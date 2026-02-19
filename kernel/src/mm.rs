//! Memory management. 


use core::alloc::*;
use core::mem::MaybeUninit;
use core::sync::atomic::*;
use core::ops::Range;

use mrld::x86::*;
use mrld::MrldBootArgs;
use mrld::paging::*;
use mrld::physmem::*;

use crate::println;
use spin::Mutex;
use uefi_raw::table::boot::{
    MemoryType, MemoryAttribute, MemoryDescriptor
};

/// The base of kernel image mapping
pub const KERNEL_TEXT_BASE: u64 = 0xffff_ffff_8000_0000;
/// The base of the kernel heap mapping
pub const KERNEL_HEAP_BASE: u64 = 0xffff_ffd0_0000_0000;
/// The size of the kernel heap mapping
pub const KERNEL_HEAP_SIZE: usize = PageSize::Size1GiB.as_usize();

/// The global allocator.
#[global_allocator]
pub static HEAP: MrldHeap = {
    MrldHeap { 
        next: AtomicPtr::new(0 as _),
        end: AtomicPtr::new(0 as _),
    }
};

/// Trivial bump allocator. 
pub struct MrldHeap { 
    next: AtomicPtr<u8>,
    end:  AtomicPtr<u8>,
}
impl MrldHeap { 

    /// Initialize this structure. 
    ///
    /// NOTE: The size and location of backing memory is fixed here.
    ///
    /// NOTE: Subsequent use of these pointers assumes that the kernel heap
    /// mapping is actually configured in page tables.
    pub unsafe fn init(&self) {
        self.next.store(KERNEL_HEAP_BASE as _, Ordering::SeqCst);
        self.end.store(
            (KERNEL_HEAP_BASE + KERNEL_HEAP_SIZE as u64) as *mut u8, 
            Ordering::SeqCst
        );
    }
}

unsafe impl GlobalAlloc for MrldHeap { 
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 { 
        let this_ptr = self.next.load(Ordering::SeqCst);
        let max_ptr = self.end.load(Ordering::SeqCst);

        let this_algn = this_ptr.align_offset(layout.align());
        let next_ptr = this_ptr.offset(
            this_algn as isize + layout.size() as isize
        );

        if next_ptr >= max_ptr { 
            panic!("uhhhhh");
        }

        //println!("[*] alloc {:x?} this={:016x?} next={:016x?}", 
        //    layout, this_ptr, next_ptr
        //);

        self.next.store(next_ptr, Ordering::SeqCst);
        this_ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // it's cheaper to just do nothing ¯\_(ツ)_/¯
    }
}

