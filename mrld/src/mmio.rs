
use core::marker::PhantomData;

/// Implemented on MMIO access types
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



#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct MmioPtr<T: MmioWidth> { 
    addr: u64,
    _t: PhantomData<T>,
}

impl <T: MmioWidth> MmioPtr<T> { 
    pub const fn new(addr: u64) -> Self { 
        Self { 
            addr, 
            _t: PhantomData,
        }
    }

    pub fn cast<U: MmioWidth>(&self) -> MmioPtr<U> { 
        MmioPtr { 
            addr: self.addr,
            _t: PhantomData,
        }
    }

    pub unsafe fn offset_bytes(self, offset: usize) -> Self { 
        Self::new(self.addr + offset as u64)
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


