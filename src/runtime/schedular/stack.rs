// HIVE (Hive Is Very Efficient)
// Copyright (C) 2026 L0rdCycl0p
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use std::alloc::{Layout, alloc, dealloc};

use crate::{error::HiveError, runtime::schedular::{api::ProcessRequest, context::HiveContext}};

use super::Pid;

pub struct Stack {
    pub start: *mut u8,
    pub end: *mut u8,
    pub size: usize,
    pub frames: Vec<StackRegion>,
}

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
    pub fn new(size: usize) -> Result<Self, HiveError> {
        if size == 0 {
            return Err(HiveError::CanNotAllocateStack)
        }

        let layout = Layout::from_size_align(size, 16)?;

        let start = unsafe { alloc(layout) };

        if start.is_null() {
            return Err(HiveError::CanNotAllocateStack)
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
    /// Allocates a new stack frame and initialised it for a new process
    /// # Safety
    /// The caller must ensure that all the pointers are valid
    /// # Errors
    /// - `HiveError::StackExhausted`
    /// - `HiveError::AttemptedToFreeUnknownStackFrame`
    pub unsafe fn new_proc_stack(
        &mut self,
        pid: u32,
        size: usize,
        process_request_ptr: *mut ProcessRequest,
        schedular_context_ptr: *mut HiveContext,
        process_context_ptr: *mut HiveContext,
    ) -> Result<StackRegion, HiveError>{
        unsafe {
            let stack = self.allocate(pid, size)?;

            let rsp = (stack.end as usize - 24) & !15;

            // [rsp + 0]  = request pointer
            // [rsp + 8]  = scheduler context pointer
            // [rsp + 16] = process context pointer

            (*process_context_ptr).rsp = rsp as u64;

            *(rsp as *mut u64) = process_request_ptr as u64;

            *((rsp + 8) as *mut u64) = schedular_context_ptr as u64;

            *((rsp + 16) as *mut u64) = process_context_ptr as u64;
            Ok(stack)
        }
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
