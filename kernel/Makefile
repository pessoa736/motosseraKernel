
RAM = 128M
THREADS = 2
PROJECT=motossera-kernel

ARCHI ?= x86_64

LIMINE_DIR = limine


.PHONY: all limine clean megaclean comrun


all: kernel.iso


limine:
	@if [ ! -d $(LIMINE_DIR) ]; then \
		git clone https://github.com/limine-bootloader/limine.git; \
	fi
	cd $(LIMINE_DIR) && ./bootstrap
	cd $(LIMINE_DIR) && ./configure  --enable-bios --enable-bios-cd --enable-uefi-$(ARCHI) --enable-uefi-cd
	cd $(LIMINE_DIR) && $(MAKE)  

kernel.bin: limine
	cargo build --release --target $(ARCHI)-unknown-none
	
	cp target/$(ARCHI)-unknown-none/release/$(PROJECT) kernel.bin

iso: kernel.bin
	mkdir -p iso/boot
	mkdir -p iso/EFI/BOOT

	cp kernel.bin iso/
	cp limine.conf iso/
	cp limine/bin/limine-bios.sys iso/
	cp limine/bin/limine-bios-cd.bin iso/
	cp limine/bin/limine-uefi-cd.bin iso/
	cp limine/bin/BOOTX64.EFI iso/EFI/BOOT/

kernel.iso: iso
	xorriso -as mkisofs \
		-b limine-bios-cd.bin \
		-no-emul-boot \
		-boot-load-size 4 \
		-boot-info-table \
		--efi-boot limine-uefi-cd.bin \
		-efi-boot-part \
		--efi-boot-image \
		--protective-msdos-label \
		iso -o kernel.iso
	./limine/bin/limine bios-install kernel.iso

run: kernel.iso limine
	qemu-system-x86_64 \
		-cdrom kernel.iso \
		-machine q35 \
		-serial stdio \
		-cpu host,+topoext \
		-smp $(THREADS) \
		-enable-kvm \
		-m $(RAM) \
		-boot d \
		-no-reboot \
		-d int,cpu_reset \
		-D qemu.log 

clean:
	rm -rf iso kernel.iso kernel.bin *.o 
	cargo clean 

megaclean: clean
	rm -rf $(LIMINE_DIR) ftp target/

comrun: clean run