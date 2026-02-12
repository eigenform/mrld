//! Support crate for the mrld bootloader and kernel.

#![no_std]
#![allow(unsafe_op_in_unsafe_fn)]
#![feature(abi_x86_interrupt)]
#![feature(adt_const_params)]

pub mod paging;
pub mod physmem;
pub mod x86; 

use core::ops::Range;
use core::ptr::NonNull;
use core::mem::MaybeUninit;


/// Function pointer reflecting the mrld kernel entrypoint. 
///
/// NOTE: This *must* match the signature of the actual kernel entrypoint. 
/// Is there any way for us to declare this *in one place*? 
pub type MrldKernelEntrypoint = extern "sysv64" fn(*const MrldBootArgs) -> !;

// NOTE: These symbols are defined in the kernel linkerscript.
unsafe extern "C" { 
    #[link_name = "_kernel_phys_base"]
    pub static KERNEL_PHYS_BASE: u64;

    #[link_name = "_kernel_virt_base"]
    pub static KERNEL_VIRT_BASE: u64;
}

/// Arguments passed from the UEFI bootloader to the kernel. 
#[repr(C)]
pub struct MrldBootArgs { 
    /// Physical address of the RSDP table
    pub rsdp_addr: u64,

    /// Physical address of the UEFI memory map
    pub uefi_map: u64,
    /// Reported size of the UEFI memory map
    pub uefi_map_size: usize,
    /// Reported descriptor size in the UEFI memory map
    pub uefi_map_desc_size: usize,
}
impl MrldBootArgs { 
    pub fn as_ptr(&self) -> *const Self { 
        self as *const Self
    }
    pub fn new_empty() -> Self { 
        Self { 
            rsdp_addr: 0,
            uefi_map: 0,
            uefi_map_size: 0,
            uefi_map_desc_size: 0,
        }
    }
}

