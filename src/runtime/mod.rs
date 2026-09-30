/*
 * @file            src/runtime/mod.rs
 * @description     
 * @author          TrollMii <trollmii@proton.me>
 * @createTime      2026-09-20 15:48:29
 * @lastModified    2026-09-30 16:04:49
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

use std::io::{Read, Seek};

use crate::{parser::LoadedFile, runtime::scheduler::Scheduler};

pub mod load_exec;
pub mod manager;
pub mod runner;
pub mod scheduler;

pub struct Runtime<S: Read + Seek> {
    pub schedulers: Vec<Scheduler<S>>,
    pub loaded_file: LoadedFile<S>,
}
