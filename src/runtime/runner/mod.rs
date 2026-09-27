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
use crate::jit::jit_function;
use crate::jit::parser::Parser;
use crate::parser::LoadedFile;
use crate::parser::load_file::load_file_by_data;
use crate::runtime::manager::Manager;
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

        let _ = source.seek(SeekFrom::Start(entry_point.code_offset));

        let init_func_id = loaded_file.header.entry_point;

        let exec_page_manager = ExecPageManager::new(1);

        let loaded_file = RwLock::new(loaded_file);
        let manager = Manager {
            exec_page_manager: RwLock::new(exec_page_manager),
            pids: RwLock::new(Vec::new()),
            loaded_file,
        };
        let manager = Arc::new(manager);
        let mut schedular = Schedular::new(manager, STACK_SIZE)?;

        unsafe { schedular.new_process(0, FunctionId(init_func_id))? };

        // ------------------------------------------------------------
        // Run
        // ------------------------------------------------------------

        unsafe {
            schedular.schedular_run();
        }
        gdb_marker!(runner_run_with_init_func_end);
        Ok(())
    } else {
        Err(HiveError::NoSourceAvailable)
    }
}
