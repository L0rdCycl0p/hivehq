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
    x86_64::{
        gen_context_switch::{ContextSwitchPatch, gen_context_switch},
        gen_load_operand::gen_load_operand,
    },
};

/// Operand, offset, lenght
pub type RequestPayloadOperand<'a> = (&'a Operand, u8, u8);
#[must_use]
pub fn gen_sched_req(
    req_op: u8,
    operands: &[RequestPayloadOperand],
    mut offset: usize,
) -> (Vec<u8>, usize, ContextSwitchPatch) {
    let mut code = Vec::new();

    // mov rdi, QWORD PTR [rsp + 0]
    //
    // rdi = pointer to ProcessRequest
    code.extend_from_slice(&[0x48, 0x8B, 0x3C, 0x24]);

    // mov BYTE PTR [rdi + 0], req_op
    code.extend_from_slice(&[0xC6, 0x47, 0x00, req_op]);

    for request_operand in operands {
        gen_request_operand(&mut code, request_operand);
    }

    offset += code.len();

    let (context_switch_code, patch) = gen_context_switch(offset);

    offset += context_switch_code.len();

    code.extend_from_slice(&context_switch_code);

    (code, offset, patch)
}

fn gen_request_operand(code: &mut Vec<u8>, request_operand: &RequestPayloadOperand) {
    match request_operand.2 {
        1 => {
            gen_load_operand(code, request_operand.0, Register::Rax);

            // mov byte ptr [rdi + offset], al
            code.extend_from_slice(&[0x88, 0x47, request_operand.1]);
        }

        4 => {
            gen_load_operand(code, request_operand.0, Register::Rax);

            // mov dword ptr [rdi + offset], eax
            code.extend_from_slice(&[0x89, 0x47, request_operand.1]);
        }

        8 => {
            gen_load_operand(code, request_operand.0, Register::Rax);

            // mov qword ptr [rdi + offset], rax
            code.extend_from_slice(&[0x48, 0x89, 0x47, request_operand.1]);
        }

        length => {
            panic!("unsupported request operand length: {length}");
        }
    }
}
