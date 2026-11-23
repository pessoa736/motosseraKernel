use std::env::{self, args};
use std::process::{exit, Command};

mod limine;
mod definitions;
mod commands;
mod utils;
mod iso;


use crate::definitions::*;
use crate::utils::{run};
use crate::limine::limine_create;
use crate::commands::{cargo};
use crate::iso::iso_create;



fn build_kernel(target: &str)
{
    cargo(
        &format!(
            "build --release --target {target} \
             -Zbuild-std=core,alloc,compiler_builtins \
             -Zbuild-std-features=compiler-builtins-mem"
        ),
        "."
    );
}


fn main() 
{
    let mut args = args().skip(1);

    let task = args.next().unwrap_or_else(|| "run".into());
    let arch = args.next().unwrap_or_else(|| env::consts::ARCH.into());


    let aarch64 = arch == "aarch64";
    let target = if aarch64 
        { "aarch64-unknown-none-softfloat".to_string() }
        else 
        { format!("{arch}-unknown-none") };


    let efi = if aarch64 { "BOOTAA64.EFI" } else { "BOOTX64.EFI" };
    let iso_dir = format!("{KERNEL_DIR}/iso");
    let iso = format!("{KERNEL_DIR}/kernel.iso");

    // 1. Limine (só se ainda não foi compilado para esta arquitetura)
    limine_create(efi, &arch);

    // 2. Kernel
    build_kernel(&target);

    // 3. ISO
    iso_create(&target, efi, aarch64);

    // 4. QEMU
    match task.as_str() {
        "run" => 
        {
            let mut q = Command::new(format!("qemu-system-{arch}"));

            q.args([
                "-drive", &format!("file={iso},media=cdrom,if=none,id=bootcd"),
                "-device", "virtio-scsi-pci", 
                "-device", "scsi-cd,drive=bootcd",
                "-display", "gtk", 
                "-serial", "stdio", 
                "-smp", "2", 
                "-m", "256M",
                "-boot", "d", 
                "-no-reboot", 
                "-d", "int,cpu_reset", 
                "-D", "qemu.log"
            ]);
            
            if aarch64 
            {q.args([
                "-machine", "virt", 
                "-device", "ramfb", 
                "-bios", "/usr/share/edk2/aarch64/QEMU_EFI.fd"
            ]);} 
            else 
            {q.args([
                "-machine", "q35"
            ]);}

            if env::consts::ARCH == arch 
            {
                q.arg("-enable-kvm");
                q.args(["-cpu", if arch == "x86_64" { "host,+topoext" } else { "host" }]);
            } 
            else 
            {
                q.args(["-cpu", if aarch64 { "cortex-a72" } else { "max" }]);
            }

            run(&mut q);
        }
        
        _ => 
        {return;}
    }

    
}