
use mrld::paging::*;
use mrld::physmem::*;
use spin::Mutex;

use crate::println;
use crate::physmem::*;
use crate::mm::{ 
    KERNEL_HEAP_BASE,
    KERNEL_TEXT_BASE
};

pub static PAGE_TABLE: Mutex<MrldPageTable> = { 
    Mutex::new(MrldPageTable::new_empty())
};

/// Helper for managing page tables.
pub struct MrldPageTable { 
    /// Physical backing region for these page tables
    desc: MrldMemoryDesc,

    /// Physical address of the next available 4KiB page 
    next_page: u64,
}
impl MrldPageTable { 
    pub const fn new_empty() -> Self { 
        Self { 
            desc: MrldMemoryDesc::new_invalid(),
            next_page: 0,
        }
    }

    /// Allocate a new 4KiB page from the physical backing
    unsafe fn allocate(&mut self) -> *mut u8 { 
        let p = self.next_page as *mut u8;
        if self.next_page + u64::from(PageSize::Size4KiB) >= self.desc.end() {
            panic!("we ran out of physical memory for page tables?")
        }

        self.next_page += u64::from(PageSize::Size4KiB);
        p
    }

    /// Initialize the page tables.
    ///
    /// This performs the following steps: 
    ///
    /// - Allocate a new PML4 table
    /// - Allocate and create some default mappings
    /// - Commit the new PML4 to CR3
    ///
    pub unsafe fn init(&mut self, 
        pt_desc: MrldMemoryDesc,
        heap_desc: MrldMemoryDesc,
    ) { 
        self.desc = pt_desc;
        self.next_page = pt_desc.start();

        // Initialize the physical backing to zero
        let mut ptr = pt_desc.start() as *mut u8;
        ptr.write_bytes(0, pt_desc.size() as _);

        let pml4_ptr = self.allocate();
        let mut pml4 = PageTable::<PML4>::mut_ref_from_ptr(pml4_ptr as _);

        self.map_pages(&mut pml4, 
            0x0000_0000_0000_0000,
            0x0000_0000_0000_0000,
            PageSize::Size1GiB,
            512
        );
        self.map_pages(&mut pml4, 
            KERNEL_TEXT_BASE,
            KERNEL_PHYS_BASE,
            PageSize::Size2MiB,
            32
        );
        self.map_pages(&mut pml4, 
            KERNEL_HEAP_BASE,
            heap_desc.start(),
            PageSize::Size1GiB,
            1
        );

        // Self::dump(&pml4);
        mrld::x86::CR3::write(pml4.as_ptr() as _);

    }

    /// Map a single page of physical memory. 
    pub unsafe fn map_page(
        &mut self, 
        pml4: &mut PageTable<PML4>,
        base_vaddr: u64,
        base_paddr: u64,
        pagesz: PageSize,
    )
    {
        assert!(base_vaddr & (u64::from(pagesz) - 1) == 0);
        assert!(base_paddr & (u64::from(pagesz) - 1) == 0);
        let vaddr = VirtAddr::from_u64(base_vaddr);
        let (pml4_idx, pdp_idx, pd_idx, pt_idx) = vaddr.decompose();

        // Get a mutable reference to the PDP table (or allocate a new one)
        let pml4e = pml4.get_mut(pml4_idx);
        let pdp = if let Some(pdp) = pml4e.as_mut_table() { 
            pdp
        } 
        else {
            let mut pdp = PageTable::<PDP>::mut_ref_from_ptr(self.allocate());
            let entry = PageTableEntry::<PML4>::new_table_ptr(pdp.as_ptr());
            pml4.set_entry(pml4_idx, entry);
            pdp
        };

        if pagesz == PageSize::Size1GiB { 
            let entry = PageTableEntry::new(
                base_paddr, 
                PTFlag::P | PTFlag::RW | PTFlag::PS
            );
            pdp.set_entry(pdp_idx, entry);
            return;
        }

        // Get a mutable reference to the PD table (or allocate a new one)
        let pdpe = pdp.get_mut(pdp_idx);
        let pd = if let Some(pd) = pdpe.as_mut_table() { 
            pd
        } else { 
            let mut pd = PageTable::<PD>::mut_ref_from_ptr(self.allocate());
            let entry = PageTableEntry::<PDP>::new_table_ptr(pd.as_ptr());
            pdp.set_entry(pdp_idx, entry);
            pd
        };

        if pagesz == PageSize::Size2MiB { 
            let entry = PageTableEntry::new(
                base_paddr, 
                PTFlag::P | PTFlag::RW | PTFlag::PS
            );
            pd.set_entry(pd_idx, entry);
            return;
        } 
            
        unimplemented!("4KiB mapping unimplemented");
    }

    /// Map one or more pages of physical memory. 
    pub unsafe fn map_pages(
        &mut self, 
        pml4: &mut PageTable<PML4>,
        base_vaddr: u64,
        base_paddr: u64,
        pagesz: PageSize,
        cnt: usize,
    ) 
    {
        assert!(cnt <= 512);
        let pg_bytes = usize::from(pagesz);
        let size_bytes = pg_bytes as u64 * cnt as u64;

        let end_vaddr = base_vaddr + size_bytes;
        let end_paddr = base_paddr + size_bytes;
        let vaddr_range = base_vaddr..end_vaddr;
        let paddr_range = base_paddr..end_paddr;

        for (vaddr, paddr) in vaddr_range.step_by(pg_bytes)
            .zip(paddr_range.step_by(pg_bytes))
        {
            self.map_page(pml4, vaddr, paddr, pagesz);
        }

    }

    pub unsafe fn dump(pml4: &PageTable<PML4>) { 
        //let pml4_ptr = mrld::x86::CR3::read();
        //let mut pml4 = PageTable::<PML4>::ref_from_ptr(pml4_ptr as _);

        println!("PML4 Table: {:016x?}", pml4.as_ptr());
        'pml4_iter: for (pml4_idx, pml4e) in pml4.iter_entries() {
            if pml4e.invalid() {
                continue 'pml4_iter;
            }
            let vaddr = VirtAddr::canonical_from_index(
                pml4_idx, 
                PageTableIdx::from(0usize), 
                PageTableIdx::from(0usize), 
                PageTableIdx::from(0usize)
            );
            println!("  {:?} {:016x?}", pml4e, vaddr);
            if pml4e.terminal() { 
                continue 'pml4_iter;
            }

            let pdp_table = unsafe { 
                PageTable::<PDP>::ref_from_ptr(pml4e.address() as *const u8)
            };

            'pdp_iter: for (pdp_idx, pdpe) in pdp_table.iter_entries() {
                let vaddr = VirtAddr::canonical_from_index(
                    pml4_idx, 
                    pdp_idx,
                    PageTableIdx::from(0usize), 
                    PageTableIdx::from(0usize)
                );
                if pdpe.invalid() { 
                    continue 'pdp_iter;
                }
                println!("   {:?} {:016x?}", pdpe, vaddr);
                if pdpe.terminal() { 
                    continue 'pdp_iter;
                }


                let pd_table = unsafe { 
                    PageTable::<PD>::ref_from_ptr(pdpe.address() as *const u8)
                };
                'pd_iter: for (pd_idx, pde) in pd_table.iter_entries() {
                    let vaddr = VirtAddr::canonical_from_index(
                        pml4_idx, 
                        pdp_idx,
                        pd_idx,
                        PageTableIdx::from(0usize)
                    );
                    if pde.invalid() { 
                        continue 'pd_iter;
                    }
                    println!("   {:?} {:016x?}", pde, vaddr);
                }

            }
        }

    }
}



