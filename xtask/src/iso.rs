

use crate::definitions::*;
use crate::utils::{cp, run};
use std::{fs, process::Command};

fn structure(target: &str, efi: &str, is_aarch64: bool)
{
    let _ = fs::remove_dir_all(ISO_DIR);
    fs::create_dir_all(format!("{ISO_DIR}/EFI/BOOT")).unwrap();
    
    cp(
        &format!("target/{target}/release/{PROJECT}"), 
        &format!("{ISO_DIR}/kernel.bin")
    );
    cp(
        "limine.conf", 
        &format!("{ISO_DIR}/limine.conf")
    );
    cp(
        &format!("{LIMINE_DIR}/bin/limine-uefi-cd.bin"), 
        &format!("{ISO_DIR}/limine-uefi-cd.bin")
    );
    cp(
        &format!("{LIMINE_DIR}/bin/{efi}"), 
        &format!("{ISO_DIR}/EFI/BOOT/{efi}")
    );
    
    if !is_aarch64
    {
        cp(
            &format!("{LIMINE_DIR}/bin/limine-bios.sys"), 
            &format!("{ISO_DIR}/limine-bios.sys")
        );
        cp(
            &format!("{LIMINE_DIR}/bin/limine-bios-cd.bin"), 
            &format!("{ISO_DIR}/limine-bios-cd.bin")
        );
    }
}


fn compile(is_aarch64: bool)
{
    let mut x = Command::new("xorriso");
    x.args([ "-as", "mkisofs"]);

    if !is_aarch64
    {x.args([
        "-b", "limine-bios-cd.bin", 
        "-no-emul-boot", 
        "-boot-load-size", "4", 
        "-boot-info-table"
    ]);}

    x.args([
        "--efi-boot", "limine-uefi-cd.bin", 
        "-efi-boot-part", 
        "--efi-boot-image",
        "--protective-msdos-label", &ISO_DIR, 
        "-o", &format!("{KERNEL_DIR}/kernel.iso")
    ]);

    run(&mut x);

    if !is_aarch64
    {
        run(
            Command::new(
                format!("{LIMINE_DIR}/bin/limine")
            )
            .args([
                "bios-install", &format!("{KERNEL_DIR}/kernel.iso")
            ])
        );
    }
    
}

pub fn iso_create(target: &str, efi: &str, is_aarch64: bool) {
    structure(target, efi, is_aarch64);
    compile(is_aarch64);
}