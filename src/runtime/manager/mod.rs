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
    sync::Arc,
};

use parking_lot::RwLock;

use crate::{
    error::HiveError,
    parser::LoadedFile,
    runtime::{
        load_exec::ExecPageManager,
        schedular::ProcessRegistryEntry,
    },
};

pub struct Manager<S: Read + Seek> {
    pub exec_page_manager: RwLock<ExecPageManager>,
    pub pids: RwLock<Vec<PidSlot>>,
    pub loaded_file: RwLock<LoadedFile<S>>,
}

pub enum PidSlot {
    Unused,
    Used(Arc<ProcessRegistryEntry>),
}

impl<S: Read + Seek> Manager<S> {
    pub fn allocate_pid(&self, process: Arc<ProcessRegistryEntry>) -> Result<u32, HiveError> {
        let mut pids = self.pids.write();

        for (pid, slot) in pids.iter_mut().enumerate() {
            if matches!(slot, PidSlot::Unused) {
                *slot = PidSlot::Used(process);
                return Ok(pid as u32);
            }
        }

        let pid = pids.len();
        if pid > u32::MAX as usize {
            return Err(HiveError::NoPidsAvailable);
        }
        pids.push(PidSlot::Used(process));
        Ok(pid as u32)
    }
}
