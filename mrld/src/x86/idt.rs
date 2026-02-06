//! Types for representing an interrupt descriptor table (IDT). 

use crate::x86::gdt::{
    KERNEL_CODE_SEL,
    KERNEL_DATA_SEL,
};

/// Marker type for an interrupt handler
#[derive(Clone, Copy)]
pub struct InterruptHandler;
/// Marker type for an interrupt handler (with an error code)
#[derive(Clone, Copy)]
pub struct InterruptHandlerErr;
/// Marker type for a diverging interrupt handler
#[derive(Clone, Copy)]
pub struct DivergingInterruptHandler;
/// Marker type for a diverging interrupt handler (with an error code)
#[derive(Clone, Copy)]
pub struct DivergingInterruptHandlerErr;

pub type InterruptHandlerFn = 
    unsafe extern "x86-interrupt" fn(InterruptStackFrame);
pub type InterruptHandlerErrFn = 
    unsafe extern "x86-interrupt" fn(InterruptStackFrame, err: u64);
pub type DivergingInterruptHandlerFn = 
    unsafe extern "x86-interrupt" fn(InterruptStackFrame) -> !;
pub type DivergingInterruptHandlerErrFn = 
    unsafe extern "x86-interrupt" fn(InterruptStackFrame, err: u64) -> !;

/// Trait for function pointers to interrupt handler routines
pub trait HandlerFn {
    fn as_usize(self) -> usize;
}
impl HandlerFn for InterruptHandlerFn { 
    fn as_usize(self) -> usize { self as _ }
}
impl HandlerFn for InterruptHandlerErrFn { 
    fn as_usize(self) -> usize { self as _ }
}
impl HandlerFn for DivergingInterruptHandlerFn { 
    fn as_usize(self) -> usize { self as _ }
}
impl HandlerFn for DivergingInterruptHandlerErrFn { 
    fn as_usize(self) -> usize { self as _ }
}

/// Marker trait for types of interrupt handler routines
pub trait HandlerKind: Clone + Copy {
    type Func: HandlerFn;
}
impl HandlerKind for InterruptHandler {
    type Func = InterruptHandlerFn;
}
impl HandlerKind for InterruptHandlerErr {
    type Func = InterruptHandlerErrFn;
}
impl HandlerKind for DivergingInterruptHandler {
    type Func = DivergingInterruptHandlerFn;
}
impl HandlerKind for DivergingInterruptHandlerErr {
    type Func = DivergingInterruptHandlerErrFn;
}



#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct InterruptStackFrame { 
    pub rip: u64,
    pub return_cs: u16,
    pub _reserved1: [u8; 6],
    pub return_rflags: u64,
    pub return_rsp: u64,
    pub return_ss: u16,
    pub _reserved2: [u8; 6],
}


#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct CallGateDescriptor { 
    tgt_off_00_15: u16,
    tgt_sel: u16,
    resv_04: u8,
    flags: u8,
    tgt_off_16_31: u16,
    tgt_off_32_63: u32,
    resv: u32,
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct InterruptGateDescriptor { 
    tgt_off_00_15: u16,
    tgt_sel: u16,
    ist: u8,
    flags: u8,
    tgt_off_16_31: u16,
    tgt_off_32_63: u32,
    resv: u32,
}


#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct IdtEntry<K: HandlerKind> {
    /// Target code segment offset [0:15]
    tgt_off_00_15: u16,
    /// Target code segment selector
    tgt_sel: u16,
    /// Interrupt Stack Table
    ist: u8,
    /// Flags (including the type, DPL, and the present bit)
    flags: u8,
    /// Target code segment offset [31:16]
    tgt_off_16_31: u16,
    /// Target code segment offset [63:32]
    tgt_off_32_63: u32,
    resv: u32,

    kind: core::marker::PhantomData<K>,
}
impl <K: HandlerKind> IdtEntry<K> { 
    const CALL_GATE_TYPE: u8 = 0b1100;
    const INT_GATE_TYPE: u8  = 0b1110;
    const TRAP_GATE_TYPE: u8 = 0b1111;
    const SIZE: usize = { 
        let sz = core::mem::size_of::<Self>();
        assert!(sz == 16, "IDT entries must be 16-bytes");
        sz
    };

    pub fn ptr_from_slice(bytes: &[u8; 16]) -> *const Self { 
        bytes.as_ptr() as *const Self
    }
    pub fn from_ptr(ptr: *const u8) -> *const Self { 
        ptr as *const Self
    }

    pub fn target_offset(&self) -> u64 { 
        (self.tgt_off_32_63 as u64) << 32 | 
        (self.tgt_off_16_31 as u64) << 16 | 
        self.tgt_off_00_15 as u64
    }

    pub fn ist_bits(&self) -> u8 { 
        self.ist & 0b0000_0111
    }

    pub fn type_bits(&self) -> u8 { 
        (self.flags & 0b0_00_0_1111)
    }
    pub fn dpl(&self) -> u8 { 
        (self.flags & 0b0_11_0_0000) >> 5
    }
    pub fn present(&self) -> bool { 
        (self.flags & 0b1_00_0_0000) != 0
    }

    pub const fn with_target_offset(mut self, offset: usize) -> Self { 
        self.tgt_off_00_15 = (offset & 0x0000_0000_0000_ffff) as u16;
        self.tgt_off_16_31 = ((offset & 0x0000_0000_ffff_0000) >> 16) as u16;
        self.tgt_off_32_63 = ((offset & 0xffff_ffff_0000_0000) >> 32) as u32;
        self
    }
    pub const fn with_target_selector(mut self, sel: u16) -> Self { 
        self.tgt_sel = sel;
        self
    }
    pub const fn with_type(mut self, ty: u8) -> Self { 
        self.flags |= ty & 0b1111;
        self
    }


    pub const fn with_present(mut self) -> Self { 
        self.flags |= 1 << 7;
        self
    }
    pub const fn with_dpl(mut self, dpl: u8) -> Self { 
        self.flags |= (dpl & 0b11) << 5;
        self
    }

    pub const fn empty() -> Self { 
        Self { 
            tgt_off_00_15: 0,
            tgt_sel: 0,
            ist: 0,
            flags: 0b0_00_0_0000,
            tgt_off_16_31: 0,
            tgt_off_32_63: 0,
            resv: 0,
            kind: core::marker::PhantomData,
        }
    }

    pub fn new_interrupt(func: K::Func) -> Self { 
        Self::empty()
            .with_type(Self::INT_GATE_TYPE)
            .with_target_selector(KERNEL_CODE_SEL.as_u16())
            .with_dpl(0b00)
            .with_present()
            .with_target_offset(func.as_usize())
    }

}

#[derive(Clone, Copy, Debug)]
pub enum IdtVector { 
    DivideByZero,
    Debug,
    Nmi,
    Breakpoint,
    Overflow,
    BoundRange,
    InvalidOpcode,
    DeviceNotAvail,
    DoubleFault,

    InvalidTss,
    SegmentNotPresent,
    Stack,
    GeneralProt,
    PageFault,

    X87Fp,
    AlignmentCheck,
    MachineCheck,
    SimdFp,

    ControlProt,
    HypervisorInj,
    VmmComm,
    Security,
}
impl Into<usize> for IdtVector { 
    fn into(self) -> usize { 
        match self { 
            Self::DivideByZero      => 0,
            Self::Debug             => 1,
            Self::Nmi               => 2,
            Self::Breakpoint        => 3,
            Self::Overflow          => 4,
            Self::BoundRange        => 5,
            Self::InvalidOpcode     => 6,
            Self::DeviceNotAvail    => 7,
            Self::DoubleFault       => 8,

            Self::InvalidTss        => 10,
            Self::SegmentNotPresent => 11,
            Self::Stack             => 12,
            Self::GeneralProt       => 13,
            Self::PageFault         => 14,

            Self::X87Fp             => 16,
            Self::AlignmentCheck    => 17,
            Self::MachineCheck      => 18,
            Self::SimdFp            => 19,

            Self::ControlProt       => 21,

            Self::HypervisorInj     => 28,
            Self::VmmComm           => 29,
            Self::Security          => 30,
        }
    }
}


#[repr(C, align(16))]
pub struct Idt { 
    pub de:  IdtEntry<InterruptHandler>,
    pub db:  IdtEntry<InterruptHandler>,
    pub nmi: IdtEntry<InterruptHandler>,
    pub bp:  IdtEntry<InterruptHandler>,
    pub of:  IdtEntry<InterruptHandler>,
    pub br:  IdtEntry<InterruptHandler>,
    pub ud:  IdtEntry<InterruptHandler>,
    pub nm:  IdtEntry<InterruptHandler>,

    pub df:  IdtEntry<InterruptHandler>,

    pub r9:  IdtEntry<InterruptHandler>,

    pub ts:  IdtEntry<InterruptHandlerErr>,
    pub np:  IdtEntry<InterruptHandlerErr>,
    pub ss:  IdtEntry<InterruptHandlerErr>,
    pub gp:  IdtEntry<InterruptHandlerErr>,

    pub pf:  IdtEntry<InterruptHandlerErr>,

    pub r15: IdtEntry<InterruptHandler>,
    pub mf:  IdtEntry<InterruptHandler>,
    pub ac:  IdtEntry<InterruptHandlerErr>,

    pub mc:  IdtEntry<InterruptHandler>,

    pub xf:  IdtEntry<InterruptHandler>,

    pub r20: IdtEntry<InterruptHandler>,

    pub cp:  IdtEntry<InterruptHandlerErr>,

    pub r22: IdtEntry<InterruptHandler>,
    pub r23: IdtEntry<InterruptHandler>,
    pub r24: IdtEntry<InterruptHandler>,
    pub r25: IdtEntry<InterruptHandler>,
    pub r26: IdtEntry<InterruptHandler>,
    pub r27: IdtEntry<InterruptHandler>,

    pub hv:  IdtEntry<InterruptHandler>,
    pub vc:  IdtEntry<InterruptHandlerErr>,
    pub sx:  IdtEntry<InterruptHandlerErr>,

    pub r31: IdtEntry<InterruptHandler>,

    pub usr: [IdtEntry<InterruptHandler>; Self::NUM_USER],

}
impl Idt { 
    // The lower 32 vectors are well-defined
    pub const NUM_USER: usize = 256 - 32;

    /// Create a new [empty] IDT. 
    pub const fn init() -> Self { 
        Self { 
            de: IdtEntry::empty(),
            db: IdtEntry::empty(),
            nmi: IdtEntry::empty(),
            bp: IdtEntry::empty(),
            of: IdtEntry::empty(),
            br: IdtEntry::empty(),
            ud: IdtEntry::empty(),
            nm: IdtEntry::empty(),
            df: IdtEntry::empty(),
            r9: IdtEntry::empty(),
            ts: IdtEntry::empty(),
            np: IdtEntry::empty(),
            ss: IdtEntry::empty(),
            gp: IdtEntry::empty(),
            pf: IdtEntry::empty(),
            r15: IdtEntry::empty(),
            mf: IdtEntry::empty(),
            ac: IdtEntry::empty(),
            mc: IdtEntry::empty(),
            xf: IdtEntry::empty(),
            r20: IdtEntry::empty(),
            cp: IdtEntry::empty(),
            r22: IdtEntry::empty(),
            r23: IdtEntry::empty(),
            r24: IdtEntry::empty(),
            r25: IdtEntry::empty(),
            r26: IdtEntry::empty(),
            r27: IdtEntry::empty(),
            hv: IdtEntry::empty(),
            vc: IdtEntry::empty(),
            sx: IdtEntry::empty(),
            r31: IdtEntry::empty(),
            usr: [IdtEntry::empty(); Self::NUM_USER],
        }
    }

    pub fn as_ptr(&self) -> *const Self { 
        self as *const Self
    }


}

