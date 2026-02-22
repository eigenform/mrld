
use mrld::x86::apic::*;
use crate::apic;
use crate::println;
use crate::trampoline;

unsafe extern "C" { 
    #[link_name = "_trampoline_start_vaddr"]
    pub static _TRAMPOLINE_START16: u64;
    #[link_name = "_trampoline_end_vaddr"]
    pub static _TRAMPOLINE_END: u64;
}

pub struct Smp;
impl Smp { 
    pub fn wait() { 
        loop {}
    }
    pub unsafe fn init() { 
        const TRAMPOLINE_PHYS: usize = 0x0000_8000;

        let mut tgt_ptr = (trampoline::Trampoline::PHYS_BASE as *mut u8);
        let src_ptr = trampoline::Trampoline::as_ptr();
        let trampoline_len = trampoline::Trampoline::DATA.len();

        tgt_ptr.write_bytes(0, 0x8000);
        tgt_ptr.copy_from_nonoverlapping(src_ptr, trampoline_len);

        apic::Lapic::send_sipi(1);

        Self::wait()


    }
}


