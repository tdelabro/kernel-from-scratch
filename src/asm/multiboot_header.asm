section .multiboot_header
align 64
header_start:
    dd 0xe85250d6                ; magic number
    dd 0                         ; 32-bit (protected) mode of i386
    dd header_end - header_start ; header length

    ; checksum
    dd 0x100000000 - (0xe85250d6 + 0 + (header_end - header_start))

    ; required end tag
    dw 0    ; type
    dw 0    ; flags
    dd 8    ; size
header_end:

section .bss
global stack_high
global stack_low
align 16
stack_low:
	resb 16384 ; 16 KiB
stack_high:

section .text
global _start
extern kernel_main
bits 32
_start:
    mov esp, stack_high
	xor ebp, ebp
	mov edi, ebx

	pushad
	call kernel_main
	popad

	cli
.hang:	hlt
	jmp .hang
.end:
