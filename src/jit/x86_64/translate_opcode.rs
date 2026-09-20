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
    opcode::{MemoryOperand, Opcode, Operand, Register, ShiftAmount},
    x86_64::{
        build_instruction::{InstructionEncoding, build_instruction},
        gen_context_switch::ContextSwitchPatch,
        gen_sched_req::gen_sched_req,
    },
};

pub const fn encode_register(register: &Register) -> u8 {
    match register {
        Register::Rax => 0,
        Register::Rcx => 1,
        Register::Rdx => 2,
        Register::Rbx => 3,
        Register::Rsp => 4,
        Register::Rbp => 5,
        Register::Rsi => 6,
        Register::Rdi => 7,
        Register::R8 => 8,
        Register::R9 => 9,
        Register::R10 => 10,
        Register::R11 => 11,
        Register::R12 => 12,
        Register::R13 => 13,
        Register::R14 => 14,
        Register::R15 => 15,
    }
}
pub struct TranslatedOpcode(pub Vec<u8>, pub usize, pub Option<ContextSwitchPatch>);
impl From<(Vec<u8>, usize)> for TranslatedOpcode {
    fn from(value: (Vec<u8>, usize)) -> Self {
        Self(value.0, value.1, None)
    }
}

pub fn translate_opcode(opcode: &Opcode, offset: usize) -> TranslatedOpcode {
    match opcode {
        Opcode::Nop => (vec![0x90], offset + 1).into(),

        Opcode::Mov8 { dst, src } => {
            match (dst, src) {
                // mov r8, r/m8
                (Operand::Register(dst), Operand::Register(src)) => {
                    let dst = encode_register(dst);
                    let src = encode_register(src);

                    let mut code = Vec::with_capacity(3);

                    let mut rex = 0x40;

                    if dst >= 8 {
                        rex |= 0x04;
                    }

                    if src >= 8 {
                        rex |= 0x01;
                    }

                    // REX is only needed for R8-R15.
                    if rex != 0x40 {
                        code.push(rex);
                    }

                    code.push(0x8A);

                    code.push(0xC0 | ((dst & 7) << 3) | (src & 7));

                    let len = code.len();
                    (code, offset + len).into()
                }

                _ => todo!(),
            }
        }

        Opcode::MovImm64 { dst, value } => {
            let dst = encode_register(dst);

            let mut code = Vec::with_capacity(10);

            code.push(0x48 | u8::from(dst >= 8));

            code.push(0xB8 + (dst & 7));

            code.extend_from_slice(&value.to_le_bytes());

            (code, offset + 10).into()
        }

        Opcode::Lea { dst, src } => {
            let dst = encode_register(dst);

            let MemoryOperand {
                base,
                index,
                scale: _,
                displacement,
            } = src;

            let base = base.map(|x| encode_register(&x));
            let index = index.map(|x| encode_register(&x));

            let mut code = Vec::new();

            let mut rex = 0x48;

            if dst >= 8 {
                rex |= 0x04;
            }

            if let Some(index) = index
                && index >= 8
            {
                rex |= 0x02;
            }

            if let Some(base) = base
                && base >= 8
            {
                rex |= 0x01;
            }

            code.push(rex);
            code.push(0x8D);

            match (base, index, displacement) {
                (Some(base), None, 0) if (base & 7) != 4 && (base & 7) != 5 => {
                    code.push(((dst & 7) << 3) | (base & 7));
                }

                _ => todo!(),
            }

            let len = code.len();
            (code, offset + len).into()
        }

        Opcode::Mov64 { dst, src } => {
            let code = build_instruction(
                InstructionEncoding::Rm {
                    opcode: 0x8B,
                    w: true,
                },
                &[dst, src],
            );

            let len = code.len();
            (code, offset + len).into()
        }

        Opcode::Add { dst, src } => {
            let encoding = match (dst, src) {
                (Operand::Register(_), Operand::Register(_) | Operand::Memory(_)) => {
                    InstructionEncoding::Rm {
                        opcode: 0x03, // ADD r64, r/m64
                        w: true,
                    }
                }

                (Operand::Memory(_), Operand::Register(_)) => {
                    InstructionEncoding::Rm {
                        opcode: 0x01, // ADD r/m64, r64
                        w: true,
                    }
                }

                (Operand::Register(_) | Operand::Memory(_), Operand::Immediate(_)) => {
                    InstructionEncoding::GroupImmediate {
                        opcode8: 0x83,
                        opcode32: 0x81,
                        group: 0,
                        w: true,
                    }
                }

                _ => todo!(),
            };

            let code = build_instruction(encoding, &[dst, src]);

            let len = code.len();
            (code, offset + len).into()
        }

        Opcode::Sub { dst, src } => {
            let encoding = match (dst, src) {
                (Operand::Register(_), Operand::Register(_) | Operand::Memory(_)) => {
                    InstructionEncoding::Rm {
                        opcode: 0x2B, // SUB r64, r/m64
                        w: true,
                    }
                }

                (Operand::Memory(_), Operand::Register(_)) => {
                    InstructionEncoding::Rm {
                        opcode: 0x29, // SUB r/m64, r64
                        w: true,
                    }
                }

                (Operand::Register(_) | Operand::Memory(_), Operand::Immediate(_)) => {
                    InstructionEncoding::GroupImmediate {
                        opcode8: 0x83,  // /5 ib
                        opcode32: 0x81, // /5 id
                        group: 5,
                        w: true,
                    }
                }

                _ => todo!(),
            };

            let code = build_instruction(encoding, &[dst, src]);

            let len = code.len();
            (code, offset + len).into()
        }
        Opcode::And { dst, src } => {
            let encoding = match (dst, src) {
                (Operand::Register(_), Operand::Register(_) | Operand::Memory(_)) => {
                    InstructionEncoding::Rm {
                        opcode: 0x23, // AND r64, r/m64
                        w: true,
                    }
                }

                (Operand::Memory(_), Operand::Register(_)) => {
                    InstructionEncoding::Rm {
                        opcode: 0x21, // AND r/m64, r64
                        w: true,
                    }
                }

                (Operand::Register(_) | Operand::Memory(_), Operand::Immediate(_)) => {
                    InstructionEncoding::GroupImmediate {
                        opcode8: 0x83,
                        opcode32: 0x81,
                        group: 4,
                        w: true,
                    }
                }

                _ => todo!(),
            };

            let code = build_instruction(encoding, &[dst, src]);

            let len = code.len();
            (code, offset + len).into()
        }

        Opcode::Or { dst, src } => {
            let encoding = match (dst, src) {
                (Operand::Register(_), Operand::Register(_) | Operand::Memory(_)) => {
                    InstructionEncoding::Rm {
                        opcode: 0x0B, // OR r64, r/m64
                        w: true,
                    }
                }

                (Operand::Memory(_), Operand::Register(_)) => {
                    InstructionEncoding::Rm {
                        opcode: 0x09, // OR r/m64, r64
                        w: true,
                    }
                }

                (Operand::Register(_) | Operand::Memory(_), Operand::Immediate(_)) => {
                    InstructionEncoding::GroupImmediate {
                        opcode8: 0x83,
                        opcode32: 0x81,
                        group: 1,
                        w: true,
                    }
                }

                _ => todo!(),
            };

            let code = build_instruction(encoding, &[dst, src]);

            let len = code.len();
            (code, offset + len).into()
        }
        Opcode::Xor { dst, src } => {
            let encoding = match (dst, src) {
                (Operand::Register(_), Operand::Register(_) | Operand::Memory(_)) => {
                    InstructionEncoding::Rm {
                        opcode: 0x33, // XOR r64, r/m64
                        w: true,
                    }
                }

                (Operand::Memory(_), Operand::Register(_)) => {
                    InstructionEncoding::Rm {
                        opcode: 0x31, // XOR r/m64, r64
                        w: true,
                    }
                }

                (Operand::Register(_) | Operand::Memory(_), Operand::Immediate(_)) => {
                    InstructionEncoding::GroupImmediate {
                        opcode8: 0x83,
                        opcode32: 0x81,
                        group: 6,
                        w: true,
                    }
                }

                _ => todo!(),
            };

            let code = build_instruction(encoding, &[dst, src]);

            let len = code.len();
            (code, offset + len).into()
        }

        Opcode::Test { left, right } => {
            let encoding = match (left, right) {
                (Operand::Register(_), Operand::Register(_) | Operand::Memory(_)) => {
                    InstructionEncoding::Rm {
                        opcode: 0x85, // TEST r/m64, r64
                        w: true,
                    }
                }

                (Operand::Memory(_), Operand::Register(_)) => InstructionEncoding::Rm {
                    opcode: 0x85,
                    w: true,
                },

                _ => todo!(),
            };

            let code = build_instruction(encoding, &[left, right]);

            let len = code.len();
            (code, offset + len).into()
        }

        Opcode::Cmp { left, right } => {
            let encoding = match (left, right) {
                (Operand::Register(_), Operand::Register(_) | Operand::Memory(_)) => {
                    InstructionEncoding::Rm {
                        opcode: 0x3B, // CMP r64, r/m64
                        w: true,
                    }
                }

                (Operand::Memory(_), Operand::Register(_)) => {
                    InstructionEncoding::Rm {
                        opcode: 0x39, // CMP r/m64, r64
                        w: true,
                    }
                }

                (Operand::Register(_) | Operand::Memory(_), Operand::Immediate(_)) => {
                    InstructionEncoding::GroupImmediate {
                        opcode8: 0x83,
                        opcode32: 0x81,
                        group: 7,
                        w: true,
                    }
                }

                _ => todo!(),
            };

            let code = build_instruction(encoding, &[left, right]);

            let len = code.len();
            (code, offset + len).into()
        }
        Opcode::Mul { src } => match src {
            Operand::Register(src) => {
                let src = encode_register(src);

                let mut code = Vec::with_capacity(3);

                code.push(0x48 | u8::from(src >= 8));

                code.push(0xF7);

                code.push(0xE0 | (src & 7));

                (code, offset + 3).into()
            }

            _ => todo!(),
        },

        Opcode::Imul { src } => match src {
            Operand::Register(src) => {
                let src = encode_register(src);

                let mut code = Vec::with_capacity(3);

                code.push(0x48 | u8::from(src >= 8));

                code.push(0xF7);

                code.push(0xE8 | (src & 7));

                (code, offset + 3).into()
            }

            _ => todo!(),
        },

        Opcode::Div { src } => match src {
            Operand::Register(src) => {
                let src = encode_register(src);

                let mut code = Vec::with_capacity(3);

                code.push(0x48 | u8::from(src >= 8));

                code.push(0xF7);

                code.push(0xF0 | (src & 7));

                (code, offset + 3).into()
            }

            _ => todo!(),
        },

        Opcode::Idiv { src } => match src {
            Operand::Register(src) => {
                let src = encode_register(src);

                let mut code = Vec::with_capacity(3);

                code.push(0x48 | u8::from(src >= 8));

                code.push(0xF7);

                code.push(0xF8 | (src & 7));

                (code, offset + 3).into()
            }

            _ => todo!(),
        },

        Opcode::Inc { operand } => match operand {
            Operand::Register(reg) => {
                let reg = encode_register(reg);

                let mut code = Vec::with_capacity(3);

                code.push(0x48 | u8::from(reg >= 8));

                code.push(0xFF);
                code.push(0xC0 | (reg & 7));

                (code, offset + 3).into()
            }

            _ => todo!(),
        },

        Opcode::Dec { operand } => match operand {
            Operand::Register(reg) => {
                let reg = encode_register(reg);

                let mut code = Vec::with_capacity(3);

                code.push(0x48 | u8::from(reg >= 8));

                code.push(0xFF);
                code.push(0xC8 | (reg & 7));

                (code, offset + 3).into()
            }

            _ => todo!(),
        },

        Opcode::Neg { operand } => match operand {
            Operand::Register(reg) => {
                let reg = encode_register(reg);

                let mut code = Vec::with_capacity(3);

                code.push(0x48 | u8::from(reg >= 8));

                code.push(0xF7);
                code.push(0xD8 | (reg & 7));

                (code, offset + 3).into()
            }

            _ => todo!(),
        },

        Opcode::Not { operand } => match operand {
            Operand::Register(reg) => {
                let reg = encode_register(reg);

                let mut code = Vec::with_capacity(3);

                code.push(0x48 | u8::from(reg >= 8));

                code.push(0xF7);
                code.push(0xD0 | (reg & 7));

                (code, offset + 3).into()
            }

            _ => todo!(),
        },

        Opcode::Shl { dst, amount } => match dst {
            Operand::Register(reg) => {
                let reg = encode_register(reg);

                let mut code = Vec::new();

                code.push(0x48 | u8::from(reg >= 8));

                match amount {
                    ShiftAmount::Immediate(amount) => {
                        code.push(0xC1);
                        code.push(0xE0 | (reg & 7));
                        code.push(*amount);
                    }

                    ShiftAmount::Cl => {
                        code.push(0xD3);
                        code.push(0xE0 | (reg & 7));
                    }
                }

                let len = code.len();
                (code, offset + len).into()
            }

            _ => todo!(),
        },

        Opcode::Shr { dst, amount } => match dst {
            Operand::Register(reg) => {
                let reg = encode_register(reg);

                let mut code = Vec::new();

                code.push(0x48 | u8::from(reg >= 8));

                match amount {
                    ShiftAmount::Immediate(amount) => {
                        code.push(0xC1);
                        code.push(0xE8 | (reg & 7));
                        code.push(*amount);
                    }

                    ShiftAmount::Cl => {
                        code.push(0xD3);
                        code.push(0xE8 | (reg & 7));
                    }
                }

                let len = code.len();
                (code, offset + len).into()
            }

            _ => todo!(),
        },

        Opcode::Sar { dst, amount } => match dst {
            Operand::Register(reg) => {
                let reg = encode_register(reg);

                let mut code = Vec::new();

                code.push(0x48 | u8::from(reg >= 8));

                match amount {
                    ShiftAmount::Immediate(amount) => {
                        code.push(0xC1);
                        code.push(0xF8 | (reg & 7));
                        code.push(*amount);
                    }

                    ShiftAmount::Cl => {
                        code.push(0xD3);
                        code.push(0xF8 | (reg & 7));
                    }
                }

                let len = code.len();
                (code, offset + len).into()
            }

            _ => todo!(),
        },

        Opcode::Rol { dst, amount } => match dst {
            Operand::Register(reg) => {
                let reg = encode_register(reg);

                let mut code = Vec::new();

                code.push(0x48 | u8::from(reg >= 8));

                match amount {
                    ShiftAmount::Immediate(amount) => {
                        code.push(0xC1);
                        code.push(0xC0 | (reg & 7));
                        code.push(*amount);
                    }

                    ShiftAmount::Cl => {
                        code.push(0xD3);
                        code.push(0xC0 | (reg & 7));
                    }
                }

                let len = code.len();
                (code, offset + len).into()
            }

            _ => todo!(),
        },

        Opcode::Ror { dst, amount } => match dst {
            Operand::Register(reg) => {
                let reg = encode_register(reg);

                let mut code = Vec::new();

                code.push(0x48 | u8::from(reg >= 8));

                match amount {
                    ShiftAmount::Immediate(amount) => {
                        code.push(0xC1);
                        code.push(0xC8 | (reg & 7));
                        code.push(*amount);
                    }

                    ShiftAmount::Cl => {
                        code.push(0xD3);
                        code.push(0xC8 | (reg & 7));
                    }
                }

                let len = code.len();
                (code, offset + len).into()
            }

            _ => todo!(),
        },

        Opcode::Movsx8 { dst, src } => match src {
            Operand::Register(src) => {
                let dst = encode_register(dst);
                let src = encode_register(src);

                let mut code = Vec::with_capacity(4);

                code.push(0x48 | if dst >= 8 { 0x04 } else { 0 } | u8::from(src >= 8));

                code.push(0x0F);
                code.push(0xBE);

                code.push(0xC0 | ((dst & 7) << 3) | (src & 7));

                (code, offset + 4).into()
            }

            _ => todo!(),
        },

        Opcode::Movsx16 { dst, src } => match src {
            Operand::Register(src) => {
                let dst = encode_register(dst);
                let src = encode_register(src);

                let mut code = Vec::with_capacity(4);

                code.push(0x48 | if dst >= 8 { 0x04 } else { 0 } | u8::from(src >= 8));

                code.push(0x0F);
                code.push(0xBF);

                code.push(0xC0 | ((dst & 7) << 3) | (src & 7));

                (code, offset + 4).into()
            }

            _ => todo!(),
        },

        Opcode::Movzx8 { dst, src } => match src {
            Operand::Register(src) => {
                let dst = encode_register(dst);
                let src = encode_register(src);

                let mut code = Vec::with_capacity(4);

                if dst >= 8 || src >= 8 {
                    code.push(0x40 | if dst >= 8 { 0x04 } else { 0 } | u8::from(src >= 8));
                }

                code.push(0x0F);
                code.push(0xB6);

                code.push(0xC0 | ((dst & 7) << 3) | (src & 7));

                let len = code.len();
                (code, offset + len).into()
            }

            _ => todo!(),
        },

        Opcode::Movzx16 { dst, src } => match src {
            Operand::Register(src) => {
                let dst = encode_register(dst);
                let src = encode_register(src);

                let mut code = Vec::with_capacity(4);

                if dst >= 8 || src >= 8 {
                    code.push(0x40 | if dst >= 8 { 0x04 } else { 0 } | u8::from(src >= 8));
                }

                code.push(0x0F);
                code.push(0xB7);

                code.push(0xC0 | ((dst & 7) << 3) | (src & 7));

                let len = code.len();
                (code, offset + len).into()
            }

            _ => todo!(),
        },

        Opcode::JmpRel8 { offset: rel } => (vec![0xEB, *rel as u8], offset + 2).into(),

        Opcode::JmpRel32 { offset: rel } => {
            let mut code = vec![0xE9];
            code.extend_from_slice(&rel.to_le_bytes());
            (code, offset + 5).into()
        }

        Opcode::JeRel8 { offset: rel } => (vec![0x74, *rel as u8], offset + 2).into(),

        Opcode::JeRel32 { offset: rel } => {
            let mut code = vec![0x0F, 0x84];
            code.extend_from_slice(&rel.to_le_bytes());
            (code, offset + 6).into()
        }

        Opcode::JneRel8 { offset: rel } => (vec![0x75, *rel as u8], offset + 2).into(),

        Opcode::JneRel32 { offset: rel } => {
            let mut code = vec![0x0F, 0x85];
            code.extend_from_slice(&rel.to_le_bytes());
            (code, offset + 6).into()
        }

        Opcode::JlRel8 { offset: rel } => (vec![0x7C, *rel as u8], offset + 2).into(),

        Opcode::JlRel32 { offset: rel } => {
            let mut code = vec![0x0F, 0x8C];
            code.extend_from_slice(&rel.to_le_bytes());
            (code, offset + 6).into()
        }

        Opcode::JleRel8 { offset: rel } => (vec![0x7E, *rel as u8], offset + 2).into(),

        Opcode::JleRel32 { offset: rel } => {
            let mut code = vec![0x0F, 0x8E];
            code.extend_from_slice(&rel.to_le_bytes());
            (code, offset + 6).into()
        }

        Opcode::JgRel8 { offset: rel } => (vec![0x7F, *rel as u8], offset + 2).into(),

        Opcode::JgRel32 { offset: rel } => {
            let mut code = vec![0x0F, 0x8F];
            code.extend_from_slice(&rel.to_le_bytes());
            (code, offset + 6).into()
        }

        Opcode::JgeRel8 { offset: rel } => (vec![0x7D, *rel as u8], offset + 2).into(),

        Opcode::JgeRel32 { offset: rel } => {
            let mut code = vec![0x0F, 0x8D];
            code.extend_from_slice(&rel.to_le_bytes());
            (code, offset + 6).into()
        }

        Opcode::JbRel8 { offset: rel } => (vec![0x72, *rel as u8], offset + 2).into(),

        Opcode::JbRel32 { offset: rel } => {
            let mut code = vec![0x0F, 0x82];
            code.extend_from_slice(&rel.to_le_bytes());
            (code, offset + 6).into()
        }

        Opcode::JaRel8 { offset: rel } => (vec![0x77, *rel as u8], offset + 2).into(),

        Opcode::JaRel32 { offset: rel } => {
            let mut code = vec![0x0F, 0x87];
            code.extend_from_slice(&rel.to_le_bytes());
            (code, offset + 6).into()
        }

        Opcode::Sete { dst } => match dst {
            Operand::Register(reg) => {
                let reg = encode_register(reg);

                let mut code = Vec::new();

                if reg >= 8 {
                    code.push(0x41);
                }

                code.extend_from_slice(&[0x0F, 0x94, 0xC0 | (reg & 7)]);

                let len = code.len();
                (code, offset + len).into()
            }

            _ => todo!(),
        },

        Opcode::Setne { dst } => match dst {
            Operand::Register(reg) => {
                let reg = encode_register(reg);

                let mut code = Vec::new();

                if reg >= 8 {
                    code.push(0x41);
                }

                code.extend_from_slice(&[0x0F, 0x95, 0xC0 | (reg & 7)]);

                let len = code.len();
                (code, offset + len).into()
            }

            _ => todo!(),
        },

        Opcode::Setl { dst } => match dst {
            Operand::Register(reg) => {
                let reg = encode_register(reg);

                let mut code = Vec::new();

                if reg >= 8 {
                    code.push(0x41);
                }

                code.extend_from_slice(&[0x0F, 0x9C, 0xC0 | (reg & 7)]);

                let len = code.len();
                (code, offset + len).into()
            }

            _ => todo!(),
        },

        Opcode::Setle { dst } => match dst {
            Operand::Register(reg) => {
                let reg = encode_register(reg);

                let mut code = Vec::new();

                if reg >= 8 {
                    code.push(0x41);
                }

                code.extend_from_slice(&[0x0F, 0x9E, 0xC0 | (reg & 7)]);

                let len = code.len();
                (code, offset + len).into()
            }

            _ => todo!(),
        },

        Opcode::Setg { dst } => match dst {
            Operand::Register(reg) => {
                let reg = encode_register(reg);

                let mut code = Vec::new();

                if reg >= 8 {
                    code.push(0x41);
                }

                code.extend_from_slice(&[0x0F, 0x9F, 0xC0 | (reg & 7)]);

                let len = code.len();
                (code, offset + len).into()
            }

            _ => todo!(),
        },

        Opcode::Setge { dst } => match dst {
            Operand::Register(reg) => {
                let reg = encode_register(reg);

                let mut code = Vec::new();

                if reg >= 8 {
                    code.push(0x41);
                }

                code.extend_from_slice(&[0x0F, 0x9D, 0xC0 | (reg & 7)]);

                let len = code.len();
                (code, offset + len).into()
            }

            _ => todo!(),
        },

        Opcode::Setb { dst } => match dst {
            Operand::Register(reg) => {
                let reg = encode_register(reg);

                let mut code = Vec::new();

                if reg >= 8 {
                    code.push(0x41);
                }

                code.extend_from_slice(&[0x0F, 0x92, 0xC0 | (reg & 7)]);

                let len = code.len();
                (code, offset + len).into()
            }

            _ => todo!(),
        },

        Opcode::Seta { dst } => match dst {
            Operand::Register(reg) => {
                let reg = encode_register(reg);

                let mut code = Vec::new();

                if reg >= 8 {
                    code.push(0x41);
                }

                code.extend_from_slice(&[0x0F, 0x97, 0xC0 | (reg & 7)]);

                let len = code.len();
                (code, offset + len).into()
            }

            _ => todo!(),
        },

        // Hive opcodes
        Opcode::Syscall => {
            let (a, b, c) = gen_sched_req(0x76, &[], offset);
            TranslatedOpcode(a, b, Some(c))
        }
        Opcode::Decl { variable: _, ty: _ } => todo!(),
        Opcode::Param {
            parameter: _,
            ty: _,
        } => todo!(),
        Opcode::EndVar => todo!(),
        Opcode::Frame {
            size: _,
            alignment: _,
        } => todo!(),
        Opcode::FrameAlloc { size: _ } => todo!(),
        Opcode::FrameFree => {
            let (a, b, c) = gen_sched_req(0x05, &[], offset);
            TranslatedOpcode(a, b, Some(c))
        }
        Opcode::Malloc { dst, size } => {
            let (a, b, c) = gen_sched_req(0x10, &[(dst, 1, 8), (size, 9, 8)], offset);
            TranslatedOpcode(a, b, Some(c))
        }
        Opcode::Calloc { dst, count, size } => {
            let (a, b, c) =
                gen_sched_req(0x11, &[(dst, 1, 8), (count, 9, 8), (size, 17, 8)], offset);
            TranslatedOpcode(a, b, Some(c))
        }
        Opcode::Realloc { dst, ptr, size } => {
            let (a, b, c) = gen_sched_req(0x12, &[(dst, 1, 8), (ptr, 9, 8), (size, 17, 8)], offset);
            TranslatedOpcode(a, b, Some(c))
        }
        Opcode::Free { ptr } => {
            let (a, b, c) = gen_sched_req(0x13, &[(ptr, 1, 8)], offset);
            TranslatedOpcode(a, b, Some(c))
        }
        Opcode::Memcpy { dst, src, size } => {
            let (a, b, c) = gen_sched_req(0x14, &[(dst, 1, 8), (src, 9, 8), (size, 17, 8)], offset);
            TranslatedOpcode(a, b, Some(c))
        }
        Opcode::Memmove { dst, src, size } => {
            let (a, b, c) = gen_sched_req(0x15, &[(dst, 1, 8), (src, 9, 8), (size, 17, 8)], offset);
            TranslatedOpcode(a, b, Some(c))
        }
        Opcode::Memset { dst, value, size } => {
            let (a, b, c) =
                gen_sched_req(0x16, &[(dst, 1, 8), (value, 9, 1), (size, 10, 8)], offset);
            TranslatedOpcode(a, b, Some(c))
        }
        Opcode::Memcmp { a, b, size } => {
            let (a, b, c) = gen_sched_req(0x17, &[(a, 1, 8), (b, 9, 8), (size, 17, 8)], offset);
            TranslatedOpcode(a, b, Some(c))
        }
        Opcode::StackAlloc { dst: _, size: _ } => todo!(),
        Opcode::StackFree { ptr: _ } => todo!(),
        Opcode::ProcSpawn {
            dst,
            function,
            args,
        } => {
            let (a, b, c) = gen_sched_req(
                0x27,
                &[(dst, 1, 8), (function, 9, 4), (args, 10, 8)],
                offset,
            );
            TranslatedOpcode(a, b, Some(c))
        }
        Opcode::ProcSelf { dst } => {
            let (a, b, c) = gen_sched_req(0x21, &[(dst, 1, 8)], offset);
            TranslatedOpcode(a, b, Some(c))
        }
        Opcode::ProcExit { reason } => {
            let (a, b, c) = gen_sched_req(0x22, &[(reason, 1, 1)], offset);
            TranslatedOpcode(a, b, Some(c))
        }
        Opcode::ProcKill { pid, reason } => {
            let (a, b, c) = gen_sched_req(0x23, &[(pid, 1, 4), (reason, 5, 1)], offset);
            TranslatedOpcode(a, b, Some(c))
        }
        Opcode::ProcState { dst, pid } => {
            let (a, b, c) = gen_sched_req(0x24, &[(dst, 1, 8), (pid, 9, 4)], offset);
            TranslatedOpcode(a, b, Some(c))
        }
        Opcode::ProcYield => {
            let (a, b, c) = gen_sched_req(0x25, &[], offset);
            TranslatedOpcode(a, b, Some(c))
        }
        Opcode::ProcSleep { duration } => {
            let (a, b, c) = gen_sched_req(0x26, &[(duration, 1, 8)], offset);
            TranslatedOpcode(a, b, Some(c))
        }
        Opcode::ProcAlive { dst, pid } => {
            let (a, b, c) = gen_sched_req(0x27, &[(dst, 1, 8), (pid, 9, 4)], offset);
            TranslatedOpcode(a, b, Some(c))
        }
        Opcode::ProcLink { pid } => {
            let (a, b, c) = gen_sched_req(0x28, &[(pid, 1, 4)], offset);
            TranslatedOpcode(a, b, Some(c))
        }
        Opcode::ProcUnlink { pid } => {
            let (a, b, c) = gen_sched_req(0x29, &[(pid, 1, 4)], offset);
            TranslatedOpcode(a, b, Some(c))
        }
        Opcode::ProcMonitor { pid } => {
            let (a, b, c) = gen_sched_req(0x2A, &[(pid, 1, 4)], offset);
            TranslatedOpcode(a, b, Some(c))
        }
        Opcode::ProcDemonitor { pid } => {
            let (a, b, c) = gen_sched_req(0x2B, &[(pid, 1, 4)], offset);
            TranslatedOpcode(a, b, Some(c))
        }
        Opcode::SchedYield => {
            let (a, b, c) = gen_sched_req(0x30, &[], offset);
            TranslatedOpcode(a, b, Some(c))
        }
        Opcode::SchedWake { pid: _ } => todo!(),
        Opcode::SchedWait { event: _ } => todo!(),
        Opcode::SchedId { dst: _ } => todo!(),
        Opcode::SchedCount { dst: _ } => todo!(),
        Opcode::SchedMigrate {
            pid: _,
            scheduler: _,
        } => todo!(),
        Opcode::MailSend { pid: _, message: _ } => todo!(),
        Opcode::MailRecv { dst: _ } => todo!(),
        Opcode::MailTryRecv { dst: _ } => todo!(),
        Opcode::MailPeek { dst: _ } => todo!(),
        Opcode::MailLen { dst: _ } => todo!(),
        Opcode::MailClear => {
            let (a, b, c) = gen_sched_req(0x45, &[], offset);
            TranslatedOpcode(a, b, Some(c))
        }
        Opcode::Await { operation: _ } => todo!(),
        Opcode::Async {
            dst: _,
            function: _,
            args: _,
        } => todo!(),
        Opcode::Resume { process: _ } => todo!(),
        Opcode::Suspend { process: _ } => todo!(),
        Opcode::ProcessDrop { pid: _ } => todo!(),
        Opcode::ProcessReset { pid: _ } => todo!(),
        Opcode::ProcessFail { reason: _ } => todo!(),
        Opcode::ProcessTrap { handler: _ } => todo!(),
        Opcode::Jit { function: _ } => todo!(),
        Opcode::HiveCall {
            function: _,
            args: _,
        } => todo!(),
        Opcode::CallWorker { worker: _, args: _ } => todo!(),
        Opcode::NativeCall {
            function: _,
            args: _,
        } => todo!(),
        Opcode::NativeReturn { value: _ } => todo!(),
        Opcode::Recall => todo!(),
        Opcode::Ret { ptr } => {
            let (a, b, c) = gen_sched_req(0x77, &[(ptr, 1, 8)], offset);
            TranslatedOpcode(a, b, Some(c))
        }
    }
}
