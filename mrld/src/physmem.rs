//! [Poor] abstractions for dealing with physical memory. 
//!

use crate::paging::PageSize;

use uefi_raw::table::boot::{
    MemoryType, MemoryAttribute, MemoryDescriptor
};


/// Describing a range of physical memory addresses. 
#[derive(Copy, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
#[repr(C, packed)]
pub struct PhysRange { 
    /// The start of this region
    start: u64,
    /// The end of this region
    end: u64,
}
impl PhysRange {
    pub const fn new(start: u64, end: u64) -> Self { 
        assert!(end >= start);
        Self { start, end } 
    }

    pub fn start(&self) -> u64 { self.start }
    pub fn end(&self) -> u64 { self.end }
    pub fn size(&self) -> u64 { self.end - self.start }

    /// Returns 'true' if 'other' is contained in this range
    pub fn contains_range(&self, other: &Self) -> bool { 
        other.start >= self.start && other.end <= self.end
    }

    /// Returns 'true' if the given address is contained in this range
    pub fn contains(&self, addr: u64) -> bool { 
        addr >= self.start && addr <= self.end
    }

    pub fn aligned_to_start(&self, other: &Self) -> bool { 
        self.start == other.start && other.end <= self.end
    }
    pub fn aligned_to_end(&self, other: &Self) -> bool { 
        self.end == other.end && other.start >= self.start
    }

    /// Returns 'true' if this range is aligned to the given page size.
    pub fn is_page_aligned(&self, pagesz: PageSize) -> bool { 
        (self.start & (u64::from(pagesz) - 1)) == 0
    }

    pub fn num_pages(&self, pagesz: PageSize) -> usize { 
        (self.size() / u64::from(pagesz)) as usize
    }

    /// If this region contains the requested number of contiguous physical
    /// pages, returns the lowest [`PhysRange`] describing the requested region. 
    pub fn try_get_pages(&self, pagesz: PageSize, cnt: usize) 
        -> Option<Self>
    { 
        let bytes = u64::from(pagesz);

        // Find the nearest aligned address
        let start_aligned = self.start.next_multiple_of(bytes);

        // This range isn't large enough to accomodate the alignment
        if start_aligned >= self.end { 
            return None;
        }
        let end = start_aligned + (bytes * (cnt as u64));

        // This range isn't large enough to accomodate the requested
        // number of pages
        if end >= self.end { 
            return None;
        }

        Some(Self::new(start_aligned, end))
    }

    // Compute the difference between this range and 'other', returning 
    // a [`PhysRangeSet`] that describes the resulting ranges. 
    pub fn split(&self, other: &Self) -> PhysRangeSet { 
        // 'other' overlaps the lower-part of this range
        if self.aligned_to_start(other) {
            PhysRangeSet::Pair { 
                new: PhysRange::new(other.start, other.end),
                old: PhysRange::new(other.end, self.end),
            }
        } 
        // 'other' overlaps the upper-part of this range
        else if self.aligned_to_end(other) {
            PhysRangeSet::Pair { 
                old: PhysRange::new(self.start, other.start),
                new: PhysRange::new(other.start, self.end),
            }
        }
        // 'other' is contained within this range
        else if self.contains_range(other) { 
            PhysRangeSet::Triad { 
                old_lo: PhysRange::new(self.start, other.start),
                new: PhysRange::new(other.start, other.end),
                old_hi: PhysRange::new(other.end, self.end),
            }
        }
        // 'other' is not contained in this range
        else { 
            PhysRangeSet::Invalid
        }
    }


}

/// The result of "splitting" a [`PhysRange`]. 
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PhysRangeSet { 
    Invalid,
    /// A pair of contiguous ranges
    Pair {
        old: PhysRange, 
        new: PhysRange
    },

    /// Three contiguous ranges
    Triad { 
        old_lo: PhysRange,
        new: PhysRange,
        old_hi: PhysRange,
    },
}

/// Describes the kind of physical memory region presented to the kernel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd)]
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

    /// Advertised as MMIO by UEFI firmware
    Mmio = 8,

    KernelPaging = 9,

    KernelHeap = 10,

    /// Advertised as "reserved" by UEFI firmware
    UefiReserved = 255,
}
impl From<MemoryType> for MrldMemoryKind { 
    fn from(t: MemoryType) -> Self { 
        match t { 
            MemoryType::LOADER_CODE |
            MemoryType::LOADER_DATA |
            MemoryType::BOOT_SERVICES_CODE |
            MemoryType::BOOT_SERVICES_DATA |
            MemoryType::CONVENTIONAL => {
                MrldMemoryKind::Available
            },
            MemoryType::RUNTIME_SERVICES_CODE |
            MemoryType::RUNTIME_SERVICES_DATA => {
                MrldMemoryKind::UefiRuntime
            },
            MemoryType::RESERVED |
            MemoryType::UNUSABLE |
            MemoryType::PAL_CODE => {
                MrldMemoryKind::UefiReserved
            },
            MemoryType::MMIO |
            MemoryType::MMIO_PORT_SPACE => { 
                MrldMemoryKind::Mmio
            }

            MemoryType::ACPI_NON_VOLATILE => {
                MrldMemoryKind::AcpiNonVolatile
            }
            _ => MrldMemoryKind::Invalid,
        }
    }
}


/// Describes a physical memory region. 
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd)]
#[repr(C)]
pub struct MrldMemoryDesc {
    /// The kind of memory region.
    pub kind: MrldMemoryKind,
    /// The physical address range defining this region.
    pub range: PhysRange,
}
impl MrldMemoryDesc { 
    pub const fn new_invalid() -> Self { 
        Self { 
            kind: MrldMemoryKind::Invalid, 
            range: PhysRange::new(0, 0),
        } 
    }
    pub fn new(range: PhysRange, kind: MrldMemoryKind) -> Self { 
        Self { 
            kind, 
            range
        }
    }

    /// Returns 'true' when 'other' can be merged into this descriptor. 
    pub fn can_merge_with(&self, other: &Self) -> bool { 
        self.kind == other.kind &&
            self.end() == other.start()
    }

    pub fn try_merge_with(&self, other: &Self) -> Option<Self> {
        if self.can_merge_with(other) { 
            let range = PhysRange::new(
                self.start(), self.end() + other.size()
            );
            Some(Self::new(range, self.kind))
        } else { 
            None
        }
    }

    pub fn set_range(&mut self, range: PhysRange) { 
        self.range = range;
    }

    pub fn range(&self) -> &PhysRange { &self.range }
    pub fn start(&self) -> u64 { self.range.start() }
    pub fn end(&self) -> u64 { self.range.end() }
    pub fn size(&self) -> u64 { self.range.size() }
}


