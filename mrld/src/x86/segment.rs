//! Types for handling x86 virtual memory segmentation.

use bitflags::bitflags;



/// Type whose variants represent a descriptor table.
pub enum TableIndicator { 
    Global = 0,
    Local  = 1,
}

/// Type whose variants represent x86 privilege levels. 
pub enum PrivilegeLevel { 
    Ring0 = 0,
    Ring1 = 1,
    Ring2 = 2,
    Ring3 = 3,
}
impl PrivilegeLevel { 
    pub fn from_u16(x: u16) -> Self { 
        match x { 
            0 => Self::Ring0,
            1 => Self::Ring1,
            2 => Self::Ring2,
            3 => Self::Ring3,
            _ => panic!("Invalid privilege level?"),
        }
    }
}

/// A "segment selector" for an entry in the global/local descriptor table.
pub struct SegmentSelector(u16);
impl SegmentSelector { 
    const RPL_MASK: u16 = 0b0000_0000_0000_0011;
    const TI_BIT: u16   = 0b0000_0000_0000_0100;
    const SI_MASK: u16  = 0b1111_1111_1111_1000;

    /// Create a new selector. 
    pub const fn new(index: u16, table: TableIndicator, rpl: PrivilegeLevel) 
        -> Self 
    { 
        Self(index << 3 | (table as u16) << 2 | rpl as u16)
    }

    /// Index into the associated descriptor table.
    pub fn index(&self) -> usize { 
        (self.0 >> 3) as usize
    }

    /// Byte offset into the associated descriptor table. 
    pub fn byte_index(&self) -> usize { 
        (self.0 & Self::SI_MASK) as usize
    }

    /// The descriptor table referenced by this selector.
    pub fn table_indicator(&self) -> TableIndicator {
        if self.0 & Self::TI_BIT != 0 {
            TableIndicator::Local
        } else {
            TableIndicator::Global
        }
    }
}

bitflags! { 
    pub struct DTEFlags: u64 { 
        /// Granularity
        const G   = (1 << 55);
        /// Default operand size
        const D   = (1 << 54);
        /// Long mode
        const L   = (1 << 53);
        /// Present
        const P   = (1 << 47);
        /// Descriptor privilege level
        const DPL = (0b11 << 45);
        /// User segment
        const S   = (1 << 44);
        /// Conforming
        const C   = (1 << 42);
        /// Readable
        const R   = (1 << 41);
        /// Accessed
        const A   = (1 << 40);

    }
}

/// Structure representing the local/global descriptor table registers.
///
/// NOTE: x86 instructions for accessing the descriptor table registers 
/// (`{L,S}GDT` and `{L,S}IDT`) expect a pointer to this structure.
///
#[repr(C, packed)]
pub struct DescriptorTablePointer {
    /// The number of bytes in the descriptor table
    pub size: u16,
    /// The virtual address of the descriptor table
    pub ptr: u64,
}
impl DescriptorTablePointer { 
    pub fn ptr(&self) -> u64 { 
        self.ptr
    }
    pub fn size(&self) -> u16 { 
        self.size
    }

    pub const fn new(size: u16, ptr: u64) -> Self { 
        Self { size, ptr } 
    }
}

/// Helper for interacting with the global descriptor table (GDT).
pub struct GDT;
impl GDT { 
    /// Read the GDT register, returning a [`DescriptorTablePointer`] with 
    /// the location and size of the GDT. 
    pub unsafe fn read() -> DescriptorTablePointer { 
        let res = DescriptorTablePointer::new(0, 0);
        core::arch::asm!(
            "sgdt [{}]",
            in(reg) &res
        );
        res
    }
}

/// Helper for interacting with the interrupt descriptor table (IDT).
pub struct IDT;
impl IDT { 
    /// Read the IDT register, returning a [`DescriptorTablePointer`] with 
    /// the location and size of the IDT. 
    pub unsafe fn read() -> DescriptorTablePointer { 
        let res = DescriptorTablePointer::new(0, 0);
        core::arch::asm!(
            "sidt [{}]",
            in(reg) &res
        );
        res
    }
}



/// A 64-bit entry in a descriptor table. 
pub struct DescriptorTableEntry(u64);
impl DescriptorTableEntry {
    pub const fn from_u64(x: u64) -> Self { 
        Self(x)
    }
}

/// The "global" descriptor table. 
pub struct GlobalDescriptorTable<const SZ: usize> { 
    entries: [DescriptorTableEntry; SZ],
}
impl <const SZ: usize> GlobalDescriptorTable<SZ> {
    //pub const fn new() -> Self { 
    //    Self { 
    //    }
    //}
}

pub struct LocalDescriptorTable { 
}



