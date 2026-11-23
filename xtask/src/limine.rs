
use std::{fs, path::Path};
use crate::commands::{git, sh};
use crate::definitions::LIMINE_DIR;

pub fn limine_create(efi: &str, arch: &str) 
{
    let stamp = format!("{LIMINE_DIR}/.built-for");
    let built_for = fs::read_to_string(&stamp).unwrap_or_default();
    let bin_ok = Path::new(&format!("{LIMINE_DIR}/bin/{efi}")).exists();

    if built_for.trim() == arch && bin_ok 
    {return;}

    if !Path::new(LIMINE_DIR).exists() 
    {
        git(
            &format!(
                "clone https://github.com/limine-bootloader/limine.git {LIMINE_DIR}"
            ),
            "."
        );
    }

    let bios = if arch == "aarch64" { "" } else { "--enable-bios --enable-bios-cd" };
    
    sh("[ -f configure ] || ./bootstrap", LIMINE_DIR);
    sh("make clean || true", LIMINE_DIR);
    sh(
        &format!("./configure {bios} --enable-uefi-{arch} --enable-uefi-cd"), 
        LIMINE_DIR
    );
    sh("make", LIMINE_DIR);
    
    fs::write(&stamp, arch).unwrap();
}