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

#[repr(C, packed)]
pub struct ProcessRequest {
    pub request: ProcessRequestTag,
    pub params: [u8; 24], //
}

#[repr(u8)]
pub enum ProcessRequestTag {
    Syscall = 0x76, // does nothing, just requeuering the process
    Recall = 0x75,
    CallWorker = 0x72,
    Call = 0x71,
    MailClear = 0x45,
    MailLen = 0x44,
    MailPeek = 0x43,
    MailTryRecv = 0x42,
    MailRec = 0x41,
    MailSend = 0x40,
    SchedMigrate = 0x35,
    SchedCount = 0x34,
    SchedId = 0x33,
    SchedWait = 0x32,
    SchedWake = 0x31,
    SchedYield = 0x30,
    ProcDemonitor = 0x2B,
    ProcMonitor = 0x2A,
    ProcUnlink = 0x29,
    ProcLink = 0x28,
    ProcAlive = 0x27,
    ProcSleep = 0x26,
    ProcYield = 0x25,
    ProcState = 0x24,
    ProcKill = 0x23,
    ProcExit = 0x22,
    ProcSelf = 0x21,
    ProcSpawn = 0x20,
    StackFree = 0x19,
    StackAlloc = 0x18,
    MemCmp = 0x17,
    MemSet = 0x16,
    MemMove = 0x15,
    MemCpy = 0x14,
    Free = 0x13,
    ReAlloc = 0x12,
    CAlloc = 0x11,
    MAlloc = 0x10,
    FrameFree = 0x05,
    FrameAlloc = 0x04,
    Frame = 0x03,
    Ret = 0x77,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct FrameParam {
    pub size: u32,
    pub alignment: u32,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct AllocParam {
    // ptr to the return value (ptr to the heap mem)
    pub dst: u64,
    pub size: u64,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct CallocParam {
    // ptr to the return value (ptr to the heap mem)
    pub dst: u64,
    pub count: u64,
    pub size: u64,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct ReallocParam {
    // ptr to the return value
    pub dst: u64,
    pub ptr: u64,
    pub size: u64,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct MemRegionParam {
    // ptr to the return value
    pub dst: u64,
    pub src: u64,
    pub size: u64,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct MemsetParam {
    // ptr to the return value
    pub dst: u64,
    pub value: u8,
    pub size: u64,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct ProcSpawnParam {
    /// ptr to the return value (pid)
    pub dst: u64,
    pub function: u32,
    /// ptr to the arguments
    pub args: u64,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct ProcKillParam {
    pub pid: u32,
    pub reason: u32,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct ProcPidParam {
    // ptr to the return value
    pub dst: u64,
    pub pid: u32,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct SchedMigrateParam {
    pub pid: u32,
    pub scheduler: u32,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct MailSendParam {
    pub pid: u32,
    pub message: (u64, u64), // len + ptr
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct AsyncParam {
    // ptr to the return value
    pub dst: u32,
    pub function: u32,
    pub args: u64, // ptr
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct CallParam {
    pub function: u32,
    pub args: u64, // ptr
}

#[repr(C, packed)]
pub union ProcessRequestParam {
    // FRAME
    pub frame: FrameParam,

    // FRAME_ALLOC
    pub frame_alloc: u32,

    // MALLOC
    pub malloc: AllocParam,

    // CALLOC
    pub calloc: CallocParam,

    // REALLOC
    pub realloc: ReallocParam,

    // FREE
    pub free: u64,

    // MEMCPY / MEMMOVE / MEMSET / MEMCMP
    pub mem_region: MemRegionParam,
    pub memset: MemsetParam,

    // STACK_ALLOC
    pub stack_alloc: AllocParam,

    // STACK_FREE
    pub stack_free: u64,

    // PROC_SPAWN
    pub proc_spawn: ProcSpawnParam,

    // PROC_SELF
    pub proc_self: u64,

    // PROC_EXIT
    pub proc_exit: u8,

    // PROC_KILL
    pub proc_kill: ProcKillParam,

    // PROC_STATE
    pub proc_state: ProcPidParam,

    // PROC_YIELD
    // PROC_SLEEP
    /// duration to sleep
    pub proc_sleep: u64,

    // PROC_ALIVE
    pub proc_alive: ProcPidParam,

    // PROC_LINK / PROC_UNLINK / PROC_MONITOR / PROC_DEMONITOR
    pub proc_pid: u32,

    // SCHED_YIELD
    // SCHED_ID
    // SCHED_COUNT
    // SCHED_WAIT
    pub sched_wait: u32,

    // SCHED_WAKE
    pub sched_wake: u32,

    // SCHED_MIGRATE
    pub sched_migrate: SchedMigrateParam,

    // MAIL_SEND
    pub mail_send: MailSendParam,

    // MAIL_RECV / MAIL_TRYRECV / MAIL_PEEK
    // ptr to the return value (mail box entry)
    pub mail_recv: u64, // ptr

    // MAIL_LEN
    // ptr to the return value (mail len)
    pub mail_len: u64,

    // AWAIT
    pub r#await: u64,

    // ASYNC
    pub r#async: AsyncParam,

    // RESUME / SUSPEND
    pub process: u32,

    // JIT
    pub jit: u32,

    // CALL
    pub call: CallParam,

    // CALL_WORKER
    pub call_worker: CallParam,

    // NATIVE_CALL
    pub native_call: CallParam,

    // NATIVE_RETURN
    pub native_return: u64, // ptr

    pub ret: u64, // ptr
}

pub const PROCESS_REQUEST_PARAM_SIZE: usize = size_of::<ProcessRequestParam>();
