

use core::ptr::NonNull;
use acpi::{
    Handler, Handle, PhysicalMapping, PciAddress,
    aml,
};


pub struct AcpiManager;
impl AcpiManager { 
    pub fn init(rsdp_addr: u64) {
    }
}


#[derive(Clone, Copy)]
pub struct MrldAcpiHandler;
impl acpi::Handler for MrldAcpiHandler {
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
        unimplemented!()
    }
    fn read_io_u16(&self, port: u16) -> u16 {
        unimplemented!()
    }
    fn read_io_u32(&self, port: u16) -> u32 {
        unimplemented!()
    }

    fn write_io_u8(&self, port: u16, val: u8) {
        unimplemented!()
    }
    fn write_io_u16(&self, port: u16, val: u16) {
        unimplemented!()
    }
    fn write_io_u32(&self, port: u16, val: u32) {
        unimplemented!()
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
