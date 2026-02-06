
use core::arch::asm;

/// Representing a register in the x86 I/O port space.
pub struct IoPort(u16);
impl IoPort { 
    pub const fn new(port: u16) -> Self { 
        Self(port)
    }

    /// Write a byte to this register.
    #[inline(never)]
    pub unsafe fn out8(&self, val: u8) {
        Io::out8(self.0, val);
    }

    /// Read a byte from this register.
    #[inline(never)]
    pub unsafe fn in8(&self) -> u8 {
        Io::in8(self.0)
    }
}

pub struct Io;
impl Io { 
    #[inline(always)]
    pub unsafe fn out8(port: u16, val: u8) {
        asm!(
            "out dx, al",
            in("al") val,
            in("dx") port,
        );
    }

    #[inline(always)]
    pub unsafe fn in8(port: u16) -> u8 {
        let res: u8;
        asm!(
            "in al, dx",
            in("dx") port,
            out("al") res,
        );
        res
    }
}


