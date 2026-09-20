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

#[derive(Debug, Clone, Copy)]
pub enum InstructionEncoding {
    /// opcode /r
    Rm { opcode: u8, w: bool },

    /// opcode /digit, r/m
    Group { opcode: u8, group: u8, w: bool },

    /// opcode /digit ib/id
    GroupImmediate {
        opcode8: u8,
        opcode32: u8,
        group: u8,
        w: bool,
    },

    /// opcode r/m, imm
    RmImmediate {
        opcode8: u8,
        opcode32: u8,
        group: u8,
        w: bool,
    },

    /// 0F opcode /r
    Rm0F { opcode: u8, w: bool },

    /// 0F opcode /r, used for 16/32/64-bit destinations
    Rm0FImmediate { opcode: u8, w: bool },

    /// mov reg, imm
    MovImmediate { w: bool },
}

pub fn build_instruction(encoding: InstructionEncoding, operands: &[&Operand]) -> Vec<u8> {
    let mut code = Vec::new();

    fn register_value(operand: &Operand) -> u8 {
        match operand {
            Operand::Register(register) => encode_register(register),

            _ => panic!("expected register operand"),
        }
    }

    fn rex_prefix(w: bool, r: bool, x: bool, b: bool) -> Option<u8> {
        let mut rex = 0x40;

        if w {
            rex |= 0x08;
        }

        if r {
            rex |= 0x04;
        }

        if x {
            rex |= 0x02;
        }

        if b {
            rex |= 0x01;
        }

        (rex != 0x40).then_some(rex)
    }

    fn emit_modrm_reg_rm(code: &mut Vec<u8>, reg: u8, operand: &Operand, w: bool, opcode: &[u8]) {
        match operand {
            Operand::Register(register) => {
                let rm = encode_register(register);

                if let Some(rex) = rex_prefix(w, reg >= 8, false, rm >= 8) {
                    code.push(rex);
                }

                code.extend_from_slice(opcode);

                code.push(0xC0 | ((reg & 7) << 3) | (rm & 7));
            }

            Operand::Memory(memory) => {
                let base = memory.base.as_ref().map(encode_register);

                let index = memory.index.as_ref().map(encode_register);

                let displacement = memory.displacement;

                let scale = match memory.scale {
                    1 => 0b00,
                    2 => 0b01,
                    4 => 0b10,
                    8 => 0b11,
                    _ => panic!("invalid scale"),
                };

                if let Some(index) = index {
                    assert!(index & 7 != 4, "RSP cannot be used as SIB index");
                }

                let needs_sib =
                    index.is_some() || base.is_none() || base.is_some_and(|base| (base & 7) == 4);

                let rex_r = reg >= 8;
                let rex_x = index.is_some_and(|index| index >= 8);
                let rex_b = base.is_some_and(|base| base >= 8);

                if let Some(rex) = rex_prefix(w, rex_r, rex_x, rex_b) {
                    code.push(rex);
                }

                code.extend_from_slice(opcode);

                let (mod_bits, rm_bits) = if needs_sib {
                    if displacement == 0 && base.is_some() && (base.unwrap() & 7) != 5 {
                        (0b00, 0b100)
                    } else if (-128..=127).contains(&displacement) {
                        (0b01, 0b100)
                    } else {
                        (0b10, 0b100)
                    }
                } else {
                    let base = base.unwrap();

                    if displacement == 0 && (base & 7) != 5 {
                        (0b00, base & 7)
                    } else if (-128..=127).contains(&displacement) {
                        (0b01, base & 7)
                    } else {
                        (0b10, base & 7)
                    }
                };

                code.push((mod_bits << 6) | ((reg & 7) << 3) | rm_bits);

                if needs_sib {
                    let sib_index = index.map_or(4, |index| index & 7);

                    let sib_base = base.map_or(5, |base| base & 7);

                    code.push((scale << 6) | (sib_index << 3) | sib_base);
                }

                match mod_bits {
                    0b01 => {
                        code.push(displacement as i8 as u8);
                    }

                    0b10 => {
                        code.extend_from_slice(&displacement.to_le_bytes());
                    }

                    0b00 if base.is_none() => {
                        code.extend_from_slice(&displacement.to_le_bytes());
                    }

                    _ => {}
                }
            }

            Operand::Immediate(_) => {
                panic!("expected register or memory operand");
            }

            Operand::Variable(_) => {
                todo!()
            }

            Operand::FrameOffset(_) => {
                todo!()
            }
        }
    }

    match encoding {
        InstructionEncoding::Rm { opcode, w } => {
            assert_eq!(operands.len(), 2, "Rm expects 2 operands");

            let reg = register_value(operands[0]);

            emit_modrm_reg_rm(&mut code, reg, operands[1], w, &[opcode]);
        }

        InstructionEncoding::Group { opcode, group, w } => {
            assert_eq!(operands.len(), 1, "Group expects 1 operand");

            let operand = &operands[0];

            match operand {
                Operand::Register(register) => {
                    let rm = encode_register(register);

                    if let Some(rex) = rex_prefix(w, false, false, rm >= 8) {
                        code.push(rex);
                    }

                    code.push(opcode);

                    code.push(0xC0 | ((group & 7) << 3) | (rm & 7));
                }

                Operand::Memory(_) => {
                    let fake_reg = Register::Rax;

                    emit_modrm_reg_rm(&mut code, group, operand, w, &[opcode]);

                    let _ = fake_reg;
                }

                _ => panic!("Group requires register or memory"),
            }
        }

        InstructionEncoding::GroupImmediate {
            opcode8,
            opcode32,
            group,
            w,
        } => {
            assert_eq!(operands.len(), 2, "GroupImmediate expects 2 operands");

            let destination = &operands[0];
            let immediate = &operands[1];

            let value = match immediate {
                Operand::Immediate(value) => *value,

                _ => panic!("expected immediate operand"),
            };

            match destination {
                Operand::Register(register) => {
                    let rm = encode_register(register);

                    if let Some(rex) = rex_prefix(w, false, false, rm >= 8) {
                        code.push(rex);
                    }

                    if (-128..=127).contains(&value) {
                        code.push(opcode8);

                        code.push(0xC0 | ((group & 7) << 3) | (rm & 7));

                        code.push(value as i8 as u8);
                    } else {
                        code.push(opcode32);

                        code.push(0xC0 | ((group & 7) << 3) | (rm & 7));

                        code.extend_from_slice(&(value as i32).to_le_bytes());
                    }
                }

                Operand::Memory(_) => {
                    // Hier brauchen wir denselben
                    // ModRM/SIB-Weg wie bei Rm,
                    // aber mit `group` als reg field.
                    //
                    // Deshalb zunächst den passenden
                    // ModRM-Teil erzeugen.
                    let opcode = if (-128..=127).contains(&value) {
                        opcode8
                    } else {
                        opcode32
                    };

                    emit_modrm_reg_rm(&mut code, group, destination, w, &[opcode]);

                    if (-128..=127).contains(&value) {
                        code.push(value as i8 as u8);
                    } else {
                        code.extend_from_slice(&(value as i32).to_le_bytes());
                    }
                }

                _ => panic!("invalid immediate destination"),
            }
        }

        InstructionEncoding::RmImmediate {
            opcode8,
            opcode32,
            group,
            w,
        } => {
            assert_eq!(operands.len(), 2, "RmImmediate expects 2 operands");

            let destination = &operands[0];

            let value = match &operands[1] {
                Operand::Immediate(value) => *value,

                _ => panic!("expected immediate operand"),
            };

            let opcode = if (-128..=127).contains(&value) {
                opcode8
            } else {
                opcode32
            };

            emit_modrm_reg_rm(&mut code, group, destination, w, &[opcode]);

            if (-128..=127).contains(&value) {
                code.push(value as i8 as u8);
            } else {
                code.extend_from_slice(&(value as i32).to_le_bytes());
            }
        }

        InstructionEncoding::Rm0F { opcode, w } => {
            assert_eq!(operands.len(), 2, "Rm0F expects 2 operands");

            let reg = register_value(operands[0]);

            emit_modrm_reg_rm(&mut code, reg, operands[1], w, &[0x0F, opcode]);
        }

        InstructionEncoding::Rm0FImmediate { opcode, w } => {
            assert_eq!(operands.len(), 3, "Rm0FImmediate expects 3 operands");

            let dst = register_value(operands[0]);

            let src = &operands[1];

            let immediate = match &operands[2] {
                Operand::Immediate(value) => *value,

                _ => panic!("expected immediate operand"),
            };

            let rm = register_value(src);

            if let Some(rex) = rex_prefix(w, dst >= 8, false, rm >= 8) {
                code.push(rex);
            }

            code.push(0x0F);
            code.push(opcode);

            code.push(0xC0 | ((dst & 7) << 3) | (rm & 7));

            code.push(immediate as u8);
        }

        InstructionEncoding::MovImmediate { w } => {
            assert_eq!(operands.len(), 2, "MovImmediate expects 2 operands");

            let dst = register_value(operands[0]);

            let value = match &operands[1] {
                Operand::Immediate(value) => *value,

                _ => panic!("expected immediate operand"),
            };

            let mut rex = 0x40;

            if w {
                rex |= 0x08;
            }

            if dst >= 8 {
                rex |= 0x01;
            }

            if rex != 0x40 {
                code.push(rex);
            }

            code.push(0xB8 | (dst & 7));

            if w {
                code.extend_from_slice(&(value as u64).to_le_bytes());
            } else {
                code.extend_from_slice(&(value as u32).to_le_bytes());
            }
        }
    }

    code
}
