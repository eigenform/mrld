
use core::alloc::*;
use core::sync::atomic::*;
use mrld::x86::*;
use mrld::paging::*;
use crate::println;

pub struct MrldAllocator { 
    next: AtomicPtr<u8>
}
unsafe impl GlobalAlloc for MrldAllocator { 
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 { 
        panic!("alloc unimpl, next={:016x?}", self.next);
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        panic!("alloc unimpl, next={:016x?}", self.next);
    }
}

unsafe extern "C" { 
    #[link_name = "_kernel_heap_base"]
    pub static KERNEL_HEAP_BASE: u64;

}

#[global_allocator]
pub static ALC: MrldAllocator = unsafe {
    MrldAllocator { 
        next: AtomicPtr::new(&KERNEL_HEAP_BASE as *const u64 as _)
    }
};

pub struct MrldPageTable;
impl MrldPageTable { 
    pub unsafe fn dump() { 
        let pml4_ptr = mrld::x86::CR3::read();
        let mut pml4 = PageTable::<PML4>::ref_from_ptr(pml4_ptr as _);

        println!("PML4 Table: {:016x?}", pml4.as_ptr());
        for (pml4_idx, pml4e) in pml4.iter_entries() {
            if pml4e.invalid() {
                continue;
            }

            let vaddr = VirtAddr::canonical_from_index(
                pml4_idx, 
                PageTableIdx::from(0usize), 
                PageTableIdx::from(0usize), 
                PageTableIdx::from(0usize)
            );
            println!("  {:?} {:016x?}", pml4e, vaddr);
            let pdp_table = unsafe { 
                PageTable::<PDP>::ref_from_ptr(pml4e.address() as *const u8)
            };
            for (pdp_idx, pdpe) in pdp_table.iter_entries() {
                let vaddr = VirtAddr::canonical_from_index(
                    pml4_idx, 
                    pdp_idx,
                    PageTableIdx::from(0usize), 
                    PageTableIdx::from(0usize)
                );
                if pdpe.invalid() { 
                    continue;
                }
                println!("   {:?} {:016x?}", pdpe, vaddr);
            }
        }

    }
}
