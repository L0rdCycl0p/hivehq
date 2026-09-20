# Hive Architecture

Hive is a native actor/process runtime inspired by the BEAM, but designed to execute functions as native/JIT-compiled code instead of interpreting them through a virtual machine.

The core idea is:

> **BEAM-like processes and scheduling, without BEAM VM execution overhead.** 

Hive is asynchronous by design. Every Hive Process is represented by a state, and the scheduler executes the work described by that state.

---

## 1. Runtime Overview

A Hive runtime consists of:

* **Schedulers** 
* **Hive Processes** 
* **Process States** 
* **Stack Frames** 
* **Process Heaps** 
* **JIT/native functions** 
* **Message passing** 

Conceptually:

```text
Hive Runtime
│
├── Scheduler
│   ├── Scheduler Stack
│   ├── Run Queue
│   └── Hive Processes
│
├── Scheduler
│   ├── Scheduler Stack
│   ├── Run Queue
│   └── Hive Processes
│
└── ...
````

Schedulers are responsible for executing Hive Processes.

A Hive Process is not an operating-system thread. It is a lightweight runtime process managed by a scheduler.

---

# 2. Hive Processes

A Hive Process contains its current state and process-local memory.

```text
Hive Process
│
├── State
│
├── Stack Frames
│
├── Process Heap
│
└── Runtime Data
```

Processes are intentionally small.

A process does not need a large permanently allocated OS stack. Stack memory is acquired as functions are executed.

This allows Hive to support very large numbers of processes without requiring a large stack allocation for every process.

---

# 3. Process State

Every Hive Process has a state.

The state describes what the process should do next.

For example:

```text
Process State
│
├── Call function
├── Wait
├── Send message
├── Spawn process
├── Return
└── Dead
```

The scheduler repeatedly evaluates the process state.

Conceptually:

```text
process
    │
    ▼
process.state
    │
    ▼
scheduler
    │
    ├── call function
    ├── wait
    ├── send
    ├── spawn
    └── terminate
```

A function does not need to block the scheduler.

Instead, asynchronous operations are represented through process state transitions.

---

# 4. Scheduler

Schedulers execute Hive Processes.

Each scheduler owns a scheduler stack.

```text
Scheduler
│
├── Scheduler Stack
│
└── Run Queue
    ├── Process A
    ├── Process B
    ├── Process C
    └── ...
```

The scheduler is responsible for:

01. Selecting a runnable process.
02. Reading its state.
03. Executing the requested operation.
04. Allocating stack frames when required.
05. Executing or JIT-compiling functions.
06. Updating the process state.
07. Returning the process to the run queue or another scheduler state.

The scheduler is therefore the main runtime component connecting Hive's asynchronous process model with native execution.

---

# 5. Scheduler Stack

The scheduler itself owns a stack.

This stack is different from the memory belonging to a Hive Process.

```text
Scheduler
│
└── Scheduler Stack
```

Hive Processes do not each require their own large operating-system stack.

Instead, the scheduler manages the execution of their functions through stack frames.

This is one of the main differences between Hive Processes and OS threads.

---

# 6. Stack Frames

Every Hive function has a statically known stack-frame layout.

When a process executes:

```text
call foo(...)
```

the scheduler allocates the stack frame required by `foo` .

Conceptually:

```text
Process
│
├── frame: current_function
│
└── frame: foo()
```

After `foo()` returns, its frame can be released.

```text
Process
│
└── frame: current_function
```

The size of a normal Hive stack frame is known ahead of execution.

Therefore, local variables stored inside a frame must have statically known sizes.

This allows stack frames to be allocated very cheaply.

---

# 7. Function Execution

A process state can request a function call:

```text
call foo(a, b)
```

The scheduler handles the transition:

```text
Process State
    │
    ▼
call foo(a, b)
    │
    ▼
allocate stack frame
    │
    ▼
JIT / native execution
    │
    ▼
return
    │
    ▼
new Process State
```

The function can either already have native code available or be JIT-compiled when required.

Hive does not require a bytecode interpreter for normal function execution.

The goal is for the actual function body to execute as native machine code.

---

# 8. JIT Execution

Hive bytecode describes functions and runtime operations, but execution does not have to remain inside a virtual machine interpreter.

A function can be JIT-compiled:

```text
Hive Function
     │
     ▼
   JIT
     │
     ▼
Native Code
     │
     ▼
 CPU
```

After compilation, subsequent executions can directly use the generated native code.

This removes the instruction-dispatch overhead normally associated with an interpreter-based VM.

---

# 9. Static Stack Frames

Normal stack frames have statically known layouts.

For example:

```text
foo(a, b)
│
├── a
├── b
├── local_a
├── local_b
└── temporary
```

The compiler/JIT knows the required frame size.

This makes frame allocation simple:

```text
frame = scheduler.allocate_frame(foo)
```

The scheduler does not need to dynamically discover the size of every local variable during execution.

---

# 10. Dynamic Memory

Static stack frames are not intended to contain every possible kind of data.

A Hive Process can allocate memory from its process-local heap.

```text
Hive Process
│
├── Stack Frames
│
└── Heap
    ├── object
    ├── array
    ├── string
    └── dynamic data
```

The heap is dynamic and can contain objects whose size is not known when the stack frame is created.

---

# 11. Dynamic Stack Memory

The process heap can also be used to support dynamically sized stack-like data.

A function is therefore not restricted to the size of its static stack frame.

For example:

```text
foo()
│
├── Static Stack Frame
│
└── Dynamic Memory
    └── 37 KiB
```

The dynamically allocated region does not need to have a fixed size such as 16 KiB.

The process can request the amount of memory it actually needs.

---

# 12. Process Memory Ownership

Process-local memory belongs to the Hive Process.

```text
Hive Process
│
├── Stack Frames
├── Dynamic Stack Memory
└── Heap
```

This creates a natural memory ownership boundary.

The process owns the memory required by its execution.

Borrowing rules can additionally be enforced by the language and compiler to prevent invalid references while the process is alive.

---

# 13. Borrowing

Hive can use ownership and borrowing rules even though process memory is eventually reclaimed as a whole.

Borrowing provides safety while the process is alive.

For example:

```text
owner
  │
  └── borrow
        │
        ▼
      function
```

The compiler can ensure that borrowed references do not outlive their valid owners.

However, Hive does not require every allocation to have a complex individual garbage-collection lifecycle.

---

# 14. Process Lifetime

The process itself defines a major memory lifetime boundary.

While a process is alive:

```text
Process
│
├── Stack
├── Heap
└── Dynamic Memory
```

When the scheduler detects that the process is dead:

```text
process.state == Dead
```

the entire process can be destroyed.

Conceptually:

```text
process.state == Dead
        │
        ▼
   drop(process)
        │
        ├── drop stack frames
        ├── release dynamic stack memory
        ├── release process heap
        └── release process resources
```

---

# 15. Process Reclamation

Hive does not need a traditional tracing garbage collector for process-local memory.

The scheduler does not need to scan the entire heap and determine which objects are reachable after a process dies.

Instead:

> **If the process is dead, its process-local memory can be dropped with the process.** 

Conceptually:

```text
Process
│
├── Heap
├── Stack Frames
└── Dynamic Memory
        │
        ▼
      Dead
        │
        ▼
  drop(process)
        │
        ▼
      freed
```

This is process-lifetime-based memory reclamation.

It behaves more like an owned memory region than a traditional garbage-collected heap.

---

# 16. Let It Fail

Hive retains the principle of **let it fail** .

A process is allowed to fail independently of other processes.

If a process reaches a fatal state:

```text
Process
    │
    ▼
failure
    │
    ▼
state = Dead
    │
    ▼
drop(process)
```

The runtime can then discard the process-local execution state.

This makes process failure cheap.

There is no requirement to manually unwind every allocation in the process heap before the process can disappear.

The process itself owns the lifetime of that memory.

---

# 17. Asynchronous Execution

Hive is asynchronous by design.

A process can return a state describing that it is waiting for an event.

```text
Process
    │
    ▼
wait for event
    │
    ▼
state = Waiting
```

The scheduler can then execute another process.

When the event occurs:

```text
event
  │
  ▼
process becomes runnable
  │
  ▼
scheduler
  │
  ▼
continue process
```

A waiting process therefore does not require a continuously executing OS thread.

---

# 18. Process Scheduling

A simplified scheduler loop looks like:

```text
while scheduler_running:

    process = next_runnable_process()

    state = process.state

    match state:

        Call(function):
            frame = allocate_frame(function)
            execute(function, frame)
            update_state(process)

        Wait(event):
            register_wait(process, event)

        Send(message):
            send_message(message)
            update_state(process)

        Spawn(process):
            spawn_process(process)
            update_state(process)

        Dead:
            drop(process)
```

The exact implementation can differ, but the fundamental model remains the same:

```text
state
  ↓
scheduler
  ↓
operation
  ↓
new state
```

---

# 19. Message Passing

Hive Processes can communicate without sharing normal process-local state.

Conceptually:

```text
Process A
    │
    │ message
    ▼
Scheduler / Mailbox
    │
    ▼
Process B
```

This allows processes to remain isolated while still supporting BEAM-like concurrent programming.

Process-local memory remains owned by the process that created it.

---

# 20. Small Processes

The architecture is specifically designed around very small processes.

A process does not require:

```text
large OS stack
+
large runtime allocation
+
VM process structure
```

Instead, it primarily consists of:

```text
Process
├── State
├── Small runtime metadata
├── Stack Frames
└── Process Heap
```

This allows Hive to create large numbers of concurrent processes.

The scheduler provides the execution resources rather than permanently assigning a thread to every process.

---

# 21. Hive vs. BEAM

Hive takes inspiration from the BEAM's concurrency model:

```text
BEAM
├── lightweight processes
├── scheduler
├── message passing
├── process isolation
└── let it fail
```

Hive keeps these concepts while changing the execution model:

```text
Hive
├── lightweight processes
├── scheduler
├── message passing
├── process isolation
├── let it fail
├── native execution
└── JIT compilation
```

The major architectural difference is that Hive is not fundamentally dependent on a VM interpreter.

Instead:

```text
Hive Function
     │
     ▼
JIT Compiler
     │
     ▼
Native Machine Code
     │
     ▼
CPU
```

---

# 22. Execution Model

The complete execution flow is:

```text
                 ┌──────────────────┐
                 │   Hive Process   │
                 │                  │
                 │      State       │
                 └────────┬─────────┘
                          │
                          ▼
                 ┌──────────────────┐
                 │    Scheduler     │
                 └────────┬─────────┘
                          │
                  Call Function
                          │
                          ▼
                 ┌──────────────────┐
                 │ Allocate Frame   │
                 └────────┬─────────┘
                          │
                          ▼
                 ┌──────────────────┐
                 │   JIT / Native   │
                 │    Execution     │
                 └────────┬─────────┘
                          │
                          ▼
                 ┌──────────────────┐
                 │   New State      │
                 └────────┬─────────┘
                          │
                          ▼
                    Scheduler
```

The scheduler therefore acts as the bridge between Hive's process model and native CPU execution.

---

# 23. Core Design Principles

Hive is built around several core principles:

01. **Processes are cheap.** 
02. **Processes are asynchronous.** 
03. **Schedulers own execution resources.** 
04. **Functions execute using scheduler-managed stack frames.** 
05. **Normal stack-frame sizes are statically known.** 
06. **Dynamic memory belongs to the process heap.** 
07. **Dynamic stack-like allocations can use process-owned memory.** 
08. **Functions can be JIT-compiled to native code.** 
09. **Process-local memory is reclaimed when the process dies.** 
10. **Borrowing provides safety while the process is alive.** 
11. **Process failure follows the `let it fail` model.** 
12. **There is no requirement for a traditional tracing GC for process-local memory.** 

The resulting architecture can be summarized as:

```text
             ┌───────────────────────┐
             │        Scheduler      │
             │                       │
             │   Scheduler Stack     │
             │   Run Queue           │
             └───────────┬───────────┘
                         │
          ┌──────────────┼──────────────┐
          │              │              │
          ▼              ▼              ▼
      Process A       Process B       Process C
          │              │              │
       State           State           State
          │              │              │
       Frames          Frames          Frames
          │              │              │
        Heap            Heap            Heap
          │              │              │
          └──────────────┼──────────────┘
                         │
                         ▼
                   JIT / Native Code
                         │
                         ▼
                        CPU
```

Hive's fundamental abstraction is therefore:

> **A lightweight asynchronous process whose state is scheduled by a native scheduler and whose functions execute as native code.** 
