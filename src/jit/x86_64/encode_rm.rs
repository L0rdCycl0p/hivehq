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

pub fn encode_rm(opcode: u8, reg: &Register, operand: &Operand, w: bool) -> Vec<u8> {
    let mut code = Vec::new();

    let reg = encode_register(reg);

    match operand {
        Operand::Register(operand) => {
            let rm = encode_register(operand);

            let mut rex = 0x40;

            if w {
                rex |= 0x08;
            }

            if reg >= 8 {
                rex |= 0x04;
            }

            if rm >= 8 {
                rex |= 0x01;
            }

            if rex != 0x40 {
                code.push(rex);
            }

            code.push(opcode);

            code.push(0xC0 | ((reg & 7) << 3) | (rm & 7));
        }

        Operand::Memory(memory) => {
            let base = memory.base.as_ref().map(encode_register);
            let index = memory.index.as_ref().map(encode_register);

            let scale = match memory.scale {
                1 => 0b00,
                2 => 0b01,
                4 => 0b10,
                8 => 0b11,
                _ => panic!("invalid scale"),
            };

            let mut rex = 0x40;

            if w {
                rex |= 0x08;
            }

            if reg >= 8 {
                rex |= 0x04;
            }

            if index.is_some_and(|x| x >= 8) {
                rex |= 0x02;
            }

            if base.is_some_and(|x| x >= 8) {
                rex |= 0x01;
            }

            if rex != 0x40 {
                code.push(rex);
            }

            code.push(opcode);

            let displacement = memory.displacement;

            match (base, index, displacement) {
                // [base]
                (Some(base), None, 0) if (base & 7) != 5 => {
                    code.push(((reg & 7) << 3) | (base & 7));
                }

                // [base + disp8]
                (Some(base), None, disp) if (-128..=127).contains(&disp) && (base & 7) != 5 => {
                    code.push(0x40 | ((reg & 7) << 3) | (base & 7));

                    code.push(disp as i8 as u8);
                }

                // [base + disp32]
                (Some(base), None, disp) => {
                    code.push(0x80 | ((reg & 7) << 3) | (base & 7));

                    code.extend_from_slice(&disp.to_le_bytes());
                }

                // [base + index * scale]
                (Some(base), Some(index), 0) => {
                    code.push(((reg & 7) << 3) | 0b100);

                    code.push((scale << 6) | ((index & 7) << 3) | (base & 7));
                }

                // [base + index * scale + disp8]
                (Some(base), Some(index), disp) if (-128..=127).contains(&disp) => {
                    code.push(0x40 | ((reg & 7) << 3) | 0b100);

                    code.push((scale << 6) | ((index & 7) << 3) | (base & 7));

                    code.push(disp as i8 as u8);
                }

                // [base + index * scale + disp32]
                (Some(base), Some(index), disp) => {
                    code.push(0x80 | ((reg & 7) << 3) | 0b100);

                    code.push((scale << 6) | ((index & 7) << 3) | (base & 7));

                    code.extend_from_slice(&disp.to_le_bytes());
                }

                // [index * scale + disp32]
                (None, Some(index), disp) => {
                    code.push(0x04 | ((reg & 7) << 3));

                    code.push((scale << 6) | ((index & 7) << 3) | 0b101);

                    code.extend_from_slice(&disp.to_le_bytes());
                }

                // [disp32]
                (None, None, disp) => {
                    code.push(0x04 | ((reg & 7) << 3));

                    code.push(0x25);

                    code.extend_from_slice(&disp.to_le_bytes());
                }
            }
        }

        Operand::Immediate(_) => {
            panic!("encode_rm cannot encode immediate operands")
        }

        // Hive-specific operands
        Operand::Variable(_) => todo!(),
        Operand::FrameOffset(_) => todo!(),
    }

    code
}
