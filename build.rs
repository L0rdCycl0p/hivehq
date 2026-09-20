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

use std::{env, path::PathBuf, process::Command};

fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let obj = out.join("jit.x86_64.context_switch.o");

    let status = Command::new("nasm")
        .args(["-f", "elf64", "src/jit/x86_64/context_switch.asm", "-o"])
        .arg(&obj)
        .status()
        .expect("failed to execute nasm");

    assert!(status.success());

    println!("cargo:rustc-link-arg={}", obj.display());
    println!("cargo:rerun-if-changed=src/jit/x86_64/context_switch.asm");
}
