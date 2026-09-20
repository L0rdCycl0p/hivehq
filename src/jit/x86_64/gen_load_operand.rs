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


use crate::jit::{
    opcode::{Operand, Register},
    x86_64::translate_opcode::encode_register,
};

pub fn gen_load_operand(code: &mut Vec<u8>, operand: &Operand, dst_reg: Register) {
    let dst = encode_register(&dst_reg);

    match operand {
        Operand::Register(src) => {
            if *src != dst_reg {
                let src = encode_register(src);

                // REX.W
                let mut rex = 0x48;

                if dst >= 8 {
                    rex |= 0x04; // REX.R
                }

                if src >= 8 {
                    rex |= 0x01; // REX.B
                }

                code.push(rex);

                // mov r64, r/m64
                code.push(0x8B);

                // mod = 11
                code.push(0xC0 | ((dst & 7) << 3) | (src & 7));
            }
        }
        Operand::Immediate(value) => {
            // REX.W
            let mut rex = 0x48;

            if dst >= 8 {
                rex |= 0x01; // REX.B
            }

            code.push(rex);

            // mov r64, imm64
            code.push(0xB8 + (dst & 7));

            code.extend_from_slice(&value.to_le_bytes());
        }

        Operand::Memory(memory) => {
            let base = memory.base.map(|x| encode_register(&x));
            let index = memory.index.map(|x| encode_register(&x));

            let displacement = memory.displacement;

            let scale = match memory.scale {
                1 => 0,
                2 => 1,
                4 => 2,
                8 => 3,
                _ => panic!("invalid x86-64 scale"),
            };

            let has_base = base.is_some();
            let has_index = index.is_some();

            let base_code = base.unwrap_or(5);
            let index_code = index.unwrap_or(4);

            let needs_sib = has_index || (base_code & 7) == 4;

            let mod_bits = if !has_base {
                0b00
            } else if displacement == 0 && (base_code & 7) != 5 {
                0b00
            } else if (-128..=127).contains(&displacement) {
                0b01
            } else {
                0b10
            };

            // REX.W
            let mut rex = 0x48;

            if dst >= 8 {
                rex |= 0x04; // REX.R
            }

            if has_index && index_code >= 8 {
                rex |= 0x02; // REX.X
            }

            if has_base && base_code >= 8 {
                rex |= 0x01; // REX.B
            }

            code.push(rex);

            // MOV r64, r/m64
            code.push(0x8B);

            let rm = if needs_sib { 4 } else { base_code & 7 };

            code.push((mod_bits << 6) | ((dst & 7) << 3) | rm);

            if needs_sib {
                code.push((scale << 6) | ((index_code & 7) << 3) | (base_code & 7));
            }

            if !has_base {
                // [index * scale + disp32]
                code.extend_from_slice(&displacement.to_le_bytes());
            } else if mod_bits == 0b01 {
                code.push(displacement as i8 as u8);
            } else if mod_bits == 0b10 {
                code.extend_from_slice(&displacement.to_le_bytes());
            }
        }

        Operand::Variable(variable) => {
            todo!("load variable {}", variable);
        }

        Operand::FrameOffset(offset) => {
            let displacement = *offset;

            let mod_bits = if (-128..=127).contains(&displacement) {
                0b01
            } else {
                0b10
            };

            // REX.W
            let mut rex = 0x48;

            if dst >= 8 {
                rex |= 0x04;
            }

            code.push(rex);

            // mov dst, [rsp + offset]
            code.push(0x8B);

            // r/m = 100 -> SIB
            code.push((mod_bits << 6) | ((dst & 7) << 3) | 0b100);

            // scale=1, index=none, base=rsp
            code.push(0x24);

            if mod_bits == 0b01 {
                code.push(displacement as i8 as u8);
            } else {
                code.extend_from_slice(&displacement.to_le_bytes());
            }
        }
    }
}
