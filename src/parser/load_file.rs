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
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
};

use binrw::BinRead as _;

use crate::{
    error::HiveError,
    parser::{
        ParsedFile,
        function_table::{FunctionDescriptor, FunctionTable},
        header,
        section_table::{self, SectionType},
        strings::StringTable,
    },
    runtime::load_exec::FunctionId,
};

#[inline]
pub fn load_file(file_path: PathBuf) -> Result<LoadedFile<std::fs::File>, HiveError> {
    let f = std::fs::File::open(file_path)?;
    load_file_by_data(f)
}
#[inline]
pub fn load_file_by_data<S: Read + Seek>(source: S) -> Result<LoadedFile<S>, HiveError> {
    ParsedFile::parse(source)
}

#[derive(Debug)]
pub struct LoadedFile<S: Read + Seek> {
    pub source: Option<S>,
    pub header: header::Header,
    pub sections: Vec<section_table::SectionDescriptor>,

    pub strings: Box<[String]>,
    pub functions: Box<[FunctionDescriptor]>,
}

impl<S: Read + Seek> Default for LoadedFile<S> {
    fn default() -> Self {
        Self {
            source: None,
            header: Default::default(),
            sections: Default::default(),
            strings: Default::default(),
            functions: Default::default(),
        }
    }
}
#[derive(Default, Debug)]
pub struct CodeSectionMapping {
    pub offset: u64,
    pub start: u64,
    pub end: u64,
}
const CODE_SECTION_MAPPING_DEFAULT: CodeSectionMapping = CodeSectionMapping {
    offset: 0,
    start: 0,
    end: 0,
};
impl ParsedFile {
    pub fn parse<S>(mut source: S) -> Result<LoadedFile<S>, HiveError>
    where
        S: Read + Seek,
    {
        let parsed = Self::read(&mut source)?;

        let mut strings = Vec::new();
        let mut functions = Vec::new();
        let mut code_section_mapping = Vec::new();
        for section in &parsed.section_table {
            match section.type_ {
                SectionType::Strings => {
                    source.seek(SeekFrom::Start(section.offset));
                    let section_strings = StringTable::read_args(&mut source, (section.size,))?;

                    strings.append(
                        &mut section_strings
                            .strings
                            .iter()
                            .map(|s| s.content.to_string())
                            .collect(),
                    );
                }
                SectionType::Functions => {
                    source.seek(SeekFrom::Start(section.offset));
                    let mut section_functions =
                        FunctionTable::read_args(&mut source, (section.size,))?;

                    functions.append(&mut section_functions.functions);
                }

                SectionType::Code => {
                    let last = code_section_mapping
                        .last()
                        .unwrap_or(&CODE_SECTION_MAPPING_DEFAULT);
                    code_section_mapping.push(CodeSectionMapping {
                        offset: section.offset,
                        start: last.end,
                        end: last.end + section.size - 1,
                    });
                }
                _ => {}
            }
        }
        let mut loaded_file = LoadedFile {
            source: Some(source),
            header: parsed.header,
            sections: parsed.section_table,
            functions: functions.into(),
            strings: strings.into(),
        };

        for (i, f) in loaded_file.functions.iter_mut().enumerate() {
            let f_start = f.code_offset; // 0
            let f_end = f_start + u64::from(f.code_size - 1); // 15
            let mut patched = false;
            for m in &code_section_mapping {
                if f_start >= m.start {
                    if f_end <= m.end {
                        f.code_offset = m.offset + (f_start - m.start);
                        patched = true;
                        break;
                    }
                    return Err(HiveError::FunctionIsInMultipleCodeChunk(FunctionId(
                        i as u32,
                    )));
                }
            }
            if !patched {
                return Err(HiveError::FunctionDoesNotFitInAnyCodeChunk(FunctionId(
                    i as u32,
                )));
            }
        }
        Ok(loaded_file)
    }
}
