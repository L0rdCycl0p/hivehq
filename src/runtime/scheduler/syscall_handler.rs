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
    arch::asm, io::{Read, Seek}, os::raw::c_void,
};

use libc::{calloc, free, malloc, realloc, syscall};
use log::info;
use parking_lot::MutexGuard;

use crate::{
    debug::gdb_marker,
    error::HiveError,
    runtime::{
        load_exec::FunctionId,
        scheduler::{
            ProcessRegistryEntry, ProcessState, Scheduler,
            api::{MailBody, ProcessRequestParam, ProcessRequestTag},
        },
    },
};
impl<S: Read + Seek> Scheduler<S> {
    /// # Safety
    /// Syscall has to be valid.
    pub unsafe fn handle_syscall(
        &mut self,
        mut process: MutexGuard<'_, ProcessRegistryEntry>,
    ) -> Result<(), HiveError> {
        // TODO!!!
        asm!(
            "mov rax, {rax}",
            "mov rdi, {rdi}",
            "mov rsi, {rsi}",
            "mov rdx, {rdx}",
            "mov r10, {r10}",
            "mov r8, {r8}",
            "mov r9, {r9}",
            "syscall",
            rax = in(reg) process.process.context.rax,
            rdi = in(reg) process.process.context.rdi,
            rsi = in(reg) process.process.context.rsi,
            rdx = in(reg) process.process.context.rdx,
            r10 = in(reg) process.process.context.r10,
            r8  = in(reg) process.process.context.r9,
            r9  = in(reg) process.process.context.r8,
            
        );
        Ok(())
    }
}
