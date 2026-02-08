//! Kernel entrypoint. 
//!
//! - Load the new GDT and switch into the new code/data segments
//! - Set the kernel stack pointer
//! - Jump into Rust code
//!
//! Implementation Notes
//! ====================
//!
//! Should be easy to turn all of this into Rust code if/when desirable.
//! Use of `global_asm!` here is mostly a matter of taste. 
//!

use mrld::MrldBootArgs;
use mrld::x86::gdt::{
    Descriptor, DFlags, GlobalDescriptorTable,
    KERNEL_CODE_SEL,
    KERNEL_DATA_SEL,
};
use mrld::x86::dtr::{DescriptorTableRegister};
use mrld::x86::segment::{SegmentSelector, PrivilegeLevel};

//pub const KERNEL_CODE_SEL: SegmentSelector = 
//    SegmentSelector::new(1, false, PrivilegeLevel::Ring0);
//pub const KERNEL_DATA_SEL: SegmentSelector = 
//    SegmentSelector::new(2, false, PrivilegeLevel::Ring0);


/// Kernel entrypoint. 
///
/// The bootloader jumps here. 
///
/// Note that page tables should have already been configured by the time
/// we've entered this function. 
#[unsafe(link_section = ".start")]
#[unsafe(no_mangle)]
#[allow(named_asm_labels)]
#[unsafe(naked)]
pub extern "sysv64" fn _start(args: *const MrldBootArgs) -> ! { unsafe { 
    core::arch::naked_asm!(r#"
        // Disable interrupts
        cli

        // Load a new GDT (defined statically in the .start.gdt section)
        lgdt [KERNEL_GDTR]

        // Switch out data segments
        mov ax, {data_sel}
        mov ds, ax
        mov es, ax
        mov fs, ax
        mov gs, ax
        mov ss, ax

        // Switch out code segment (using the far return `retfq`)
        push {code_sel}
        lea rax, [2f]
        push rax
        retfq
    2:

        // Set the kernel stack pointer
        movabs rsp, offset _kernel_stack_hi
        sub rsp, 4096

        // Jump into the kernel - execution continues in 'src/main.rs'
        jmp {main}

    3:
        // Unreachable
        ud2
        jmp 3b
    "#,

    data_sel = const KERNEL_DATA_SEL.as_u16(),
    code_sel = const KERNEL_CODE_SEL.as_u16(),
    main = sym crate::kernel_main,
    );
} }


const KERNEL_TEXT_DESC: Descriptor = Descriptor::new(
    0x0000_0000, PrivilegeLevel::Ring0, 0xffff, DFlags::CODE
);
const KERNEL_DATA_DESC: Descriptor = Descriptor::new(
    0x0000_0000, PrivilegeLevel::Ring0, 0xffff, DFlags::DATA
);

// Build the '.start.gdt' section contents.
core::arch::global_asm!(r#"
.section .start.gdt
.global KERNEL_GDT
.align 64
KERNEL_GDT:
    .quad 0x0000000000000000
    .quad {text}
    .quad {data}

.align 64
.global KERNEL_GDTR
KERNEL_GDTR:
    .word (KERNEL_GDTR - KERNEL_GDT - 1)
    .quad KERNEL_GDT
"#,
text = const KERNEL_TEXT_DESC.as_u64(),
data = const KERNEL_DATA_DESC.as_u64(),
);


