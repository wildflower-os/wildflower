[bits 64]

section .data
msg: db 27, "[93m", "Halting WildflowerOS.", 27, "[0m", 10
len: equ $-msg

global _start
section .text
_start:
  mov rax, 4                  ; syscall number for WRITE
  mov rdi, 1                  ; standard output
  mov rsi, msg                ; addr of string
  mov rdx, len                ; size of string
  int 0x80

  mov rax, 0xB                ; syscall number for SLEEP
  mov rdi, __?float64?__(0.5) ; duration
  int 0x80

  mov rax, 0xA                ; syscall number for STOP
  mov rdi, 0xDEAD             ; halt code
  int 0x80
