#![no_std]
#![no_main]

mod x86;
mod bup;
mod pxe;

// We're using the 'global_allocator' feature in the 'uefi' crate. 
extern crate alloc;
//use alloc::boxed::Box;
//use alloc::vec::Vec;
//use core::ptr::NonNull;

use uefi::prelude::*;
use uefi::println;
use uefi::runtime::ResetType;
//use uefi::proto::pi::mp::MpServices;

// NOTE: Apparently you can run on other APs with MP_SERVICES
//fn run_on_all(cb: fn()) -> uefi::Result<()> {
//    let handle = uefi::boot::get_handle_for_protocol::<MpServices>()?;
//    let mp_services = uefi::boot::open_protocol_exclusive::<MpServices>(handle)?;
//    cb();
//    if let Err(e) = mp_services.startup_all_aps(true, run_cb, cb as *mut _, None, None) {
//        if e.status() != Status::NOT_STARTED {
//            return Err(e);
//        }
//    }
//    Ok(())
//}
//
//extern "efiapi" fn run_cb(content: *mut core::ffi::c_void) {
//    let cb: fn() = unsafe { core::mem::transmute(content) };
//    cb();
//}


#[entry]
fn efi_main() -> Status {
    uefi::helpers::init().unwrap();

    bup::do_console_init();
    println!("[*] HELO from mrld :^)");
    bup::do_acpi_init();

    //let mm = uefi::boot::memory_map(MemoryType::LOADER_DATA).unwrap();
    //println!("[*] Memory map:");
    //for entry in mm.entries() { 
    //    println!("  phys={:016x} virt={:016x}", entry.phys_start, entry.virt_start);
    //}

    println!("[*] mrld entrypoint completed");

    wait_for_shutdown();
    //Status::SUCCESS
}

fn wait_for_shutdown() -> ! {
    println!("[*] Press any key to shut down the machine ...");
    let key_event = uefi::system::with_stdin(|stdin| { 
        stdin.wait_for_key_event().unwrap()
    });
    let mut events = [ key_event ];
    uefi::boot::wait_for_event(&mut events).unwrap();
    println!("[*] Shutting down in five seconds ...");
    uefi::boot::stall(5_000_000);
    uefi::runtime::reset(ResetType::SHUTDOWN, Status::SUCCESS, None);
}


