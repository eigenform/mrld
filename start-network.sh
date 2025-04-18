#!/bin/bash 

if [[ $EUID != 0 ]]; then
	echo "Must be root"
	exit -1;
fi

# Interface connected to the target
TGT_IF="enp12s0f3u1"

ip link set dev ${TGT_IF} up
ip addr add 10.200.200.1/24 dev ${TGT_IF}
ip link set dev ${TGT_IF} up

# Start DHCP server
dhcpd -cf ./dhcpd.conf ${TGT_IF} &
sleep 1
DHCP_PID=$(cat /var/run/dhcpd.pid)
echo "[*} DHCP server PID: ${DHCP_PID}"

# SIGINT handler
shutdown_services()
{
	echo "[*] Got SIGINT - shutting down DHCP server"
	kill ${DHCP_PID}
	exit 123
}
trap shutdown_services SIGINT

# Start TFTP server
tftpd -i 10.200.200.1 -r -d ./pxe/

shutdown_services()

