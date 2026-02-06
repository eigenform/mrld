//! 'mrld' kernel. 

#![allow(unsafe_op_in_unsafe_fn)]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]

#![no_std]
#![no_main]

mod macros;
mod serial;
mod util;
mod mm; 
mod start;
mod panic;
mod interrupt;
mod acpi;

use spin;
use mrld::x86::*;


use mrld::{
    MrldBootArgs
};

//unsafe fn dump_dtrs() {
//    println!("[*] Current kernel GDTR/IDTR:");
//    let gdtr = mrld::x86::GDTR::read();
//    println!("  GDTR @ {:016x?} ({}B)", gdtr.ptr(), gdtr.size());
//    for idx in 0..(gdtr.size() / 8) {
//        let ptr = gdtr.ptr().offset(idx as isize);
//        let val = ptr.read();
//        let d = mrld::x86::gdt::Descriptor::new_from_u64(val);
//        println!("    [{:04}]: {:x?}", idx, d);
//    }
//
//    //let idtr = mrld::x86::IDTR::read();
//    //println!("  IDTR @ {:016x?} ({}B)", idtr.ptr(), idtr.size());
//    //for idx in 0..(idtr.size() / 16) {
//    //    let ptr = idtr.ptr().offset(idx as isize *2);
//    //    //let ptr = IdtEntry::from_ptr(ptr as _);
//    //    println!("{:016x}", ptr.as_ref().unwrap().target_offset(), *ptr);
//    //}
//
//    //for idx in 0..(idtr.size() / 8) {
//    //    let ptr = idtr.ptr().offset(idx as isize);
//    //    println!("    [{:04}]: {:016x}", idx, ptr.read());
//    //}
//}


/// Kernel entrypoint [in Rust].
/// This function is entered from `_start()` in `src/start.rs`. 
#[unsafe(link_section = ".text")]
#[unsafe(no_mangle)]
pub extern "sysv64" fn kernel_main(args: *const MrldBootArgs) -> ! { 
    let args = unsafe { args.as_ref().unwrap() };

    unsafe { 
        // Initialize the COM1 port
        serial::COM1.lock().init();

        // Initialize the new IDT
        interrupt::IdtManager::init();

        acpi::AcpiManager::init(args.rsdp_addr);
    }

    let patch_level = unsafe { 
        Msr::rdmsr(Msr::PATCH_LEVEL)
    };

    println!("[*] HELO from the mrld kernel :^)");
    println!("Patch level {:08x}", patch_level);
    println!("[*] Waiting for messages ...");

    unsafe { mm::MrldPageTable::dump(); }


    let x = unsafe { 
        use core::alloc::*;
        core::ptr::read_volatile::<u64>(0xffff_ffff_c000_0000usize as _);
        mm::ALC.alloc(Layout::new::<[u8; 0x256]>());
    };

    unsafe { 
        core::arch::asm!("ud2");
    }

    panic!("and?");

    loop {}
}


