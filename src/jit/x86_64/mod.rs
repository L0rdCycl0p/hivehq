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

#[cfg(test)]
use crate::jit::opcode::{Opcode, Operand, Register};
use crate::jit::{
    opcode::{self},
    x86_64::translate_opcode::{TranslatedOpcode, translate_opcode},
};
mod build_instruction;
mod encode_rm;
mod gen_context_switch;
mod gen_load_operand;
mod gen_sched_req;
mod translate_opcode;
pub use gen_context_switch::ContextSwitchPatch;
#[must_use]
pub fn jit_x86_64(code: &[opcode::Opcode]) -> (Box<[u8]>, Box<[ContextSwitchPatch]>) {
    let mut offset = 0;
    let mut jit_code = Vec::with_capacity(50);
    let mut patches = Vec::with_capacity(10);
    for opc in code {
        let TranslatedOpcode(c, o, p) = translate_opcode(opc, offset);
        offset += o;
        jit_code.extend(c);
        if let Some(p) = p {
            patches.push(p);
        }
    }
    (jit_code.into_boxed_slice(), patches.into_boxed_slice())
}

#[test]
fn test_jit() {
    let opcodes: Box<[Opcode]> = Box::from([
        Opcode::Nop,
        Opcode::Mov64 {
            dst: Operand::Register(Register::Rax),
            src: Operand::Register(Register::Rbx),
        },
        Opcode::Add {
            dst: Operand::Register(Register::Rax),
            src: Operand::Register(Register::Rcx),
        },
        Opcode::ProcExit {
            reason: Operand::Register(Register::Rax),
        },
    ]);
    let (jit_code, patches) = jit_x86_64(&opcodes);
    for (index, opcode) in jit_code.iter().enumerate() {
        println!("{opcode:#04x}");
    }
}
/*
hive asm
param $0, type
var $1, type
mov64 $1, 42
add $1, $0
proc.exit 1

asm
_entry:
    ;; schedular_request_ptr = [rsp+0] = rsp
    ;; schedular_context_ptr = [rsp+8]
    ;; process_context_ptr = [rsp+16]
    ;; swap_context_func_addr = [rsp+24]
    ;; param0 = [rsp+32]
    ;; var1 = [rsp+40]
    ;;
    mov QWORD PTR [rsp+40], 42
    add QWORD PTR [rsp+40], QWORD PTR [rsp+32]
    mov rdi, QWORD PTR [rsp+0]
    mov BYTE PTR [rdi + 0], 0x22 ;; REQUEST OP
    mov BYTE PTR [rdi + 1], 0x1 ;; REQUEST PAYLOAD

    mov rdi, QWORD PTR [rsp+16]
    mov rsi, QWORD PTR [rsp+8]
    lea rcx, [rel .resume0]
    jmp context_switch
.resume0:

*/
