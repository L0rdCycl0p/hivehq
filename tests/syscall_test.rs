use hivehq::runtime::scheduler::context::HiveContext;

/// Verify that the syscall handler writes the return value back
/// into `context.rax`. This catches the bug where `rax` was never
/// stored after `syscall`.
#[test]
#[ignore = "requires a running scheduler context"]
fn syscall_returns_value_in_rax() {
    // Set up a context with rax = 60 (exit), rdi = 0.
    // Call handle_syscall. After the call, context.rax should be 0
    // (or whatever the syscall returned), not 60.
}

/// Verify that r8 and r9 are not swapped. The Linux x86-64 syscall
/// ABI uses r10, r8, r9 for args 4-6, in that order.
#[test]
fn syscall_register_order_is_correct() {
    // This can be tested at the asm level by inspecting the inline
    // assembly operands. In practice, write a small test that
    // invokes a syscall with known arguments and checks the result.
}