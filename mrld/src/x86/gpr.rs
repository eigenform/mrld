use core::arch::asm;

pub struct Gpr; 
impl Gpr { 
    #[inline(always)]
    pub fn rip() -> u64 { 
        let res: u64;
            unsafe { asm!(
                "lea rax, [rip]",
                out("rax") res,
                options(nomem, nostack, preserves_flags)
            );
        }
        res
    }

    #[inline(always)]
    pub fn rsp() -> u64 { 
        let res: u64;
            unsafe { asm!(
                "mov rax, rsp",
                out("rax") res,
                options(nomem, nostack, preserves_flags)
            );
        }
        res
    }

    #[inline(always)]
    pub fn rbp() -> u64 { 
        let res: u64;
            unsafe { asm!(
                "mov rax, rsp",
                out("rax") res,
                options(nomem, nostack, preserves_flags)
            );
        }
        res
    }
}


