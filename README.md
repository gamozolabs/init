# Summary

High-performance `/sbin/init` program for Linux

This is designed to do literally nothing but accept binaries over the
network and run them as a child of init.

If you pipe a file to `<server ip>:1234` it will run it and pipe the
stderr and stdout back to you

If you connect to `<server ip>:1235` init will send SIGKILL to all
processes on the system but itself. This is a measure to allow resetting
the system if a binary was uploaded that had issues. This port neither
sends or recieves anything, it simply kills upon getting a TCP connection.

For a simple headless Linux machine running a basic kernel, you'll want
flags like this:

```
console=ttyS1,115200 rw root=/dev/sda ip=dhcp
```

This enables a console on ttyS1 (in my case that's COM2, the
Serial-over-LAN port for IPMI), `rw` specifies that the root mount should
be read-writable (required since we drop a file), root specifies the root
filesystem device (in our case we used an unpartioned flash drive with
vfat), and `ip=dhcp` is the coolest part, this allows the kernel to get
a DHCP lease on any active NICs. This is mandatory because we use
networking in `init` without any configuration of the network.

TL;DR: Build a kernel that can read from your drive and do networking, put this
       on it at `/sbin/init`, boot the kernel and it will run this program.

