/*
 * @file            src/jit/x86_64/mod.rs
 * @description
 * @author          TrollMii <trollmii@proton.me>
 * @createTime      2026-09-20 15:48:29
 * @lastModified    2026-09-30 16:02:45
 * Copyright ©L0rdCycl0p
                    HIVE (Hive Is Very Efficient)
Copyright (C) 2026 L0rdCycl0p

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>.

*/

use crate::jit::{
    opcode::{self},
    x86_64::translate_opcode::{TranslatedOpcode, translate_opcode},
};
mod build_instruction;
mod gen_context_switch;
mod gen_load_operand;
mod gen_sched_req;
mod translate_opcode;
pub use gen_context_switch::ContextSwitchPatch;
#[must_use]
pub fn jit_x86_64(code: &[opcode::Opcode]) -> Box<[u8]> {
    let mut offset = 0;
    let mut jit_code = Vec::with_capacity(50);
    for opc in code {
        let TranslatedOpcode(c, o) = translate_opcode(opc, offset);
        offset += o;
        jit_code.extend(c);
    }
    jit_code.into_boxed_slice()
}
