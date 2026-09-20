;; HIVE (Hive Is Very Efficient)
;; Copyright (C) 2026 L0rdCycl0p
;;
;; This program is free software: you can redistribute it and/or modify
;; it under the terms of the GNU General Public License as published by
;; the Free Software Foundation, either version 3 of the License, or
;; (at your option) any later version.
;;
;; This program is distributed in the hope that it will be useful,
;; but WITHOUT ANY WARRANTY; without even the implied warranty of
;; MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
;; GNU General Public License for more details.
;;
;; You should have received a copy of the GNU General Public License
;; along with this program. If not, see <https://www.gnu.org/licenses/>.


BITS 64

global hive_context_switch

section .text

; void hive_context_switch(
;     HiveContext *old,       ; RDI
;     HiveContext *new,       ; RSI
;     void *return_addr       ; RDX
; )

hive_context_switch:

    ; ============================================================
    ; Save old context
    ; ============================================================

    mov [rdi + 0],   rax
    mov [rdi + 8],   rbx
    mov [rdi + 16],  rcx
    mov [rdi + 24],  rdx
    mov [rdi + 32],  rsi
    mov [rdi + 40],  rdi
    mov [rdi + 48],  rbp
    mov [rdi + 56],  rsp

    mov [rdi + 64],  r8
    mov [rdi + 72],  r9
    mov [rdi + 80],  r10
    mov [rdi + 88],  r11
    mov [rdi + 96],  r12
    mov [rdi + 104], r13
    mov [rdi + 112], r14
    mov [rdi + 120], r15

    ; RIP
    mov [rdi + 128], rdx


    ; ============================================================
    ; Keep new context pointer
    ; ============================================================

    mov r10, rsi


    ; ============================================================
    ; Load new context
    ; ============================================================

    mov rax, [r10 + 0]
    mov rbx, [r10 + 8]
    mov rcx, [r10 + 16]
    mov rdx, [r10 + 24]

    mov rsi, [r10 + 32]
    mov rdi, [r10 + 40]
    mov rbp, [r10 + 48]

    mov rsp, [r10 + 56]

    mov r8,  [r10 + 64]
    mov r9,  [r10 + 72]

    mov r11, [r10 + 88]
    mov r12, [r10 + 96]
    mov r13, [r10 + 104]
    mov r14, [r10 + 112]
    mov r15, [r10 + 120]


    ; ============================================================
    ; Load RIP
    ; ============================================================

    mov r10, [r10 + 128]

    jmp r10