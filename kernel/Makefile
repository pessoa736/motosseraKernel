
RAM = 128M
THREADS = 2
PROJECT=motossera-kernel

LIMINE_DIR = limine
ISO_DIR = iso
TARGET_DIR = target


ARCHI ?= x86_64
ifeq ($(ARCHI), aarch64)
	ARCHI_ext ?= $(ARCHI)-linux-android
else 
	ARCHI_ext ?= $(ARCHI)-unknown-none
endif




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
	cargo build --release --target $(ARCHI_ext)
	
	cp $(TARGET_DIR)/$(ARCHI_ext)/release/$(PROJECT) kernel.bin


iso: kernel.bin
	mkdir -p $(ISO_DIR)/boot
	mkdir -p $(ISO_DIR)/EFI/BOOT

	cp kernel.bin  $(ISO_DIR)/
	cp limine.conf $(ISO_DIR)/

	cp $(LIMINE_DIR)/bin/limine-bios.sys    $(ISO_DIR)/
	cp $(LIMINE_DIR)/bin/limine-bios-cd.bin $(ISO_DIR)/
	cp $(LIMINE_DIR)/bin/limine-uefi-cd.bin $(ISO_DIR)/
	cp $(LIMINE_DIR)/bin/BOOTX64.EFI        $(ISO_DIR)/EFI/BOOT/

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

	$(LIMINE_DIR)/bin/limine bios-install kernel.iso

run: kernel.iso limine
	qemu-system-$(ARCHI) \
		-cdrom kernel.iso \
		-machine q35 \
		-serial stdio \
		-cpu host,+topoext \
		-smp $(THREADS) \
		$(if $(filter x86_64,$(ARCHI)), -enable-kvm) \
		-m $(RAM) \
		-boot d \
		-no-reboot \
		-d int,cpu_reset \
		-D qemu.log  


clean:
	rm -rf iso kernel.iso kernel.bin *.o 
	cargo clean 

megaclean: clean
	rm -rf $(LIMINE_DIR)/ ftp $(TARGET_DIR)/

comrun: clean run