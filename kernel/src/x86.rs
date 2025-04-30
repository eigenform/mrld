use core::arch::asm;

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


#[inline(always)]
pub unsafe fn rdmsr(msr: u32) -> u64 { 
    let lo: u32;
    let hi: u32;
    asm!(
        "rdmsr",
        in("rcx") msr,
        out("rax") lo,
        out("rdx") hi,
        options(nomem, nostack, preserves_flags)
    );
    (hi as u64) << 32 | lo as u64
}

#[inline(always)]
pub unsafe fn wrmsr(msr: u32, val: u64) {
    let lo: u32 = val as u32;
    let hi: u32 = (val >> 32) as u32;
    asm!(
        "wrmsr",
        in("rcx") msr,
        in("rax") lo,
        in("rdx") hi,
        options(nomem, nostack, preserves_flags)
    );
}

