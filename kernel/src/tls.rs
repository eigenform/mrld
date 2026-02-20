
use mrld::x86::msr::*;
use mrld::x86::segment::GS;
use core::alloc::*;
use crate::mm;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum ThreadState { 
    Uninit = 0,
    Init = 0xff,
}

/// Local storage for a hardware thread. 
#[repr(C)]
pub struct Tls { 
    _p: *const Self,
    core_id: usize,
    state: ThreadState,
}
impl Tls { 
    /// Initialize local storage for this hardware thread. 
    pub unsafe fn init(core_id: usize) { 
        // Allocate some backing memory
        let ptr: *mut Self = mm::HEAP.alloc(Layout::new::<Self>()) as _;

        let mut tls = ptr.as_mut_unchecked();
        tls._p = ptr;
        tls.state = ThreadState::Init;
        tls.core_id = core_id;

        // Let the GS segment point to local storage
        Msr::wrmsr(Msr::GS_BASE, ptr as _);
    }
}

impl Tls { 
    pub fn is_bsp(&self) -> bool { 
        self.core_id == 0
    }
    pub fn state(&self) -> ThreadState { 
        self.state
    }
}


/// Synthesize a static reference to this object. 
///
/// This assumes that `GS_BASE` is always set to the start of this object,
/// and that the first member in [`Tls`] is also a pointer to this object.
/// 
impl Tls {
    unsafe fn ptr_from_gs() -> *mut Self { 
        let x: u64;
        core::arch::asm!("mov {}, gs:[0]", out(reg) x);
        x as _
    }

    pub fn as_ref() -> &'static Self {
        unsafe { Self::ptr_from_gs().as_ref_unchecked() }
    }
    pub fn as_mut() -> &'static mut Self { 
        unsafe { Self::ptr_from_gs().as_mut_unchecked() }
    }
}
