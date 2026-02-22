
use mrld::x86::{ 
    msr::*,
    apic::*,
    cpuid,
};
use mrld::mmio::*;
use crate::println;


pub struct ApicMmio(pub MmioPtr<u32>);
impl ApicMmio { 
    pub fn new() -> Self { 
        let bar = ApicBar::from(Msr::rdmsr(Msr::APIC_BAR));
        Self(MmioPtr::new(bar.base_address()))
    }
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

    pub unsafe fn interrupt_command_lo(&self) -> MmioPtr<u32> { 
        self.0.offset_bytes(0x300)
    }
    pub unsafe fn interrupt_command_hi(&self) -> MmioPtr<u32> { 
        self.0.offset_bytes(0x310)
    }



}

/// Helper for interactions with the local APIC. 
pub struct Lapic; 
impl Lapic { 
    pub unsafe fn init() {
        let bar = ApicBar::from(Msr::rdmsr(Msr::APIC_BAR));
        println!("[*] APIC BAR at {:016x}", bar.base_address());
        if bar.bsc() { 
            println!("[*] This is the bootstrap core");
        }
        if bar.ae() { 
            println!("[*] APIC enabled");
        } else { 
            panic!("expected APIC to be enabled");
        }
        if bar.extd() {
            println!("[*] X2APIC mode enabled");
        }
    }

    pub unsafe fn mmio() -> ApicMmio { 
        let bar = ApicBar::from(Msr::rdmsr(Msr::APIC_BAR));
        let mut mmio = ApicMmio(
            MmioPtr::new((bar.aba() << 12))
        );
        mmio
    }

    // The 8-bit vector 
    //
    // When receiving SIPI, the entrypoint is a 20-bit physical address 
    // where the high 8 bits are the vector. 
    // 0x3_c000
    pub unsafe fn send_sipi(dest: usize) {
        const TRAMPOLINE_PHYS: usize = 0x0000_8000;

        let init_cmd = mrld::x86::apic::IntrCommand::new()
            .with_l(true)
            .with_tgm(true)
            .with_des(dest as _)
            .with_mt(MessageType::Init as _);

        let startup_cmd = mrld::x86::apic::IntrCommand::new()
            .with_vec((TRAMPOLINE_PHYS >> 12) as _)
            .with_mt(MessageType::Startup as _)
            .with_des(dest as _)
            .with_l(true);

        let mut mmio = ApicMmio::new();
        let startup_cmd = u64::from_le_bytes(startup_cmd.into_bytes());
        let init_cmd = u64::from_le_bytes(init_cmd.into_bytes());

        mmio.interrupt_command_hi().write(
            ((init_cmd & 0xffff_ffff_0000_0000) >> 32) as _
        );
        mmio.interrupt_command_lo().write(
            ((init_cmd & 0x0000_0000_ffff_ffff)) as _
        );
        mmio.interrupt_command_hi().write(
            ((init_cmd & 0xffff_ffff_0000_0000) >> 32) as _
        );
        mmio.interrupt_command_lo().write(
            ((init_cmd & 0x0000_0000_ffff_ffff)) as _
        );



        mmio.interrupt_command_hi().write(
            ((startup_cmd & 0xffff_ffff_0000_0000) >> 32) as _
        );
        mmio.interrupt_command_lo().write(
            ((startup_cmd & 0x0000_0000_ffff_ffff)) as _
        );



    }




}
