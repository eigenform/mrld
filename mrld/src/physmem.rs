//! Abstractions for dealing with physical memory. 
//!
//! Notes
//! =====
//!

use core::ops::Range;

///// Base of physical memory allocated (by the bootloader) for [`MrldBootArgs`].
//pub const BOOT_ARGS_PHYS_BASE: u64  = 0x0000_0000_0010_0000;

///// Base of physical memory allocated (by the bootloader) for page tables. 
//pub const PAGE_TABLE_PHYS_BASE: u64 = 0x0000_0000_0020_0000;

///// Base of physical memory allocated (by the bootloader) for the kernel.
//pub const KERNEL_PHYS_BASE: u64     = 0x0000_0000_0020_0000;


/// Describes the kind of physical memory region presented to the kernel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum MrldMemoryKind { 
    Invalid = 0,

    /// Physical memory available for use by the kernel. 
    Available = 1,

    /// Physical memory allocated by UEFI firmware and/or the bootloader. 
    /// Eligible to be reclaimed by the kernel. 
    Reclaimable = 2,

    /// Arguments passed from the bootloader to the kernel. 
    BootArgs = 3,

    /// Physical memory allocated by the bootloader for the kernel image. 
    KernelImage = 4,

    /// Advertised as "ACPI non-volatile" by UEFI firmware
    AcpiNonVolatile = 5,

    /// UEFI runtime services
    UefiRuntime = 7,

    /// Advertised as "reserved" by UEFI firmware
    UefiReserved = 255,
}

/// Describes a physical memory region presented to the kernel at boot-time.
#[derive(Debug)]
#[repr(C)]
pub struct MrldMemoryDesc {
    /// The kind of memory region.
    pub kind: MrldMemoryKind,
    /// The physical address range defining this region.
    pub range: Range<u64>,
}
impl MrldMemoryDesc { 
    pub fn is_valid(&self) -> bool { 
        (self.kind != MrldMemoryKind::Invalid) && 
        self.range.start != 0 && self.range.end != 0
    }
    pub fn new_invalid() -> Self { 
        Self { 
            kind: MrldMemoryKind::Invalid, 
            range: 0..0,
        } 
    }
}

/// Describes a set of physical memory regions presented to the kernel.
#[repr(C)]
pub struct MrldMemoryMap { 
    /// Set of physical memory regions.
    pub entries: [MrldMemoryDesc; Self::NUM_ENTRIES],
}
impl MrldMemoryMap { 
    /// Fixed number of entries in this memory map.
    ///
    /// WARNING: The memory map presented after exiting UEFI boot services 
    /// may have more entries. The bootloader is responsible for merging 
    /// regions so that they fit into this map. 
    pub const NUM_ENTRIES: usize = 128;

    /// Create a new [empty] memory map. 
    pub fn new_empty() -> Self { 
        Self { 
            entries: core::array::from_fn(|_| MrldMemoryDesc::new_invalid()),
        }
    }
}


