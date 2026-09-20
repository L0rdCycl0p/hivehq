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


use thiserror::Error;

use crate::jit::opcode::{MemoryOperand, Opcode, Operand, Register, Type, Variable};

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("UnexpectedEof")]
    UnexpectedEof,
    #[error("Unknown Opcode `{0}` at `{1}`")]
    UnknownOpcode(u8, usize),
    #[error("Unknown Hive Opcode `{0}` at `{1}`")]
    UnknownHiveOpcode(u8, usize),
    #[error("Invalid Operand `{0}` at `{1}`")]
    InvalidOperand(u8, usize),
}

pub struct Parser<'a> {
    code: &'a [u8],
    ptr: usize,
}

impl<'a> Parser<'a> {
    #[must_use]
    pub const fn new(code: &'a [u8]) -> Self {
        Self { code, ptr: 0 }
    }

    pub fn parse(mut self) -> Result<Box<[Opcode]>, ParseError> {
        let mut instructions = Vec::new();

        while self.ptr < self.code.len() {
            instructions.push(self.parse_opcode()?);
        }

        Ok(instructions.into_boxed_slice())
    }

    fn parse_opcode(&mut self) -> Result<Opcode, ParseError> {
        match self.read_u8()? {
            // x86-64
            0x90 => Ok(Opcode::Nop),

            0x88 => Ok(Opcode::Mov8 {
                dst: self.read_operand()?,
                src: self.read_operand()?,
            }),

            0x89 => Ok(Opcode::Mov64 {
                dst: self.read_operand()?,
                src: self.read_operand()?,
            }),

            0x8A => Ok(Opcode::Mov8 {
                dst: self.read_operand()?,
                src: self.read_operand()?,
            }),

            0x8B => Ok(Opcode::Mov64 {
                dst: self.read_operand()?,
                src: self.read_operand()?,
            }),

            0x8D => Ok(Opcode::Lea {
                dst: self.read_register()?,
                src: self.read_memory_operand()?,
            }),

            0x01 | 0x03 => Ok(Opcode::Add {
                dst: self.read_operand()?,
                src: self.read_operand()?,
            }),

            0x29 | 0x2B => Ok(Opcode::Sub {
                dst: self.read_operand()?,
                src: self.read_operand()?,
            }),

            0xF7 => self.parse_f7(),

            0xFF => self.parse_ff(),

            0x21 => Ok(Opcode::And {
                dst: self.read_operand()?,
                src: self.read_operand()?,
            }),

            0x09 => Ok(Opcode::Or {
                dst: self.read_operand()?,
                src: self.read_operand()?,
            }),

            0x31 => Ok(Opcode::Xor {
                dst: self.read_operand()?,
                src: self.read_operand()?,
            }),

            0x85 => Ok(Opcode::Test {
                left: self.read_operand()?,
                right: self.read_operand()?,
            }),

            0x39 | 0x3B => Ok(Opcode::Cmp {
                left: self.read_operand()?,
                right: self.read_operand()?,
            }),

            0xE9 => Ok(Opcode::JmpRel32 {
                offset: self.read_i32()?,
            }),

            0xEB => Ok(Opcode::JmpRel8 {
                offset: self.read_i8()?,
            }),

            0x74 => Ok(Opcode::JeRel8 {
                offset: self.read_i8()?,
            }),

            0x75 => Ok(Opcode::JneRel8 {
                offset: self.read_i8()?,
            }),

            0x7C => Ok(Opcode::JlRel8 {
                offset: self.read_i8()?,
            }),

            0x7E => Ok(Opcode::JleRel8 {
                offset: self.read_i8()?,
            }),

            0x7F => Ok(Opcode::JgRel8 {
                offset: self.read_i8()?,
            }),

            0x7D => Ok(Opcode::JgeRel8 {
                offset: self.read_i8()?,
            }),

            0x72 => Ok(Opcode::JbRel8 {
                offset: self.read_i8()?,
            }),

            0x77 => Ok(Opcode::JaRel8 {
                offset: self.read_i8()?,
            }),

            // Hive
            0xF0 => self.parse_hive_opcode(),

            op => Err(ParseError::UnknownOpcode(op, self.ptr)),
        }
    }

    fn parse_f7(&mut self) -> Result<Opcode, ParseError> {
        let sub = self.read_u8()?;

        match sub {
            4 => Ok(Opcode::Mul {
                src: self.read_operand()?,
            }),

            5 => Ok(Opcode::Imul {
                src: self.read_operand()?,
            }),

            6 => Ok(Opcode::Div {
                src: self.read_operand()?,
            }),

            7 => Ok(Opcode::Idiv {
                src: self.read_operand()?,
            }),

            3 => Ok(Opcode::Neg {
                operand: self.read_operand()?,
            }),

            2 => Ok(Opcode::Not {
                operand: self.read_operand()?,
            }),

            _ => Err(ParseError::UnknownOpcode(0xF7, self.ptr)),
        }
    }

    fn parse_ff(&mut self) -> Result<Opcode, ParseError> {
        let sub = self.read_u8()?;

        match sub {
            0 => Ok(Opcode::Inc {
                operand: self.read_operand()?,
            }),

            1 => Ok(Opcode::Dec {
                operand: self.read_operand()?,
            }),

            _ => Err(ParseError::UnknownOpcode(0xFF, self.ptr)),
        }
    }

    fn parse_hive_opcode(&mut self) -> Result<Opcode, ParseError> {
        match self.read_u8()? {
            0x00 => Ok(Opcode::Decl {
                variable: self.read_variable()?,
                ty: self.read_type()?,
            }),

            0x01 => Ok(Opcode::Param {
                parameter: self.read_variable()?,
                ty: self.read_type()?,
            }),

            0x02 => Ok(Opcode::EndVar),

            0x03 => Ok(Opcode::Frame {
                size: self.read_u64()?,
                alignment: self.read_u64()?,
            }),

            0x04 => Ok(Opcode::FrameAlloc {
                size: self.read_u64()?,
            }),

            0x05 => Ok(Opcode::FrameFree),

            0x10 => Ok(Opcode::Malloc {
                dst: self.read_operand()?,
                size: self.read_operand()?,
            }),

            0x11 => Ok(Opcode::Calloc {
                dst: self.read_operand()?,
                count: self.read_operand()?,
                size: self.read_operand()?,
            }),

            0x12 => Ok(Opcode::Realloc {
                dst: self.read_operand()?,
                ptr: self.read_operand()?,
                size: self.read_operand()?,
            }),

            0x13 => Ok(Opcode::Free {
                ptr: self.read_operand()?,
            }),

            0x14 => Ok(Opcode::Memcpy {
                dst: self.read_operand()?,
                src: self.read_operand()?,
                size: self.read_operand()?,
            }),

            0x15 => Ok(Opcode::Memmove {
                dst: self.read_operand()?,
                src: self.read_operand()?,
                size: self.read_operand()?,
            }),

            0x16 => Ok(Opcode::Memset {
                dst: self.read_operand()?,
                value: self.read_operand()?,
                size: self.read_operand()?,
            }),

            0x17 => Ok(Opcode::Memcmp {
                a: self.read_operand()?,
                b: self.read_operand()?,
                size: self.read_operand()?,
            }),

            0x18 => Ok(Opcode::StackAlloc {
                dst: self.read_operand()?,
                size: self.read_operand()?,
            }),

            0x19 => Ok(Opcode::StackFree {
                ptr: self.read_operand()?,
            }),

            0x20 => Ok(Opcode::ProcSpawn {
                dst: self.read_operand()?,
                function: self.read_operand()?,
                args: self.read_operand()?,
            }),

            0x21 => Ok(Opcode::ProcSelf {
                dst: self.read_operand()?,
            }),

            0x22 => Ok(Opcode::ProcExit {
                reason: self.read_operand()?,
            }),

            0x23 => Ok(Opcode::ProcKill {
                pid: self.read_operand()?,
                reason: self.read_operand()?,
            }),

            0x24 => Ok(Opcode::ProcState {
                dst: self.read_operand()?,
                pid: self.read_operand()?,
            }),

            0x25 => Ok(Opcode::ProcYield),

            0x26 => Ok(Opcode::ProcSleep {
                duration: self.read_operand()?,
            }),

            0x27 => Ok(Opcode::ProcAlive {
                dst: self.read_operand()?,
                pid: self.read_operand()?,
            }),

            0x28 => Ok(Opcode::ProcLink {
                pid: self.read_operand()?,
            }),

            0x29 => Ok(Opcode::ProcUnlink {
                pid: self.read_operand()?,
            }),

            0x2A => Ok(Opcode::ProcMonitor {
                pid: self.read_operand()?,
            }),

            0x2B => Ok(Opcode::ProcDemonitor {
                pid: self.read_operand()?,
            }),

            0x30 => Ok(Opcode::SchedYield),

            0x31 => Ok(Opcode::SchedWake {
                pid: self.read_operand()?,
            }),

            0x32 => Ok(Opcode::SchedWait {
                event: self.read_operand()?,
            }),

            0x33 => Ok(Opcode::SchedId {
                dst: self.read_operand()?,
            }),

            0x34 => Ok(Opcode::SchedCount {
                dst: self.read_operand()?,
            }),

            0x35 => Ok(Opcode::SchedMigrate {
                pid: self.read_operand()?,
                scheduler: self.read_operand()?,
            }),

            0x40 => Ok(Opcode::MailSend {
                pid: self.read_operand()?,
                message: self.read_operand()?,
            }),

            0x41 => Ok(Opcode::MailRecv {
                dst: self.read_operand()?,
            }),

            0x42 => Ok(Opcode::MailTryRecv {
                dst: self.read_operand()?,
            }),

            0x43 => Ok(Opcode::MailPeek {
                dst: self.read_operand()?,
            }),

            0x44 => Ok(Opcode::MailLen {
                dst: self.read_operand()?,
            }),

            0x45 => Ok(Opcode::MailClear),

            0x50 => Ok(Opcode::Await {
                operation: self.read_operand()?,
            }),

            0x51 => Ok(Opcode::Async {
                dst: self.read_operand()?,
                function: self.read_operand()?,
                args: self.read_operand()?,
            }),

            0x52 => Ok(Opcode::Resume {
                process: self.read_operand()?,
            }),

            0x53 => Ok(Opcode::Suspend {
                process: self.read_operand()?,
            }),

            0x60 => Ok(Opcode::ProcessDrop {
                pid: self.read_operand()?,
            }),

            0x61 => Ok(Opcode::ProcessReset {
                pid: self.read_operand()?,
            }),

            0x62 => Ok(Opcode::ProcessFail {
                reason: self.read_operand()?,
            }),

            0x63 => Ok(Opcode::ProcessTrap {
                handler: self.read_operand()?,
            }),

            0x70 => Ok(Opcode::Jit {
                function: self.read_operand()?,
            }),

            0x71 => Ok(Opcode::HiveCall {
                function: self.read_operand()?,
                args: self.read_operand()?,
            }),

            0x72 => Ok(Opcode::NativeCall {
                function: self.read_operand()?,
                args: self.read_operand()?,
            }),

            0x73 => Ok(Opcode::NativeReturn {
                value: self.read_operand()?,
            }),

            0x74 => Ok(Opcode::Recall),
            0x75 => Ok(Opcode::Syscall),
            0x77 => Ok(Opcode::Ret {
                ptr: self.read_operand()?,
            }),
            op => Err(ParseError::UnknownHiveOpcode(op, self.ptr)),
        }
    }

    // ─────────────────────────────
    // Primitive readers
    // ─────────────────────────────

    fn read_u8(&mut self) -> Result<u8, ParseError> {
        let value = *self.code.get(self.ptr).ok_or(ParseError::UnexpectedEof)?;

        self.ptr += 1;

        Ok(value)
    }

    fn read_i8(&mut self) -> Result<i8, ParseError> {
        Ok(self.read_u8()? as i8)
    }

    fn read_u16(&mut self) -> Result<u16, ParseError> {
        let bytes = self.read_array::<2>()?;
        Ok(u16::from_le_bytes(bytes))
    }

    fn read_i32(&mut self) -> Result<i32, ParseError> {
        let bytes = self.read_array::<4>()?;
        Ok(i32::from_le_bytes(bytes))
    }

    fn read_u64(&mut self) -> Result<u64, ParseError> {
        let bytes = self.read_array::<8>()?;
        Ok(u64::from_le_bytes(bytes))
    }
    fn read_u32(&mut self) -> Result<u32, ParseError> {
        let bytes = self.read_array::<4>()?;
        Ok(u32::from_le_bytes(bytes))
    }

    fn read_array<const N: usize>(&mut self) -> Result<[u8; N], ParseError> {
        if self.ptr + N > self.code.len() {
            return Err(ParseError::UnexpectedEof);
        }

        let bytes = self.code[self.ptr..self.ptr + N].try_into().unwrap();

        self.ptr += N;

        Ok(bytes)
    }
}

impl Parser<'_> {
    fn read_register(&mut self) -> Result<Register, ParseError> {
        match self.read_u8()? {
            0 => Ok(Register::Rax),
            1 => Ok(Register::Rbx),
            2 => Ok(Register::Rcx),
            3 => Ok(Register::Rdx),
            4 => Ok(Register::Rsi),
            5 => Ok(Register::Rdi),
            6 => Ok(Register::Rbp),
            7 => Ok(Register::Rsp),
            8 => Ok(Register::R8),
            9 => Ok(Register::R9),
            10 => Ok(Register::R10),
            11 => Ok(Register::R11),
            12 => Ok(Register::R12),
            13 => Ok(Register::R13),
            14 => Ok(Register::R14),
            15 => Ok(Register::R15),
            op => Err(ParseError::InvalidOperand(op, self.ptr)),
        }
    }

    fn read_operand(&mut self) -> Result<Operand, ParseError> {
        match self.read_u8()? {
            // register
            0x00 => Ok(Operand::Register(self.read_register()?)),

            // immediate i64
            0x01 => Ok(Operand::Immediate(self.read_u64()? as i64)),

            // variable u32
            0x02 => {
                let id = self.read_u64()? as u32;
                Ok(Operand::Variable(id))
            }

            // frame offset i32
            0x03 => {
                let offset = self.read_i32()?;
                Ok(Operand::FrameOffset(offset))
            }

            // memory
            0x04 => Ok(Operand::Memory(self.read_memory_operand()?)),

            op => Err(ParseError::InvalidOperand(op, self.ptr)),
        }
    }

    fn read_memory_operand(&mut self) -> Result<MemoryOperand, ParseError> {
        let base = if self.read_u8()? == 0xFF {
            None
        } else {
            self.ptr -= 1;
            Some(self.read_register()?)
        };

        let index = if self.read_u8()? == 0xFF {
            None
        } else {
            self.ptr -= 1;
            Some(self.read_register()?)
        };

        let scale = self.read_u8()?;
        let displacement = self.read_i32()?;

        Ok(MemoryOperand {
            base,
            index,
            scale,
            displacement,
        })
    }

    fn read_variable(&mut self) -> Result<Variable, ParseError> {
        Ok(Variable {
            id: self.read_u32()?,
        })
    }
    fn read_type(&mut self) -> Result<Type, ParseError> {
        Ok(Type {
            id: self.read_u32()?,
        })
    }
}

pub fn parse(code: Box<[u8]>) -> Result<Box<[Opcode]>, ParseError> {
    Parser::new(&code).parse()
}

#[cfg(test)]
const EXAMPLE_CODE: &[u8] = &[
    // NOP
    0x90, // MOV variable 0, immediate 42
    0x89, // dst = variable 0
    0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // src = immediate 42
    0x01, 0x2A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // ADD variable 0, immediate 8
    0x01, // dst = variable 0
    0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // src = immediate 8
    0x01, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PROC_EXIT(0)
    0xF0, 0x22, // reason = immediate 0
    0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
#[test]
fn test_parser() -> Result<(), ParseError> {
    let opcodes = parse(EXAMPLE_CODE.into())?;

    for (index, opcode) in opcodes.iter().enumerate() {
        println!("{index:04}: {opcode:#?}");
    }
    Ok(())
}
