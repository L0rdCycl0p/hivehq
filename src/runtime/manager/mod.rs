/*
 * @file            src/runtime/manager/mod.rs
 * @description     
 * @author          TrollMii <trollmii@proton.me>
 * @createTime      2026-09-20 15:48:29
 * @lastModified    2026-09-30 16:05:07
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
    sync::Arc,
};

use crossbeam::queue::ArrayQueue;
use parking_lot::{Mutex, RwLock};
type ProcessRef = Arc<Mutex<ProcessRegistryEntry>>;
use crate::{
    error::HiveError,
    parser::LoadedFile,
    runtime::{load_exec::ExecPageManager, scheduler::ProcessRegistryEntry},
};
/// The schedulers shares this struct
pub struct Manager<S: Read + Seek> {
    /// Shared `ExecPageManager`
    pub exec_page_manager: RwLock<ExecPageManager>,
    /// A list of `PidSlot`'s
    pub pids: RwLock<Vec<PidSlot>>,
    /// Shared loaded file
    pub loaded_file: RwLock<LoadedFile<S>>,
    pub process_queue: ArrayQueue<ProcessRef>,
}

pub enum PidSlot {
    Unused,
    Used(ProcessRef),
}

impl PidSlot {
    pub fn deconstruct<S: Read + Seek>(mut self) {
        match self {
            Self::Unused => (),
            Self::Used(mutex) => {
                drop(mutex);
                self = Self::Unused;
            }
        }
    }
}

impl<S: Read + Seek> Manager<S> {
    /// Allocates a new pid to a process
    /// # Errors
    /// - `HiveError::NoPidsAvailable` if no pids are available
    pub fn allocate_pid(&self, process: ProcessRef) -> Result<u32, HiveError> {
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

    pub fn free_pid(&self, pid: u32) -> Result<(), HiveError> {
        let pids = self.pids.read();

        let proc = pids.get(pid as usize).ok_or(HiveError::PidNotFound)?;
        match proc {
            PidSlot::Unused => return Err(HiveError::PidAlreadyFree),
            PidSlot::Used(mutex) => {
                let mut proc = mutex.lock();
                proc.state = super::scheduler::ProcessState::Dead;
            }
        }
        drop(pids);
        let mut pids = self.pids.write();

        let proc = pids.get_mut(pid as usize).ok_or(HiveError::PidNotFound)?;
        *proc = PidSlot::Unused;
        drop(pids);
        Ok(())
    }
}
