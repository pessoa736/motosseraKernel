use core::arch::asm;
use core::panic::PanicInfo;
use core::fmt::{ Error, Result, Write};


#[cfg(target_arch = "aarch64")] const UARCH0_BASE: *mut u8 = 0x0900_0000 as *mut u8; 
#[cfg(target_arch = "x86_64")]  const COM1: u16 = 0x3f8;

struct SerialWriter;

impl Write for SerialWriter {


    #[inline(always)]
    fn write_str(&mut self, s: &str) -> Result {
        
        #[cfg(target_arch = "aarch64")]
        {
            unsafe 
            {
                for byte in s.bytes() { UARCH0_BASE.write_volatile(byte) };
            }
            return Ok(());
        }

        #[cfg(target_arch = "x86_64")]
        { 
            unsafe {
                for byte in s.bytes() {
                    asm!(
                        "out dx, al", 
                        in("dx") COM1, 
                        in("al") byte, 
                        options(nomem, preserves_flags, nostack)
                    );
                }
            }
            return Ok(());
        }

        Err(Error)
    }
}

#[inline(always)]
pub fn dwrite(s: &str) {
    let mut writer = SerialWriter;
    let _ = writer.write_str(s);
}

pub fn dprint(title: Option<&str>, s: &str) {
    if let Some(t) = title {
        dwrite("[ ");
        dwrite(t);
        dwrite(" ]");
    }
    dwrite(s);
    dwrite("\n");
}

#[panic_handler]
pub fn panic(_info: &PanicInfo) -> ! {
    let mut writer = SerialWriter;
    
    #[cfg(target_arch = "x86_64")]{
        let _ = write!(writer, "\n[ panic ] ");
        let _ = write!(writer, "{}", _info);
    }
    loop {}
}
