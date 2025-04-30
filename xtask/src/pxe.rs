
use clap::Parser;
use anyhow::{anyhow, Result};

#[derive(Parser)]
pub struct PxeArgs { 
    /// Name of the host network interface to listen on
    #[arg(long, default_value = "enp12s0f3u1")]
    interface: String,

    /// Host IP address to listen on
    #[arg(long, default_value = "10.200.200.1")]
    server_ip: String,

    /// Requested IP address of the target machine
    #[arg(long, default_value = "10.200.200.200")]
    target_ip: String,

    /// MAC address of the target machine
    target_mac: String,

    /// DHCP network 
    #[arg(long, default_value = "10.200.200.0")]
    net: String,

    /// DHCP network mask 
    #[arg(long, default_value = "255.255.255.0")]
    netmask: String,
}

// NOTE: Hardcoded defaults specific to *my* setup. 
const DEFAULT_PXE_INTERFACE: &'static str = "enp12s0f3u1";
const DEFAULT_PXE_NET: &'static str       = "10.200.200.0";
const DEFAULT_PXE_NETMASK: &'static str   = "255.255.255.0";
const DEFAULT_PXE_SERVER_IP: &'static str = "10.200.200.1";
const DEFAULT_PXE_TARGET_IP: &'static str = "10.200.200.200";
const DEFAULT_BOOTFILE: &'static str      = "mrld-boot.efi";

pub fn bringup_interface(args: &PxeArgs) -> Result<()> { 
    Ok(())
}

pub fn write_dhcpd_conf(args: &PxeArgs) -> Result<()> { 
    let content = format!("\n\
    option subnet-mask {};\n\
    subnet {} netmask {} {{\n\
        \tnext-server {};\n\
    }}\n\

    host mrld-target {{\n\
        \thardware ethernet {};\n\
        \tfixed-address {};\n\
        \tfilename \"{}\";\n\
    }}\n\
    ",
        args.netmask,
        args.net,
        args.netmask,
        args.server_ip,
        args.target_mac,
        args.target_ip,
        DEFAULT_BOOTFILE,
    );

    println!("{}", content);

    Ok(())
}

