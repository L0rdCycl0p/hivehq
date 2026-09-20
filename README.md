# HIVE

**HIVE (Hive Is Very Efficient)** is a lightweight native runtime inspired by the BEAM, designed to run large numbers of small, isolated processes with low runtime overhead.

HIVE aims to provide the benefits of process-based concurrency — lightweight processes, message passing, isolation, and efficient scheduling — without requiring a virtual machine.

Instead of running on a traditional VM, HIVE focuses on native execution and a minimal runtime.

## Features

* Lightweight isolated processes
* Process-based concurrency
* Message passing
* Efficient scheduling
* Native execution
* Low runtime overhead
* Custom `.hive` executable format
* JIT compilation
* Scheduler-managed stack frames
* Shared stack memory between processes
* Designed for large numbers of small processes

## Project Status

HIVE is currently in early development.

**Current version: 0.1.0**

This release marks the first major milestone after weeks of development and establishes the initial runtime, executable format, process model, and execution infrastructure.

The architecture is still evolving and APIs may change significantly between releases.

## Architecture

HIVE is structured around a runtime containing multiple schedulers. Each scheduler can manage multiple processes.

Every scheduler owns its own stack, which is allocated on the heap. The processes assigned to that scheduler share this stack while they are executed.

```text
                         ┌─────────────────────┐
                         │     HIVE Program    │
                         │       .hive         │
                         └──────────┬──────────┘
                                    │
                                    ▼
                         ┌─────────────────────┐
                         │    HIVE Runtime     │
                         └──────────┬──────────┘
                                    │
                ┌───────────────────┼───────────────────┐
                │                   │                   │
                ▼                   ▼                   ▼
        ┌───────────────┐   ┌───────────────┐   ┌───────────────┐
        │  Scheduler 1  │   │  Scheduler 2  │   │  Scheduler N  │
        │               │   │               │   │               │
        │  ┌─────────┐  │   │  ┌─────────┐  │   │  ┌─────────┐  │
        │  │Process 1│  │   │  │Process 4│  │   │  │Process N│  │
        │  │Process 2│  │   │  │Process 5│  │   │  │Process N│  │
        │  │Process 3│  │   │  │Process 6│  │   │  │Process N│  │
        │  └─────────┘  │   │  └─────────┘  │   │  └─────────┘  │
        │       │       │   │       │       │   │       │       │
        │       ▼       │   │       ▼       │   │       ▼       │
        │ ┌───────────┐ │   │ ┌───────────┐ │   │ ┌───────────┐ │
        │ │Heap Stack │ │   │ │Heap Stack │ │   │ │Heap Stack │ │
        │ │           │ │   │ │           │ │   │ │           │ │
        │ │ ┌───────┐ │ │   │ │ ┌───────┐ │ │   │ │ ┌───────┐ │ │
        │ │ │ Frame │ │ │   │ │ │ Frame │ │ │   │ │ │ Frame │ │ │
        │ │ ├───────┤ │ │   │ │ ├───────┤ │ │   │ │ ├───────┤ │ │
        │ │ │ Frame │ │ │   │ │ │ Frame │ │ │   │ │ │ Frame │ │ │
        │ │ ├───────┤ │ │   │ │ ├───────┤ │ │   │ │ ├───────┤ │ │
        │ │ │ Frame │ │ │   │ │ │ Frame │ │ │   │ │ │ Frame │ │ │
        │ │ └───────┘ │ │   │ │ └───────┘ │ │   │ | └───────┘ │ |
        │ └───────────┘ │   │ └───────────┘ │   │ └───────────┘ │
        └───────────────┘   └───────────────┘   └───────────────┘
```

A scheduler can therefore manage many processes without allocating a separate stack for every process.

### Schedulers

Schedulers are the execution units of HIVE. Each scheduler manages a set of processes and owns one heap-allocated stack.

```text
Scheduler
    │
    ├── Process A
    ├── Process B
    ├── Process C
    ├── Process D
    │
    └── Scheduler Stack
             │
             ├── Function Frame
             ├── Function Frame
             └── Function Frame
```

The scheduler switches between its processes and uses its stack for their current execution.

Multiple schedulers can operate independently:

```text
                 HIVE Runtime
                      │
          ┌───────────┼───────────┐
          ▼           ▼           ▼
     Scheduler 1  Scheduler 2  Scheduler N
          │           │           │
       Processes   Processes   Processes
          │           │           │
      Heap Stack  Heap Stack  Heap Stack
```

### Shared Scheduler Stacks

The stack belongs to the **scheduler**, not to an individual process.

When a process needs to execute a function, the scheduler creates the required function frame on its stack. When execution switches to another process, the stack can be used for that process as well.

```text
                 Scheduler
                     │
                     ▼
             ┌───────────────┐
             │  Heap Stack   │
             ├───────────────┤
             │ Process A     │
             │ Function      │
             │ Frame         │
             ├───────────────┤
             │ Process A     │
             │ Function      │
             │ Frame         │
             └───────────────┘

                     │
              process switch
                     ▼

             ┌───────────────┐
             │  Heap Stack   │
             ├───────────────┤
             │ Process B     │
             │ Function      │
             │ Frame         │
             ├───────────────┤
             │ Process B     │
             │ Function      │
             │ Frame         │
             └───────────────┘
```

Function frame sizes are statically known. This allows the scheduler to determine how much stack space is required when a function is executed.

The result is that many processes can share the stack memory belonging to their scheduler instead of requiring every process to have its own permanently allocated **8MB OS-stack**.

## Native Execution

HIVE does not require a traditional virtual machine to execute its programs.

The `.hive` executable is loaded by the runtime, while the schedulers manage processes and their execution. The HIVE JIT compiles code to native machine code.

```text
        .hive executable
               │
               ▼
        ┌─────────────┐
        │    Loader   │
        └──────┬──────┘
               │
               ▼
        ┌─────────────┐
        │     JIT     │
        └──────┬──────┘
               │
               ▼
        ┌─────────────┐
        │ Native Code │
        └──────┬──────┘
               │
               ▼
              CPU
```

The runtime therefore provides the process and scheduling model while the actual program execution uses native machine code.

## Concept

HIVE is heavily inspired by the process model of the BEAM.

Instead of relying on operating-system threads for every unit of concurrent work, HIVE provides its own lightweight processes managed by the runtime.

The goal is to make it practical to run a very large number of small processes while keeping scheduling and runtime overhead low.

Unlike the BEAM, HIVE does not require a virtual machine to execute its programs. HIVE is designed around native execution and JIT compilation.

## Installation

HIVE can be installed from crates.io:

```bash
cargo install hivehq
```

Or built from source:

```bash
git clone https://github.com/L0rdCycl0p/hivehq.git
cd hivehq
cargo build --release
```

## Usage

Run a HIVE program with:

```bash
hive program.hive
```

## Development

Build HIVE in debug mode:

```bash
cargo build
```

Run the test suite:

```bash
cargo test
```

Build an optimized release:

```bash
cargo build --release
```

## License

HIVE is licensed under the **GNU General Public License v3.0**.

See [`LICENSE`](LICENSE) for the complete license text.
