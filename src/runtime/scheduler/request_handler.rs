/*
 * @file            src/runtime/scheduler/request_handler.rs
 * @description     
 * @author          TrollMii <trollmii@proton.me>
 * @createTime      2026-09-27 15:55:31
 * @lastModified    2026-09-29 20:31:19
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
    io::{Read, Seek},
    os::raw::c_void,
};

use chrono::Local;
use libc::{calloc, free, malloc, realloc};
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
    /// Process Request has to be valid.
    /// # Errors
    /// `HiveError::PidNotFound`
    pub unsafe fn handle_request(
        &mut self,
        mut process: MutexGuard<'_, ProcessRegistryEntry>,
    ) -> Result<(), HiveError> {
        unsafe {
            gdb_marker!(sched_req_handler_entry);
            let req = &process.process.request.request;
            let param = process.process.request.params;

            let param: ProcessRequestParam = std::mem::transmute(param);
            gdb_marker!(sched_req_handler_match);
            match req {
                ProcessRequestTag::Syscall => {
                    self.handle_syscall(process)?;
                }
                // YIELD
                ProcessRequestTag::Recall
                | ProcessRequestTag::SchedYield
                | ProcessRequestTag::ProcYield => {}
                ProcessRequestTag::CallWorker => {
                    let param = param.call_worker;
                    let pid = self.new_process(FunctionId(param.function), param.args)?;
                    *(param.dst as *mut u32) = pid;
                }
                ProcessRequestTag::Call => {
                    let param = param.call;
                    self.call_function(process.pid, param.function, param.args as *const c_void)?;
                }
                ProcessRequestTag::MailClear => {
                    while process.mailbox.pop().is_some() {} // TODO!!! That can be better
                }
                ProcessRequestTag::MailLen => {
                    let param = param.mail_len;
                    let dst = param as *mut u64;
                    *dst = process.mailbox.len() as u64;
                }
                ProcessRequestTag::MailPeek => {
                    let param = param.mail_recv;
                    let _dst = param as *mut MailBody;
                    todo!();
                }
                ProcessRequestTag::MailTryRecv => {
                    let param = param.mail_recv;
                    let dst = param as *mut MailBody;
                    *dst = process.mailbox.pop().unwrap_or_default();
                }
                ProcessRequestTag::MailRecv => {
                    let param = param.mail_recv;
                    let dst = param as *mut MailBody;
                    let msg = process.mailbox.pop();
                    if let Some(msg) = msg {
                        *dst = msg;
                    } else {
                        process.state = ProcessState::WaitingRecv { dst: dst as u64 }
                    }
                }
                ProcessRequestTag::MailSend => {
                    let param = param.mail_send;
                    let pids = self.manager.pids.read();
                    let pid_slot = pids.get(param.pid as usize).ok_or(HiveError::PidNotFound)?;

                    match pid_slot {
                        crate::runtime::manager::PidSlot::Unused => {
                            return Err(HiveError::PidNotFound);
                        }
                        crate::runtime::manager::PidSlot::Used(mutex) => {
                            mutex.lock().mailbox.push(param.message);
                        }
                    }
                    drop(pids);
                }
                ProcessRequestTag::SchedMigrate => todo!(),
                ProcessRequestTag::SchedCount => todo!(),
                ProcessRequestTag::SchedId => todo!(),
                ProcessRequestTag::SchedWait => todo!(),
                ProcessRequestTag::SchedWake => todo!(),
                ProcessRequestTag::ProcDemonitor => todo!(),
                ProcessRequestTag::ProcMonitor => todo!(),
                ProcessRequestTag::ProcUnlink => todo!(),
                ProcessRequestTag::ProcLink => todo!(),
                ProcessRequestTag::ProcAlive => todo!(),
                ProcessRequestTag::ProcSleep => {
                    let start = Local::now().timestamp_micros();
                    let duration = param.proc_sleep;
                    let end = start + duration as i64;
                    process.state = ProcessState::Sleep { end };
                }
                ProcessRequestTag::ProcState => todo!(),
                ProcessRequestTag::ProcKill => {
                    let param = param.proc_kill;
                    let pids = self.manager.pids.write();
                    if let Some(crate::runtime::manager::PidSlot::Used(_p)) =
                        pids.get(param.pid as usize)
                    {}
                }
                ProcessRequestTag::ProcExit => {
                    let exit_code = param.proc_exit;

                    let i = param.mem_region;
                    let a = i.dst.to_le_bytes();
                    let b = i.src.to_le_bytes();
                    let c = i.dst.to_le_bytes();
                    let mut result = [0u8; 24];

                    result[0..8].copy_from_slice(&a);
                    result[8..16].copy_from_slice(&b);
                    result[16..24].copy_from_slice(&c);
                    let pid = process.pid;
                    drop(process);
                    #[cfg(feature = "cfg_log_proc_exit")]
                    info!("Process `{pid}` exited with exit code: `{exit_code}`");
                    self.manager.free_pid(pid)?;
                }
                ProcessRequestTag::ProcSelf => {
                    let param = param.proc_self;
                    free(param as *mut c_void);
                }
                ProcessRequestTag::ProcSpawn => {
                    let param = param.proc_spawn;
                    let pid = self.new_process(FunctionId(param.function), param.args)?;
                    *(param.dst as *mut u32) = pid;
                }
                ProcessRequestTag::StackFree => todo!(),
                ProcessRequestTag::StackAlloc => todo!(),
                ProcessRequestTag::MemCmp => todo!(),
                ProcessRequestTag::MemSet => todo!(),
                ProcessRequestTag::MemMove => todo!(),
                ProcessRequestTag::MemCpy => todo!(),
                ProcessRequestTag::Free => {
                    let param = param.free;
                    free(param as *mut c_void);
                }
                ProcessRequestTag::ReAlloc => {
                    let param = param.realloc;
                    let dst_ptr: *mut *mut c_void = param.dst as *mut *mut c_void;
                    let ptr = realloc(param.ptr as *mut c_void, param.size as usize);
                    *dst_ptr = ptr;
                }
                ProcessRequestTag::CAlloc => {
                    let param = param.calloc;
                    let dst_ptr: *mut *mut c_void = param.dst as *mut *mut c_void;
                    let ptr = calloc(param.count as usize, param.size as usize);
                    *dst_ptr = ptr;
                }
                ProcessRequestTag::MAlloc => {
                    let param = param.malloc;
                    let dst_ptr: *mut *mut c_void = param.dst as *mut *mut c_void;
                    let ptr = malloc(param.size as usize);
                    *dst_ptr = ptr;
                }
                ProcessRequestTag::FrameFree => todo!(),
                ProcessRequestTag::FrameAlloc => todo!(),
                ProcessRequestTag::Frame => todo!(),
                ProcessRequestTag::Ret => {
                    let _param = param.ret;
                }
            }
            Ok(())
        }
    }
}
