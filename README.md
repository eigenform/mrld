# mrld ("emerald")

```
boot/            - mrld UEFI boot stub
pxe/             - [Untracked] directory for serving files with PXE/TFTP
README.md        - You are here
start-network.sh - Script for serving bootloader with PXE
```

## Configuration

You'll also have to write your own `dhcpd.conf` and place it in this directory. 
For instance, mine looks like this: 

```
# dhcpd.conf
option subnet-mask 255.255.255.0;
option routers 10.200.200.1;

subnet 10.200.200.0 netmask 255.255.255.0 {
	range 10.200.200.40 10.200.200.49;
}

host target {
	hardware ethernet <target mac address>
	fixed-address 10.200.200.200;
	filename "boot.efi";
}
```

Images are served from the [`pxe/`](./pxe) directory. 
I have `boot.efi` symlinked, ie. 

```
$ ln -s target/x86_64-unknown-uefi/release/boot.efi pxe/boot.efi
```


## Usage

The intended workflow here is: 

- The target machine is connected on a point-to-point ethernet link
- Start a DHCP server for negotiating PXE boot on the target
- Start a TFTP server for serving the actual UEFI binary 
- Start the target machine

I'm using the ISC DHCP server and [altugbakan/rs-tftpd](https://github.com/altugbakan/rs-tftpd) 
(which you can easily install with `cargo install tftpd`).

For now, this is all automated with [`start-network.sh`](./start-network.sh). 
You will probably have to tweak this to reflect your own setup.


