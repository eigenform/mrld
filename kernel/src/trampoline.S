.intel_syntax noprefix

.set MAGIC, 0xb007c0de

.set MSR_EFER, 0xc0000080
.set EFER_LME, (1 << 8)

// Protected mode
.set CR0_PE, (1 << 0)
// Cache Disable
.set CR0_CD, (1 << 30)
// Paging
.set CR0_PG, (1 << 31)

/// Debug Enable
.set CR4_DE,  (1 << 3)
// Physical Address Extensions
.set CR4_PAE, (1 << 5)
// Page-Global Enable
.set CR4_PGE,  (1 << 7)
// PMC enable
.set CR4_PCE, (1 << 8)

// Hack for serial output in QEMU
.macro com2 byte
    mov dx, 0x2f8 
    mov al, \byte
    out dx, al
.endm

// NOTE: The linkerscript limits the size of the .text section to 512 bytes.
// (Hopefully the linker will complain if you accidentally exceed the limit)

.section .text.code16
.code16

.set CODE_DESC, 0x0008
.set DATA_DESC, 0x0010

_trampoline_start:
    // Disable interrupts
    cli

	mov eax, [_header_magic]
	cmp eax, MAGIC
	jne _trampoline_dead
    com2 0x30

    // Clear all of the data segment selectors
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax

	// Load the provisional GDT
    lgdt ds:_trampoline_gdtr
    com2 0x31

    // Enable protected mode
    mov eax, cr0
    or eax, CR0_PE
    mov cr0, eax
    com2 0x32

	// Far jump into protected mode
    jmp far ptr CODE_DESC:_trampoline_start32

.align 16
_trampoline_dead:
	com2 0x44
	com2 0x45
	com2 0x41
	com2 0x44
	hlt

.align 16
.code32
_trampoline_start32:
	// Set up all of the data segments
    xor eax, eax
    mov ax, DATA_DESC
    mov es, ax
    mov ds, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    com2 0x33

	// Enable PAE
	mov eax, cr4
	or eax, (CR4_PAE | CR4_PGE)
	mov cr4, eax
    com2 0x34

	// Use PML4 from the bootstrap core
	// NOTE: This might rely on BSC tables being mapped at a 32-bit address?
	mov eax, dword ptr [_header_pml4]
	mov cr3, eax
    com2 0x35

	// Enable long mode
	mov ecx, MSR_EFER
	rdmsr
	or eax, EFER_LME
	wrmsr
    com2 0x36

	// Enable paging
	mov eax, cr0
	or eax, CR0_PG
	mov cr0, eax
    com2 0x37

	// Load provisional GDT (again)
	lgdt [_trampoline_gdtr64]
    com2 0x38

	// Switch to new data segment
    mov ax, DATA_DESC
    mov es, ax
    mov ds, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

	// Enter long mode
    jmp far ptr CODE_DESC:_trampoline_start64

.code64
_trampoline_start64:
    com2 0x39
	lea rax, [_header_stack_base]
	mov rsp, [rax]
	sub rsp, 0x40

    com2 0x40
    com2 0x0a

	lea rax, [_header_entry]
	jmp [rax]

// NOTE: The linkerscript always places this section at some fixed offset.
// These values are replaced by the kernel during runtime.
// The layout of this section *must* match the 'TrampolineHeader' type
// defined in 'kernel/src/trampoline.rs' and whatever is in the linkerscript. 
.section .data.header
_header_magic:
	.long 0
_header_pml4:
	.long 0
_header_entry:
	.quad 0
_header_stack_base:
	.quad 0

// ========================================================
.section .data.gdt
.global _trampoline_gdt32
_trampoline_gdt32:
    .quad 0x0000000000000000
    .quad 0x00cf9a000000ffff
    .quad 0x00cf93000000ffff
_trampoline_gdt32_end:

.align 64
.global _trampoline_gdtr32
_trampoline_gdtr:
    .word (_trampoline_gdt32_end - _trampoline_gdt32 - 1)
    .long _trampoline_gdt32

.align 64
.global _trampoline_gdt64
_trampoline_gdt64:
    .quad 0x0000000000000000
    .quad 0x00209a0000000000
    .quad 0x0000920000000000
_trampoline_gdt64_end:

.align 64
.global _trampoline_gdtr64
_trampoline_gdtr64:
    .word (_trampoline_gdt64_end - _trampoline_gdt64 - 1)
    .long _trampoline_gdt64


