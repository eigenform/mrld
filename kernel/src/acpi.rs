
use core::ptr::NonNull;
use alloc::vec;
use alloc::vec::Vec;
use acpi::{
    Handler, Handle, PhysicalMapping, PciAddress,
    aml,
    AcpiTables,
    AcpiTable,
    AmlTable,

    platform::{
        AcpiPlatform,
        ProcessorState,
    },
    aml::{
        Interpreter,
        object::*,
        namespace::*,
        AmlError
    },

    rsdp::Rsdp,
    sdt::{
        SdtHeader,
        Signature,
        fadt::Fadt,
        facs::Facs,
        madt::*,
    },
            
};
use crate::println;
use spin::Mutex;
use mrld::x86::io::Io;
use mrld::mmio::*;
use core::mem::MaybeUninit;
use core::pin::Pin;

/// Simple helper for dealing with ACPI. 
pub struct MrldAcpiManager { 
    platform: AcpiPlatform<MrldAcpiHandler>,
    //interpreter: Interpreter<MrldAcpiHandler>,
    maybe_guest: bool,
}
impl MrldAcpiManager { 
    pub unsafe fn new(rsdp_addr: u64) -> Self { 
        let tables: AcpiTables<MrldAcpiHandler>;
        let Ok(tables): Result<AcpiTables<MrldAcpiHandler>, _> = 
            AcpiTables::from_rsdp(MrldAcpiHandler, rsdp_addr as _) 
        else {
            panic!("Couldn't parse ACPI tables from RSDP?");
        };

        let Ok(platform) = AcpiPlatform::new(tables, MrldAcpiHandler) else {
            panic!("Couldn't create AcpiPlatform?");
        };

        // NOTE: For now, try to detect virtualized hardware with the OEM ID. 
        // In QEMU, it seems like most of the tables are 'BOCHS'.
        println!("[*] ACPI tables:");
        let mut maybe_guest = false;
        for (idx, hdr) in platform.tables.table_headers() {
            let oem_id = hdr.oem_id.as_ascii().unwrap();
            let creator_id = hdr.creator_id.as_ascii().unwrap();
            if oem_id.as_str().contains("BOCHS") {
                maybe_guest = true;
            }
            println!("  [{}] {} {}", hdr.signature, oem_id.as_str(), creator_id.as_str());
        }


        let dsdt = platform.tables.dsdt().unwrap();
        let dsdt_revision = dsdt.revision;
        let facs = platform.tables.find_table::<Fadt>()
            .and_then(|fadt| fadt.facs_address().ok())
            .map(|facs_address| unsafe { 
                platform.handler.map_physical_region::<Facs>(facs_address, 
                    core::mem::size_of::<Facs>()
                )
            });

        //let mut interpreter = Interpreter::new(
        //    MrldAcpiHandler,
        //    dsdt_revision,
        //    platform.registers.clone(),
        //    facs
        //);

        // NOTE: Maybe one day we'll be able to parse the DSDT ..
        //let dsdt_mapping = platform.handler.map_physical_region::<SdtHeader>(
        //        dsdt.phys_address, dsdt.length as _
        //);
        //let dsdt_stream_ptr = dsdt_mapping.virtual_start.as_ptr()
        //    .byte_add(core::mem::size_of::<SdtHeader>()) as *const u8;
        //let dsdt_stream = core::slice::from_raw_parts(
        //    dsdt_stream_ptr,
        //    dsdt.length as usize - core::mem::size_of::<SdtHeader>(),
        //);
        //interpreter.load_table(dsdt_stream).unwrap();

        Self { 
            platform,
            //interpreter,
            maybe_guest,
                
        }
    }

    pub fn maybe_guest(&self) -> bool { 
        self.maybe_guest
    }

    unsafe fn get_fadt(&self) -> PhysicalMapping<MrldAcpiHandler, Fadt> { 
        self.platform.tables.find_table::<Fadt>().unwrap()
    }
    unsafe fn get_madt(&self) -> PhysicalMapping<MrldAcpiHandler, Madt> { 
        self.platform.tables.find_table::<Madt>().unwrap()
    }

    pub unsafe fn init(&mut self) { 
        let fadt = self.get_fadt();

        let pm1a_ctr_port = fadt.pm1a_control_block().unwrap().address as u16;
        let smi_cmd_port = (fadt.smi_cmd_port & 0xffff) as u16;
        let acpi_enable = fadt.acpi_enable;
        let mut pm1a_ctr = mrld::x86::io::IoPort::new(pm1a_ctr_port);
        let mut smi = mrld::x86::io::IoPort::new(smi_cmd_port);

        // Switch from legacy to ACPI mode
        let x = pm1a_ctr.in16();
        if x & 0b1 != 0 { 
            println!("[*] FADT says we're in ACPI mode ...");
        } else { 
            println!("[*] FADT says we're in legacy mode ...");
            println!("[*] Waiting for SCI_EN ...");
            smi.out8(acpi_enable);
            loop { 
                if pm1a_ctr.in16() & 0b1 != 0 { 
                    break;
                }
            }
            println!("[*] Switched to ACPI mode");
        }

        let madt = self.get_madt();
        let lapic_addr = madt.get().local_apic_address;
        println!("{:08x?}", lapic_addr);


        for entry in madt.get().entries() {
            println!("{:x?}", entry);
        }

        if let Some(info) = &self.platform.processor_info { 
            println!("{:?}", info.boot_processor);
            for p in &info.application_processors { 
                if p.state == ProcessorState::Disabled { 
                    continue;
                }
                println!("{:?}", p);
            }
        }


    }
}


/// Power management
impl MrldAcpiManager {
    pub unsafe fn system_reset(&self) -> ! { 
        let fadt = self.get_fadt();
        mrld::x86::io::Io::out8(fadt.reset_reg.address as _, fadt.reset_value);
        core::arch::asm!("hlt");
        loop {}
    }

    /// Enter some sleep state (???)
    ///
    /// NOTE: This is only tested with S5 
    pub unsafe fn enter_s5_state(&self, slp_typ: u8) {
        assert!(slp_typ <= 5);
        assert!(slp_typ == 5, "only S5 is implemented/tested...");

        const SLP_EN: u16 = (1 << 13);
        const SLP_TYP_LSB: u16 = 10;

        let fadt = self.get_fadt();

        // Hacky reproduction of the _PTS method on my test machine ..
        CezanneACPI::do_pts(slp_typ);

        // PM1 event block
        //
        // NOTE: Kinda just took this from ACPI debug logs on Linux.
        // You should probably uhhh, figure out why this works. 
        let pm1a_evt = fadt.pm1a_event_block;
        let mut evt = mrld::x86::io::IoPort::new(pm1a_evt as _);
        let x = evt.in16();
        let y = 0b1100_0111_0011_0001;
        //println!("PM1_EVT: {:04x} -> {:04x}", x, y);
        evt.out16(y);

        // "Program the SLP_TYPx fields with the values contained in the 
        // selected sleeping object"
        let pm1a = fadt.pm1a_control_block();
        if let Ok(pm1a) = pm1a { 
            let mut p = mrld::x86::io::IoPort::new(pm1a.address as _);

            // Write SLP_TYPx first
            let val = p.in16();
            let next_val = (
                ((slp_typ as u16 & 0b111) << SLP_TYP_LSB) | val
            );
            p.out16(next_val);
            core::arch::asm!("mfence; lfence");

            // ... then another write to set SLP_EN
            let next_val = next_val | SLP_EN;
            p.out16(next_val);
            core::arch::asm!("mfence; lfence");
        }

        if slp_typ >= 4 {
            println!("[!] Failed to sleep? (why are you still here?)");
            core::arch::asm!("hlt");
        }

        // NOTE: My test machine doesn't have the PM1B block
        //let pm1b = fadt.pm1b_control_block();
        //if let Ok(Some(pm1b)) = pm1b { 
        //    let mut p = mrld::x86::io::IoPort::new(pm1b.address as _);
        //    let val = (
        //        (pm1b_slp_typ as u16 & 0b111) << SLP_TYP_LSB | SLP_EN
        //    );
        //    p.out16(val);
        //}


    }

}


/// Helper struct implementing [`acpi::Handler`].
#[derive(Clone, Copy)]
pub struct MrldAcpiHandler;
impl acpi::Handler for MrldAcpiHandler {

    // NOTE: This assumes that these regions are identity-mapped. 
    unsafe fn map_physical_region<T>(&self, 
        paddr: usize, size: usize,
    ) -> PhysicalMapping<Self, T> {
        PhysicalMapping { 
            physical_start: paddr,
            virtual_start: NonNull::new_unchecked(paddr as *mut _),
            region_length: size,
            mapped_length: size,
            handler: *self,
        }
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {}

    fn read_u8(&self, address: usize) -> u8 { 
        unsafe { (address as *const u8).read_volatile() }
    }
    fn read_u16(&self, address: usize) -> u16 { 
        unsafe { (address as *const u16).read_volatile() }
    }
    fn read_u32(&self, address: usize) -> u32 { 
        unsafe { (address as *const u32).read_volatile() }
    }
    fn read_u64(&self, address: usize) -> u64 { 
        unsafe { (address as *const u64).read_volatile() }
    }

    fn write_u8(&self, address: usize, value: u8) {
        unsafe { (address as *mut u8).write_volatile(value) }
    }

    fn write_u16(&self, address: usize, value: u16) {
        unsafe { (address as *mut u16).write_volatile(value) }
    }

    fn write_u32(&self, address: usize, value: u32) {
        unsafe { (address as *mut u32).write_volatile(value) }
    }

    fn write_u64(&self, address: usize, value: u64) {
        unsafe { (address as *mut u64).write_volatile(value) }
    }

    fn read_io_u8(&self, port: u16) -> u8 {
        unsafe { Io::in8(port) }
    }
    fn read_io_u16(&self, port: u16) -> u16 {
        unsafe { Io::in16(port) }
    }
    fn read_io_u32(&self, port: u16) -> u32 {
        unsafe { Io::in32(port) }
    }

    fn write_io_u8(&self, port: u16, val: u8) {
        unsafe { Io::out8(port, val) }
    }
    fn write_io_u16(&self, port: u16, val: u16) {
        unsafe { Io::out16(port, val) }
    }
    fn write_io_u32(&self, port: u16, val: u32) {
        unsafe { Io::out32(port, val) }
    }


    fn read_pci_u8(&self, address: PciAddress, off: u16) -> u8 {
        unimplemented!()
    }
    fn read_pci_u16(&self, address: PciAddress, off: u16) -> u16 {
        unimplemented!()
    }
    fn read_pci_u32(&self, address: PciAddress, off: u16) -> u32 {
        unimplemented!()
    }
    fn write_pci_u8(&self, address: PciAddress, off: u16, val: u8) {
        unimplemented!()
    }
    fn write_pci_u16(&self, address: PciAddress, off: u16, val: u16) {
        unimplemented!()
    }
    fn write_pci_u32(&self, address: PciAddress, off: u16, val: u32) {
        unimplemented!()
    }

    fn nanos_since_boot(&self) -> u64 { 
        0
    }
    fn stall(&self, us: u64) {
    }
    fn sleep(&self, ms: u64) {
    }
    fn create_mutex(&self) -> Handle {
        Handle(0)
    }
    fn acquire(&self, mutex: Handle, timeout: u16) -> Result<(), aml::AmlError> {
        Ok(())
    }
    fn release(&self, mutex: Handle) { 
    }

}


/// ACPI sleep-state helper for my test machine. 
///
/// NOTE: This is a hack for dealing with our lack of an AML parser for now.
/// Otherwise, we'd just perform whatever ops are prescribed in the DSDT. 
/// This all comes from (a) looking at my DSDT with 'iasl', and (b) looking
/// at some of the headers in 'coreboot' :< 
///
pub struct CezanneACPI;
impl CezanneACPI { 
    const ACPI_MMIO_BASE: u64 = 0xfed8_0000;

    const ACPI_SMI_BASE: u64  = Self::ACPI_MMIO_BASE + 0x200;
    const ACPI_SMI_88: u64  = Self::ACPI_SMI_BASE + 0x88;
    const ACPI_SMI_96: u64  = Self::ACPI_SMI_BASE + 0x96;
    const ACPI_SMI_B0: u64  = Self::ACPI_SMI_BASE + 0xb0;

    const ACPI_PMIO_BASE: u64 = Self::ACPI_MMIO_BASE + 0x300;
    const ACPI_PMIO_BB: u64 = Self::ACPI_PMIO_BASE + 0xbb;
    const ACPI_PMIO_BE: u64 = Self::ACPI_PMIO_BASE + 0xbe;
    const ACPI_PMIO_E4: u64 = Self::ACPI_PMIO_BASE + 0xe4;
    const ACPI_PMIO_F0: u64 = Self::ACPI_PMIO_BASE + 0xf0;

    const ACPI_PM1_CNT: u64 = Self::ACPI_MMIO_BASE + 0x804;
    const ACPI_PM1_STS: u64 = Self::ACPI_MMIO_BASE + 0x800;

    // Toggle bit 3 in PMIO register 0xf0. 
    // Not actually sure what this does? Some older 'coreboot' headers say 
    // this register is PMIOA 'UsbControl'? 
    const RSTU: MmioPtr<u16> = MmioPtr::new(Self::ACPI_PMIO_F0);
    pub unsafe fn set_rstu(val: bool) { 
        Self::RSTU.toggle(3, val)
    }

    // Set bits [0:1] in PMIO register 0xe4. 
    // No idea what this does (is BLNK supposed to mean "blink"?)
    const BLNK: MmioPtr<u8> = MmioPtr::new(Self::ACPI_PMIO_E4);
    pub unsafe fn set_blnk(val: u8) { 
        assert!(val <= 0b11);
        Self::BLNK.write_mask(0b11, val & 0b11);
    }

    // No idea what this is but it's definitely necessary
    const PWDE: MmioPtr<u8> = MmioPtr::new(Self::ACPI_PMIO_BB);
    pub unsafe fn set_pwde(val: bool) { 
        Self::PWDE.toggle(6, val)
    }

    // No idea what this is; presumably related to firing some SMI
    const CLPS: MmioPtr<u32> = MmioPtr::new(Self::ACPI_SMI_88);
    pub unsafe fn set_clps(val: bool) { 
        Self::CLPS.toggle(1, val);
    }

    // No idea what this is; presumably related to firing some SMI
    const SLPS: MmioPtr<u32> = MmioPtr::new(Self::ACPI_SMI_B0);
    pub unsafe fn set_slps(val: u8) { 
        Self::SLPS.write_mask(0b1100, ((val as u32)& 0b11) << 2)
    }


    // This *should* be what the _PTS method does on my machine. 
    pub unsafe fn do_pts(slp_type: u8) { 
        assert!(slp_type <= 5);

        // I think port80 is usually used for debug logging, 
        // so maybe this is unnecessary
        mrld::x86::io::Io::out16(0x80, slp_type as _);

        // method RPOP
        {
            // NOTE: I'm pretty sure this is an SMI request 
            if slp_type == 5 { 
                // SMIR = WVAL (0xe5)
                mrld::x86::io::Io::out8(0xb2, 0xe5);
            }

            // SSSK = Arg0
            // NOTE: Is this writing to SMM memory or something? 
            (0xbc254a98 as *mut u8).write_volatile(slp_type);
        }


        if slp_type == 3 { 
            Self::set_blnk(0b01);
        }
        if slp_type == 4 || slp_type == 5 { 
            Self::set_blnk(0b00);
        }
        if slp_type == 3 { 
            Self::set_rstu(false);
        }

        Self::set_clps(true);
        Self::set_slps(0b01);

        if slp_type == 3 { 
            Self::set_slps(0b01);
        }
        if slp_type == 4 { 
            Self::set_slps(0b01);
            Self::set_rstu(true);
        }
        if slp_type == 5 { 
            Self::set_pwde(true);
        }

    }

}



