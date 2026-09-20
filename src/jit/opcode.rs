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

#[derive(Debug, Clone)]
pub enum Opcode {
    Nop,

    // ─────────────────────────────
    // x86-64 instructions
    // ─────────────────────────────
    Mov8 {
        dst: Operand,
        src: Operand,
    },

    Mov64 {
        dst: Operand,
        src: Operand,
    },

    MovImm64 {
        dst: Register,
        value: u64,
    },

    Lea {
        dst: Register,
        src: MemoryOperand,
    },

    Add {
        dst: Operand,
        src: Operand,
    },

    Sub {
        dst: Operand,
        src: Operand,
    },

    Mul {
        src: Operand,
    },

    Imul {
        src: Operand,
    },

    Div {
        src: Operand,
    },

    Idiv {
        src: Operand,
    },

    Inc {
        operand: Operand,
    },

    Dec {
        operand: Operand,
    },

    Neg {
        operand: Operand,
    },

    And {
        dst: Operand,
        src: Operand,
    },

    Or {
        dst: Operand,
        src: Operand,
    },

    Xor {
        dst: Operand,
        src: Operand,
    },

    Not {
        operand: Operand,
    },

    Test {
        left: Operand,
        right: Operand,
    },

    Cmp {
        left: Operand,
        right: Operand,
    },

    Shl {
        dst: Operand,
        amount: ShiftAmount,
    },

    Shr {
        dst: Operand,
        amount: ShiftAmount,
    },

    Sar {
        dst: Operand,
        amount: ShiftAmount,
    },

    Rol {
        dst: Operand,
        amount: ShiftAmount,
    },

    Ror {
        dst: Operand,
        amount: ShiftAmount,
    },

    Movsx8 {
        dst: Register,
        src: Operand,
    },

    Movsx16 {
        dst: Register,
        src: Operand,
    },

    Movzx8 {
        dst: Register,
        src: Operand,
    },

    Movzx16 {
        dst: Register,
        src: Operand,
    },

    JmpRel8 {
        offset: i8,
    },

    JmpRel32 {
        offset: i32,
    },

    JeRel8 {
        offset: i8,
    },

    JeRel32 {
        offset: i32,
    },

    JneRel8 {
        offset: i8,
    },

    JneRel32 {
        offset: i32,
    },

    JlRel8 {
        offset: i8,
    },

    JlRel32 {
        offset: i32,
    },

    JleRel8 {
        offset: i8,
    },

    JleRel32 {
        offset: i32,
    },

    JgRel8 {
        offset: i8,
    },

    JgRel32 {
        offset: i32,
    },

    JgeRel8 {
        offset: i8,
    },

    JgeRel32 {
        offset: i32,
    },

    JbRel8 {
        offset: i8,
    },

    JbRel32 {
        offset: i32,
    },

    JaRel8 {
        offset: i8,
    },

    JaRel32 {
        offset: i32,
    },

    Sete {
        dst: Operand,
    },

    Setne {
        dst: Operand,
    },

    Setl {
        dst: Operand,
    },

    Setle {
        dst: Operand,
    },

    Setg {
        dst: Operand,
    },

    Setge {
        dst: Operand,
    },

    Setb {
        dst: Operand,
    },

    Seta {
        dst: Operand,
    },
    Syscall,

    // ─────────────────────────────
    // Hive frame instructions
    // ─────────────────────────────
    Decl {
        variable: Variable,
        ty: Type,
    },

    Param {
        parameter: Variable,
        ty: Type,
    },

    EndVar,

    Frame {
        size: u64,
        alignment: u64,
    },

    FrameAlloc {
        size: u64,
    },

    FrameFree,

    // ─────────────────────────────
    // Memory
    // ─────────────────────────────
    Malloc {
        dst: Operand,
        size: Operand,
    },

    Calloc {
        dst: Operand,
        count: Operand,
        size: Operand,
    },

    Realloc {
        dst: Operand,
        ptr: Operand,
        size: Operand,
    },

    Free {
        ptr: Operand,
    },

    Memcpy {
        dst: Operand,
        src: Operand,
        size: Operand,
    },

    Memmove {
        dst: Operand,
        src: Operand,
        size: Operand,
    },

    Memset {
        dst: Operand,
        value: Operand,
        size: Operand,
    },

    Memcmp {
        a: Operand,
        b: Operand,
        size: Operand,
    },

    StackAlloc {
        dst: Operand,
        size: Operand,
    },

    StackFree {
        ptr: Operand,
    },

    // ─────────────────────────────
    // Processes
    // ─────────────────────────────
    ProcSpawn {
        dst: Operand,
        function: Operand,
        args: Operand,
    },

    ProcSelf {
        dst: Operand,
    },

    ProcExit {
        reason: Operand,
    },

    ProcKill {
        pid: Operand,
        reason: Operand,
    },

    ProcState {
        dst: Operand,
        pid: Operand,
    },

    ProcYield,

    ProcSleep {
        duration: Operand,
    },

    ProcAlive {
        dst: Operand,
        pid: Operand,
    },

    ProcLink {
        pid: Operand,
    },

    ProcUnlink {
        pid: Operand,
    },

    ProcMonitor {
        pid: Operand,
    },

    ProcDemonitor {
        pid: Operand,
    },

    // ─────────────────────────────
    // Scheduler
    // ─────────────────────────────
    SchedYield,

    SchedWake {
        pid: Operand,
    },

    SchedWait {
        event: Operand,
    },

    SchedId {
        dst: Operand,
    },

    SchedCount {
        dst: Operand,
    },

    SchedMigrate {
        pid: Operand,
        scheduler: Operand,
    },

    // ─────────────────────────────
    // Mailbox
    // ─────────────────────────────
    MailSend {
        pid: Operand,
        message: Operand,
    },

    MailRecv {
        dst: Operand,
    },

    MailTryRecv {
        dst: Operand,
    },

    MailPeek {
        dst: Operand,
    },

    MailLen {
        dst: Operand,
    },

    MailClear,

    // ─────────────────────────────
    // Async
    // ─────────────────────────────
    Await {
        operation: Operand,
    },

    Async {
        dst: Operand,
        function: Operand,
        args: Operand,
    },

    Resume {
        process: Operand,
    },

    Suspend {
        process: Operand,
    },

    // ─────────────────────────────
    // Process lifecycle
    // ─────────────────────────────
    ProcessDrop {
        pid: Operand,
    },

    ProcessReset {
        pid: Operand,
    },

    ProcessFail {
        reason: Operand,
    },

    ProcessTrap {
        handler: Operand,
    },

    // ─────────────────────────────
    // JIT / calls
    // ─────────────────────────────
    Jit {
        function: Operand,
    },

    HiveCall {
        function: Operand,
        args: Operand,
    },

    CallWorker {
        worker: Operand,
        args: Operand,
    },

    NativeCall {
        function: Operand,
        args: Operand,
    },

    NativeReturn {
        value: Operand,
    },

    Recall,

    Ret {
        ptr: Operand,
    },
}

#[derive(Debug, Clone)]
pub enum Operand {
    Register(Register),
    Immediate(i64),
    Memory(MemoryOperand),
    Variable(u32),
    FrameOffset(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register {
    Rax,
    Rbx,
    Rcx,
    Rdx,
    Rsi,
    Rdi,
    Rbp,
    Rsp,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

#[derive(Debug, Clone)]
pub struct MemoryOperand {
    pub base: Option<Register>,
    pub index: Option<Register>,
    pub scale: u8,
    pub displacement: i32,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub id: u32,
}
#[derive(Debug, Clone, Copy)]
pub struct Type {
    pub id: u32,
}

#[derive(Debug, Clone)]
pub enum ShiftAmount {
    Immediate(u8),
    Cl,
}
