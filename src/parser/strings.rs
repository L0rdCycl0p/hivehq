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


use std::io::SeekFrom;

use binrw::{BinRead, BinWrite, NullString};


#[derive(Debug, BinRead, BinWrite)]
#[brw(little)]
pub struct StringEntry {
    pub offset: u64,
    #[br(seek_before = SeekFrom::Start(offset), restore_position)]
    pub content: NullString,
}

#[derive(Debug, BinRead, BinWrite)]
#[brw(little)]
#[brw(import(size: u64))]
pub struct StringTable {
    #[br(count=size/8)]
    pub strings: Vec<StringEntry>,
}
