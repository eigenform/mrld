
use uefi::{println, print};
use uefi::boot::ScopedProtocol;
use uefi::proto::{
    network::{
        IpAddress,
        pxe::{ Packet, BaseCode, DhcpV4Packet, UdpOpFlags, },
    },
};
use core::ffi::CStr;

// NOTE: Fixed PXE server address for now
const SERVER_IP: IpAddress = IpAddress::new_v4([10, 200, 200, 1]);
const KERNEL_FILENAME: &'static uefi::CStr8 = {
    uefi::cstr8!("mrld-kernel")
};


pub fn download_kernel() -> uefi::Result<()> {
    use uefi::boot::{
        get_handle_for_protocol,
        open_protocol_exclusive,
    };

    let handle = get_handle_for_protocol::<BaseCode>()?;
    let mut base_code = open_protocol_exclusive::<BaseCode>(handle)?;

    // NOTE: Currently, we only expect users to be PXE booting 'mrld-boot'.
    // Just return an error if PXE is not already started. 
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
    if !base_code.mode().dhcp_ack_received { 
        return Err(uefi::Error::new(uefi::Status::NOT_READY, ()));
    }

    let ack: &DhcpV4Packet = base_code.mode().dhcp_ack.as_ref();

    println!("[*] ciaddr: {:?}", ack.bootp_ci_addr);
    println!("[*] yiaddr: {:?}", ack.bootp_yi_addr);
    println!("[*] siaddr: {:?}", ack.bootp_si_addr);

    let server_ip = IpAddress::new_v4(ack.bootp_si_addr);
    println!("[*] Server IP: {:?}", server_ip);

    // Get the address of the DHCP server, which we *assume* is also acting 
    // as the TFTP server hosting the kernel image. 
    if ack.bootp_si_addr == [0, 0, 0, 0] { 
        println!("[!] DHCPv4 ACK had no server address (SIADDR)?");
        return Err(uefi::Error::new(uefi::Status::NOT_FOUND, ()));
    }

    let boot_file = CStr::from_bytes_until_nul(&ack.bootp_boot_file).unwrap();
    println!("[*] Boot file: {}", boot_file.to_str().unwrap());

    //let mut buf = alloc::boxed::Box::new([0u8; 0x1000]);

    //// Download the kernel from the PXE server
    //let res = base_code.tftp_read_file(
    //    &server_ip, 
    //    &KERNEL_FILENAME, 
    //    Some(buf.as_mut_slice())
    //);

    //match res { 
    //    Ok(_) => {},
    //    Err(e) => {
    //        match e.status() {
    //            uefi::Status::TIMEOUT => {
    //                println!("[!] PXE timed out while requesting kernel?");
    //            },
    //            _ => {
    //                println!("[!] PXE error {}", e);
    //            },
    //        }
    //        return Err(e);
    //    }
    //}

    //drop(buf);

    Ok(())
}

fn send_udp_msg(base_code: &mut ScopedProtocol<BaseCode>, payload: &[u8]) {
    let this_ip = base_code.mode().station_ip;
    let header = [payload.len() as u8];
    let mut write_src_port = 0;
    base_code.udp_write(
        UdpOpFlags::ANY_SRC_PORT,
        &SERVER_IP,
        666,
        None,
        Some(&this_ip),
        Some(&mut write_src_port),
        Some(&header),
        payload,
    ).expect("couldn't write udp packet");
}


