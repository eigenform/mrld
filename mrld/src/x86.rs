
/// Implemented on marker types for an MTRR. 
pub trait MTRRId { const IDX: u32; }
impl MTRRId for MTRR0 { const IDX: u32 = 0; }
impl MTRRId for MTRR1 { const IDX: u32 = 1; }
impl MTRRId for MTRR2 { const IDX: u32 = 2; }
impl MTRRId for MTRR3 { const IDX: u32 = 3; }
impl MTRRId for MTRR4 { const IDX: u32 = 4; }
impl MTRRId for MTRR5 { const IDX: u32 = 5; }
impl MTRRId for MTRR6 { const IDX: u32 = 6; }
impl MTRRId for MTRR7 { const IDX: u32 = 7; }

pub struct MTRR0;
pub struct MTRR1;
pub struct MTRR2;
pub struct MTRR3;
pub struct MTRR4;
pub struct MTRR5;
pub struct MTRR6;
pub struct MTRR7;

pub struct VariableMTRR<I: MTRRId>(core::marker::PhantomData<I>);
impl <I: MTRRId> VariableMTRR<I> {
    const MSR_MTRR_VAR_BASE: u32 = (0x0000_0200 + (I::IDX * 2));
    const MSR_MTRR_VAR_MASK: u32 = Self::MSR_MTRR_VAR_BASE + 1;

    #[inline(always)]
    pub unsafe fn read() -> (u64, u64) { 
        (Self::read_base(), Self::read_mask())
    }
    #[inline(always)]
    pub unsafe fn read_base() -> u64 { 
        rdmsr(Self::MSR_MTRR_VAR_BASE)
    }
    #[inline(always)]
    pub unsafe fn read_mask() -> u64 { 
        rdmsr(Self::MSR_MTRR_VAR_MASK)
    }
}

pub struct CR3;
impl CR3 { 
    #[inline(always)]
    pub unsafe fn write(val: u64) {
        core::arch::asm!( "mov cr3, rax", in("rax") val);
    }
    #[inline(always)]
    pub unsafe fn read() -> u64 { 
        let val: u64;
        core::arch::asm!( "mov rax, cr3", out("rax") val);
        val
    }
}

pub struct CR4;
impl CR4 { 
    #[inline(always)]
    pub unsafe fn write(val: u64) {
        core::arch::asm!( "mov cr4, rax", in("rax") val);
    }
    #[inline(always)]
    pub unsafe fn read() -> u64 { 
        let val: u64;
        core::arch::asm!( "mov rax, cr4", out("rax") val);
        val
    }
}

#[inline(always)]
pub fn rdmsr(msr: u32) -> u64 { 
    let lo: u32;
    let hi: u32;
    unsafe { 
        core::arch::asm!(
            "rdmsr",
            in("rcx") msr,
            out("rax") lo,
            out("rdx") hi,
            options(nomem, nostack, preserves_flags)
        );
    }
    (hi as u64) << 32 | lo as u64
}

#[inline(always)]
pub fn wrmsr(msr: u32, val: u64) {
    let lo: u32 = val as u32;
    let hi: u32 = (val >> 32) as u32;
    unsafe { 
        core::arch::asm!(
            "wrmsr",
            in("rcx") msr,
            in("rax") lo,
            in("rdx") hi,
            options(nomem, nostack, preserves_flags)
        );
    }
}


