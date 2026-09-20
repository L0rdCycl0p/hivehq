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

use binrw::{BinRead, BinWrite};


pub mod errors;
pub mod function_table;
pub mod header;
pub mod load_file;
pub mod section_table;
pub mod strings;
pub mod symbol_table;

pub use load_file::{LoadedFile, load_file, load_file_by_data};

#[derive(Debug, BinRead, BinWrite)]
#[brw(little)]
pub struct ParsedFile {
    pub header: header::Header,

    #[br(
        count = header.section_count,
        seek_before = SeekFrom::Start(header.section_table)
    )]
    pub section_table: Vec<section_table::SectionDescriptor>,
}
