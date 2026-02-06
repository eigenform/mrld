//! Types for interacting with x86 descriptor table registers.

/// Structure representing the contents of the interrupt/global descriptor 
/// table registers (GDTR/IDTR). 
///
/// The GDTR/IDTR are 80-bit registers that contain: 
///
/// - The number of bytes in the target descriptor table, minus one
/// - The virtual address of a a descriptor table
///
/// The x86 instructions for accessing the GDTR (`{L,S}GDT`) and IDTR 
/// (`{L,S}IDT`) expect a pointer to values of this type.
///
#[derive(Debug)]
#[repr(C, packed)]
pub struct DescriptorTableRegister {
    /// The number of bytes in the descriptor table, minus one. 
    limit: u16,
    /// The virtual address of the descriptor table
    ptr: *const u64,
}
impl DescriptorTableRegister { 
    pub const fn as_ptr(&self) -> *const Self { 
        self as *const Self
    }

    /// Return the virtual address of the associated descriptor table.
    pub const fn ptr(&self) -> *const u64 { 
        self.ptr
    }

    /// Return the 16-bit size of the associated descriptor table.
    pub const fn limit(&self) -> u16 { 
        self.limit
    }

    /// Return the size of the associated descriptor table (in bytes).
    pub const fn size(&self) -> usize { 
        (self.limit as usize) + 1
    }
    pub const unsafe fn new(limit: u16, ptr: *const u64) 
        -> DescriptorTableRegister 
    { 
        DescriptorTableRegister { limit, ptr }
    }

}

/// Helper for interacting with the global descriptor table register (GDTR).
pub struct GDTR;
impl GDTR { 

    /// Read the GDTR, returning a [`DescriptorTableRegister`].
    pub unsafe fn read() -> DescriptorTableRegister { 
        let mut res = DescriptorTableRegister { 
            limit: 0,
            ptr: core::ptr::null(),
        };
        core::arch::asm!(
            "sgdt [{}]",
            in(reg) &mut res,
            options(nostack, preserves_flags),
        );
        res
    }

    /// Write the GDTR.
    pub unsafe fn write(gdt: &DescriptorTableRegister) {
        core::arch::asm!(
            "lgdt [{}]",
            in(reg) gdt, 
            options(readonly, nostack, preserves_flags),
        );
    }
}

/// Helper for interacting with the interrupt descriptor table register (IDTR).
pub struct IDTR;
impl IDTR { 
    /// Read the IDTR, returning a [`DescriptorTableRegister`].
    pub unsafe fn read() -> DescriptorTableRegister { 
        let mut res = DescriptorTableRegister { 
            limit: 0,
            ptr: core::ptr::null(),
        };
        core::arch::asm!(
            "sidt [{}]",
            in(reg) &mut res,
            options(nostack, preserves_flags),
        );
        res
    }

    /// Write the IDTR.
    pub unsafe fn write(idt: &DescriptorTableRegister) {
        core::arch::asm!(
            "lidt [{}]",
            in(reg) idt, 
            options(readonly, nostack, preserves_flags),
        );
    }
}


