.intel_syntax noprefix

// Hack for serial output in QEMU
.macro com2 byte
    mov dx, 0x2f8 
    mov al, \byte
    out dx, al
.endm

// ==============================================================
.section .text.code16
.code16

.set CODE_DESC, 0x0008
.set DATA_DESC, 0x0010

.global _trampoline_start
_trampoline_start:
    // Disable interrupts
    cli

    com2 0x30

    // Clear all of the data segment selectors
	// Load the provisional GDT
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    lgdt ds:_trampoline_gdtr

    com2 0x31

    // Enable protected mode
    mov eax, cr0
    or eax, 1
    mov cr0, eax

    com2 0x32

	// Jump into 
    jmp far ptr CODE_DESC:_trampoline_start32


.align 64
.code32
.global _trampoline_start32
_trampoline_start32:
    com2 0x33

	// Set up all of the data segments
    xor eax, eax
    mov ax, DATA_DESC
    mov es, ax
    mov ds, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    nop
    nop
    nop
    nop
    nop
    nop
    nop
1:
    pause
    jz 1b


.section .data
.align 64
.global _trampoline_gdt
_trampoline_gdt:
    .quad 0x0000000000000000
    .quad 0x00cf9a000000ffff
    .quad 0x00cf93000000ffff
_trampoline_gdt_end:


// Protected mode GDT 
.align 64
.global _trampoline_gdtr
_trampoline_gdtr:
    .word (_trampoline_gdt_end - _trampoline_gdt - 1)
    .long _trampoline_gdt


.section .end

