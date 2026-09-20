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


use binrw::{BinRead, BinWrite};
use bitflags::bitflags;

pub const MAGIC: [u8; 4] = [0x48, 0x49, 0x56, 0x45]; // HIVE

#[derive(Debug, BinRead, BinWrite, Default)]
#[brw(repr = u16, little)]
pub enum FileType {
    #[default]
    Executable = 0x1,
    Module = 0x2,
    Library = 0x3,
    Object = 0x4,
}

bitflags! {
    #[derive(Debug, Default)]
    pub struct FileFlags: u32 {
        const EXECUTABLE          = 1 << 0;
        const POSITION_INDEPENDENT = 1 << 1;
        const HAS_RELOCATIONS     = 1 << 2;
        const HAS_DEBUG           = 1 << 3;
        const HAS_JIT_INFO        = 1 << 4;
    }
}
#[derive(Debug, Default, BinRead, BinWrite)]
#[brw(magic = b"HIVE", little)]
pub struct Header {
    pub version: u16,
    pub r#type: FileType,
    #[br(map = FileFlags::from_bits_retain)]
    #[bw(map = |flags: &FileFlags| flags.bits())]
    pub flags: FileFlags,
    pub file_size: u64,
    pub section_table: u64,
    pub section_count: u32,
    pub entry_point: u32,
    /// Header size
    size: u16,
    reserved: u16,
}
