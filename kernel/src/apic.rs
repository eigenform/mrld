
use mrld::x86::{ 
    msr::*,
    apic::*,
    cpuid,
};
use mrld::mmio::*;
use crate::println;


pub struct ApicMmio(pub MmioPtr<u32>);
impl ApicMmio { 
    pub unsafe fn apic_id(&self) -> MmioPtr<u32> { 
        self.0.offset_bytes(0x20)
    }
    pub unsafe fn apic_version(&self) -> MmioPtr<u32> { 
        self.0.offset_bytes(0x30)
    }
    pub unsafe fn task_priority(&self) -> MmioPtr<u32> { 
        self.0.offset_bytes(0x80)
    }
    pub unsafe fn arbitration_priority(&self) -> MmioPtr<u32> { 
        self.0.offset_bytes(0x90)
    }
    pub unsafe fn processor_priority(&self) -> MmioPtr<u32> { 
        self.0.offset_bytes(0xa0)
    }
    pub unsafe fn end_of_interrupt(&self) -> MmioPtr<u32> { 
        self.0.offset_bytes(0xb0)
    }
    pub unsafe fn remote_read(&self) -> MmioPtr<u32> { 
        self.0.offset_bytes(0xc0)
    }
    pub unsafe fn logical_destination(&self) -> MmioPtr<u32> { 
        self.0.offset_bytes(0xd0)
    }
    pub unsafe fn destination_format(&self) -> MmioPtr<u32> { 
        self.0.offset_bytes(0xe0)
    }
    pub unsafe fn spurious_interrupt_vector(&self) -> MmioPtr<u32> { 
        self.0.offset_bytes(0xf0)
    }

}

/// Helper for interactions with the local APIC. 
pub struct Lapic; 
impl Lapic { 
    pub unsafe fn init() {
        let bar = ApicBar::from(Msr::rdmsr(Msr::APIC_BAR));
        if bar.bsc() { 
            println!("[*] This is the bootstrap core");
        }
        if bar.ae() { 
            println!("[*] APIC enabled");
        }
        if bar.extd() {
            println!("[*] X2APIC mode enabled");
        }
    }
}
