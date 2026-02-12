
use core::ptr::NonNull;
use acpi::{
    Handler, Handle, PhysicalMapping, PciAddress,
    aml,
    AcpiTables,
    AcpiTable,

    rsdp::Rsdp,
    sdt::{
        Signature,
        fadt::Fadt,
        madt::*,
    },
            
};
use crate::println;
use spin::Mutex;
use mrld::x86::io::Io;
use core::mem::MaybeUninit;
use core::sync::atomic::*;

pub static ACPI: AcpiManager = {
    AcpiManager::new()
};

pub struct AcpiManager { 
    /// Physical address of the RSDP
    rsdp: AtomicU64,
    /// Identity-mapped pointer to the FADT
    fadt: AtomicPtr<Fadt>,
    /// Identity-mapped pointer to the MADT
    madt: AtomicPtr<Madt>,
}
impl AcpiManager { 
    pub const fn new() -> Self { 
        Self { 
            rsdp: AtomicU64::new(0),
            fadt: AtomicPtr::new(core::ptr::null_mut()),
            madt: AtomicPtr::new(core::ptr::null_mut()),
        }
    }

    /// Initialize this structure, using the physical address of the RSDP
    /// to find pointers to other relevant ACPI tables
    pub unsafe fn init(&self, rsdp_addr: u64) {
        let tables: AcpiTables<MrldAcpiHandler>;

        let Ok(tables): Result<AcpiTables<MrldAcpiHandler>, _> = 
            AcpiTables::from_rsdp(MrldAcpiHandler, rsdp_addr as _) 
        else {
            panic!("Couldn't parse ACPI tables from RSDP?");
        };

        for (phys, e) in tables.table_headers() { 
            println!("{:016x}: {}", phys, e.signature.as_str());
        }
 
        let Some(x) = tables.find_table::<Fadt>() else {
            panic!("Couldn't find FADT?");
        };
        let mut fadt_ptr: NonNull<Fadt> = x.virtual_start;

        let Some(x) = tables.find_table::<Madt>() else {
            panic!("Couldn't find MADT?");
        };
        let mut madt_ptr: NonNull<Madt> = x.virtual_start;

        let x = core::pin::Pin::static_ref(madt_ptr.as_ref());
        for entry in x.entries() { 
            println!("{:x?}", x);
        }

        self.rsdp.store(rsdp_addr, Ordering::SeqCst);
        self.fadt.store(fadt_ptr.as_mut(), Ordering::SeqCst);
        self.madt.store(madt_ptr.as_mut(), Ordering::SeqCst);

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
