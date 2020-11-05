global start

section .text
bits 32
start:
    mov esp, stack_top
	extern kernel_main
	jmp kernel_main
	cli
.hang:	hlt
	jmp .hang
.end:

section .bss
align 16
stack_bottom:
	resb 16384 ; 16 KiB
stack_top:

