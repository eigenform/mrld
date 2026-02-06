//! Types for handling x86 virtual memory segmentation.

use bitflags::bitflags;

/// Type whose variants represent x86 privilege levels. 
pub enum PrivilegeLevel { 
    Ring0 = 0,
    Ring1 = 1,
    Ring2 = 2,
    Ring3 = 3,
}
impl PrivilegeLevel { 
    pub const fn from_u16(x: u16) -> Self { 
        match x { 
            0 => Self::Ring0,
            1 => Self::Ring1,
            2 => Self::Ring2,
            3 => Self::Ring3,
            _ => panic!("Invalid privilege level?"),
        }
    }
    pub const fn as_u64(&self) -> u64 { 
        match self { 
            Self::Ring0 => 0,
            Self::Ring1 => 1,
            Self::Ring2 => 2,
            Self::Ring3 => 3,
        }
    }
}

/// A "segment selector" for an entry in the global/local descriptor table.
pub struct SegmentSelector(u16);
impl SegmentSelector { 
    const RPL_MASK: u16 = 0b0000_0000_0000_0011;
    const TI_BIT: u16   = 0b0000_0000_0000_0100;
    const SI_MASK: u16  = 0b1111_1111_1111_1000;

    /// Create a new selector where:
    ///
    /// - `index` is the index into a descriptor table
    /// - `rpl` is the requested [`PrivilegeLevel`]
    /// - `local` indicates a local descriptor table when true
    ///   (or, a global descriptor table when false)
    ///
    pub const fn new(index: u16, local: bool, rpl: PrivilegeLevel) 
        -> Self 
    { 
        Self(index << 3 | (local as u16) << 2 | rpl as u16)
    }

    /// Index into the associated descriptor table.
    pub fn index(&self) -> usize { 
        (self.0 >> 3) as usize
    }

    /// Byte offset into the associated descriptor table. 
    pub fn byte_index(&self) -> usize { 
        (self.0 & Self::SI_MASK) as usize
    }

    /// Returns 'true' if this selector refers to a local descriptor table
    /// (otherwise, 'false' for a global descriptor table). 
    pub fn is_local(&self) -> bool {
        (self.0 & Self::TI_BIT) != 0
    }

    pub const fn as_u16(&self) -> u16 { self.0 }
}

pub trait Segment {
    unsafe fn write(selector: SegmentSelector);
}

/// Code segment. 
pub struct CS;
impl Segment for CS { 
    unsafe fn write(selector: SegmentSelector) {
        core::arch::asm!(r#"
            push {s:x}
            lea {tmp}, [rip + 2f]
            push {tmp}
            retfq
        2:
        "#,
        s = in(reg) selector.as_u16(),
        tmp = lateout(reg) _,
        options(preserves_flags),
        );
    }
}

pub struct SS;
impl Segment for SS { 
    unsafe fn write(selector: SegmentSelector) {
        core::arch::asm!(
            "mov ss, {0:x}",
            in(reg) selector.as_u16(),
            options(nostack, preserves_flags)
        );
    }
}

pub struct DS;
impl Segment for DS { 
    unsafe fn write(selector: SegmentSelector) {
        core::arch::asm!(
            "mov ds, {0:x}",
            in(reg) selector.as_u16(),
            options(nostack, preserves_flags)
        );
    }
}


pub struct ES;
impl Segment for ES { 
    unsafe fn write(selector: SegmentSelector) {
        core::arch::asm!(
            "mov es, {0:x}",
            in(reg) selector.as_u16(),
            options(nostack, preserves_flags)
        );

    }
}


pub struct FS;
impl Segment for FS { 
    unsafe fn write(selector: SegmentSelector) {
        core::arch::asm!(
            "mov fs, {0:x}",
            in(reg) selector.as_u16(),
            options(nostack, preserves_flags)
        );

    }
}

pub struct GS;
impl Segment for GS { 
    unsafe fn write(selector: SegmentSelector) {
        core::arch::asm!(
            "mov gs, {0:x}",
            in(reg) selector.as_u16(),
            options(nostack, preserves_flags)
        );

    }
}


