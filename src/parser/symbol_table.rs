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


use std::io::{Read, Seek, SeekFrom};

use crate::parser::{errors::LoadError, section_table::SectionDescriptor};
#[repr(u16)]
#[derive(Debug)]
pub enum SymbolType {
    Function = 1,
    Worker = 2,
    Data = 3,
    Atom = 4,
    Type = 5,
    External = 6,
}
impl TryFrom<u16> for SymbolType {
    type Error = LoadError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Function),
            2 => Ok(Self::Worker),
            3 => Ok(Self::Data),
            4 => Ok(Self::Atom),
            5 => Ok(Self::Type),
            6 => Ok(Self::External),
            _ => Err(LoadError::InvalidSymbolType(value)),
        }
    }
}

#[repr(u16)]
#[derive(Debug)]
pub enum SymbolBindings {
    Local = 1,
    Global = 2,
    Weak = 3,
    Import = 4,
    Export = 5,
}
impl TryFrom<u16> for SymbolBindings {
    type Error = LoadError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Local),
            2 => Ok(Self::Global),
            3 => Ok(Self::Weak),
            4 => Ok(Self::Import),
            5 => Ok(Self::Export),
            _ => Err(LoadError::InvalidSymbolBindings(value)),
        }
    }
}

#[derive(Debug)]
pub struct SymbolDescriptor {
    pub name_string_id: u32,
    pub value: u64,
    pub size: u32,
    pub r#type: SymbolType,
    pub bindings: SymbolBindings,
}
pub type SymbolTable = Box<[SymbolDescriptor]>;

pub fn decode_symbol_table<S>(
    source: &mut S,
    section_entry: &SectionDescriptor,
) -> Result<SymbolTable, LoadError>
where
    S: Read + Seek,
{
    let size = section_entry.size;
    let symbol_count = (size as usize) / 24;
    source.seek(SeekFrom::Start(section_entry.offset))?;

    let mut bytes = vec![0_u8; size as usize];
    bytes.len();
    source.read_exact(&mut bytes)?;
    let mut symbol_table_vec: Vec<SymbolDescriptor> = Vec::with_capacity(symbol_count);
    let mut offset = 0;
    #[allow(clippy::indexing_slicing)]
    for _i in 0..symbol_count {
        let name: [u8; 4] = bytes[offset..offset + 4].try_into()?;
        let value: [u8; 8] = bytes[offset + 4..offset + 12].try_into()?;
        let size: [u8; 4] = bytes[offset + 12..offset + 16].try_into()?;
        let r#type: [u8; 2] = bytes[offset + 16..offset + 18].try_into()?;
        let bindings: [u8; 2] = bytes[offset + 18..offset + 20].try_into()?;

        let reserved: [u8; 4] = bytes[offset + 20..offset + 24].try_into()?;
        if reserved != [0_u8; 4] {
            return Err(LoadError::BadAlignment);
        }

        let name = u32::from_le_bytes(name);
        let value = u64::from_le_bytes(value);
        let size = u32::from_le_bytes(size);
        let r#type = u16::from_le_bytes(r#type);
        let bindings = u16::from_le_bytes(bindings);

        let r#type = SymbolType::try_from(r#type)?;
        let bindings = SymbolBindings::try_from(bindings)?;

        let symbol_descriptor = SymbolDescriptor {
            name_string_id: name,
            value,
            size,
            r#type,
            bindings,
        };
        symbol_table_vec.push(symbol_descriptor);
        offset += 24;
    }
    Ok(symbol_table_vec.into())
}
