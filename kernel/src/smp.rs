
use mrld::x86::apic::*;
use mrld::physmem::*;
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

    // FIXME: Actually do this in a sane way
    pub unsafe fn init() { 
        use core::alloc::*;
        use alloc::alloc::alloc;

        let stac_lo = alloc(
            core::alloc::Layout::new::<[u8; 0x1000]>()
            .align_to(0x1000).unwrap()
        );
        stac_lo.write_bytes(0, 0x1000);
        let stac_hi = stac_lo.offset(0x1000);

        trampoline::Trampoline::write(
            ap_entry as _,
            mrld::x86::CR3::read(),
            stac_hi as u64 - 16,
        );

        apic::Lapic::send_sipi(1);

        // FIXME: Wait around for APs to check back in
        unsafe { 
            loop { mrld::x86::pause(); }
        }


    }
}


// NOTE: APs enter this from the trampoline in 64-bit mode with paging. 
//
// FIXME:
// - There's no IDT
// - There's some provisional GDT
// - We're using PML4 from the bootstrap core
// - Actually do something
//
pub unsafe fn ap_entry() -> ! { 
    use crate::tls::*;
    let apic_id = mrld::x86::cpuid(0xb, 0).edx;
    println!("HELO from AP {}", apic_id);

    Tls::init(apic_id as _);

    unsafe { 
        loop { mrld::x86::pause(); }
    }
}
