[bits 64]

section .data
message: db "WildflowerOS is rebooting.", 0xa
message_len: equ $ - message

section .text
global _start
_start:
  ; Write message to stdout
  mov rax, 4            ; syscall: write
  mov rdi, 1            ; stdout
  lea rsi, [rel message]
  mov rdx, message_len
  int 0x80

  ; Call the kernel STOP syscall (0x0A) to trigger reboot with exit code 0xCAFE
  mov rax, 0x0A         ; syscall: stop
  mov rdi, 0xCAFE       ; exit/reboot code
  int 0x80