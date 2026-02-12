//! Physical Memory
//!
//! The [`MrldMemoryMap`] type describes the high-level layout of physical 
//! memory, and only supports a fixed number of regions.
//!
//! The static [`MEMORY_MAP`] contains the state of this map during runtime. 
//! This is initialized immediately after boot by walking the UEFI memory map 
//! passed to the kernel in [`MrldBootArgs`]. 
//!
//! A [`MrldMemoryKind::KernelHeap`] region always describes a 1GiB-aligned 
//! region of physical memory dedicated to the global allocator. 
//!
//! A [`MrldMemoryKind::KernelPaging`] region always describes a 2MiB-aligned
//! region of physical memory dedicated to storing page tables. 
//!
//! Page Tables
//! ===========
//!
//! The kernel boots with provisional page tables defined in the bootloader
//! (see `boot/src/bup.rs`), which are roughly: 
//!
//! - 512GiB identity mapping
//!     - Physical : 0x0000_0000_0000_0000 - 0x0000_0080_0000_0000 
//!     - Virtual  : 0x0000_0000_0000_0000 - 0x0000_0080_0000_0000
//!
//! - Kernel image (32 2MiB pages)
//!     - Physical : 0x0000_0000_0400_0000 - 0x0000_0000_0800_0000 
//!     - Virtual  : 0xffff_ffff_8000_0000 - 0xffff_ffff_8400_0000 
//!
//! After we've defined a physical region for paging, a new set of tables 
//! is created with the following mappings: 
//!
//! - 512GiB identity mapping
//!     - Physical : 0x0000_0000_0000_0000 - 0x0000_0080_0000_0000 
//!     - Virtual  : 0x0000_0000_0000_0000 - 0x0000_0080_0000_0000
//!
//! - Kernel heap (one 1GiB page)
//!     - Physical : The [`MrldMemoryKind::KernelHeap`] region
//!     - Virtual  : 0xffff_ffd0_0000_0000 - 0xffff_ffd0_4000_0000 
//!
//! - Kernel image (32 2MiB pages)
//!     - Physical : 0x0000_0000_0400_0000 - 0x0000_0000_0800_0000 
//!     - Virtual  : 0xffff_ffff_8000_0000 - 0xffff_ffff_8400_0000 
//!

use mrld::physmem::*;
use mrld::paging::*;
use mrld::MrldBootArgs;
use crate::println;
use spin::Mutex;
use uefi_raw::table::boot::{
    MemoryType, MemoryAttribute, MemoryDescriptor
};

/// Base of physical memory allocated (by the bootloader) for the kernel.
pub const KERNEL_PHYS_BASE: u64     = 0x0000_0000_0400_0000;

/// The physical memory map.
pub static MEMORY_MAP: Mutex<MrldMemoryMap> = {
    Mutex::new(MrldMemoryMap::new_empty())
};

/// Describes a set of physical memory regions managed by the kernel. 
#[repr(C)]
pub struct MrldMemoryMap { 
    /// Set of physical memory regions.
    pub entries: [Option<MrldMemoryDesc>; Self::NUM_ENTRIES],
}
impl MrldMemoryMap { 
    /// Fixed number of entries in this memory map.
    pub const NUM_ENTRIES: usize = 128;

    /// Create a new [empty] memory map. 
    pub const fn new_empty() -> Self { 
        Self { 
            entries: [ const { None }; Self::NUM_ENTRIES ],
        }
    }

    /// Return a mutable reference to a particular entry in the map.
    pub fn get_mut(&mut self, idx: usize) -> &mut Option<MrldMemoryDesc> { 
        assert!(idx < Self::NUM_ENTRIES);
        &mut self.entries[idx]
    }

    /// Invalidate a particular entry in the map. 
    pub fn invalidate(&mut self, idx: usize) { 
        assert!(idx < Self::NUM_ENTRIES);
        self.entries[idx] = None;
    }

    /// Return an iterator over references to valid entries in the map.
    pub fn iter_valid(&self) -> impl Iterator<Item = &MrldMemoryDesc> { 
        self.entries.iter().filter_map(|x| x.as_ref())
    }

    /// Return an iterator over mutable references to valid entries in the map.
    pub fn iter_mut_valid(&mut self) -> impl Iterator<Item = &mut MrldMemoryDesc> { 
        self.entries.iter_mut().filter_map(|x| x.as_mut())
    }

    /// Return a reference to the first entry satisfying the closure 'f' 
    /// (if it exists).
    pub fn find_with(&mut self, f: impl Fn(&&MrldMemoryDesc) -> bool)
        -> Option<&MrldMemoryDesc>
    { 
        self.iter_valid().find(f)
    }

    /// Return a mutable reference to the first entry satisfying the closure 
    /// 'f' (if it exists).
    pub fn find_mut_with(&mut self, f: impl FnMut(&&mut MrldMemoryDesc) -> bool)
        -> Option<&mut MrldMemoryDesc>
    { 
        self.iter_mut_valid().find(f)
    }

    /// Insert a new valid region into the map.
    pub unsafe fn allocate_new_region(&mut self, desc: MrldMemoryDesc) { 
        if let Some(e) = self.entries.iter_mut().find(|x| x.is_none()) { 
            *e = Some(desc);
        } else { 
            panic!("couldn't allocate memory map region");
        }
    }

}

impl MrldMemoryMap { 
    /// Initialize the map by parsing the UEFI memory map passed from the 
    /// bootloader. 
    pub unsafe fn init(&mut self, boot_args: &MrldBootArgs) { 
        let base_ptr = boot_args.uefi_map as *const u8;
        let desc_sz = boot_args.uefi_map_desc_size;
        let sz = boot_args.uefi_map_size;
        let num_entries = sz / desc_sz;

        let mut prev: Option<(usize, MrldMemoryDesc)> = None;
        let mut pidx = 0;

        for uidx in 0..num_entries { 
            let ptr = base_ptr.offset(uidx as isize * desc_sz as isize);
            let uefi_desc: *const MemoryDescriptor = ptr.cast();

            if let Some(d) = uefi_desc.as_ref() { 
                let range = PhysRange::new(
                    d.phys_start, d.phys_start + (d.page_count * 0x1000)
                );
                let kind = MrldMemoryKind::from(d.ty);
                let desc = MrldMemoryDesc::new(range, kind);

                if let Some((idx, p)) = prev { 
                    if let Some(merged) = p.try_merge_with(&desc) { 
                        self.entries[idx] = Some(merged);
                        prev = Some((idx, merged));
                    } 
                    else { 
                        self.entries[pidx] = Some(desc);
                        prev = Some((pidx, desc));
                        pidx += 1;
                    }
                } else { 
                    prev = Some((pidx, desc));
                    self.entries[pidx] = Some(desc);
                    pidx += 1;
                }
            }
        }

        // Reserve a region for the kernel image
        let desc = self.allocate_at(
            KERNEL_PHYS_BASE,
            mrld::paging::PageSize::Size2MiB, 
            32, 
            mrld::physmem::MrldMemoryKind::KernelImage,
        );
        if desc.is_none() { 
            panic!("Couldn't reserve physical region for kernel image?");
        }


    }

    /// Create a new region with the given base address, page size, and page 
    /// count. 
    pub unsafe fn allocate_at(&mut self, 
        addr: u64, 
        pagesz: PageSize,
        cnt: usize,
        kind: MrldMemoryKind,
    ) -> Option<MrldMemoryDesc>
    {
        assert!( (addr & (u64::from(pagesz) - 1)) == 0);
        let requested_size = u64::from(pagesz) * cnt as u64;
        let requested_range = PhysRange::new(addr, addr + requested_size);

        let candidate_desc = self.find_mut_with(|ref desc| {
            desc.kind == MrldMemoryKind::Available && 
            desc.range().contains_range(&requested_range)
        });

        if let Some(candidate_desc) = candidate_desc { 
            match candidate_desc.range().split(&requested_range) { 
                PhysRangeSet::Pair { old, new } => {
                    candidate_desc.set_range(old);
                    let new_desc = MrldMemoryDesc::new(new, kind);
                    self.allocate_new_region(new_desc);
                    Some(new_desc)
                },
                PhysRangeSet::Triad { old_lo, new, old_hi } => {
                    candidate_desc.set_range(old_lo);
                    let new_desc = MrldMemoryDesc::new(new, kind);
                    let new_desc_hi = MrldMemoryDesc::new(
                        old_hi, candidate_desc.kind
                    );

                    self.allocate_new_region(new_desc);
                    self.allocate_new_region(new_desc_hi);
                    Some(new_desc)
                },
                PhysRangeSet::Invalid => { 
                    panic!("couldn't split range?");
                },
            }
        } else { 
            None
        }
    }

    /// Find an existing *available* region satisfying the constraints, and 
    /// then create a new region ("allocated" from the available region) with 
    /// the given page size and count.
    pub unsafe fn allocate(&mut self, pagesz: PageSize, cnt: usize, 
        kind: MrldMemoryKind) -> Option<MrldMemoryDesc>
    { 
        let candidate_desc = self.find_mut_with(|ref desc| {
            desc.kind == MrldMemoryKind::Available && 
            desc.range().try_get_pages(pagesz, cnt).is_some()
        });

        if let Some(candidate_desc) = candidate_desc { 
            let requested_range = candidate_desc.range()
                .try_get_pages(pagesz, cnt).unwrap();

            // Split the candidate range into the appropriate parts
            match candidate_desc.range().split(&requested_range) { 
                PhysRangeSet::Pair { old, new } => {
                    candidate_desc.set_range(old);
                    let new_desc = MrldMemoryDesc::new(new, kind);
                    self.allocate_new_region(new_desc);
                    Some(new_desc)
                },
                PhysRangeSet::Triad { old_lo, new, old_hi } => {
                    candidate_desc.set_range(old_lo);
                    let new_desc = MrldMemoryDesc::new(new, kind);
                    let new_desc_hi = MrldMemoryDesc::new(
                        old_hi, candidate_desc.kind
                    );

                    self.allocate_new_region(new_desc);
                    self.allocate_new_region(new_desc_hi);
                    Some(new_desc)
                },
                PhysRangeSet::Invalid => { 
                    panic!("couldn't split range?");
                },
            }
        } 
        // We couldn't find a region to allocate from
        else { 
            None
        }
    }
}

