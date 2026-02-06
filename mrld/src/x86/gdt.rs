//! Types for representing a global descriptor table (GDT). 

use bitflags::bitflags;

use crate::x86::dtr::DescriptorTableRegister;
use crate::x86::segment::{ PrivilegeLevel, SegmentSelector };

bitflags! { 
    /// Descriptor flags
    #[derive(Clone, Copy, Debug)]
    pub struct DFlags: u64 { 
        /// Granularity
        const G   = (1 << 55);
        /// Default operand size
        const D   = (1 << 54);
        /// Long mode
        const L   = (1 << 53);
        /// (Available to OS)
        const AVL = (1 << 52);
        /// Present
        const P   = (1 << 47);
        /// User segment
        const S   = (1 << 44);
        /// Executable
        const E   = (1 << 43);
        /// Conforming
        const C   = (1 << 42);
        /// Writable [for data] / Readable [for code]
        const W   = (1 << 41);
        /// Accessed
        const A   = (1 << 40);
    }
}

impl DFlags {
    // Present | User | Granularity | Writable/Readable | Accessed
    pub const DEFAULT: Self = Self::from_bits_truncate(
        Self::P.bits() | 
        Self::S.bits() | 
        Self::G.bits() | 
        Self::W.bits() | 
        Self::A.bits()
    );

    pub const CODE: Self = Self::from_bits_truncate(
        Self::DEFAULT.bits() | Self::L.bits() | Self::E.bits()
    );

    pub const DATA: Self = Self::from_bits_truncate(
        Self::DEFAULT.bits() | Self::L.bits()
    );
}

/// A 64-bit "user" descriptor.
pub struct Descriptor(u64);
impl Descriptor { 
    /// Segment limit [15:0]
    const SEGMENT_LIMIT_15_0_MASK: u64  = 0x0000_0000_0000_ffff;

    /// Base address [23:0]
    const BASE_ADDR_LO_MASK: u64        = 0x0000_00ff_ffff_0000;

    /// Segment limit [19:16]
    const SEGMENT_LIMIT_MASK_19_16: u64 = 0x0000_000f_0000_0000;

    /// Privilege level
    //const DPL_MASK: u64                 = 0x0000_6000_0000_0000;
    const DPL_MASK: u64                 = (0b11 << 45);

    /// Base address [31:24]
    const BASE_ADDR_HI_MASK: u64        = 0xff00_0000_0000_0000;

    /// Return the set of [`DFlags`] associated with this descriptor.
    pub fn flags(&self) -> DFlags {
        DFlags::from_bits_truncate(self.0)
    }

    /// Return the 16-bit segment limit associated with this descriptor.
    pub fn segment_limit(&self) -> u16 { 
        (self.0 & Self::SEGMENT_LIMIT_15_0_MASK) as u16
    }

    /// Return the 32-bit base address associated with this descriptor.
    pub fn base_addr(&self) -> u32 {
        let hi = (self.0 & Self::BASE_ADDR_HI_MASK) >> 32;
        let lo = (self.0 & Self::BASE_ADDR_LO_MASK) >> 16;
        (hi | lo) as u32
    }

    /// Return the 64-bit value of this descriptor.
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    /// Create a new "null" descriptor.
    pub const fn new_null() -> Self { 
        Self(0)
    }

    pub const fn new_from_u64(x: u64) -> Self { 
        Self(x)
    }


    /// Create a new descriptor. 
    pub const fn new(
        base_addr: u32, 
        dpl: PrivilegeLevel, 
        segment_limit: u16,
        flags: DFlags
    ) -> Self 
    {
        let hi = (base_addr as u64 & 0xff00_0000) << 32;
        let lo = (base_addr as u64 & 0x00ff_ffff) << 16;
        let dpl = dpl.as_u64() << 45;
        let flags = flags.bits();
        let limit = segment_limit as u64;
        Self(hi | lo | dpl | flags | limit)
    }
}

impl core::fmt::Debug for Descriptor {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut f = fmt.debug_struct("Descriptor");
        f.field("flags", &self.flags());
        f.field("base_addr", &self.base_addr());
        f.field("limit", &self.segment_limit());
        f.finish()
    }
}


pub enum SystemDescriptor { 
    LdtPointer(u64, u64),
    TssAvailable(u64, u64),
    TssBusy(u64, u64),
    CallGate(u64, u64),
    InterruptGate(u64, u64),
    TrapGate(u64, u64),
}
impl SystemDescriptor {
    pub fn as_u64(&self) -> (u64, u64) { 
        match self { 
            Self::LdtPointer(x, y) | 
            Self::TssAvailable(x, y) |
            Self::TssBusy(x, y) |
            Self::CallGate(x, y) |
            Self::InterruptGate(x, y) |
            Self::TrapGate(x, y) => (*x, *y),
        }
    }
}

pub enum GdtEntry { 
    /// Code descriptor
    Code(DFlags, PrivilegeLevel),
    /// Data descriptor
    Data(DFlags, PrivilegeLevel),
    /// System descriptor
    System(SystemDescriptor),
}

/// Default code segment selector.
/// FIXME: Move this to mrld-kernel?
pub const KERNEL_CODE_SEL: SegmentSelector = 
    SegmentSelector::new(1, false, PrivilegeLevel::Ring0);

/// Default data segment selector.
/// FIXME: Move this to mrld-kernel?
pub const KERNEL_DATA_SEL: SegmentSelector = 
    SegmentSelector::new(2, false, PrivilegeLevel::Ring0);


/// An x86 global descriptor table. 
///
/// `SZ` is the number of 64-bit words in the table. 
///
/// NOTE: This is currently only used for *building* a GDT.
pub struct GlobalDescriptorTable<const SZ: usize> {
    pub entries: [u64; SZ],
    pub cursor: usize,
}
impl <const SZ: usize> GlobalDescriptorTable<SZ> {

    // NOTE: Try to [statically] prevent the user from creating a type where 
    // the size in bytes would exceed the 16-bit 'limit' in the GDTR. 
    // In practice, this means that we cannot safely let the user create a GDT 
    // where the number of 64-bit entries exceeds 8192. 
    const SZ_BYTES: usize = {
        let sz = core::mem::size_of::<[u64; SZ]>();
        assert!(sz <= (u16::MAX as usize) + 1,
            "Number of GDT entries cannot be greater than 8192"
        );
        sz
    };

    /// Return the size of this table (in bytes).
    pub const fn size(&self) -> usize { 
        Self::SZ_BYTES
    }

    /// Return the 16-bit size of this table (in bytes), minus one. 
    pub const fn limit(&self) -> u16 { 
        Self::SZ_BYTES as u16 - 1
    }

    /// Create a new empty table
    pub const fn new_zeroed() -> Self { 
        Self { entries: [0; SZ], cursor: 0 }
    }

    pub const fn as_ptr(&self) -> *const u64 {
        &self.entries as *const u64
    }

}

impl <const SZ: usize> GlobalDescriptorTable<SZ> {

    /// Write a null entry to the GDT.
    pub const fn push_null_desc(mut self) -> Self { 
        assert!(self.cursor < SZ);
        self.entries[self.cursor] = 0;
        self.cursor += 1;
        self
    }

    /// Write a 64-bit user descriptor to this GDT.
    ///
    /// FIXME: You're *assuming* this is only ever used in long mode. 
    /// FIXME: You're *assuming* the segment is ignored (because paging).
    ///
    pub const fn push_user_desc(mut self, entry: Descriptor) -> Self {
        assert!(self.cursor < SZ);
        self.entries[self.cursor] = entry.as_u64();
        self.cursor += 1;
        self
    }

    /// Write a 128-bit system descriptor to this GDT.
    pub fn push_sys_desc(mut self, entry: (u64, u64)) -> Self { 
        assert!(self.cursor + 1 < SZ);
        self.entries[self.cursor] = entry.0;
        self.entries[self.cursor + 1] = entry.1;
        self.cursor += 2;
        self
    }
}


impl <const SZ: usize> GlobalDescriptorTable<SZ> {
    /// Synthesize a static reference to the GDT described by the given 
    /// [`DescriptorTableRegister`]. 
    ///
    /// Safety
    /// ======
    ///
    /// In general, this function is not safe: 
    ///
    /// - The number of 64-bit entries in a GDT is not known statically 
    ///   In this case, we're just asserting that the number of entries for 
    ///   this particular type is consistent with the size provided by the GDTR.
    ///
    /// - The layout of [`GlobalDescriptorTable`] is an array of `u64`, and 
    ///   the `SZ` actually reflect the number of entries because system 
    ///   descriptors are 128-bit
    ///
    pub unsafe fn ref_from_gdtr(gdtr: &DescriptorTableRegister) 
        -> &'static Self 
    {
        assert!(gdtr.size() / 8 == SZ, "Inconsistent GDT size");
        gdtr.ptr().cast::<Self>().as_ref().unwrap()
    }
}
