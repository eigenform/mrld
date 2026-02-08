//! 'mrld' PXE kernel loader

use uefi::{
    println, CStr8, cstr8,
    boot::{
        AllocateType,
        MemoryType,
        get_handle_for_protocol,
        open_protocol_exclusive,
    },
    proto::{
        network::{
            //IpAddress,
            pxe::{ BaseCode, DhcpV4Packet },
        },
    },
};
use core::ptr::NonNull;
use core::net::{ IpAddr, Ipv4Addr };

/// Helper for allocating/downloading/loading an 'mrld' kernel ELF. 
pub struct KernelImage { 
    /// Pointer to the kernel ELF
    pub ptr: NonNull<u8>,
    /// Size of the kernel ELF (in bytes)
    pub size: usize,
}
impl KernelImage {
    /// Fixed remote filename on the PXE server
    pub const REMOTE_FILENAME: &'static CStr8 = cstr8!("mrld-kernel");

    pub fn as_mut_slice(&mut self) -> &mut [u8] { 
        unsafe { 
            NonNull::slice_from_raw_parts(self.ptr, self.size).as_mut()
        }
    }
}

impl KernelImage {
    /// Download the kernel image with the UEFI PXE protocol.
    ///
    /// NOTE: Currently, we *expect* the bootloader itself has been loaded 
    /// over PXE, and we return an error if PXE is not already started. 
    pub fn download() -> uefi::Result<Self> { 
        let handle = get_handle_for_protocol::<BaseCode>()?;
        let mut base_code = open_protocol_exclusive::<BaseCode>(handle)?;

        match base_code.start(false) { 
            Ok(_) => {
                println!("[!] PXE services were not already started?");
                return Err(uefi::Error::new(uefi::Status::NOT_READY, ()));
            },
            Err(e) => {
                match e.status() { 
                    uefi::Status::ALREADY_STARTED => {},
                    _ => return Err(e),
                }
            },
        }
        if !base_code.mode().dhcp_ack_received() { 
            return Err(uefi::Error::new(uefi::Status::NOT_READY, ()));
        }

        // Get the address of the DHCP server, which we *assume* is also 
        // acting as the TFTP server hosting our kernel image. 
        let ack: &DhcpV4Packet = base_code.mode().dhcp_ack().as_ref();
        if ack.bootp_si_addr == [0, 0, 0, 0] { 
            println!("[!] DHCPv4 ACK had no server address (SIADDR)?");
            return Err(uefi::Error::new(uefi::Status::NOT_FOUND, ()));
        }

        let server_ip = IpAddr::V4(Ipv4Addr::from_octets(ack.bootp_si_addr));
        let kernel_sz = base_code.tftp_get_file_size(
            &server_ip, 
            &Self::REMOTE_FILENAME
        )?;
        let ptr: NonNull<u8> = uefi::boot::allocate_pages(
            AllocateType::AnyPages,
            MemoryType::LOADER_DATA, 
            (kernel_sz as usize / uefi::boot::PAGE_SIZE) + 1,
        )?;


        let mut res = KernelImage { 
            size: kernel_sz as usize,
            ptr,
        };
        base_code.tftp_read_file(
            &server_ip, 
            &Self::REMOTE_FILENAME,
            Some(res.as_mut_slice())
        )?;

        match base_code.stop() {
            Ok(_) => {},
            Err(e) => {
                match e.status() { 
                    _ => return Err(e),
                }
            },
        }

        Ok(res)
    }

    /// Load the kernel into physical memory and return the entrypoint.
    ///
    /// Conventions
    /// ===========
    ///
    /// - The *load address* of a segment is a physical address (which we can
    ///   expect to be identity mapped when running in the bootloader here)
    ///
    /// - Non-loadable segments are ignored
    ///
    /// - The entrypoint has the type [`mrld::MrldKernelEntrypoint`]
    ///
    pub unsafe fn load(&self) -> uefi::Result<mrld::MrldKernelEntrypoint> {
        use elf::{
            endian::LittleEndian,
            abi::PT_LOAD,
            ElfBytes,
        };
        println!("[*] Loading kernel ...");
        let elf = {
            let slice = unsafe { 
                NonNull::slice_from_raw_parts(self.ptr, self.size).as_ref()
            };
            ElfBytes::<LittleEndian>::minimal_parse(slice).unwrap()
        };
        println!("  Kernel entrypoint: {:016x}", elf.ehdr.e_entry);
        let entrypt = elf.ehdr.e_entry;


        for seg in elf.segments().unwrap() {
            println!("  Kernel segment: p={:016x} v={:016x}",
                seg.p_paddr, seg.p_vaddr
            );
            if seg.p_type != PT_LOAD { continue; }
            if seg.p_paddr == 0 { continue; }
            let tgt = seg.p_paddr as *mut u8;

            if seg.p_memsz > seg.p_filesz { 
                tgt.write_bytes(0, seg.p_memsz as _);
            }

            let src = self.ptr.offset(seg.p_offset as isize);
            tgt.copy_from(src.as_ptr(), seg.p_filesz as usize);
        }
        unsafe { 
            Ok(core::mem::transmute(entrypt))
        }
    }
}

