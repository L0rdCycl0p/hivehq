# Hive Opcodes

| Opcode     | Name   | Operands      | Description                              |
|------------|--------|---------------|------------------------------------------|
| `90` | `NOP` | —             | No operation.                            |
| `88 /r` | `MOV` | `r/m8, r8` | Move an 8-bit value.                     |
| `89 /r` | `MOV` | `r/m64, r64` | Move a 64-bit value.                     |
| `8A /r` | `MOV` | `r8, r/m8` | Load an 8-bit value.                     |
| `8B /r` | `MOV` | `r64, r/m64` | Load a 64-bit value.                     |
| `B8+rd io` | `MOV` | `r64, imm64` | Move an immediate 64-bit value.          |
| `8D /r` | `LEA` | `r64, m` | Calculate an effective address.          |
| `01 /r` | `ADD` | `r/m64, r64` | Integer addition.                        |
| `03 /r` | `ADD` | `r64, r/m64` | Integer addition.                        |
| `29 /r` | `SUB` | `r/m64, r64` | Integer subtraction.                     |
| `2B /r` | `SUB` | `r64, r/m64` | Integer subtraction.                     |
| `F7 /4` | `MUL` | `r/m64` | Unsigned multiplication.                 |
| `F7 /5` | `IMUL` | `r/m64` | Signed multiplication.                   |
| `F7 /6` | `DIV` | `r/m64` | Unsigned division.                       |
| `F7 /7` | `IDIV` | `r/m64` | Signed division.                         |
| `FF /0` | `INC` | `r/m64` | Increment by one.                        |
| `FF /1` | `DEC` | `r/m64` | Decrement by one.                        |
| `F7 /3` | `NEG` | `r/m64` | Two's-complement negation.               |
| `21 /r` | `AND` | `r/m64, r64` | Bitwise AND.                             |
| `09 /r` | `OR` | `r/m64, r64` | Bitwise OR.                              |
| `31 /r` | `XOR` | `r/m64, r64` | Bitwise XOR.                             |
| `F7 /2` | `NOT` | `r/m64` | Bitwise NOT.                             |
| `85 /r` | `TEST` | `r/m64, r64` | Bitwise test without storing the result. |
| `39 /r` | `CMP` | `r/m64, r64` | Compare two values.                      |
| `3B /r` | `CMP` | `r64, r/m64` | Compare two values.                      |
| `C1 /4 ib` | `SHL` | `r/m64, imm8` | Logical left shift.                      |
| `D3 /4` | `SHL` | `r/m64, CL` | Logical left shift by CL.                |
| `C1 /5 ib` | `SHR` | `r/m64, imm8` | Logical right shift.                     |
| `D3 /5` | `SHR` | `r/m64, CL` | Logical right shift by CL.               |
| `C1 /7 ib` | `SAR` | `r/m64, imm8` | Arithmetic right shift.                  |
| `D3 /7` | `SAR` | `r/m64, CL` | Arithmetic right shift by CL.            |
| `C1 /0 ib` | `ROL` | `r/m64, imm8` | Rotate left.                             |
| `C1 /1 ib` | `ROR` | `r/m64, imm8` | Rotate right.                            |
| `0F 05` | `SYSCALL` | - | Syscall, but the syscall goes to the schedular
| `0F BE /r` | `MOVSX` | `r64, r/m8` | Sign-extend 8-bit value.                                                                                                                                         |
| `0F BF /r` | `MOVSX` | `r64, r/m16` | Sign-extend 16-bit value.                                                                                                                                        |
| `0F B6 /r` | `MOVZX` | `r64, r/m8` | Zero-extend 8-bit value.                                                                                                                                         |
| `0F B7 /r` | `MOVZX` | `r64, r/m16` | Zero-extend 16-bit value.                                                                                                                                        |
| `E9 cd` | `JMP` | `rel32` | Unconditional jump.                                                                                                                                              |
| `EB cb` | `JMP` | `rel8` | Short unconditional jump.                                                                                                                                        |
| `74 cb` | `JE` | `rel8` | Jump if equal / zero.                                                                                                                                            |
| `0F 84 cd` | `JE` | `rel32` | Near jump if equal / zero.                                                                                                                                       |
| `75 cb` | `JNE` | `rel8` | Jump if not equal / non-zero.                                                                                                                                    |
| `0F 85 cd` | `JNE` | `rel32` | Near jump if not equal / non-zero.                                                                                                                               |
| `7C cb` | `JL` | `rel8` | Jump if signed less-than.                                                                                                                                        |
| `0F 8C cd` | `JL` | `rel32` | Near jump if signed less-than.                                                                                                                                   |
| `7E cb` | `JLE` | `rel8` | Jump if signed less-than-or-equal.                                                                                                                               |
| `0F 8E cd` | `JLE` | `rel32` | Near jump if signed less-than-or-equal.                                                                                                                          |
| `7F cb` | `JG` | `rel8` | Jump if signed greater-than.                                                                                                                                     |
| `0F 8F cd` | `JG` | `rel32` | Near jump if signed greater-than.                                                                                                                                |
| `7D cb` | `JGE` | `rel8` | Jump if signed greater-than-or-equal.                                                                                                                            |
| `0F 8D cd` | `JGE` | `rel32` | Near jump if signed greater-than-or-equal.                                                                                                                       |
| `72 cb` | `JB` | `rel8` | Jump if unsigned below.                                                                                                                                          |
| `0F 82 cd` | `JB` | `rel32` | Near jump if unsigned below.                                                                                                                                     |
| `77 cb` | `JA` | `rel8` | Jump if unsigned above.                                                                                                                                          |
| `0F 87 cd` | `JA` | `rel32` | Near jump if unsigned above.                                                                                                                                     |
| `0F 94 /r` | `SETE` | `r/m8` | Set byte if equal.                                                                                                                                               |
| `0F 95 /r` | `SETNE` | `r/m8` | Set byte if not equal.                                                                                                                                           |
| `0F 9C /r` | `SETL` | `r/m8` | Set byte if signed less-than.                                                                                                                                    |
| `0F 9E /r` | `SETLE` | `r/m8` | Set byte if signed less-than-or-equal.                                                                                                                           |
| `0F 9F /r` | `SETG` | `r/m8` | Set byte if signed greater-than.                                                                                                                                 |
| `0F 9D /r` | `SETGE` | `r/m8` | Set byte if signed greater-than-or-equal.                                                                                                                        |
| `0F 92 /r` | `SETB` | `r/m8` | Set byte if unsigned below.                                                                                                                                      |
| `0F 97 /r` | `SETA` | `r/m8` | Set byte if unsigned above.                                                                                                                                      |
| `F0 00` | `DECL` | `variable id, type` | Declares a local variable in the current function frame.                                                                                                         |
| `F0 01` | `PARAM` | `parameter, type` | Declares a function parameter in the current function frame. Parameters are part of the statically known frame layout and may be modified by the function.       |
| `F0 02` | `END_VAR` | —                     | Terminates the parameter and variable declaration section.                                                                                                       |
| `F0 03` | `FRAME` | `size, alignment` | Describes the complete statically allocated frame, including parameters and local variables.                                                                     |
| `F0 04` | `FRAME_ALLOC` | `size` | Allocates a function frame.                                                                                                                                      |
| `F0 05` | `FRAME_FREE` | —                     | Releases the current function frame.                                                                                                                             |
| `F0 10` | `MALLOC` | `dst, size` | Allocates memory from the current process heap.                                                                                                                  |
| `F0 11` | `CALLOC` | `dst, count, size` | Allocates zero-initialized process heap memory.                                                                                                                  |
| `F0 12` | `REALLOC` | `dst, ptr, size` | Resizes a process heap allocation.                                                                                                                               |
| `F0 13` | `FREE` | `ptr` | Releases process heap memory.                                                                                                                                    |
| `F0 14` | `MEMCPY` | `dst, src, size` | Copies a memory region.                                                                                                                                          |
| `F0 15` | `MEMMOVE` | `dst, src, size` | Moves a memory region while supporting overlap.                                                                                                                  |
| `F0 16` | `MEMSET` | `dst, value, size` | Fills a memory region.                                                                                                                                           |
| `F0 17` | `MEMCMP` | `a, b, size` | Compares two memory regions.                                                                                                                                     |
| `F0 18` | `STACK_ALLOC` | `dst, size` | Allocates dynamically sized process-owned stack memory.                                                                                                          |
| `F0 19` | `STACK_FREE` | `ptr` | Releases dynamic stack memory.                                                                                                                                   |
| `F0 20` | `PROC_SPAWN` | `dst, function, args` | Creates a new Hive process.                                                                                                                                      |
| `F0 21` | `PROC_SELF` | `dst` | Gets the current process ID.                                                                                                                                     |
| `F0 22` | `PROC_EXIT` | `reason` | Terminates the current process.                                                                                                                                  |
| `F0 23` | `PROC_KILL` | `pid, reason` | Terminates another process.                                                                                                                                      |
| `F0 24` | `PROC_STATE` | `dst, pid` | Gets the state of a process.                                                                                                                                     |
| `F0 25` | `PROC_YIELD` | —                     | Yields execution to the scheduler.                                                                                                                               |
| `F0 26` | `PROC_SLEEP` | `duration` | Suspends the current process. (duration in ms)                                                                                                                                    |
| `F0 27` | `PROC_ALIVE` | `dst, pid` | Checks whether a process is alive.                                                                                                                               |
| `F0 28` | `PROC_LINK` | `pid` | Links two processes.                                                                                                                                             |
| `F0 29` | `PROC_UNLINK` | `pid` | Removes a process link.                                                                                                                                          |
| `F0 2A` | `PROC_MONITOR` | `pid` | Starts monitoring a process.                                                                                                                                     |
| `F0 2B` | `PROC_DEMONITOR` | `pid` | Stops monitoring a process.                                                                                                                                      |
| `F0 30` | `SCHED_YIELD` | —                     | Explicitly yields execution to the scheduler.                                                                                                                    |
| `F0 31` | `SCHED_WAKE` | `pid` | Wakes a waiting process.                                                                                                                                         |
| `F0 32` | `SCHED_WAIT` | `filedescriptor` | Puts the current process into a waiting state.                                                                                                                   |
| `F0 33` | `SCHED_ID` | `dst` | Gets the current scheduler ID.                                                                                                                                   |
| `F0 34` | `SCHED_COUNT` | `dst` | Gets the number of schedulers.                                                                                                                                   |
| `F0 35` | `SCHED_MIGRATE` | `pid, scheduler` | Requests migration of a process to another scheduler.                                                                                                            |
| `F0 40` | `MAIL_SEND` | `pid, message` | Sends a message to another process mailbox.                                                                                                                      |
| `F0 41` | `MAIL_RECV` | `dst` | Receives the next message and waits if none is available.                                                                                                        |
| `F0 42` | `MAIL_TRYRECV` | `dst` | Attempts to receive a message without blocking.                                                                                                                  |
| `F0 43` | `MAIL_PEEK` | `dst` | Reads the next mailbox message without consuming it.                                                                                                             |
| `F0 44` | `MAIL_LEN` | `dst` | Gets the number of queued messages.                                                                                                                              |
| `F0 45` | `MAIL_CLEAR` | —                     | Clears the current mailbox.                                                                                                                                      |
| `F0 50` | `AWAIT` | `operation` | Suspends the current process until an asynchronous operation completes.                                                                                          |
| `F0 51` | `ASYNC` | `dst, function, args` | Starts an asynchronous operation.                                                                                                                                |
| `F0 52` | `RESUME` | `process` | Resumes a suspended process.                                                                                                                                     |
| `F0 53` | `SUSPEND` | `process` | Suspends a process.                                                                                                                                              |
| `F0 70` | `JIT` | `function` | Requests JIT compilation of a function.                                                                                                                          |
| `F0 71` | `CALL` | `function, args` | Executes a JIT-compiled function.                                                                                                                                |
| `F0 72` | `CALL_WORKER` | `worker, args` | Executes a JIT-compiled function.                                                                                                                                |
| `F0 73` | `NATIVE_CALL` | `function, args` | Calls a native function.                                                                                                                                         |
| `F0 74` | `NATIVE_RETURN` | `value` | Returns from a native boundary.                                                                                                                                  |
| `F0 75` | `RECALL` | -                     | Recalls the current function, so no stack frame has to be allocated. This only works at worker functions, because they don't have a return value. Good for loops |
| `F0 76` | `SYSCALL` | -                     | Just an alias for `0F 05` 
| `F0 77` | `RET` | `ptr to value` | Returns |
