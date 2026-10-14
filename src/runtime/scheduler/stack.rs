/*
 * @file            src/runtime/scheduler/stack.rs
 * @description
 * @author          TrollMii <trollmii@proton.me>
 * @createTime      2026-09-27 15:55:31
 * @lastModified    2026-10-14 15:58:23
 * Copyright ©L0rdCycl0p
                    HIVE (Hive Is Very Efficient)
Copyright (C) 2026 L0rdCycl0p

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>.

*/

use std::{
    alloc::{Layout, alloc, dealloc},
    os::raw::c_void,
};

use crate::{
    error::HiveError,
    runtime::scheduler::{api::ProcessRequest, context::HiveContext},
};

use super::Pid;

pub struct Stack {
    pub start: *mut u8,
    pub end: *mut u8,
    pub size: usize,
    pub frames: Vec<StackRegion>,
}
// SAFETY: Stack is in a mutex
unsafe impl Send for Stack {}
unsafe impl Sync for Stack {}
pub struct StackRegion {
    pub start: *mut u8,
    pub end: *mut u8,
    pub state: FrameState,
}

pub enum FrameState {
    Used(Pid),
    Free,
}

impl Stack {
    /// Creates a new Hive Stack
    /// # Errors
    /// - `HiveError::CanNotAllocateStack`
    /// - `HiveError::LayoutError`
    pub fn new(size: usize) -> Result<Self, HiveError> {
        if size == 0 {
            return Err(HiveError::CanNotAllocateStack);
        }

        let layout = Layout::from_size_align(size, 16)?;

        let start = unsafe { alloc(layout) };

        if start.is_null() {
            return Err(HiveError::CanNotAllocateStack);
        }
        let end = unsafe { start.add(size) };

        Ok(Self {
            start,
            end,
            size,
            frames: vec![StackRegion {
                start,
                end,
                state: FrameState::Free,
            }],
        })
    }
    /// Allocates a `StackRegion`
    /// # Errors
    /// - `HiveError::StackExhausted`
    pub fn allocate(&mut self, pid: Pid, size: usize) -> Result<StackRegion, HiveError> {
        for i in 0..self.frames.len() {
            let frame = &mut self.frames[i];

            if !matches!(frame.state, FrameState::Free) {
                continue;
            }

            let available = frame.end as usize - frame.start as usize;

            if available < size {
                continue;
            }

            let start = frame.start;
            let end = unsafe { start.add(size) };
            let old_end = frame.end;

            frame.end = end;
            frame.state = FrameState::Used(pid);

            if available > size {
                self.frames.insert(
                    i + 1,
                    StackRegion {
                        start: end,
                        end: old_end,
                        state: FrameState::Free,
                    },
                );
            }

            return Ok(StackRegion {
                start,
                end,
                state: FrameState::Used(pid),
            });
        }

        Err(HiveError::StackExhausted)
    }
    /// Frees a stack frame
    /// # Errors
    /// - `HiveError::AttemptedToFreeUnknownStackFrame`
    pub fn free(&mut self, start: *mut u8) -> Result<(), HiveError> {
        for frame in &mut self.frames {
            if frame.start == start {
                frame.state = FrameState::Free;
                return Ok(());
            }
        }

        Err(HiveError::AttemptedToFreeUnknownStackFrame)
    }
    /// Compact the stack
    pub fn compact(&mut self) {
        let mut i = 0;

        while i + 1 < self.frames.len() {
            let current_free = matches!(self.frames[i].state, FrameState::Free);

            let next_free = matches!(self.frames[i + 1].state, FrameState::Free);

            if current_free && next_free {
                let next_end = self.frames[i + 1].end;

                self.frames[i].end = next_end;
                self.frames.remove(i + 1);

                continue;
            }

            i += 1;
        }
    }
    /// Allocates a new stack frame and initialises it for a new process.
    ///
    /// # Layout
    /// On return, the process context's `rsp` points at four consecutive
    /// `u64` slots in the region (low → high):
    ///
    /// ```text
    /// [rsp +  0] = ProcessRequest*
    /// [rsp +  8] = HiveContext*   (scheduler)
    /// [rsp + 16] = HiveContext*   (process)
    /// [rsp + 24] = params
    /// ```
    ///
    /// `rsp` is 16-byte aligned. At least 128 bytes of SysV red zone are
    /// reserved above the slots.
    ///
    /// # Safety
    /// - All pointer arguments must be non-null and point to live values
    ///   for at least as long as the process runs.
    /// - `scheduler_context_ptr` and `process_context_ptr` must be distinct.
    /// - `params` must remain valid until the process reads it.
    ///
    /// # Errors
    /// - [`HiveError::StackExhausted`] if `size` is too small.
    /// - [`HiveError::AttemptedToFreeUnknownStackFrame`]
    /// - A "null pointer" error variant (see `NullPointer` below).
    #[allow(clippy::panic_in_result_fn)] // no longer needed, but harmless
    pub unsafe fn new_proc_stack(
        &mut self,
        pid: u32,
        size: usize,
        process_request_ptr: *mut ProcessRequest,
        scheduler_context_ptr: *mut HiveContext,
        process_context_ptr: *mut HiveContext,
        params: *const c_void,
    ) -> Result<StackRegion, HiveError> {
        use std::mem::{align_of, size_of};

        const RED_ZONE: usize = 128; // x86-64 SysV red zone
        const N_SLOTS: usize = 4;
        const SLOT_BYTES: usize = N_SLOTS * size_of::<u64>();
        const ALIGN: usize = align_of::<u64>(); // 8; rsp is still 16-byte aligned

        // Require enough room for the slots + red zone + alignment slack.
        let minimum = SLOT_BYTES
            .checked_add(RED_ZONE)
            .and_then(|n| n.checked_add(ALIGN - 1))
            .ok_or(HiveError::StackExhausted)?;
        if size < minimum {
            return Err(HiveError::StackExhausted);
        }

        // Reject null pointers *before* we allocate, so there's nothing to free.
        if process_request_ptr.is_null()
            || scheduler_context_ptr.is_null()
            || process_context_ptr.is_null()
        {
            return Err(HiveError::NullPointer); // add this variant if missing
        }

        // `allocate` is safe; keep it out of any unsafe block.
        let stack = self.allocate(pid, size)?;

        // Stack grows down: `end` is the high address.
        debug_assert!(stack.end as usize > stack.start as usize);

        let rsp = match (stack.end as usize)
            .checked_sub(RED_ZONE)
            .and_then(|a| a.checked_sub(SLOT_BYTES))
            .map(|a| a & !(ALIGN - 1))
        {
            Some(rsp) if rsp >= stack.start as usize => rsp,
            _ => {
                // Give the frame back; don't leak on the error path.
                let _ = self.free(stack.start);
                return Err(HiveError::StackExhausted);
            }
        };

        // SAFETY: `rsp` is 16-byte aligned and we checked that `rsp..rsp+32`
        // lies within `stack`. `align_of::<u64>() == 8` divides `ALIGN`, so
        // every slot is aligned.
        let slots: &mut [u64; N_SLOTS] = unsafe { &mut *(rsp as *mut [u64; N_SLOTS]) };
        slots[0] = process_request_ptr as u64;
        slots[1] = scheduler_context_ptr as u64;
        slots[2] = process_context_ptr as u64;
        slots[3] = params as u64;

        // SAFETY: caller promised `process_context_ptr` is valid and writable.
        unsafe {
            (*process_context_ptr).rsp = rsp as u64;
        }

        Ok(stack)
    }
}

impl Drop for Stack {
    fn drop(&mut self) {
        let size = self.end as usize - self.start as usize;

        let layout = Layout::from_size_align(size, 16).expect("invalid stack layout");

        unsafe {
            dealloc(self.start, layout);
        }
    }
}
