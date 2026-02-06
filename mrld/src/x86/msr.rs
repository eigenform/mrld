

pub struct Msr;
impl Msr { 
    pub const APIC_BAR: u32 = 0x0000_001b;
    pub const PATCH_LEVEL: u32 = 0x0000_008b;
    pub const SPEC_CTRL: u32 = 0x0000_0048;

    pub const EFER: u32 = 0xc000_0080;

    pub const SYS_CFG: u32 = 0xc001_0010;
    pub const HWCR: u32    = 0xc001_0015;
    pub const VM_CR: u32   = 0xc001_0114;

    pub const PERF_CTL: [u32; 6]  = [
        0xc001_0200,
        0xc001_0202,
        0xc001_0204,
        0xc001_0206,
        0xc001_0208,
        0xc001_020a,
    ];

    pub const PERF_CTR: [u32; 6]  = [
        0xc001_0201,
        0xc001_0203,
        0xc001_0205,
        0xc001_0207,
        0xc001_0209,
        0xc001_020b,
    ];

    pub const LS_CFG: u32  = 0xc001_1020;
    pub const IC_CFG: u32  = 0xc001_1021;
    pub const DC_CFG: u32  = 0xc001_1022;
    pub const FP_CFG: u32  = 0xc001_1028;
    pub const DE_CFG: u32  = 0xc001_1029;

    pub const IBS_FETCH_CTL: u32     = 0xc001_1030;
    pub const IBS_FETCH_LINADDR: u32 = 0xc001_1031;
    pub const IBS_FETCH_PHYADDR: u32 = 0xc001_1032;
    pub const IBS_OP_CTL: u32        = 0xc001_1033;
    pub const IBS_OP_RIP: u32        = 0xc001_1034;
    pub const IBS_OP_DATA: u32       = 0xc001_1035;
    pub const IBS_OP_DATA2: u32      = 0xc001_1036;
    pub const IBS_OP_DATA3: u32      = 0xc001_1037;
    pub const IBS_DC_LINADDR: u32    = 0xc001_1038;
    pub const IBS_DC_PHYADDR: u32    = 0xc001_1039;
    pub const IBS_CTL: u32           = 0xc001_103a;
    pub const BP_IBSTGT_RIP: u32     = 0xc001_103b;

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

}

