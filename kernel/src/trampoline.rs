//! Trampoline used to boot APs starting in real mode.
//!
//! The trampoline code is assembled and linked seperately in `build.rs`. 
//!
//! Notes
//! =====
//!
//! An AP begins fetching instructions at '_trampoline_start' after receiving 
//! a startup IPI (SIPI) from the bootstrap core. The 8-bit vector associated 
//! with the SIPI becomes the index of a 4KiB page in the low 1MiB of physical 
//! memory (from `0x0_0000` to `0xf_f000`). 
//!
//! In our case, we assume this code will always be located at `0x8000`.
//! This is baked into the linkerscript. 
//!


pub struct Trampoline;
impl Trampoline { 
    pub const PHYS_BASE: u64 = 0x8000;
    pub const DATA: &'static [u8; 1024] = 
        include_bytes!("../../target/mrld-kernel/trampoline.bin");

    pub fn as_ptr() -> *const u8 { 
        Self::DATA.as_ptr()
    }
}

