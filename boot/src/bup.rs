//! System bring-up during UEFI boot services. 

use core::ptr::NonNull;
use uefi::println;
use uefi::mem::memory_map::*;
use uefi::boot::{
    AllocateType,
    MemoryType
};

use mrld;
use mrld::physmem::{
    MrldMemoryMap, 
    MrldMemoryKind
};

/// Wait [indefinitely] for user input, then shut down the machine.
pub fn wait_for_shutdown() -> ! {
    use uefi::runtime::ResetType;
    use uefi::Status;
    println!("[*] Press any key to shut down the machine ...");
    let key_event = uefi::system::with_stdin(|stdin| { 
        stdin.wait_for_key_event().unwrap()
    });
    let mut events = [ key_event ];
    uefi::boot::wait_for_event(&mut events).unwrap();
    println!("[*] Shutting down in five seconds ...");
    uefi::boot::stall(core::time::Duration::from_secs(5));
    uefi::runtime::reset(ResetType::SHUTDOWN, Status::SUCCESS, None);
}

pub fn dump_memory_map() -> uefi::Result<()> { 
    use uefi::mem::memory_map::{
        MemoryType,
        MemoryMap,
    };
    let mm = uefi::boot::memory_map(MemoryType::LOADER_DATA)?;
    println!("[*] Memory descriptor size: {}B", mm.meta().desc_size);
    println!("[*] {} entries", mm.meta().entry_count());
    for entry in mm.entries() { 
        println!("{:<42?} {:016x} {:016x} {}", 
            entry.ty,
            entry.phys_start, entry.virt_start, entry.page_count
        );
    }
    Ok(())
}

/// Return the physical address of the RSDP table.
pub fn get_rsdp_addr() -> u64 { 
    use uefi::table::cfg::ACPI2_GUID;
    uefi::system::with_config_table(|tbl| {
        let rdsp = tbl.iter().find(|e| e.guid == ACPI2_GUID).unwrap();
        rdsp.address as u64
    })
}

/// Switch the UEFI console to mode 0.
pub fn do_console_init() {
    //use uefi::proto::console::text::OutputMode;
    uefi::system::with_stdout(|stdout| { 
        let tgt_mode = stdout.modes().find(|m| m.index() == 0).unwrap();
        stdout.set_mode(tgt_mode).unwrap();
        stdout.clear().unwrap();
    });
}

pub fn build_memory_map(uefi_map: &MemoryMapOwned, mrld_map: &mut MrldMemoryMap) {
    let mut idx = 0;
    for entry in uefi_map.entries() {

        // FIXME: Since this is running after exiting boot services, 
        // what is the correct way to fail here? 
        if idx >= MrldMemoryMap::NUM_ENTRIES {
            //println!("[*] Memory map capacity exceeded?");
            break;
        }

        let size = entry.page_count * (1 << 12);
        let paddr_base = entry.phys_start;
        let range = paddr_base..(paddr_base + size);
        let kind = match entry.ty { 
            MemoryType::LOADER_CODE |
            MemoryType::LOADER_DATA |
            MemoryType::BOOT_SERVICES_CODE |
            MemoryType::BOOT_SERVICES_DATA |
            MemoryType::CONVENTIONAL => {
                MrldMemoryKind::Available
            },
            MemoryType::RUNTIME_SERVICES_CODE |
            MemoryType::RUNTIME_SERVICES_DATA => {
                MrldMemoryKind::UefiRuntime
            },
            MemoryType::RESERVED |
            MemoryType::UNUSABLE |
            MemoryType::MMIO |
            MemoryType::MMIO_PORT_SPACE |
            MemoryType::PAL_CODE => {
                MrldMemoryKind::UefiReserved
            },
            MemoryType::ACPI_NON_VOLATILE => {
                MrldMemoryKind::AcpiNonVolatile
            }
            _ => MrldMemoryKind::Invalid,
        };

        // Just merge contiguous regions with the same type (?)
        let is_contiguous = if idx > 0 {
            range.start == mrld_map.entries[idx-1].range.end && 
            mrld_map.entries[idx-1].kind == kind
        } else { 
            false
        };
        if is_contiguous { 
            mrld_map.entries[idx-1].range.end += size;
        } else { 
            mrld_map.entries[idx].range = range;
            mrld_map.entries[idx].kind = kind;
            idx += 1;
        }
    }
}

/// Build a small set of page tables.
///
/// 0x0000_0000_0000_0000 - 0x0000_0080_0000_0000:  identity mapped
/// 0xffff_ffff_8000_0000 - 0xffff_ffff_c000_0000:  mrld kernel
///
/// NOTE: This is probably fine; we'll probably just be rebuilding these 
/// after booting into the kernel anyway.
///
pub unsafe fn build_page_tables() -> NonNull<u8> {
    use mrld::paging::*;

    let pml4t_ptr: NonNull<u8> = uefi::boot::allocate_pages(
        AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        1
    ).unwrap();
    let ident_pdp_ptr: NonNull<u8> = uefi::boot::allocate_pages(
        AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        1
    ).unwrap();

    let kernel_pdp_ptr: NonNull<u8> = uefi::boot::allocate_pages(
        AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        1
    ).unwrap();
    let kernel_pd_ptr: NonNull<u8> = uefi::boot::allocate_pages(
        AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        1
    ).unwrap();


    //let mut builder = PTBuilder::new();
    let pml4t = PageTable::<PML4>::mut_ref_from_ptr(pml4t_ptr.as_ptr());

    // Use 1GiB pages to identity map the low ~512GiB of physical memory.
    let pdpt = PageTable::<PDP>::mut_ref_from_ptr(ident_pdp_ptr.as_ptr());
    for idx in 0..512 { 
        pdpt.set_entry(PageTableIdx::new(idx), PageTableEntry::new(
            (idx as u64 * (1<<30)),
            PTFlag::P | PTFlag::RW | PTFlag::PS
        ));
    }
    pml4t.set_entry(PageTableIdx::new(0), PageTableEntry::new(
        pdpt.as_ptr() as u64,
        PTFlag::P | PTFlag::RW
    ));

    // Create a handful of 2MiB pages for the kernel mapping. 
    let v = VirtAddr::from_u64(0xffff_ffff_8000_0000);
    let (pml4_idx, pdp_idx, pd_idx, pt_idx) = v.decompose();
    let pdpt = PageTable::<PDP>::mut_ref_from_ptr(kernel_pdp_ptr.as_ptr());
    let pdt = PageTable::<PD>::mut_ref_from_ptr(kernel_pd_ptr.as_ptr());

    pml4t.set_entry(pml4_idx, PageTableEntry::new(
        pdpt.as_ptr() as u64,
        PTFlag::P | PTFlag::RW
    ));
    pdpt.set_entry(pdp_idx, PageTableEntry::new(
        pdt.as_ptr() as u64,
        PTFlag::P | PTFlag::RW
    ));

    // FIXME: This assumes a physical address
    for idx in 0..32 { 
        pdt.set_entry(PageTableIdx::new(idx), PageTableEntry::new(
            0x0400_0000 + (idx as u64 * (1 << 21)),
            PTFlag::P | PTFlag::RW | PTFlag::PS
        ));
    }
    pml4t_ptr
}

pub unsafe fn dump_dtrs() {
    println!("[*] Current UEFI GDTR/IDTR:");
    let gdtr = mrld::x86::GDTR::read();
    println!("  GDTR @ {:016x?} ({}B)", gdtr.ptr(), gdtr.size());
    for idx in 0..(gdtr.size() / 8) {
        let ptr = gdtr.ptr().offset(idx as isize);
        let val = ptr.read();
        let d = mrld::x86::gdt::Descriptor::new_from_u64(val);
        println!("    [{:04}]: {:x?}", idx, d);
    }

    let idtr = mrld::x86::IDTR::read();
    println!("  IDTR @ {:016x?} ({}B)", idtr.ptr(), idtr.size());
    for idx in 0..(idtr.size() / 8) {
        let ptr = idtr.ptr().offset(idx as isize);
        println!("    [{:04}]: {:016x}", idx, ptr.read());
    }
}


pub fn dump_pgtable(ptr: *const u8) {
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

