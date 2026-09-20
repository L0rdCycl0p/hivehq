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


use std::env::args;

use hivehq::{
    error::HiveError,
    runtime::runner::run_hive_data_stream,
};

fn main() -> Result<(), HiveError> {
    let mut args = args();
    args.next();
    let file = args.next().unwrap();
    let path = std::path::PathBuf::from(file);
    run_hive_data_stream(std::fs::File::open(path)?)
}
