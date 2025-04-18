
use acpi::*;
use core::ptr::NonNull;
use uefi::println;

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

pub fn do_acpi_init() {
    use uefi::table::cfg::ACPI2_GUID;
    use acpi::platform::ProcessorState;

    let rsdp_addr = uefi::system::with_config_table(|tbl| {
        let rdsp = tbl.iter().find(|e| e.guid == ACPI2_GUID).unwrap();
        rdsp.address as usize
    });

    let acpi_tables = unsafe { 
        AcpiTables::from_rsdp(MrldAcpiHandler, rsdp_addr).unwrap()
    };

    let platform_info = acpi_tables.platform_info().unwrap();
    let proc_info = platform_info.processor_info.unwrap();
    println!("BSP proc_id={} lapic_id={}", 
        proc_info.boot_processor.processor_uid,
        proc_info.boot_processor.local_apic_id,
    );
    for ap in proc_info.application_processors.iter() {
        if ap.state == ProcessorState::Disabled {
            continue;
        }
        println!("AP  proc_id={} lapic_id={}", 
            ap.processor_uid, 
            ap.local_apic_id,
        );
    }
}

pub fn do_console_init() {
    use uefi::proto::console::text::OutputMode;

    uefi::system::with_stdout(|stdout| { 
        let tgt_mode = stdout.modes().find(|m| m.index() == 1).unwrap();
        stdout.set_mode(tgt_mode).unwrap();
        stdout.clear().unwrap();
    });
}


