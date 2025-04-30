
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


