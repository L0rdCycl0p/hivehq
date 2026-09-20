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


use std::{
    io, ptr,
    sync::{Arc, Weak},
};

use libc::{
    MAP_ANONYMOUS, MAP_FAILED, MAP_PRIVATE, PROT_EXEC, PROT_READ, PROT_WRITE, mmap, mprotect,
    munmap,
};

const PAGE_SIZE: usize = 4096;
/// The `FunctionId`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId(pub u32);
/// A struct that represents a loaded Page in memory
pub struct ExecPage {
    ptr: *mut u8,
    size: usize,
}

impl ExecPage {
    fn new() -> io::Result<Self> {
        let ptr = unsafe {
            mmap(
                ptr::null_mut(),
                PAGE_SIZE,
                PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS,
                -1,
                0,
            )
        };

        if ptr == MAP_FAILED {
            return Err(io::Error::last_os_error());
        }

        Ok(Self {
            ptr: ptr.cast(),
            size: PAGE_SIZE,
        })
    }

    fn make_writable(&self) -> io::Result<()> {
        let result = unsafe { mprotect(self.ptr.cast(), self.size, PROT_READ | PROT_WRITE) };

        if result != 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(())
    }

    fn make_executable(&self) -> io::Result<()> {
        let result = unsafe { mprotect(self.ptr.cast(), self.size, PROT_READ | PROT_EXEC) };

        if result != 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(())
    }
}

impl Drop for ExecPage {
    fn drop(&mut self) {
        unsafe {
            munmap(self.ptr.cast(), self.size);
        }
    }
}

/// A struct that represents a loaded func
pub struct LoadedFunc {
    pub id: FunctionId,
    pub ptr: *mut u8,
    pub len: usize,

    pages: Box<[Arc<ExecPage>]>,
}

impl LoadedFunc {
    /// getter for `self.ptr`
    #[must_use]
    pub const fn ptr(&self) -> *mut u8 {
        self.ptr
    }
    /// getter for `self.len`
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }
    /// checks if `self.len` is equal to zero
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
    /// getter for `self.id`
    #[must_use]
    pub const fn id(&self) -> FunctionId {
        self.id
    }
}

struct ManagedPage {
    page: Weak<ExecPage>,
    used: usize,
}

impl ManagedPage {
    const fn remaining(&self) -> usize {
        PAGE_SIZE - self.used
    }
}

pub struct ExecPageManager {
    pages: Vec<ManagedPage>,
    functions: Vec<Option<Weak<LoadedFunc>>>,
}

impl ExecPageManager {
    /// Creates a new `ExecPageManager`
    #[must_use]
    pub fn new(num_funcs: usize) -> Self {
        let mut functions: Vec<Option<Weak<LoadedFunc>>> = Vec::with_capacity(num_funcs);
        for _ in 0..num_funcs {
            functions.push(None);
        }
        Self {
            pages: Vec::new(),
            functions
        }
    }
    /// Gets a function if it is already loaded into memory via `mmap`
    pub fn get_func(&mut self, id: FunctionId) -> io::Result<Arc<LoadedFunc>> {
        let index = id.0 as usize;

        if self.functions.len() <= index {
            self.functions.resize(index + 1, None);
        }

        if let Some(weak_func) = &self.functions[index]
            && let Some(func) = weak_func.upgrade()
        {
            return Ok(func);
        }

        Err(io::Error::new(io::ErrorKind::NotFound, ""))
    }
    /// Checks whether a function exists
    #[must_use]
    pub fn func_exists(&self, id: FunctionId) -> bool {
        let index = id.0 as usize;

        let Some(Some(weak_func)) = self.functions.get(index) else {
            return false;
        };

        weak_func.strong_count() > 0
    }
    /// Loads a function into memory via `mmap`
    pub fn load_func(&mut self, id: FunctionId, code: Box<[u8]>) -> io::Result<Arc<LoadedFunc>> {
        let len = code.len();

        if len == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "cannot load empty function",
            ));
        }

        self.pages.retain(|managed| managed.page.strong_count() > 0);

        let mut remaining = len;
        let mut code_offset = 0usize;

        let mut function_ptr: Option<*mut u8> = None;

        let mut used_pages: Vec<Arc<ExecPage>> = Vec::new();

        for managed in &mut self.pages {
            if remaining == 0 {
                break;
            }

            let Some(page) = managed.page.upgrade() else {
                continue;
            };

            let available = managed.remaining();

            if available == 0 {
                continue;
            }

            let amount = available.min(remaining);

            page.make_writable()?;

            let dst = unsafe { page.ptr.add(managed.used) };

            unsafe {
                ptr::copy_nonoverlapping(code.as_ptr().add(code_offset), dst, amount);
            }

            managed.used += amount;

            if function_ptr.is_none() {
                function_ptr = Some(dst);
            }

            remaining -= amount;
            code_offset += amount;

            page.make_executable()?;

            used_pages.push(page);
        }

        while remaining > 0 {
            let page = Arc::new(ExecPage::new()?);

            let amount = remaining.min(PAGE_SIZE);

            let dst = page.ptr;

            unsafe {
                ptr::copy_nonoverlapping(code.as_ptr().add(code_offset), dst, amount);
            }

            page.make_executable()?;

            self.pages.push(ManagedPage {
                page: Arc::downgrade(&page),
                used: amount,
            });

            used_pages.push(page);

            if function_ptr.is_none() {
                function_ptr = Some(dst);
            }

            remaining -= amount;
            code_offset += amount;
        }

        let ptr = function_ptr.expect("function length was checked to be non-zero");
        let loaded_func = LoadedFunc {
            id,
            ptr,
            len,
            pages: used_pages.into_boxed_slice(),
        };
        let loaded_func = Arc::new(loaded_func);
        if self.functions.len() <= id.0 as usize {
            let mut c = id.0 as usize - self.functions.len(); // [0, 1, 2, 3, 4, 5, 6, 7, 8] 9
            while c != 0 {
                self.functions.push(None);
                c-=1;
            }
            self.functions.push(None);
        } else {
            self.functions[id.0 as usize] = Some(Arc::downgrade(&loaded_func));
        }
        Ok(loaded_func)
    }
}

impl Default for ExecPageManager {
    fn default() -> Self {
        Self::new(0)
    }
}
