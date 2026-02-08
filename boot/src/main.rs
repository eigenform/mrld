//! 'mrld' UEFI bootloader.
//!
//! The process is here is probably going to be something like:
//!
//! - Set up arguments passed to the kernel
//! - Download the kernel over PXE
//! - Load the kernel into physical memory
//! - Set up and switch into a new set of page tables
//! - Set up and switch into new interrupt tables
//! - Jump into the kernel

#![no_std]
#![no_main]
#![allow(unsafe_op_in_unsafe_fn)]

// We're using the 'global_allocator' feature in the 'uefi' crate. 
// We can probably reclaim memory later when the kernel is running.
#![feature(allocator_api)]

mod bup;
mod pxe;
mod smp;

use core::ptr::NonNull;
use uefi::prelude::*;
use uefi::println;
use uefi::runtime::ResetType;
use uefi::boot::{ AllocateType, MemoryType };
use mrld::{ MrldBootArgs, };
use mrld::x86;

#[entry]
fn efi_main() -> Status {
    uefi::helpers::init().unwrap();
    bup::do_console_init();

    println!("[*] HELO from the mrld boot-stub :^)");
    println!("  Firmware Vendor:   {}", uefi::system::firmware_vendor());
    println!("  Firmware Revision: {}", uefi::system::firmware_revision());

    // Allocate for boot arguments and synthesize a mutable reference to them.
    let boot_args: &mut MrldBootArgs = unsafe { 
        let ptr: NonNull<u8> = uefi::boot::allocate_pages(
            AllocateType::AnyPages,
            MemoryType::LOADER_DATA,
            (core::mem::size_of::<MrldBootArgs>() / uefi::boot::PAGE_SIZE) + 1
        ).unwrap();

        let mut boot_args_ptr: NonNull<MrldBootArgs> = ptr.cast();
        boot_args_ptr.write(MrldBootArgs::new_empty());
        boot_args_ptr.as_mut()
    };

    // Fill in the physical address of the RDSP table.
    // NOTE: We can parse ACPI tables in the kernel later if we need to.
    boot_args.rsdp_addr = { 
        use uefi::table::cfg::ACPI2_GUID;
        uefi::system::with_config_table(|tbl| {
            let rdsp = tbl.iter().find(|e| e.guid == ACPI2_GUID).unwrap();
            rdsp.address as u64
        })
    };

    // Download the kernel image via PXE.
    let img = pxe::KernelImage::download().map_err(|e| {
        println!("[!] Error downloading kernel: {}", e);
        bup::wait_for_shutdown();
    }).unwrap();
    println!("[!] Downloaded kernel ...");

    // Load the kernel into physical memory and find the entrypoint
    let kernel_entrypt = unsafe { img.load().unwrap() };
    println!("[!] Loaded kernel into memory ...");

    // Build a new set of page tables
    let pml4_ptr = unsafe { 
        let res = bup::build_page_tables();
        //dump_pgtable(res.as_ptr());
        res
    };
    println!("[!] Wrote provisional page tables ...");


    unsafe { 
        // Exit UEFI boot services
        let uefi_map = uefi::boot::exit_boot_services(None);

        // Build the memory map passed to the kernel
        bup::build_memory_map(&uefi_map, &mut boot_args.memory_map);

        // Switch to the new set of page tables
        mrld::x86::CR3::write(pml4_ptr.as_ptr() as u64);

        // Transfer control into the kernel
        kernel_entrypt(boot_args.as_ptr());
    }
}


