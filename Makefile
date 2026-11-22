SHELL := /bin/bash

RAM = 256M
THREADS = 2
PROJECT=motossera-kernel

KERNEL_DIR = kernel
LIMINE_DIR = $(KERNEL_DIR)/limine
ISO_DIR = $(KERNEL_DIR)/iso
TARGET_DIR = target


HOST_ARCHI = $(shell uname -m)

ARCHI ?= $(HOST_ARCHI)
ifeq ($(ARCHI), aarch64)
	ARCHI_ext ?= aarch64-unknown-none-softfloat
	EFI_BIN := BOOTAA64.EFI
	LIMINE_BIOS_FLAGS :=
	XORRISO_BIOS :=
else
	ARCHI_ext ?= $(ARCHI)-unknown-none
	EFI_BIN := BOOTX64.EFI
	LIMINE_BIOS_FLAGS := --enable-bios --enable-bios-cd
	XORRISO_BIOS := -b limine-bios-cd.bin -no-emul-boot -boot-load-size 4 -boot-info-table
endif

ifeq ($(HOST_ARCHI),$(ARCHI))
    QEMU_ACCEL := -enable-kvm
    ifeq ($(ARCHI),x86_64)
        QEMU_CPU := -cpu host,+topoext
    else
        QEMU_CPU := -cpu host
    endif
else
    QEMU_ACCEL :=
    ifeq ($(ARCHI),aarch64)
        QEMU_CPU := -cpu cortex-a72
    else
        QEMU_CPU := -cpu max
    endif
endif


.PHONY: all limine iso kernel.bin kernel.iso run clean megaclean comrun


all: kernel.iso


limine:
	@if [ ! -d $(LIMINE_DIR) ]; then \
		git clone https://github.com/limine-bootloader/limine.git  $(LIMINE_DIR); \
	fi
	cd $(LIMINE_DIR) && ./bootstrap
	cd $(LIMINE_DIR) && ./configure $(LIMINE_BIOS_FLAGS) --enable-uefi-$(ARCHI) --enable-uefi-cd
	cd $(LIMINE_DIR) && $(MAKE)


kernel.bin: limine
	cargo build --release --target $(ARCHI_ext)
	cp $(TARGET_DIR)/$(ARCHI_ext)/release/$(PROJECT) $(KERNEL_DIR)/kernel.bin


iso: kernel.bin
	mkdir -p $(ISO_DIR)/{boot,EFI/BOOT}

	cp $(KERNEL_DIR)/kernel.bin $(ISO_DIR)/
	cp limine.conf $(ISO_DIR)/

ifeq ($(ARCHI),x86_64)
	cp $(LIMINE_DIR)/bin/limine-bios.sys $(ISO_DIR)/
	cp $(LIMINE_DIR)/bin/limine-bios-cd.bin $(ISO_DIR)/
endif
	cp $(LIMINE_DIR)/bin/limine-uefi-cd.bin $(ISO_DIR)/
	cp $(LIMINE_DIR)/bin/$(EFI_BIN) $(ISO_DIR)/EFI/BOOT/

kernel.iso: iso
	xorriso -as mkisofs \
		$(XORRISO_BIOS) \
		--efi-boot limine-uefi-cd.bin \
		-efi-boot-part \
		--efi-boot-image \
		--protective-msdos-label \
		$(ISO_DIR) -o $(KERNEL_DIR)/kernel.iso
ifeq ($(ARCHI),x86_64)
	$(LIMINE_DIR)/bin/limine bios-install $(KERNEL_DIR)/kernel.iso
endif

run: kernel.iso
	qemu-system-$(ARCHI) \
		-drive file=./$(KERNEL_DIR)/kernel.iso,media=cdrom,if=none,id=bootcd \
		-device virtio-scsi-pci \
		-device scsi-cd,drive=bootcd \
		-machine $(or $(if $(filter x86_64,$(ARCHI)), q35), virt) \
		$(if $(filter aarch64,$(ARCHI)), -device ramfb -bios /usr/share/edk2/aarch64/QEMU_EFI.fd) \
		-display gtk \
		-serial stdio \
		$(QEMU_CPU) $(QEMU_ACCEL) \
		-smp $(THREADS) \
		-m $(RAM) \
		-boot d \
		-no-reboot \
		-d int,cpu_reset \
		-D qemu.log

clean:
	rm -rf $(ISO_DIR)/ $(KERNEL_DIR)/kernel.iso $(KERNEL_DIR)/kernel.bin *.o
	cargo clean

megaclean: clean
	rm -rf $(LIMINE_DIR)/ $(KERNEL_DIR)/ftp $(TARGET_DIR)/

comrun: clean run