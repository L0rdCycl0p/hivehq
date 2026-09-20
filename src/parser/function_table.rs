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
#[repr(u16)]
#[derive(Debug)]
pub enum FunctionType {
    Function = 1,
    Worker = 2,
    Data = 3,
    Atom = 4,
    Type = 5,
    External = 6,
}

bitflags! {
    #[derive(Debug)]
    pub struct FunctionFlags: u32 {
        const ENTRY         = 1 << 0;
        const EXPORTED      = 1 << 1;
        const IMPORTED      = 1 << 2;
        const VARIADIC      = 1 << 3;
        const NO_RETURN     = 1 << 4;
    }
}

#[derive(Debug, BinRead, BinWrite)]
pub struct FunctionDescriptor {
    pub symbol: u32,
    pub code_offset: u64,
    pub code_size: u32,
    pub frame_size: u32,
    #[br(map = FunctionFlags::from_bits_retain)]
    #[bw(map = |flags: &FunctionFlags| flags.bits())]
    pub flags: FunctionFlags,
    pub param_type_id: u32,
    pub return_type_id: u32,
}
#[derive(Debug, BinRead, BinWrite)]
#[brw(little)]
#[brw(import(size: u64))]
pub struct FunctionTable {
    #[br(count=size/32)]
    pub functions: Vec<FunctionDescriptor>,
}
