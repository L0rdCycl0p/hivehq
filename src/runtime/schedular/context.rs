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


use std::arch::asm;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct HiveContext {
    pub rax: u64,    // 0
    pub rbx: u64,    // 8
    pub rcx: u64,    // 16
    pub rdx: u64,    // 24
    pub rsi: u64,    // 32
    pub rdi: u64,    // 40
    pub rbp: u64,    // 48
    pub rsp: u64,    // 56
    pub r8: u64,     // 64
    pub r9: u64,     // 72
    pub r10: u64,    // 80
    pub r11: u64,    // 88
    pub r12: u64,    // 96
    pub r13: u64,    // 104
    pub r14: u64,    // 112
    pub r15: u64,    // 120
    pub rip: u64,    // 128
    pub rflags: u64, // 136
}

pub unsafe fn swapcontext(old: &mut HiveContext, new: &HiveContext) {
    unsafe {
        asm!(
            // ============================================================
            // RDI = old
            // RSI = new
            // ============================================================

            // ------------------------------------------------------------
            // Save old GPRs
            // ------------------------------------------------------------

            "mov [rdi + 0],   rax",
            "mov [rdi + 8],   rbx",
            "mov [rdi + 16],  rcx",
            "mov [rdi + 24],  rdx",
            "mov [rdi + 32],  rsi",
            "mov [rdi + 40],  rdi",
            "mov [rdi + 48],  rbp",
            "mov [rdi + 56],  rsp",

            "mov [rdi + 64],  r8",
            "mov [rdi + 72],  r9",
            "mov [rdi + 80],  r10",
            "mov [rdi + 88],  r11",
            "mov [rdi + 96],  r12",
            "mov [rdi + 104], r13",
            "mov [rdi + 112], r14",
            "mov [rdi + 120], r15",

            // ------------------------------------------------------------
            // Save RIP
            //
            // The return address is the instruction after `ret`.
            // We create it with a local label.
            // ------------------------------------------------------------

            "lea rax, [rip + 2f]",
            "mov [rdi + 128], rax",

            // ------------------------------------------------------------
            // Save RFLAGS
            // ------------------------------------------------------------

            "pushfq",
            "pop rax",
            "mov [rdi + 136], rax",

            // ============================================================
            // Prepare new context
            // ============================================================

            // We still have RSI = new here.

            // Load new RSP into RCX temporarily.
            "mov rcx, [rsi + 56]",

            // Load new RIP into RAX temporarily.
            "mov rax, [rsi + 128]",

            // Switch to the new stack.
            "mov rsp, rcx",

            // Put the new RIP onto the new stack.
            //
            // After restoring all registers we execute `ret`,
            // which pops this address into RIP.
            "push rax",

            // ------------------------------------------------------------
            // Restore RFLAGS
            // ------------------------------------------------------------

            "mov rax, [rsi + 136]",
            "push rax",
            "popfq",

            // ------------------------------------------------------------
            // Restore registers
            //
            // RSI and RDI must be restored LAST because they are our
            // pointers to `new`.
            // ------------------------------------------------------------

            "mov rax, [rsi + 0]",
            "mov rbx, [rsi + 8]",
            "mov rcx, [rsi + 16]",
            "mov rdx, [rsi + 24]",

            "mov rbp, [rsi + 48]",

            "mov r8,  [rsi + 64]",
            "mov r9,  [rsi + 72]",
            "mov r10, [rsi + 80]",
            "mov r11, [rsi + 88]",
            "mov r12, [rsi + 96]",
            "mov r13, [rsi + 104]",
            "mov r14, [rsi + 112]",
            "mov r15, [rsi + 120]",

            // Restore RSI/RDI LAST.
            "mov rdi, [rsi + 40]",
            "mov rsi, [rsi + 32]",

            // Jump to new RIP.
            //
            // RIP was pushed onto the new stack above.
            "ret",

            // ============================================================
            // Old context resumes here.
            // ============================================================
            "2:",

            in("rdi") old,
            in("rsi") new,

            options(nostack)
        );
    }
}
