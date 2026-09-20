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

#[derive(Debug, Clone, Copy)]
pub struct ContextSwitchPatch {
    pub patch_offset: usize,
    pub rip_offset: usize,
}

#[must_use]
pub fn gen_context_switch(offset: usize) -> (Vec<u8>, ContextSwitchPatch) {
    let context_switch_addr = hive_context_switch as *const () as usize;
    let mut code = Vec::new();

    // mov rdi, [rsp + 16]
    code.extend_from_slice(&[0x48, 0x8B, 0x7C, 0x24, 0x10]);

    // mov rsi, [rsp + 8]
    code.extend_from_slice(&[0x48, 0x8B, 0x74, 0x24, 0x08]);

    // lea rcx, [rip + disp32]
    //
    // 48 8D 0D xx xx xx xx
    //
    // xx xx xx xx = Patch
    let lea_start = code.len();

    code.extend_from_slice(&[0x48, 0x8D, 0x0D, 0x00, 0x00, 0x00, 0x00]);

    let rip_offset = code.len();

    // mov rax, <context_switch_addr>
    code.extend_from_slice(&[0x48, 0xB8]);
    code.extend_from_slice(&context_switch_addr.to_le_bytes());

    // jmp rax
    code.extend_from_slice(&[0xFF, 0xE0]);

    (
        code,
        ContextSwitchPatch {
            patch_offset: offset + lea_start + 3,

            rip_offset: offset + rip_offset,
        },
    )
}

pub fn patch_context_switches(address: usize, patches: &[ContextSwitchPatch]) {
    for patch in patches {
        let patch_address = address + patch.patch_offset;
        let rip_address = address + patch.rip_offset;

        let next_rip = patch_address + 4;

        let displacement = rip_address as isize - next_rip as isize;

        assert!(
            i32::try_from(displacement).is_ok(),
            "RIP-relative displacement does not fit in i32"
        );

        unsafe {
            std::ptr::write_unaligned(patch_address as *mut i32, displacement as i32);
        }
    }
}

unsafe extern "C" {
    unsafe fn hive_context_switch();
}
