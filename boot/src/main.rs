#![no_std]
#![no_main]

#![feature(allocator_api)]

mod x86;
mod bup;
mod pxe;
mod smp;

// We're using the 'global_allocator' feature in the 'uefi' crate. 
extern crate alloc;
//use alloc::boxed::Box;
//use alloc::vec::Vec;
//use core::ptr::NonNull;

use uefi::prelude::*;
use uefi::println;
use uefi::runtime::ResetType;

#[entry]
fn efi_main() -> Status {
    uefi::helpers::init().unwrap();

    bup::do_console_init();
    println!("[*] HELO from mrld :^)");
    bup::do_acpi_init();

    use acpi::platform::ProcessorState;
    let platform_info = bup::get_platform_info();
    if let Some(proc_info) = &platform_info.processor_info {
        for ap in proc_info.application_processors.iter() {
            if ap.state == ProcessorState::Disabled {
                continue;
            }
            println!("ap #{}, lapic_id={}", ap.processor_uid, ap.local_apic_id);
        }
    }

    println!("[*] mrld entrypoint completed");

    pxe::download_kernel().unwrap();

    //match pxe::download_kernel() {
    //    Ok(_) => {},
    //    Err(e) => match e.status() {
    //        uefi::Status::TIMEOUT => {
    //            println!("[!] PXE timed out while requesting kernel?");
    //        },
    //        _ => {
    //            println!("[!] PXE error {}", e);
    //        },
    //    },
    //} 

    wait_for_shutdown();
    //Status::SUCCESS
}

//fn wait_for_kernel() -> ! { 
//    println!("[*] Press any key to jump into the kernel...");
//    let key_event = uefi::system::with_stdin(|stdin| { 
//        stdin.wait_for_key_event().unwrap()
//    });
//    let mut events = [ key_event ];
//    uefi::boot::wait_for_event(&mut events).unwrap();
//    unimplemented!("kernel entrypoint");
//}

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


