section .multiboot_header
header_start:
    dd 0xe85250d6                ; magic number
    dd 0                         ; protected mode code
    dd header_end - header_start ; header length

    ; checksum
    dd 0x100000000 - (0xe85250d6 + 0 + (header_end - header_start))

    ; required end tag
    dw 0    ; type
    dw 0    ; flags
    dd 8    ; size
header_end:

section .bss
align 16
stack_bottom:
	resb 16384 ; 16 KiB
stack_top:

section .text
global _start
bits 32
_start:
    mov esp, stack_top

	; GDT
	; PAGING

	extern kernel_main
	jmp kernel_main
	
	cli
.hang:	hlt
	jmp .hang
.end:
