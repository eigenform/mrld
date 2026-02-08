
use clap::Parser;
use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::Read;
use std::process::*;
use std::path::Path;
use std::env;

//#[derive(Parser)]
//pub struct PxeArgs { 
//    /// Name of the host network interface to listen on
//    #[arg(long, default_value = "enp12s0f3u1")]
//    interface: String,
//
//    /// Host IP address to listen on
//    #[arg(long, default_value = "10.200.200.1")]
//    server_ip: String,
//
//    /// Requested IP address of the target machine
//    #[arg(long, default_value = "10.200.200.200")]
//    target_ip: String,
//
//    /// MAC address of the target machine
//    target_mac: String,
//
//    /// DHCP network 
//    #[arg(long, default_value = "10.200.200.0")]
//    net: String,
//
//    /// DHCP network mask 
//    #[arg(long, default_value = "255.255.255.0")]
//    netmask: String,
//}

// NOTE: Hardcoded defaults specific to *my* setup. 
const DEFAULT_PXE_INTERFACE: &'static str = "enp12s0f3u1";
const DEFAULT_PXE_NET: &'static str       = "10.200.200.0";
const DEFAULT_PXE_NETMASK: &'static str   = "255.255.255.0";
const DEFAULT_PXE_SERVER_IP: &'static str = "10.200.200.1";
const DEFAULT_PXE_TARGET_IP: &'static str = "10.200.200.200";
const DEFAULT_BOOTFILE: &'static str      = "mrld-boot.efi";

pub fn start(root: &Path) -> Result<()> { 
    let pxe_dir = root.join("/pxe/");
    let dhcp_conf = root.join("/dhcpd.conf");

    bringup_interface()?;

    Ok(())
}

pub fn bringup_interface() -> Result<()> { 
    let mut f = File::open(
        format!("/sys/class/net/{}/operstate", DEFAULT_PXE_INTERFACE)
    )?;
    let mut state = String::new();
    f.read_to_string(&mut state)?;
    match state.as_str() { 
        "up" => {},
        "down" => {

            let addr = format!("{}/24", DEFAULT_PXE_SERVER_IP);
            let cmd = Command::new("sudo").args([
                "ip", "addr", "add", &addr, "dev", DEFAULT_PXE_INTERFACE,
            ]).spawn()?.wait()?;
            if let Some(code) = cmd.code() { 
                if code != 0 { 
                    return Err(anyhow!("Failed to assign address?"));
                }
            }

            let cmd = Command::new("sudo").args([
                "ip", "link", "set", "addr", DEFAULT_PXE_INTERFACE, "up"
            ]).spawn()?.wait()?;
            if let Some(code) = cmd.code() { 
                if code != 0 { 
                    return Err(anyhow!("Failed to bring up {}", DEFAULT_PXE_INTERFACE));
                }
            }


        },
        _ => unreachable!("{:02x?}", state.as_bytes()),
    }

    Ok(())
}

//pub fn write_dhcpd_conf(args: &PxeArgs) -> Result<()> { 
//    let content = format!("\n\
//    option subnet-mask {};\n\
//    subnet {} netmask {} {{\n\
//        \tnext-server {};\n\
//    }}\n\
//
//    host mrld-target {{\n\
//        \thardware ethernet {};\n\
//        \tfixed-address {};\n\
//        \tfilename \"{}\";\n\
//    }}\n\
//    ",
//        args.netmask,
//        args.net,
//        args.netmask,
//        args.server_ip,
//        args.target_mac,
//        args.target_ip,
//        DEFAULT_BOOTFILE,
//    );
//
//    println!("{}", content);
//
//    Ok(())
//}

