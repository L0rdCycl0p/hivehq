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

use std::{array::TryFromSliceError, io, str::Utf8Error};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadError {
    #[error("Invalid Magic, expected [0x48, 0x49, 0x56, 0x45] (HIVE)")]
    InvalidMagic,
    #[error("Invalid Header Size")]
    InvalidHeaderSize(u16),
    #[error("Invalid File Format Version In Header: `{0}`")]
    InvalidVersion(u16),
    #[error("Invalid File Type In Header: `{0}`")]
    InvalidFileType(u16),
    #[error("Invalid Section Type: `{0}`")]
    InvalidSectionType(u32),
    #[error("Invalid Symbol Type: `{0}`")]
    InvalidSymbolType(u16),
    #[error("Invalid Symbol Bindings: `{0}`")]
    InvalidSymbolBindings(u16),
    #[error("IO Error: {0}")]
    IOError(#[from] io::Error),
    #[error("TryFromSliceError: {0}")]
    TryFromSliceError(#[from] TryFromSliceError),
    #[error("UTF8Error: {0}")]
    UTF8Error(#[from] Utf8Error),

    #[error("Bad Flags")]
    BadFlags,

    #[error("Reserved Bytes aren't null (e. g. [0, 0, 0, 0])")]
    ReservedBytesNotNull,

    #[error("Bad Alignment")]
    BadAlignment,

    #[error("Bad Null String")]
    BadNullString,
}
