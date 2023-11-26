all: disk
	qemu-system-x86_64 -accel kvm -m 4G -kernel linux-6.6.2/arch/x86/boot/bzImage -append "console=ttyS0,115200 rw root=/dev/sda1 ip=dhcp" -drive file=fat:rw:boot,format=raw -nic user,model=virtio,hostfwd=tcp::1234-:1234,hostfwd=tcp::1235-:1235 -nographic -no-reboot

disk:
	cargo build --target x86_64-unknown-linux-musl
	cp target/x86_64-unknown-linux-musl/debug/init boot/sbin

