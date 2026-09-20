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
    io::{Read, Seek},
    os::raw::c_void,
};

use libc::{calloc, free, malloc, realloc};

use crate::{
    debug::gdb_marker,
    runtime::schedular::{
        Schedular,
        api::{ProcessRequestParam, ProcessRequestTag},
    },
};
/// # Safety
/// Process Request has to be valid.
pub unsafe fn handle_request<S: Read + Seek>(process_i: usize, schedular: &mut Schedular<S>) {
    gdb_marker!(sched_req_handler_entry);
    let process = unsafe { schedular.process_registry.get_unchecked(process_i) };
    let req = &process.process.request.request;
    let param = process.process.request.params;

    let param: ProcessRequestParam = unsafe { std::mem::transmute(param) };
    gdb_marker!(sched_req_handler_match);
    match req {
        ProcessRequestTag::Syscall => todo!(),
        ProcessRequestTag::Recall => {}
        ProcessRequestTag::CallWorker => todo!(),
        ProcessRequestTag::Call => {
            let _param = unsafe { param.call };
        }
        ProcessRequestTag::MailClear => todo!(),
        ProcessRequestTag::MailLen => todo!(),
        ProcessRequestTag::MailPeek => todo!(),
        ProcessRequestTag::MailTryRecv => todo!(),
        ProcessRequestTag::MailRec => todo!(),
        ProcessRequestTag::MailSend => todo!(),
        ProcessRequestTag::SchedMigrate => todo!(),
        ProcessRequestTag::SchedCount => todo!(),
        ProcessRequestTag::SchedId => todo!(),
        ProcessRequestTag::SchedWait => todo!(),
        ProcessRequestTag::SchedWake => todo!(),
        ProcessRequestTag::SchedYield => todo!(),
        ProcessRequestTag::ProcDemonitor => todo!(),
        ProcessRequestTag::ProcMonitor => todo!(),
        ProcessRequestTag::ProcUnlink => todo!(),
        ProcessRequestTag::ProcLink => todo!(),
        ProcessRequestTag::ProcAlive => todo!(),
        ProcessRequestTag::ProcSleep => todo!(),
        ProcessRequestTag::ProcYield => todo!(),
        ProcessRequestTag::ProcState => todo!(),
        ProcessRequestTag::ProcKill => {
            let param = unsafe { param.proc_kill };
            for i in 0..schedular.process_registry.len() - 1 {
                if unsafe { schedular.process_registry.get_unchecked(i).pid } == param.pid {
                    schedular.process_registry.swap_remove(i);
                }
            }
        }
        ProcessRequestTag::ProcExit => {
            let exit_code = unsafe { param.proc_exit };

            let i = unsafe { param.mem_region };
            let a = i.dst.to_le_bytes();
            let b = i.src.to_le_bytes();
            let c = i.dst.to_le_bytes();
            let mut result = [0u8; 24];

            result[0..8].copy_from_slice(&a);
            result[8..16].copy_from_slice(&b);
            result[16..24].copy_from_slice(&c);

            println!("[EXIT:PID({})] exit code: {}", process.pid, exit_code);
            schedular.process_registry.remove(process_i);
        }
        ProcessRequestTag::ProcSelf => {
            let param = unsafe { param.proc_self };
            unsafe {
                free(param as *mut c_void);
            };
        }
        ProcessRequestTag::ProcSpawn => {
            let param = unsafe { param.proc_spawn };
            let _args_ptr = param.args as *const c_void;
            //unsafe { schedular.add_process(param.function, args_ptr) };
            todo!()
        }
        ProcessRequestTag::StackFree => todo!(),
        ProcessRequestTag::StackAlloc => todo!(),
        ProcessRequestTag::MemCmp => todo!(),
        ProcessRequestTag::MemSet => todo!(),
        ProcessRequestTag::MemMove => todo!(),
        ProcessRequestTag::MemCpy => todo!(),
        ProcessRequestTag::Free => {
            let param = unsafe { param.free };
            unsafe {
                free(param as *mut c_void);
            };
        }
        ProcessRequestTag::ReAlloc => {
            let param = unsafe { param.realloc };
            let dst_ptr: *mut *mut c_void = param.dst as *mut *mut c_void;
            unsafe {
                let ptr = realloc(param.ptr as *mut c_void, param.size as usize);
                *dst_ptr = ptr;
            };
        }
        ProcessRequestTag::CAlloc => {
            let param = unsafe { param.calloc };
            let dst_ptr: *mut *mut c_void = param.dst as *mut *mut c_void;
            unsafe {
                let ptr = calloc(param.count as usize, param.size as usize);
                *dst_ptr = ptr;
            };
        }
        ProcessRequestTag::MAlloc => {
            let param = unsafe { param.malloc };
            let dst_ptr: *mut *mut c_void = param.dst as *mut *mut c_void;
            unsafe {
                let ptr = malloc(param.size as usize);
                *dst_ptr = ptr;
            };
        }
        ProcessRequestTag::FrameFree => todo!(),
        ProcessRequestTag::FrameAlloc => todo!(),
        ProcessRequestTag::Frame => todo!(),
        ProcessRequestTag::Ret => {
            let _param = unsafe { param.ret };
        }
    }
}
