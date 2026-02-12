
use mrld::x86::idt::*;
use mrld::x86::dtr::*;
use mrld::x86::cr::*;
use crate::util;
use crate::println;

use spin;

pub static IDT: spin::Mutex<Idt> = {
    spin::Mutex::new(Idt::init())
};

/// Helper for managing the interrupt descriptor table.
pub struct IdtManager;
impl IdtManager { 
    /// Initialize the IDT and update the IDTR
    pub unsafe fn init() {
        let idt_ptr = { 
            let mut idt = IDT.lock();

            idt.de = IdtEntry::new_interrupt(de_handler as _);
            idt.db = IdtEntry::new_interrupt(db_handler as _);
            idt.nmi= IdtEntry::new_interrupt(nmi_handler as _);
            idt.bp = IdtEntry::new_interrupt(bp_handler as _);
            idt.of = IdtEntry::new_interrupt(of_handler as _);
            idt.br = IdtEntry::new_interrupt(br_handler as _);
            idt.ud = IdtEntry::new_interrupt(ud_handler as _);
            idt.nm = IdtEntry::new_interrupt(nm_handler as _);
            idt.df = IdtEntry::new_interrupt(df_handler as _);

            idt.ts = IdtEntry::new_interrupt(ts_handler as _);
            idt.np = IdtEntry::new_interrupt(np_handler as _);
            idt.ss = IdtEntry::new_interrupt(ss_handler as _);
            idt.gp = IdtEntry::new_interrupt(gp_handler as _);
            idt.pf = IdtEntry::new_interrupt(pf_handler as _);

            idt.mf = IdtEntry::new_interrupt(mf_handler as _);
            idt.ac = IdtEntry::new_interrupt(ac_handler as _);
            idt.mc = IdtEntry::new_interrupt(mc_handler as _);
            idt.xf = IdtEntry::new_interrupt(xf_handler as _);
            idt.cp = IdtEntry::new_interrupt(cp_handler as _);
            idt.hv = IdtEntry::new_interrupt(hv_handler as _);
            idt.vc = IdtEntry::new_interrupt(vc_handler as _);
            idt.sx = IdtEntry::new_interrupt(sx_handler as _);
            idt.as_ptr()
        };

        IDTR::write(&DescriptorTableRegister::new(512, idt_ptr as _));

    }
}


fn generic_handler_panic(s: &'static str, f: &InterruptStackFrame, err: Option<u64>) {
    println!("err={:016x?}", err);
    println!("CR2={:016x}", unsafe { CR2::read() });
    println!("{:x?}", f);
    panic!("panic for #{} at {:016x}!", s, f.rip);
}

macro_rules! decl_generic_handler {
    ($func:ident, $name:literal) => { 
        unsafe extern "x86-interrupt" fn $func(f: InterruptStackFrame) { 
            generic_handler_panic($name, &f, None);
        }
    }
}
macro_rules! decl_generic_handler_err {
    ($func:ident, $name:literal) => { 
        unsafe extern "x86-interrupt" fn $func(f: InterruptStackFrame, err: u64) { 
            generic_handler_panic($name, &f, Some(err));
        }
    }
}

decl_generic_handler!(de_handler, "de");
decl_generic_handler!(db_handler, "db");
decl_generic_handler!(nmi_handler, "nmi");
decl_generic_handler!(bp_handler, "bp");
decl_generic_handler!(of_handler, "of");
decl_generic_handler!(br_handler, "br");
decl_generic_handler!(ud_handler, "ud");
decl_generic_handler!(nm_handler, "nm");
decl_generic_handler!(df_handler, "df");

decl_generic_handler_err!(ts_handler, "ts");
decl_generic_handler_err!(np_handler, "np");
decl_generic_handler_err!(ss_handler, "ss");
decl_generic_handler_err!(gp_handler, "gp");
//decl_generic_handler_err!(pf_handler, "pf");
unsafe extern "x86-interrupt" fn pf_handler(f: InterruptStackFrame, err: u64) { 
    generic_handler_panic("PF", &f, Some(err));
}

decl_generic_handler!(mf_handler, "mf");

decl_generic_handler_err!(ac_handler, "ac");

decl_generic_handler!(mc_handler, "mc");
decl_generic_handler!(xf_handler, "xf");

decl_generic_handler_err!(cp_handler, "cp");

decl_generic_handler!(hv_handler, "hv");

decl_generic_handler_err!(vc_handler, "vc");
decl_generic_handler_err!(sx_handler, "sx");


