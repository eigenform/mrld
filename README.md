# mrld

`mrld` (like "emerald") is a UEFI bootloader and tiny "kernel" for bare-metal 
x86 experiments.

```
boot/            - mrld UEFI bootloader
kernel/          - mrld kernel
mrld/            - mrld support library
pxe/             - [Untracked] directory for serving files with PXE/TFTP
xtask/           - mrld build tooling
README.md        - You are here
start-network.sh - Script for serving bootloader with PXE
```

## Building this Project

The [`xtask`](./xtask) crate is used to automate building and testing.
Run `cargo xtask help` to see more information about available commands. 

- `cargo xtask build` invokes `cargo build` for the bootloader and kernel
- `cargo xtask qemu` attempts to PXE boot on QEMU 

### Build Notes

There are three parts to this: 

- A UEFI boot-stub/bootloader, using the `x86_64-unknown-uefi` target
- A kernel using the [`mrld-kernel.json`](./mrld-kernel.json) target
- A support library (which should be compatible with both targets)


## Using this Project

For now, `mrld` is only intended to be booted over the network with PXE. 
A DHCP/TFTP server is expected to serve the bootloader and kernel to the 
target machine. 

### PXE Configuration

This project expects that the [`pxe/`](./pxe) directory is used when serving 
files to the target machine. `cargo xtask build` automatically creates 
symlinks in this directory to the bootloader UEFI executable and kernel ELF.

When booting on real hardware, the current workflow assumes that you're using the 
typical ISC DHCP server (`dhcpd`) and [altugbakan/rs-tftpd](https://github.com/altugbakan/rs-tftpd)
(which can be easily installed with `cargo install tftpd`).
I'm currently using [`start-network.sh`](./start-network.sh) to control 
these. You will probably have to change this to reflect your setup. 

This also assumes that the user has created a [`dhcpd.conf`](./dhcpd.conf) in 
the project root. Here's what mine looks like, assuming the PXE server is 
running on `10.200.200.1`: 

```
# dhcpd.conf
option subnet-mask 255.255.255.0;
option routers 10.200.200.1;

subnet 10.200.200.0 netmask 255.255.255.0 {
	range 10.200.200.40 10.200.200.49;
    next-server 10.200.200.1; 
}

host target {
	hardware ethernet <target mac address>
	fixed-address 10.200.200.200;
	filename "mrld-boot.efi";
}
```

At some point, this will all be replaced with an `xtask` command. 

### Using QEMU

UEFI support in QEMU relies on OVMF. The paths to the appropriate OVMF images 
are hardcoded here, and you may have to change them for your own setup. 
See [`xtask`](./xtask/src/main.rs) for more information. 

QEMU has a built-in DHCP/TFTP server that transparently handles PXE, and 
no other configuration should be required for using PXE. 

The current process is:

1. Run `cargo xtask build`
2. Run `cargo xtask qemu`

### Using Real Hardware

The current process is:

1. The target machine is connected over ethernet to the host machine
2. Run `cargo xtask build`
3. Run `start-network.sh` (in the future, `cargo xtask pxe`)
4. Start the target machine

