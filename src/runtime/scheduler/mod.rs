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

use chrono::Utc;
use crossbeam::queue::{ArrayQueue, SegQueue};
use log::{info, warn};
use parking_lot::Mutex;
use static_assertions::assert_impl_all;
use std::{
    collections::VecDeque,
    io::{Read, Seek, SeekFrom},
    os::{fd::RawFd, raw::c_void},
    sync::Arc,
};

pub mod api;
pub mod context;
pub mod request_handler;
pub mod stack;

use api::ProcessRequest;
use stack::{Stack, StackRegion};

use crate::{
    debug::gdb_marker, error::HiveError, jit::jit_function, runtime::{
        load_exec::{FunctionId, LoadedFunc}, manager::{Manager, PidSlot}, scheduler::{
            api::{MailBody, ProcessRequestTag}, context::{HiveContext, swapcontext},
        },
    },
};

pub type Pid = u32;

pub struct Scheduler<S: Read + Seek> {
    //pub process_registry: Vec<ProcessRegistryEntry>,
    pub context: HiveContext,
    pub stack: Stack,
    pub manager: Arc<Manager<S>>,
}

pub struct CallRetStack {
    pub stack_region: StackRegion,
    pub function: Arc<LoadedFunc>,
}

pub struct ProcessRegistryEntry {
    pub pid: Pid,

    /// The process' CALL/RET stack.
    pub stack_frames: Vec<CallRetStack>,

    pub process: Box<Process>,
    pub state: ProcessState,
    /// SegQueue of ptr to data
    pub mailbox: SegQueue<MailBody>, 
}

#[repr(C)]
pub struct Process {
    pub context: HiveContext,
    pub request: ProcessRequest,
}

pub enum ProcessState {
    Running,
    WaitingRecv { dst: u64 },
    WaitingFD { fd: RawFd },
    Sleep { end: i64 },
    Dead,
}
impl<S: Read + Seek> Scheduler<S> {
    pub fn new(manager: Arc<Manager<S>>, stack_size: usize) -> Result<Self, HiveError> {
        Ok(Self {
            //process_registry: Vec::with_capacity(100),
            context: HiveContext::default(),
            stack: Stack::new(stack_size)?,
            manager,
        })
    }
}
impl<S: Read + Seek> Scheduler<S> {
    pub unsafe fn scheduler_run(&mut self) -> Result<(), HiveError> {
        unsafe {
            gdb_marker!(sched_run);

            while !self.manager.process_queue.is_empty() {
                if let Some(p) = self.manager.process_queue.pop() {
                    let mut process = p.lock();
                    match process.state {
                        ProcessState::Running => {}

                        ProcessState::WaitingRecv { dst } => {
                            let Some(msg) = process.mailbox.pop() else {
                                drop(process);
                                self.manager
                                    .process_queue
                                    .push(p)
                                    .map_err(|_| HiveError::QueueIsFull)?;
                                continue;
                            };

                            *(dst as *mut MailBody) = msg;

                            process.state = ProcessState::Running;
                        }

                        ProcessState::WaitingFD { .. } => {
                            todo!();
                        }


                        ProcessState::Sleep { end } if Utc::now().timestamp_micros() >= end => {
                            process.state = ProcessState::Running;
                        }

                        ProcessState::Sleep { .. } => {
                            drop(process);
                            self.manager
                                .process_queue
                                .push(p)
                                .map_err(|_| HiveError::QueueIsFull)?;
                            continue;
                        }
                        ProcessState::Dead => {
                            drop(process);
                            drop(p);
                            continue;
                        }
                    }

                    gdb_marker!(sched_process_context_swap);

                    swapcontext(&mut self.context, &process.process.context);

                    self.handle_request(process)?;
                    self.manager
                        .process_queue
                        .push(p)
                        .map_err(|_| HiveError::QueueIsFull)?;
                } else {
                    return Err(HiveError::Unknown(None))
                }
            }
            warn!("All processes are done");
        }
        Ok(())
    }
    pub unsafe fn new_process(&mut self, function_id: FunctionId, args: u64) -> Result<Pid, HiveError> {
        unsafe {
            let (code_offset, code_size, frame_size) = {
                let loaded_file = self.manager.loaded_file.read();

                let function = loaded_file.functions[function_id.0 as usize];
                drop(loaded_file);

                (
                    function.code_offset,
                    function.code_size,
                    function.frame_size,
                )
            };

            let loaded_func = {
                let mut exec_page_manager = self.manager.exec_page_manager.write();

                if exec_page_manager.func_exists(function_id) {
                    exec_page_manager.get_func(function_id)?
                } else {
                    let mut loaded_file = self.manager.loaded_file.write();

                    let source = loaded_file
                        .source
                        .as_mut()
                        .ok_or(HiveError::NoSourceAvailable)?;
                    source.seek(SeekFrom::Start(code_offset))?;

                    let mut buf = vec![0; code_size as usize];

                    source.read_exact(&mut buf)?;

                    drop(loaded_file);
                    let code = jit_function(buf.into_boxed_slice());

                    exec_page_manager.load_func(function_id, code)?
                }
            };

            // ------------------------------------------------------------
            // Initial CPU context
            // ------------------------------------------------------------

            let mut context = HiveContext::default();

            context.rip = loaded_func.ptr as u64;

            let process = Process {
                context,
                request: ProcessRequest {
                    request: ProcessRequestTag::SchedYield,
                    params: [0; 24],
                },
            };

            let process = ProcessRegistryEntry {
                pid: 0,
                stack_frames: Vec::with_capacity(5),
                process: Box::new(process),
                state: ProcessState::Running,
                mailbox: SegQueue::new(),
            };
            let process = Arc::new(Mutex::new(process));
            let pid = self.manager.allocate_pid(process.clone())?;
            let mut process_guard = process.lock();
            process_guard.pid = pid;
            let frame = self.stack.new_proc_stack(
                pid,
                frame_size as usize,
                &raw mut process_guard.process.request,
                &raw mut self.context,
                &raw mut process_guard.process.context,
                args
            )?;
            let frame = CallRetStack {
                stack_region: frame,
                function: loaded_func,
            };
            process_guard.stack_frames.push(frame);
            drop(process_guard);
            self.manager.process_queue.push(process);

            Ok(pid)
        }
    }
    pub fn call_function(
        &mut self,
        pid: Pid,
        function_id: u32,
        args: *const c_void
    ) -> Result<(), HiveError> {
        let (code_offset, code_size, frame_size) = {
            let loaded_file = self.manager.loaded_file.read();

            let function = &loaded_file.functions[function_id as usize];

            (
                function.code_offset,
                function.code_size,
                function.frame_size,
            )
        };
        let frame = self.stack.allocate(pid, frame_size as usize)?;
        let loaded_func = {
            let mut exec_page_manager = self.manager.exec_page_manager.write();

            if exec_page_manager.func_exists(FunctionId(function_id)) {
                exec_page_manager.get_func(FunctionId(function_id))?
            } else {
                let mut loaded_file = self.manager.loaded_file.write();

                let source = loaded_file
                    .source
                    .as_mut()
                    .ok_or(HiveError::NoSourceAvailable)?;
                source.seek(SeekFrom::Start(code_offset))?;

                let mut buf = vec![0; code_size as usize];

                source.read_exact(&mut buf)?;

                drop(loaded_file);
                let code = jit_function(buf.into_boxed_slice());

                exec_page_manager.load_func(FunctionId(function_id), code)?
            }
        };
        let frame = CallRetStack {
            stack_region: frame,
            function: loaded_func,
        };
        match &self.manager.pids.read()[pid as usize] {
            PidSlot::Unused => todo!(),
            PidSlot::Used(mutex) => mutex.lock().stack_frames.push(frame),
        };

        // Initialize frame/function arguments here.
        let _ = function_id;
        let _ = args;
        Ok(())
    }

    pub fn return_function(&mut self, pid: Pid) -> Result<(), HiveError> {
        match &self.manager.pids.read()[pid as usize] {
            PidSlot::Unused => todo!(),
            PidSlot::Used(mutex) => {
                let frame = mutex
                    .lock()
                    .stack_frames
                    .pop()
                    .ok_or(HiveError::RetOnEmptyStack)?;

                self.stack.free(frame.stack_region.start)?;
                Ok(())
            }
        }
    }
}
