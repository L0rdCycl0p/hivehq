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

//! HIVE (Hive Is Very Efficient) — a lightweight native runtime inspired by BEAM, designed to run large numbers of small, isolated processes without a virtual machine.

#[cfg(not(target_os = "linux"))]
compile_error!(
    "HIVE only supports Linux. \
    HIVE is intentionally designed as a Linux-only runtime because it directly \
    relies on Linux system interfaces such as mmap and mprotect, and is primarily \
    intended for server workloads where Linux is the dominant platform. \
    Restricting HIVE to Linux allows the runtime to specialize for performance, \
    reduce platform-specific bugs, avoid unnecessary development and maintenance \
    effort, and keep the overall architecture and implementation complexity under \
    control instead of introducing abstractions for operating systems that HIVE \
    is not designed to support."
);
#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
compile_error!("HIVE currently supports only x86_64 Linux.");

/// Debug Tools
pub mod debug;
/// CLI Parsing with clap
pub mod cli;
/// errors
pub mod error;
/// Just in Time compilation
pub mod jit;
/// `.hive` parser (binrw)
pub mod parser;
/// Execution runtime
pub mod runtime;
pub use debug::gdb_marker;
