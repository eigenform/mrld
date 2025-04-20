//! System bring-up during UEFI boot services. 

use acpi::*;
use core::ptr::NonNull;
use core::ffi::c_void;
use uefi::println;
use spin::Once;
use crate::x86;

#[derive(Clone)]
pub struct MrldAcpiHandler;
impl acpi::AcpiHandler for MrldAcpiHandler {
    fn unmap_physical_region<T>(_region: &acpi::PhysicalMapping<Self, T>) {}
    unsafe fn map_physical_region<T>(&self, addr: usize, size: usize)
        -> acpi::PhysicalMapping<Self, T>
    {
        unsafe { 
            acpi::PhysicalMapping::new(
                addr, 
                NonNull::new(addr as *mut T).unwrap(), 
                size, 
                size, 
                self.clone()
            )
        }
    }
}

static PLATFORM_INFO: Once<PlatformInfo<'static, alloc::alloc::Global>> = Once::new();

pub fn get_platform_info() -> &'static PlatformInfo<'static,  alloc::alloc::Global> {
    PLATFORM_INFO.get().unwrap()
}


pub fn do_acpi_init() {
    use uefi::table::cfg::ACPI2_GUID;

    let rsdp_addr = uefi::system::with_config_table(|tbl| {
        let rdsp = tbl.iter().find(|e| e.guid == ACPI2_GUID).unwrap();
        rdsp.address as usize
    });

    if let Ok(acpi_tables) = unsafe { 
        AcpiTables::from_rsdp(MrldAcpiHandler, rsdp_addr)
    } {
        let platform_info = PlatformInfo::new(&acpi_tables).unwrap();
        PLATFORM_INFO.call_once(|| platform_info);
    }
}

pub fn do_console_init() {
    use uefi::proto::console::text::OutputMode;

    uefi::system::with_stdout(|stdout| { 
        let tgt_mode = stdout.modes().find(|m| m.index() == 0).unwrap();
        stdout.set_mode(tgt_mode).unwrap();
        stdout.clear().unwrap();
    });
}


