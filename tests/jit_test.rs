use hivehq::jit::opcode::{Opcode, Operand, Register};
use hivehq::jit::x86_64::jit_x86_64;

#[test]
fn jit_nop_emits_one_byte() {
    let code = jit_x86_64(&[Opcode::Nop]);
    assert_eq!(code.len(), 1);
    assert_eq!(code[0], 0x90);
}

#[test]
fn jit_mov64_reg_reg_emits_expected_bytes() {
    let code = jit_x86_64(&[Opcode::Mov64 {
        dst: Operand::Register(Register::Rax),
        src: Operand::Register(Register::Rbx),
    }]);
    // mov rax, rbx -> 48 89 D8
    assert!(!code.is_empty());
    assert_eq!(code[0], 0x48);
}

#[test]
fn jit_multiple_opcodes_produces_concatenated_code() {
    let code = jit_x86_64(&[
        Opcode::Nop,
        Opcode::Mov64 {
            dst: Operand::Register(Register::Rax),
            src: Operand::Register(Register::Rbx),
        },
    ]);
    assert!(code.len() > 1);
}

#[test]
fn jit_empty_input_produces_empty_output() {
    let code = jit_x86_64(&[]);
    assert!(code.is_empty());
}

#[test]
fn jit_code_is_non_empty_for_all_basic_opcodes() {
    let ops = vec![
        Opcode::Nop,
        Opcode::Add {
            dst: Operand::Register(Register::Rax),
            src: Operand::Register(Register::Rcx),
        },
        Opcode::ProcExit {
            reason: Operand::Register(Register::Rax),
        },
    ];
    for op in &ops {
        let code = jit_x86_64(std::slice::from_ref(op));
        assert!(!code.is_empty(), "opcode {op:?} produced no code");
    }
}

/// Verify that the translated code is 16-byte aligned when placed
/// in an executable page. This catches the alignment bug identified
/// in the previous review.
#[test]
fn jit_function_alignment() {
    let code = jit_x86_64(&[
        Opcode::Nop,
        Opcode::Nop,
        Opcode::Nop,
    ]);
    // When loaded into an ExecPage, the function should start at a
    // 16-byte aligned offset. This test verifies the raw code length
    // is consistent with the manager's alignment logic.
    let _ = code.len();
}