

use modular_bitfield::prelude::*;
use modular_bitfield::bitfield;

#[bitfield] 
#[repr(u64)]
pub struct ApicBar { 
    _r0: B8,
    /// Bootstrap core
    pub bsc: bool,
    _r9: B1,
    /// X2APIC enable
    pub extd: bool,
    /// APIC enable
    pub ae: bool,
    /// APIC base address
    pub aba: B40,
    _r52: B12,
}


#[bitfield] 
#[repr(u32)]
pub struct LocalApicId { 
    _r: B24,
    pub aid: B8,
}

#[bitfield] 
#[repr(u32)]
pub struct ApicVersion { 
    pub ver: B8,
    _r8: B8,
    pub mle: B8,
    _r24: B7,
    pub eas: bool,
}

#[bitfield] 
#[repr(u32)]
pub struct XapicFeature { 
    pub inc: bool,
    pub snic: bool,
    pub xaidc: bool,
    _r3: B13,
    pub xlc: B8,
    _r24: B8,
}

#[bitfield] 
#[repr(u32)]
pub struct XapicControl { 
    pub iern: bool,
    pub sn: bool,
    pub xaidn: bool,
    pub _r3: B29,
}

#[bitfield] 
#[repr(u32)]
pub struct GeneralLvtRegister { 
    pub vec: B8,
    pub mt: B3,
    _r11: B1,
    pub ds: bool,
    _r13: B1,
    pub rir: bool,
    pub tgm: bool,
    pub m: bool,
    pub tmm: bool,
    pub _r18: B14,
}

#[bitfield] 
#[repr(u32)]
pub struct TimerLvtRegister { 
    pub vec: B8,
    pub _r8: B4,
    pub ds: bool,
    _r13: B3,
    pub m: bool,
    pub tmm: bool,
    pub _r18: B14,
}

#[bitfield] 
#[repr(u32)]
pub struct DivideConfig { 
    pub dv_1_0: B2,
    pub _r2: B1,
    pub dv_2: B1,
    pub _r4: B28,
}

#[bitfield] 
#[repr(u32)]
pub struct LocalIntLvtRegister { 
    pub vec: B8,
    pub mt: B3,
    _r11: B1,
    pub ds: bool,
    _r13: B3,
    pub rir: bool,
    pub tgm: bool,
    pub m: bool,
    pub _r17: B13,
}

#[bitfield] 
#[repr(u32)]
pub struct PmcLvtRegister { 
    pub vec: B8,
    pub mt: B3,
    _r11: B1,
    pub ds: bool,
    _r13: B3,
    pub m: bool,
    pub _r17: B15,
}

#[bitfield] 
#[repr(u32)]
pub struct ApicErrLvtRegister { 
    pub vec: B8,
    pub mt: B3,
    _r11: B1,
    pub ds: bool,
    _r13: B3,
    pub m: bool,
    pub _r17: B15,
}

#[bitfield] 
#[repr(u32)]
pub struct ApicErrStatus { 
    _r0: B2,
    pub sae: bool,
    pub rae: bool,
    _r4: B1,
    pub siv: bool,
    pub riv: bool,
    pub ira: bool,
    _r8: B24,
}

#[bitfield] 
#[repr(u32)]
pub struct SpuriousIntr { 
    pub vec: B8,
    pub ase: bool,
    pub fcc: bool,
    _r10: B22,
}

#[bitfield] 
#[repr(u64)]
pub struct IntrCommand { 
    pub vec: B8,
    pub mt: B3,
    pub dm: bool,
    pub ds: bool,
    _r13: B1,
    pub l: bool,
    pub tgm: bool,
    pub rrs: B2,
    pub dsh: B2,
    _r20: B36,
    pub des: B8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType { 
    Fixed      = 0b000,
    LowestPri  = 0b001,
    Smi        = 0b010,
    RemoteRead = 0b011,
    Nmi        = 0b100,
    Init       = 0b101,
    Startup    = 0b110,
    ExtIntr    = 0b111,
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Destination { 
    Destination  = 0b00,
    This         = 0b01,
    AllIncl      = 0b10,
    AllExcl      = 0b11,
}










