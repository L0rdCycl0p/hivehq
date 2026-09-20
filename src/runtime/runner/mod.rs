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

use parking_lot::RwLock;

use crate::gdb_marker;
use crate::jit::parser::Parser;
use crate::parser::LoadedFile;
use crate::parser::load_file::load_file_by_data;
use crate::runtime::manager::Manager;
use crate::runtime::schedular::stack::Stack;
use crate::{
    error::HiveError,
    jit::{opcode::Opcode, x86_64::jit_x86_64},
    runtime::{
        load_exec::{ExecPageManager, FunctionId},
        schedular::{
            Process, Schedular,
            api::{ProcessRequest, ProcessRequestTag},
            context::HiveContext,
        },
    },
};
use std::io::{Read, Seek, SeekFrom};
use std::sync::Arc;
const STACK_SIZE: usize = 64 * 1024;

pub fn run_hive_data_stream<S: Read + Seek>(source: S) -> Result<(), HiveError> {
    let loaded_file = load_file_by_data(source)?;
    run_with_loaded_file(loaded_file)
}

pub fn run_with_loaded_file<S: Read + Seek>(
    mut loaded_file: LoadedFile<S>,
) -> Result<(), HiveError> {
    if let Some(source) = &mut loaded_file.source {
        let entry_point = &loaded_file.functions[loaded_file.header.entry_point as usize];

        source.seek(SeekFrom::Start(entry_point.code_offset));

        // SAFETY: buf is overwritten by read
        #[allow(clippy::uninit_vec)]
        let mut buf = Vec::with_capacity(entry_point.code_size as usize);
        #[allow(clippy::uninit_vec)]
        unsafe {
            buf.set_len(entry_point.code_size as usize);
        };
        #[allow(clippy::read_zero_byte_vec)]
        let _ = source.read_exact(&mut buf);
        let opcodes = Parser::new(&buf).parse()?;
        run_with_init_func(opcodes, loaded_file)
    } else {
        Err(HiveError::NoSourceAvailable)
    }
}
/// # Panics
pub fn run_with_init_func<S: Read + Seek>(
    opcodes: Box<[Opcode]>,
    loaded_file: LoadedFile<S>,
) -> Result<(), HiveError> {
    let init_func_id = loaded_file.header.entry_point;
    let init_func = &loaded_file.functions[init_func_id as usize];
    let _init_func_frame_size = init_func.frame_size as usize;
    let code = jit_x86_64(&opcodes);
    // ------------------------------------------------------------
    // Executable memory
    // ------------------------------------------------------------

    let mut exec_page_manager = ExecPageManager::new(1);

    let loaded_func = exec_page_manager.load_func(FunctionId(0), code.0)?;
    // ------------------------------------------------------------
    // Initial CPU context
    // ------------------------------------------------------------

    let mut context = HiveContext::default();

    context.rip = loaded_func.ptr as u64;

    // 5 + 12 = 17
    context.rbx = 5;
    context.r12 = 12;

    // ------------------------------------------------------------
    // Process
    // ------------------------------------------------------------

    let _process = Process {
        context,
        request: ProcessRequest {
            request: ProcessRequestTag::SchedYield,
            params: [0x0; 24],
        },
    };
    let loaded_file = RwLock::new(loaded_file);
    let manager = Manager {
        exec_page_manager: RwLock::new(exec_page_manager),
        pids: RwLock::new(Vec::new()),
        loaded_file,
    };
    let manager = Arc::new(manager);
    let stack = Stack::new(STACK_SIZE);
    let mut schedular = Schedular {
        process_registry: Vec::with_capacity(100),
        context,
        manager,
        stack,
    };

    // ------------------------------------------------------------
    // Process stack
    // ------------------------------------------------------------
    unsafe { schedular.new_process(0, FunctionId(init_func_id)) };

    // ------------------------------------------------------------
    // Run
    // ------------------------------------------------------------

    unsafe {
        schedular.schedular_run();
    }
    gdb_marker!(runner_run_with_init_func_end);
    Ok(())
}
