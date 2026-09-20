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


#[doc(hidden)]
#[macro_export]
macro_rules! __gdb_marker {
    ($name:ident) => {
        #[cfg(feature = "debug")]
        unsafe {
            #![allow(named_asm_labels)]
            std::arch::asm!(
                concat!(".globl ", stringify!($name), "\n", stringify!($name), ":"),
                options(nostack, preserves_flags)
            );
        }
    };
}

#[doc(inline)]
pub use __gdb_marker as gdb_marker;
