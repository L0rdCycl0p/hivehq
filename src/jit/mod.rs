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


pub mod opcode;
pub mod parser;
#[cfg(feature = "x86_64")]
pub mod x86_64;
#[inline]
#[must_use]
pub fn jit_function(code: Box<[u8]>) -> Box<[u8]> {
    let parsed = parser::parse(code).unwrap();
    gen_code(&parsed).0
}

cfg_select! {
    feature = "x86_64" => {

        fn gen_code(code: &[opcode::Opcode]) -> (Box<[u8]>, Box<[x86_64::ContextSwitchPatch]>) {
            x86_64::jit_x86_64(code)
        }
    }
}
