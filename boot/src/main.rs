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

pub const BOOT_ARGS_DATA:  MemoryType = MemoryType::custom(0x8000_0000);
pub const KERNEL_IMG_DATA: MemoryType = MemoryType::custom(0x8000_0001);
pub const PAGE_TABLE_DATA: MemoryType = MemoryType::custom(0x8000_0002);

/// Default page size (4KiB)
pub const PAGE_SZ: usize = (1 << 12);

#[entry]
fn efi_main() -> Status {
    uefi::helpers::init().unwrap();

    bup::do_console_init();

    println!("[*] HELO from the mrld boot-stub :^)");
    println!("[*] Firmware Vendor:   {}", uefi::system::firmware_vendor());
    println!("[*] Firmware Revision: {}", uefi::system::firmware_revision());

    // Allocate for boot arguments and synthesize a mutable reference to them.
    let boot_args: &mut MrldBootArgs = unsafe { 
        let num_pages: usize = {
            (core::mem::size_of::<MrldBootArgs>() / PAGE_SZ) + 1
        };
        let ptr: NonNull<u8> = uefi::boot::allocate_pages(
            AllocateType::AnyPages,
            BOOT_ARGS_DATA,
            num_pages
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
        wait_for_shutdown();
    }).unwrap();

    // Load the kernel into physical memory and find the entrypoint
    let kernel_entrypt = unsafe { img.load() };

    // Build a new set of page tables
    let pml4_ptr = unsafe { 
        let res = bup::build_page_tables();
        //dump_pgtable(res.as_ptr());
        res
    };

    unsafe { 
        // Exit UEFI boot services
        let uefi_map = uefi::boot::exit_boot_services(
            MemoryType::BOOT_SERVICES_DATA
        );

        // Copy over the memory map into our boot args
        bup::build_memory_map(&uefi_map, &mut boot_args.memory_map);

        // Switch to the new set of page tables
        mrld::x86::CR3::write(pml4_ptr.as_ptr() as u64);

        // Transfer control into the kernel
        kernel_entrypt(boot_args.as_ptr());
    }
}

fn dump_pgtable(ptr: *const u8) {
    use mrld::paging::*;
    let pml4_table = unsafe { 
        PageTable::<PML4>::ref_from_ptr(ptr) 
    };
    println!("PML4 Table: {:016x?}", pml4_table.as_ptr());
    let mut cnt = 0;
    for pml4e in pml4_table.entries() {
        if pml4e.invalid() {
            continue;
        }
        if cnt > 0 { break; }
        println!("  {:?}", pml4e);
        let pdp_table = unsafe { 
            PageTable::<PDP>::ref_from_ptr(pml4e.address() as *const u8)
        };
        for pdpe in pdp_table.entries() {
            println!("   {:?}", pdpe);
        }
        cnt += 1;
    }
}


/// Wait [indefinitely] for user input, then shut down the machine.
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

