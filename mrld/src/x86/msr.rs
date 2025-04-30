
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


