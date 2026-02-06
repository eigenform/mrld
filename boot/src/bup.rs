//! System bring-up during UEFI boot services. 

use core::ptr::NonNull;
use core::ffi::c_void;
use uefi::println;
use uefi::mem::memory_map::*;
use uefi::boot::{
    AllocateType,
    MemoryType
};

use crate::pxe::KernelImage;
use mrld;
use mrld::physmem::{
    MrldMemoryMap, 
    MrldMemoryKind
};

pub fn dump_memory_map() -> uefi::Result<()> { 
    use uefi::mem::memory_map::{
        MemoryType,
        MemoryMap,
    };
    let mm = uefi::boot::memory_map(MemoryType::BOOT_SERVICES_DATA)?;
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
        if idx >= MrldMemoryMap::NUM_ENTRIES {
            println!("[*] Memory map capacity exceeded?");
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

            crate::BOOT_ARGS_DATA => {
                MrldMemoryKind::BootArgs
            },
            KernelImage::MEMORY_TYPE => {
                MrldMemoryKind::KernelImage
            },
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


/// Ad-hoc helper for managing the physical backing for a set of page tables.
pub struct PTBuilder {
    base: NonNull<u8>,
    pages: usize,
    next: usize,
}
impl PTBuilder { 
    pub fn new() -> Self { 
        let num_pages = 2048;
        let ptr: NonNull<u8> = uefi::boot::allocate_pages(
            AllocateType::AnyPages,
            crate::PAGE_TABLE_DATA,
            num_pages
        ).unwrap();

        Self { 
            base: ptr,
            pages: num_pages, 
            next: 0,
        }
    }
    pub fn alloc(&mut self) -> *mut u8 {
        if self.next >= self.pages { 
            panic!("oops");
        }
        let res = unsafe { 
            self.base.offset((1<<12) * self.next as isize)
        };
        self.next += 1;
        res.as_ptr()
    }
}

/// Build a small set of page tables.
///
/// 0x0000_0000_0000_0000 - 0x0000_0080_0000_0000
/// 0xffff_ffff_8000_0000 - 0xffff_ffff_8000_0000
///
pub unsafe fn build_page_tables() -> NonNull<u8> {
    use mrld::paging::*;

    let mut builder = PTBuilder::new();
    let pml4t = PageTable::<PML4>::mut_ref_from_ptr(builder.alloc());

    // Use 1GiB pages to identity map the low ~512GiB of physical memory.
    let pdpt = PageTable::<PDP>::mut_ref_from_ptr(builder.alloc());
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
    let pdpt = PageTable::<PDP>::mut_ref_from_ptr(builder.alloc());
    let pdt = PageTable::<PD>::mut_ref_from_ptr(builder.alloc());

    pml4t.set_entry(pml4_idx, PageTableEntry::new(
        pdpt.as_ptr() as u64,
        PTFlag::P | PTFlag::RW
    ));
    pdpt.set_entry(pdp_idx, PageTableEntry::new(
        pdt.as_ptr() as u64,
        PTFlag::P | PTFlag::RW
    ));

    for idx in 0..32 { 
        pdt.set_entry(PageTableIdx::new(idx), PageTableEntry::new(
            0x0400_0000 + (idx as u64 * (1 << 21)),
            PTFlag::P | PTFlag::RW | PTFlag::PS
        ));
    }
    builder.base
}


