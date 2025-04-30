//! Types for dealing with x86 paging.

use core::ptr::NonNull;
use bitflags::bitflags;

/// Number of 64-bit entries in a page table. 
///
/// NOTE: In our case, this is fixed at 512 entries for all levels. 
const NUM_ENTRIES: usize = 512;

/// Type representing different supported page sizes. 
pub enum PageSize { Size4KiB, Size2MiB, Size1GiB, }
impl PageSize { 
    pub fn as_usize(&self) -> usize { 
        match self {
            Self::Size4KiB => 1 << 12,
            Self::Size2MiB => 1 << 21,
            Self::Size1GiB => 1 << 30,
        }
    }
}

/// Type representing different kinds of page tables.
/// Variants of type correspond to types implementing [`PageTableKind`].
pub enum PageTableLevel { PML4, PDP, PD, PT, }
impl PageTableLevel {
    pub fn next_level(&self) -> Option<Self> {
        match self { 
            Self::PML4 => Some(Self::PDP),
            Self::PDP  => Some(Self::PD),
            Self::PD   => Some(Self::PT),
            Self::PT   => None,
        }
    }
}

/// Interface for marker types that represent a particular kind of page table.
pub trait PageTableKind {
    /// The 'enum'-like representation for this kind of page table.
    const LEVEL: PageTableLevel;
    /// The 'enum'-like representation for the next level of page table.
    const NEXT_LEVEL: Option<PageTableLevel>;
    /// Human-readable name for this kind of page table.
    const NAME: &'static str;
    /// Human-readable name for entries in this kind of page table
    const ENTRY_NAME: &'static str;
    /// The size of terminal entries in this kind of page table.
    const TERMINAL_SIZE: Option<PageSize>;
}

/// Marker type for a page map level-4 (PML4) table.
pub struct PML4;

/// Marker type for a page directory pointer (PDP) table.
pub struct PDP;

/// Marker type for a page directory (PD) table.
pub struct PD;

/// Marker type for a terminal page table (PT).
pub struct PT;

impl PageTableKind for PML4 {
    const LEVEL: PageTableLevel = PageTableLevel::PML4;
    const NEXT_LEVEL: Option<PageTableLevel> = Some(PageTableLevel::PDP);
    const NAME: &'static str = "PML4";
    const ENTRY_NAME: &'static str = "PML4E";
    const TERMINAL_SIZE: Option<PageSize> = None;
}
impl PageTableKind for PDP {
    const LEVEL: PageTableLevel = PageTableLevel::PDP;
    const NEXT_LEVEL: Option<PageTableLevel> = Some(PageTableLevel::PD);
    const NAME: &'static str = "PDP";
    const ENTRY_NAME: &'static str = "PDPE";
    const TERMINAL_SIZE: Option<PageSize> = Some(PageSize::Size1GiB);
}
impl PageTableKind for PD {
    const LEVEL: PageTableLevel = PageTableLevel::PD;
    const NEXT_LEVEL: Option<PageTableLevel> = Some(PageTableLevel::PT);
    const NAME: &'static str = "PD";
    const ENTRY_NAME: &'static str = "PDE";
    const TERMINAL_SIZE: Option<PageSize> = Some(PageSize::Size2MiB);
}
impl PageTableKind for PT {
    const LEVEL: PageTableLevel = PageTableLevel::PT;
    const NEXT_LEVEL: Option<PageTableLevel> = None;
    const NAME: &'static str = "PT";
    const ENTRY_NAME: &'static str = "PTE";
    const TERMINAL_SIZE: Option<PageSize> = Some(PageSize::Size4KiB);
}

/// Type alias for PML4 table entries.
pub type PML4Entry = PageTableEntry<PML4>;

/// Type alias for PDP table entries.
pub type PDPEntry  = PageTableEntry<PDP>;

/// Type alias for page directory table entries.
pub type PDEntry   = PageTableEntry<PD>;

/// Type alias for last-level page table entries.
pub type PTEntry   = PageTableEntry<PT>;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct PTFlag: u64 {
        const P   = (1 << 0);
        const RW  = (1 << 1);
        const US  = (1 << 2);
        const PWT = (1 << 3);
        const PCD = (1 << 4);
        const A   = (1 << 5);
        const D   = (1 << 6);
        const PS  = (1 << 7);
        const G   = (1 << 8);
        const NX  = (1 << 63);
    }
}

impl PTFlag { 
    pub fn as_u64(&self) -> u64 { 
        self.bits() as u64
    }
}


/// A 64-bit page table entry. 
#[repr(transparent)]
pub struct PageTableEntry<K: PageTableKind> {
    val: u64, 
    _level: core::marker::PhantomData<K>
}
impl <K: PageTableKind> PageTableEntry<K> {
    pub fn from_u64(val: u64) -> Self { 
        Self { 
            val, 
            _level: core::marker::PhantomData,
        }
    }

    pub fn new(address: u64, flags: PTFlag) -> Self { 
        let mut val = 0;
        val |= address & Self::ADDRESS_MASK;
        val |= flags.as_u64();

        Self { 
            val, 
            _level: core::marker::PhantomData,
        }
    }

    pub fn level(&self) -> PageTableLevel { 
        K::LEVEL
    }

    pub fn flags(&self) -> PTFlag { 
        PTFlag::from_bits_retain(self.val)
    }

    /// "Base [physical] address"
    const ADDRESS_MASK: u64 = 0x0fff_ffff_ffff_f000;
    /// "Available-to-software" bits
    const AVL_MASK: u64     = 0x0000_0000_0000_0e00;

    /// Is this a "terminal" page table entry? 
    ///
    /// WARNING: This *assumes* we are in long mode with PAE. 
    pub fn terminal(&self) -> bool { 
        match K::LEVEL { 
            PageTableLevel::PML4 => {
                false
            },
            PageTableLevel::PDP => {
                self.flags().contains(PTFlag::PS)
            },
            PageTableLevel::PD => {
                self.flags().contains(PTFlag::PS)
            },
            PageTableLevel::PT => true,
        }
    }

    pub fn invalid(&self) -> bool { 
        self.val == 0
    }
    pub fn address(&self) -> u64 { 
        self.val & Self::ADDRESS_MASK
    }
}
impl <K: PageTableKind> core::fmt::Debug for PageTableEntry<K> { 
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let flags = self.flags();
        //let w = flags.rw() { "rw" } else { "ro" };
        //let u = if self.user() { "usr" } else { "sys" };
        //let wt = if self.writethrough() { "wt" } else { "wb" };
        //let c = if self.cacheable() { "cc" } else { "uc" };

        //let p = if self.present() { "p" } else { "-" };
        //let a = if self.accessed() { "a" } else { "-" };
        //let d = if self.dirty() { "d" } else { "-" };
        //let s = if self.ps() { "t" } else { "-" };
        //let g = if self.global() { "g" } else { "-" };
        //let x = if self.nonexecutable() { "x" } else { "-" };

        write!(fmt, "{}[{:016x}][{:?}]",
            K::ENTRY_NAME,
            self.address(), 
            self.flags(),
        )
    }
}


/// A page table. 
#[repr(C, align(0x1000))]
pub struct PageTable<K: PageTableKind> {
    entries: [PageTableEntry<K>; NUM_ENTRIES],
}
impl <K: PageTableKind> PageTable<K> {
    pub fn as_ptr(&self) -> *const Self { 
        self.entries.as_ptr() as *const Self
    }

    /// Synthesize a reference to a [`PageTable`] from a pointer.
    pub unsafe fn ref_from_ptr(ptr: *const u8) -> &'static Self { 
        let nn = NonNull::new(ptr as *mut u8).unwrap();
        nn.cast().as_ref()
    }

    /// Synthesize a mutable reference to a [`PageTable`] from a pointer.
    pub unsafe fn mut_ref_from_ptr(ptr: *mut u8) -> &'static mut Self { 
        let nn = NonNull::new(ptr).unwrap();
        nn.cast().as_mut()
    }

    pub fn set_entry(&mut self, idx: usize, entry: PageTableEntry<K>) {
        assert!(idx < NUM_ENTRIES);
        self.entries[idx] = entry;
    }

    pub fn clear_entry(&mut self, idx: usize) {
        assert!(idx < NUM_ENTRIES);
        self.entries[idx] = PageTableEntry::from_u64(0);
    }


    /// Return a slice of the entries in this page table.
    pub fn entries(&self) -> &[PageTableEntry<K>] {
        &self.entries
    }

    /// Return a mutable slice of the entries in this page table.
    pub fn entries_mut(&mut self) -> &mut [PageTableEntry<K>] {
        &mut self.entries
    }

    // FIXME: This is fine for now (returning static references). 
    // At some point, you might consider actually tracking the lifetime
    // of a particular PML4 pointer. 

    pub unsafe fn from_cr3() -> &'static Self { 
        Self::ref_from_ptr(crate::x86::CR3::read() as *const u8)
    }

    pub unsafe fn from_cr3_mut() -> &'static mut Self { 
        Self::mut_ref_from_ptr(crate::x86::CR3::read() as *mut u8)
    }

}
impl <K: PageTableKind> core::ops::Index<usize> for PageTable<K> {
    type Output = PageTableEntry<K>;
    fn index(&self, idx: usize) -> &Self::Output { 
        &self.entries[idx]
    }
}
impl <K: PageTableKind> core::ops::IndexMut<usize> for PageTable<K> {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output { 
        &mut self.entries[idx]
    }
}


/// A virtual memory address. 
pub struct VirtAddr(u64);
impl VirtAddr { 
    const SEXT_MASK:     u64 = 0xffff_0000_0000_0000;
    const PML4_IDX_MASK: u64 = 0x0000_ff80_0000_0000;
    const PDP_IDX_MASK:  u64 = 0x0000_007f_c000_0000;
    const PD_IDX_MASK:   u64 = 0x0000_0000_3fe0_0000;
    const PT_IDX_MASK:   u64 = 0x0000_0000_001f_f000;

    /// Decompose this virtual address into its components. 
    pub fn decompose(&self) -> (usize, usize, usize, usize) { 
        let pml4_idx = self.pml4_idx();
        let pdp_idx = self.pdp_idx();
        let pd_idx = self.pd_idx();
        let pt_idx = self.pt_idx();
        (pml4_idx, pdp_idx, pd_idx, pt_idx)
    }

    pub fn is_canonical(&self) -> bool {
        if self.0 & (1<<47) != 0 {
            self.0 & Self::SEXT_MASK == Self::SEXT_MASK
        } else { 
            self.0 & Self::SEXT_MASK == 0
        }
    }

    pub fn from_u64(val: u64) -> Self { 
        Self(val)
    }
    pub fn pml4_idx(&self) -> usize { 
        ((self.0 & Self::PML4_IDX_MASK) >> 39) as usize
    }
    pub fn pdp_idx(&self) -> usize { 
        ((self.0 & Self::PDP_IDX_MASK) >> 30) as usize
    }
    pub fn pd_idx(&self) -> usize { 
        ((self.0 & Self::PD_IDX_MASK) >> 21) as usize
    }
    pub fn pt_idx(&self) -> usize { 
        ((self.0 & Self::PT_IDX_MASK) >> 12) as usize
    }
}
