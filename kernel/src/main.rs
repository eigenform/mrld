//! 'mrld' kernel. 

#![allow(unsafe_op_in_unsafe_fn)]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]
#![feature(ascii_char)]

#![no_std]
#![no_main]

mod macros;
mod serial;
mod util;
mod mm; 
mod physmem;
mod paging;
mod start;
mod panic;
mod interrupt;
mod tls;
mod acpi;
mod apic; 
mod smp;
mod trampoline; 

extern crate alloc;

use mrld::x86::*;
use mrld::{
    MrldBootArgs
};

/// Kernel entrypoint [in Rust].
/// This function is entered from `_start()` in `src/start.rs`. 
#[unsafe(link_section = ".text")]
#[unsafe(no_mangle)]
pub extern "sysv64" fn kernel_main(args: *const MrldBootArgs) -> ! { 
    let args = unsafe { args.as_ref().unwrap() };

    // I guess we can use the APIC ID as a core ID for now
    let apic_id = mrld::x86::cpuid(0xb, 0).edx;

    unsafe {
        // Initialize serial port as soon as possible
        serial::COM2.lock().init();
        println!("[*] HELO from the mrld kernel, on core {} :^)", apic_id);

        // Write and switch into a new IDT
        interrupt::IdtManager::init();

        apic::Lapic::init();
    }

    // Initialize our memory map with data passed from UEFI. 
    // Reserve physical regions for paging and the heap. 
    let (pt_desc, heap_desc) = unsafe { 
        let mut mmap = physmem::MEMORY_MAP.lock();
        mmap.init(&args);

        let Some(pt_desc) = mmap.allocate(
            mrld::paging::PageSize::Size2MiB, 
            8, 
            mrld::physmem::MrldMemoryKind::KernelPaging
        ) else { 
            panic!("Couldn't reserve physical memory for page tables?");
        };

        let Some(heap_desc) = mmap.allocate(
            mrld::paging::PageSize::Size1GiB, 
            1, 
            mrld::physmem::MrldMemoryKind::KernelHeap
        ) else {
            panic!("Couldn't reserve physical memory for kernel heap?");
        };
        (pt_desc, heap_desc)
    };

    unsafe { 
        // Initialize page tables
        let mut pt = paging::PAGE_TABLE.lock();
        pt.init(pt_desc, heap_desc);

        // Initialize the global allocator and kernel heap
        mm::HEAP.init();

        // Initialize thread-local storage
        tls::Tls::init(apic_id as _);
    }

    // Initialize ACPI
    let mut acpi = unsafe { 
        let mut mgr = acpi::MrldAcpiManager::new(args.rsdp_addr);
        mgr.init();
        if mgr.maybe_guest() { 
            println!("[*] Assuming you're on virtualized hardware ...");
        }
        mgr
    };



    println!("[*] Memory map:");
    { 
        let map = physmem::MEMORY_MAP.lock();
        for entry in map.iter_valid() { 
            println!("  {:016x}:{:016x} {:?}", 
                entry.start(), entry.end(), entry.kind
            );
        }
    }

    let cpuid = mrld::x86::cpuid(1, 0).eax; 
    let patch_level = Msr::rdmsr(Msr::PATCH_LEVEL);
    println!("[*] CPUID: {:08x}, patch level {:08x}", cpuid, patch_level);

    let x = tls::Tls::as_ref().state();

    unsafe { 
        smp::Smp::init();
    }



    unsafe { 
        println!("[!] Going for shutdown (hopefully) ...");
        acpi.enter_s5_state(5);
    }

    unsafe { 
        core::arch::asm!("ud2");
    }

    panic!("and?");

    loop {}
}


