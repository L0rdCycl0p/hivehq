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

use crate::parser::errors::LoadError;
use binrw::{BinRead, BinWrite};
use bitflags::bitflags;
use std::io::{Read, Seek};

#[derive(Debug, BinRead, BinWrite)]
#[brw(little, repr=u32)]
pub enum SectionType {
    Metadata = 1,
    Strings = 2,
    Symbols = 3,
    Types = 4,
    Atoms = 5,
    Functions = 6,
    Workers = 7,
    Code = 8,
    Data = 9,
    Relocations = 10,
    Debug = 11,
    JitInfo = 12,
}

bitflags! {
    #[derive(Debug)]
    pub struct SectionFlags: u32 {
        const READ      = 1 << 0;
        const WRITE     = 1 << 1;
        const EXEC      = 1 << 2;
        const ALLOC     = 1 << 3;
        const OPTIONAL  = 1 << 4;
    }
}

#[derive(Debug, BinRead, BinWrite)]
#[brw(little)]
pub struct SectionDescriptor {
    pub type_: SectionType,
    #[br(map = SectionFlags::from_bits_retain)]
    #[bw(map = |flags: &SectionFlags| flags.bits())]
    pub flags: SectionFlags,
    pub offset: u64,
    pub size: u64,
    pub alignment: u32,
    pub reserved: u32,
}
pub type SectionTable = Box<[SectionDescriptor]>;
#[allow(clippy::indexing_slicing)]
pub fn decode_section_table<S>(
    _source: &mut S,
    _section_table_offset: u64,
    _section_count: u32,
) -> Result<SectionTable, LoadError>
where
    S: Read + Seek,
{
    todo!()
}
