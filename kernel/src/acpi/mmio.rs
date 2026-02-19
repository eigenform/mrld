
pub struct PM1Status(pub u16);
impl PM1Status { 
    const TMR_STS: u16         = (1 << 0);
    const BM_STS: u16          = (1 << 4);
    const GBL_STS: u16         = (1 << 5);
    const PWRBTN_STS: u16      = (1 << 8);
    const SLPBTN_STS: u16      = (1 << 9);
    const RTC_STS: u16         = (1 << 10);
    const PICEXP_WAKE_STS: u16 = (1 << 14);
    const WAK_STS: u16         = (1 << 15);
}
impl Into<u16> for PM1Status { 
    fn into(self) -> u16 { self.0 }
}

pub struct PM1Enable(pub u16);
impl PM1Enable { 
    const TMR_EN: u16          = (1 << 0);
    const GBL_EN: u16          = (1 << 5);
    const PWRBTN_EN: u16       = (1 << 8);
    const SLPBTN_EN: u16       = (1 << 9);
    const RTC_EN: u16          = (1 << 10);
    const PICEXP_WAKE_DIS: u16 = (1 << 14);
}
impl Into<u16> for PM1Enable { 
    fn into(self) -> u16 { self.0 }
}


pub struct PM1Control(pub u16);
impl PM1Enable { 
    const SCI_EN: u16   = (1 << 0);
    const BM_RLD: u16   = (1 << 1);
    const GBL_RLS: u16  = (1 << 2);
    const SLP_TYPx: u16 = (0b111 << 10);
    const SLP_EN: u16   = (1 << 13);
}
impl Into<u16> for PM1Control { 
    fn into(self) -> u16 { self.0 }
}

pub trait MmioWidth: 
    From<bool> + 
    core::ops::Shl<Self, Output=Self> +
    core::ops::Shl<usize, Output=Self> +
    core::ops::BitAnd<Self, Output=Self> +
    core::ops::BitOr<Self, Output=Self> + 
    core::ops::Not<Output=Self>
{
    const BITS: usize;
    const ONE: Self;
}
impl MmioWidth for u8 { 
    const BITS: usize = 8;
    const ONE: Self = 1;
}
impl MmioWidth for u16 { 
    const BITS: usize = 16;
    const ONE: Self = 1;
}
impl MmioWidth for u32 { 
    const BITS: usize = 32;
    const ONE: Self = 1;
}



#[repr(transparent)]
pub struct MmioPtr<T: MmioWidth> { 
    addr: u64,
    _t: core::marker::PhantomData<T>,
}
impl <T: MmioWidth> MmioPtr<T> { 
    pub const fn new(addr: u64) -> Self { 
        Self { 
            addr, 
            _t: core::marker::PhantomData,
        }
    }

    unsafe fn as_mut(&self) -> *mut T { 
        self.addr as _
    }
    pub unsafe fn write(&self, val: T) { 
        self.as_mut().write_volatile(val)
    }
    pub unsafe fn read(&self) -> T {
        self.as_mut().read_volatile()
    }
    pub unsafe fn toggle(&self, idx: usize, en: bool) {
        assert!(idx < T::BITS);
        let val = self.read();
        let bit: T = (T::ONE << idx);
        let next = (val & !bit) | (T::from(en)).shl(idx);
        self.write(next);
    }
    pub unsafe fn write_mask(&self, mask: T, val: T) {
        let x = self.read();
        let next = (x & !mask) | val;
        self.write(next);
    }



}


