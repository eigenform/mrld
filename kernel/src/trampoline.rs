//! Trampoline used to boot APs starting in real mode.
//!
//! The trampoline code is assembled and linked seperately. 
//! See `kernel/src/trampoline.S`, `trampoline.ld`, and `kernel/build.rs` for 
//! important details about all of this. 
//!
//! Notes
//! =====
//!
//! An AP begins fetching instructions at '_trampoline_start' after receiving 
//! a startup IPI (SIPI) from the bootstrap core. The 8-bit vector associated 
//! with the SIPI becomes the index of a 4KiB page in the low 1MiB of physical 
//! memory (from `0x0_0000` to `0xf_f000`). I *think* this immediately changes
//! the CS segment, and the instruction pointer is set to zero. 
//!
//! In our case, we assume this code will always be located at `0x8000`.
//! This is baked into the linkerscript. 
//! The length of the resulting binary is padded to 4096 bytes with `objdump`. 
//!
//! Debugging
//! =========
//!
//! tl;dr GOOD LUCK, it's basically impossible to use QEMU's GDB stub for 
//! looking at code across different x86 operating modes. Instead of debugging,
//! consider immediately writing the bugfree code and not making any mistakes.
//!
//! After having trouble with GDB, I had a little bit of success gleaning 
//! information from KVM and ftrace, ie. 
//!
//! ```shell
//! $ echo 1 > /sys/kernel/tracing/events/kvm/enable
//! $ cat /sys/kernel/tracing/trace_pipe
//! ```

/// 32-byte metadata used by the trampoline 
#[repr(C)]
pub struct TrampolineHeader { 
    magic: u32,
    pml4: u32,
    entry: u64,
    stack_base: u64,
}
impl TrampolineHeader { 
    pub const MAGIC: u32 = 0xb007c0de;
    pub const fn new(entry: u64, pml4: u32, stack_base: u64) -> Self { 
        Self { 
            magic: 0xb007c0de,
            pml4,
            entry,
            stack_base,
        }
    }
}

#[repr(C)]
pub struct Trampoline;
impl Trampoline { 
    /// Fixed physical address of the trampoline
    pub const PHYS_BASE: u64 = 0x8000;

    /// Fixed offset to [`TrampolineHeader`] 
    pub const HDR_OFF: isize = 0x200;

    pub const DATA: &'static [u8; 0x1000] = 
        include_bytes!("../../target/mrld-kernel/trampoline.bin");

    pub fn as_ptr() -> *const u8 { 
        Self::DATA.as_ptr()
    }

    /// Write the trampoline binary into physical memory.
    pub unsafe fn write(entry: u64, pml4: u64, stack_base: u64) { 
        let tgt = (Self::PHYS_BASE as *mut u8);
        let src = Self::DATA.as_ptr();
        tgt.copy_from_nonoverlapping(src, Self::DATA.len());

        // FIXME: For now, I think we need the physical address of PML4
        // to be a 32-bit address... 
        assert!(pml4 & 0xffff_ffff_0000_0000 == 0);

        let hdr = TrampolineHeader::new(
            entry,
            pml4 as _,
            stack_base,
        );
        let tgt_hdr = (tgt.offset(Self::HDR_OFF) as *mut TrampolineHeader);
        tgt_hdr.write_volatile(hdr);
    }
}

