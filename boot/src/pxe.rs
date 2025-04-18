
use uefi::println;
use uefi::boot::ScopedProtocol;
use uefi::proto::{
    network::{
        IpAddress,
        pxe::{ Packet, BaseCode, DhcpV4Packet, UdpOpFlags, },
    },
};

// NOTE: Fixed PXE server address for now
const SERVER_IP: IpAddress = IpAddress::new_v4([10, 200, 200, 1]);

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


