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
use log::info;

use crate::{
    debug::gdb_marker, error::HiveError, runtime::{load_exec::FunctionId, scheduler::{
        ProcessRegistryEntry, ProcessState, Scheduler,
        api::{MailBody, ProcessRequestParam, ProcessRequestTag},
    }},
};
impl<S: Read + Seek> Scheduler<S> {
    /// # Safety
    /// Process Request has to be valid.
    pub unsafe fn handle_request(
        &mut self,
        process: &mut ProcessRegistryEntry,
    ) -> Result<(), HiveError> {
        gdb_marker!(sched_req_handler_entry);
        let req = &process.process.request.request;
        let param = process.process.request.params;

        let param: ProcessRequestParam = unsafe { std::mem::transmute(param) };
        gdb_marker!(sched_req_handler_match);
        match req {
            ProcessRequestTag::Syscall => todo!(),
            ProcessRequestTag::Recall => {}
            ProcessRequestTag::CallWorker => {
                let param = unsafe {param.call_worker};
                let pid = unsafe { self.new_process(FunctionId(param.function), param.args)? };
                unsafe { *(param.dst as *mut u32) = pid };
            },
            ProcessRequestTag::Call => {
                let param = unsafe { param.call };
                self.call_function(process.pid, param.function, param.args as *const c_void)?;
            }
            ProcessRequestTag::MailClear => todo!(),
            ProcessRequestTag::MailLen => {
                let param = unsafe { param.mail_len };
                let dst = param as *mut u64;
                unsafe { *dst = process.mailbox.len() as u64 };
            }
            ProcessRequestTag::MailPeek => {
                let param = unsafe { param.mail_recv };
                let dst = param as *mut MailBody;
                todo!();
            }
            ProcessRequestTag::MailTryRecv => {
                let param = unsafe { param.mail_recv };
                let dst = param as *mut MailBody;
                unsafe { *dst = process.mailbox.pop().unwrap_or(MailBody::default()) };
            }
            ProcessRequestTag::MailRecv => {
                let param = unsafe { param.mail_recv };
                let dst = param as *mut MailBody;
                let msg = process.mailbox.pop();
                if let Some(msg) = msg {
                    unsafe { *dst = msg };
                } else {
                    process.state = ProcessState::WaitingRecv { dst: dst as u64 }
                }
            }
            ProcessRequestTag::MailSend => {
                let param = unsafe { param.mail_send };
                let pids = self.manager.pids.read();
                let pid_slot = pids.get(param.pid as usize).ok_or(HiveError::PidNotFound)?;
                match pid_slot {
                    crate::runtime::manager::PidSlot::Unused => return Err(HiveError::PidNotFound),
                    crate::runtime::manager::PidSlot::Used(mutex) => {
                        mutex.lock().mailbox.push(param.message);
                    }
                }
            }
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
                let pids = self.manager.pids.write();
                if let Some(crate::runtime::manager::PidSlot::Used(p)) =
                    pids.get(param.pid as usize)
                {}
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

                info!(
                    "Process `{}` exited with exit code: `{}`",
                    process.pid, exit_code
                );
                self.manager.free_pid(process.pid)?;
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
                //unsafe { scheduler.add_process(param.function, args_ptr) };
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
        };
        Ok(())
    }
}
