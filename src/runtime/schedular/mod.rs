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
    debug::gdb_marker,
    error::HiveError,
    jit::jit_function,
    runtime::{
        load_exec::FunctionId,
        manager::Manager,
        schedular::{
            api::ProcessRequestTag,
            context::{HiveContext, swapcontext},
            request_handler::handle_request,
        },
    },
};

pub type Pid = u32;

pub struct Schedular<S: Read + Seek> {
    pub process_registry: Vec<ProcessRegistryEntry>,
    pub context: HiveContext,
    pub stack: Stack,
    pub manager: Arc<Manager<S>>,
}

pub struct ProcessRegistryEntry {
    pub pid: Pid,

    // The process' CALL/RET stack.
    pub stack_frames: Vec<StackRegion>,

    pub process: Box<Process>,
    pub state: ProcessState,
    pub mailbox: VecDeque<u64>,
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
}
impl<S: Read + Seek> Schedular<S> {
    pub fn new(manager: Arc<Manager<S>>, stack_size: usize) -> Self {
        Self {
            process_registry: Vec::with_capacity(100),
            context: HiveContext::default(),
            stack: Stack::new(stack_size),
            manager,
        }
    }
}
impl<S: Read + Seek> Schedular<S> {
    pub unsafe fn schedular_run(&mut self) { unsafe {
        gdb_marker!(sched_run);

        loop {
            if self.process_registry.is_empty() {
                println!("All process done");
                break;
            }

            for i in 0..self.process_registry.len() {
                let process = self.process_registry.get_unchecked_mut(i);

                match process.state {
                    ProcessState::Running => {}

                    ProcessState::WaitingRecv { dst } => {
                        let Some(msg) = process.mailbox.pop_front() else {
                            continue;
                        };

                        *(dst as *mut *mut c_void) = msg as *mut c_void;

                        process.state = ProcessState::Running;
                    }

                    ProcessState::WaitingFD { .. } => {
                        todo!();
                    }

                    ProcessState::Sleep { end } if Utc::now().timestamp_micros() >= end => {
                        process.state = ProcessState::Running;
                    }

                    ProcessState::Sleep { .. } => {
                        continue;
                    }
                }

                gdb_marker!(sched_process_context_swap);

                swapcontext(&mut self.context, &process.process.context);

                handle_request(i, self);
            }
        }
    }}
    pub unsafe fn new_process(
        &mut self,
        pid: u32,
        function_id: FunctionId,
    ) -> Result<(), HiveError> { unsafe {
        let (code_offset, code_size, frame_size) = {
            let loaded_file = self.manager.loaded_file.read();

            let function = &loaded_file.functions[function_id.0 as usize];

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

                let code = jit_function(buf.into_boxed_slice());

                Arc::new(exec_page_manager.load_func(function_id, code)?)
            }
        };

        // ------------------------------------------------------------
        // Initial CPU context
        // ------------------------------------------------------------

        let mut context = HiveContext::default();

        context.rip = loaded_func.ptr as u64;

        context.rbx = 5;
        context.r12 = 12;

        let process = Process {
            context,
            request: ProcessRequest {
                request: ProcessRequestTag::SchedYield,
                params: [0; 24],
            },
        };

        let mut process = ProcessRegistryEntry {
            pid,
            stack_frames: Vec::new(),
            process: Box::new(process),
            state: ProcessState::Running,
            mailbox: VecDeque::new(),
        };

        self.stack.new_proc_stack(
            pid,
            frame_size as usize,
            &raw mut process.process.request,
            &raw mut self.context,
            &raw mut process.process.context,
        );

        self.process_registry.push(process);

        Ok(())
    }}
    pub fn call_function(
        &mut self,
        pid: Pid,
        function_id: u32,
        args: *const c_void,
        frame_size: usize,
    ) {
        let frame = self
            .stack
            .allocate(pid, frame_size)
            .expect("HIVE stack exhausted");

        self.process_registry[pid as usize].stack_frames.push(frame);

        // Initialize frame/function arguments here.
        let _ = function_id;
        let _ = args;
    }

    pub fn return_function(&mut self, pid: Pid) {
        let process = &mut self.process_registry[pid as usize];

        let frame = process.stack_frames.pop().expect("RET on empty stack");

        self.stack.free(frame.start);
    }
}
