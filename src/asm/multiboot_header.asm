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
extern kernel_main
global _start
bits 32
_start:
    mov esp, stack_high
	xor ebp, ebp

	; Save caller state
	push eax
	push ecx
	push edx

	; push arguments
	; https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Boot-information-format
	push ebx ; physical address of the Multiboot2 information structure
	push eax ; magic value for MultiBoot2 compliant bootloader, 0x36d76289

	call kernel_main

	; pop arguments
	pop eax
	pop eax

	; Restore caller state
	pop edx
	pop ecx
	pop eax

	cli
.hang:	hlt
	jmp .hang
.end:
