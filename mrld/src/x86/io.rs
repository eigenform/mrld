
use core::arch::asm;

/// Representing a register in the x86 I/O port space.
pub struct IoPort(u16);
impl IoPort { 
    pub const fn new(port: u16) -> Self { 
        Self(port)
    }

    /// Write a byte to this register.
    #[inline(always)]
    pub unsafe fn out8(&self, val: u8) {
        Io::out8(self.0, val);
    }

    /// Read a byte from this register.
    #[inline(always)]
    pub unsafe fn in8(&self) -> u8 {
        Io::in8(self.0)
    }

    /// Write a 32-bit word to this register.
    #[inline(always)]
    pub unsafe fn out32(&self, val: u32) {
        Io::out32(self.0, val);
    }

    /// Read a 32-bit word from this register.
    #[inline(always)]
    pub unsafe fn in32(&self) -> u32 {
        Io::in32(self.0)
    }

}

pub struct Io;
impl Io { 
    #[inline(always)]
    pub unsafe fn out8(port: u16, val: u8) {
        asm!("out dx, al", in("al") val, in("dx") port,
            options(nomem, nostack, preserves_flags)
        );
    }

    #[inline(always)]
    pub unsafe fn in8(port: u16) -> u8 {
        let res: u8;
        asm!("in al, dx", in("dx") port, out("al") res,
            options(nomem, nostack, preserves_flags)
        );
        res
    }

    #[inline(always)]
    pub unsafe fn out16(port: u16, val: u16) {
        asm!("out dx, ax", in("ax") val, in("dx") port,
            options(nomem, nostack, preserves_flags)
        );
    }

    #[inline(always)]
    pub unsafe fn in16(port: u16) -> u16 {
        let res: u16;
        asm!("in ax, dx", in("dx") port, out("ax") res,
            options(nomem, nostack, preserves_flags)
        );
        res
    }

    #[inline(always)]
    pub unsafe fn out32(port: u16, val: u32) {
        asm!("out dx, eax", in("eax") val, in("dx") port,
            options(nomem, nostack, preserves_flags)
        );
    }

    #[inline(always)]
    pub unsafe fn in32(port: u16) -> u32 {
        let res: u32;
        asm!("in eax, dx", in("dx") port, out("eax") res,
            options(nomem, nostack, preserves_flags)
        );
        res
    }

}


