use core::fmt::{self, Write};
use core::panic::PanicInfo;

struct SerialWriter;

#[cfg(target_arch = "x86_64")]
mod arch {
    use core::arch::asm;
    use crate::definitions::COM1;

    #[inline(always)]
    unsafe fn outb(port: u16, data: u8) {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") data,
            options(nomem, nostack, preserves_flags)
        );
    }

    #[inline(always)]
    unsafe fn inb(port: u16) -> u8 {
        let ret: u8;
        asm!(
            "in al, dx",
            in("dx") port,
            out("al") ret,
            options(nomem, nostack, preserves_flags)
        );
        ret
    }

    pub fn write_byte(b: u8) {
        unsafe {
            while inb(COM1 + 5) & 0x20 == 0 {}
            outb(COM1, b);
        }
    }

    pub fn halt() -> ! {
        loop {
            unsafe { asm!("hlt", options(nomem, nostack, preserves_flags)); }
        }
    }
}


#[cfg(target_arch = "aarch64")]
mod arch {
    use core::sync::atomic::{AtomicUsize, Ordering};
    use core::arch::asm;
    
    use crate::definitions::*;


    pub static  HHDM: AtomicUsize = AtomicUsize::new(0xffff_8000_0000_0000);

    #[inline(always)]
    pub fn uart_dr() -> *mut u32 {
        (HHDM.load(Ordering::Relaxed) + UART0_PHYS) as *mut u32
    }

    #[inline(always)]
    pub fn uart_fr() -> *mut u32 {
        (HHDM.load(Ordering::Relaxed) + UART0_PHYS + 0x18) as *mut u32
    }

    pub fn write_byte(b: u8) {
        unsafe {
            while (uart_dr().read_volatile() & TXFF) != 0 {}
            UARCH0_DR.write_volatile(b as u32);
        }
    }

    pub fn halt() -> ! {
        loop {
            unsafe { asm!("wfe", options(nomem, nostack, preserves_flags)); }
        }
    }
}

impl Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            if b == b'\n' {
                arch::write_byte(b'\r');
            }
            arch::write_byte(b);
        }
        Ok(())
    }
}

#[inline(always)]
pub fn dwrite(s: &str) {
    let _ = SerialWriter.write_str(s);
}

pub fn dprint(title: Option<&str>, s: &str) {
    if let Some(t) = title {
        dwrite("[ ");
        dwrite(t);
        dwrite(" ] ");
    }
    dwrite(s);
    dwrite("\n");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut w = SerialWriter;
    let _ = write!(w, "\n[ panic ] {}\n", info);
    arch::halt()
}