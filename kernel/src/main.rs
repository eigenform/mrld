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
mod physmem;
mod paging;
mod start;
mod panic;
mod interrupt;
mod acpi;

extern crate alloc;

use spin;
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

    // Initialize serial port
    unsafe { 
        serial::COM2.lock().init();
    }

    println!("[*] HELO from the mrld kernel :^)");

    // Write and switch into a new IDT
    unsafe {
        interrupt::IdtManager::init();
    }

    // Initialize our memory map with data passed from UEFI. 
    // Reserve physical regions for paging and the heap. 
    let (pt_desc, heap_desc) = unsafe { 
        let mut mmap = physmem::MEMORY_MAP.lock();
        mmap.init(&args);

        let Some(pt_desc) = mmap.allocate(
            mrld::paging::PageSize::Size2MiB, 
            32, 
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

    // Initialize page tables
    unsafe { 
        let mut pt = paging::PAGE_TABLE.lock();
        pt.init(pt_desc, heap_desc);
    }

    // Initialize the kernel heap
    unsafe { 
        mm::HEAP.init();
    }

    // Initialize ACPI
    unsafe { 
        acpi::ACPI.init(args.rsdp_addr);
    }


    { 
        let map = physmem::MEMORY_MAP.lock();
        for entry in map.iter_valid() { 
            println!("{:016x}:{:016x} {:?}", 
                entry.start(), entry.end(), entry.kind
            );
        }
    }

    let patch_level = Msr::rdmsr(Msr::PATCH_LEVEL);
    println!("Patch level {:08x}", patch_level);
    println!("[*] Waiting for messages ...");

    let x = unsafe { 
        use core::alloc::*;
        mm::HEAP.alloc(Layout::new::<[u8; 0x1000]>());
    };

    unsafe { 
        core::arch::asm!("ud2");
    }

    panic!("and?");

    loop {}
}


