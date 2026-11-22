


#[cfg(target_arch = "x86_64")] 
mod arch {
    use core::{arch::asm};
    use crate::debuglog::dprint;

    #[inline(always)]
    pub unsafe fn outb(port: u16, data: u8) {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") data,
            options(nomem, nostack, preserves_flags)
        );
    }

    #[inline(always)]
    pub unsafe fn inb(port: u16) -> u8
    {
        let ret: u8;
        asm!(
            "in al, dx",
            in("dx") port,
            out("al") ret,
            options(nomem, nostack, preserves_flags)
        );

        return ret;
    }  

    use crate::definitions::io::COM1;

    pub unsafe fn io_read() -> u8
    {
        while (inb(COM1+5) & 0x01) == 0 {};
        inb(COM1)
    }

    pub unsafe fn io_write(data: u8)
    {
        while (inb(COM1 + 5) & 0x20) == 0 {};
        outb(COM1, data);
    }

     pub fn init() {
        unsafe 
        {
            dprint(Some("IO.1"), "interrupições dessativadas");
            outb(COM1 + 1, 0x00);
            dprint(Some("IO.2"), "ativou o DLAB");
            outb(COM1 + 3, 0x80);
            
            outb(COM1 + 0, 0x03);
            outb(COM1 + 1, 0x00);
            dprint(Some("IO.3"), "Definiu a velocidade");
            dprint(Some("IO.4"), "desativou o DLAB");

            outb(COM1 + 3, 0x03);
            dprint(Some("IO.5"), "Configurou os buffers FIFO");
            outb(COM1 + 2, 0x07);
            dprint(Some("IO.6"), "Preparar os sinais de controle");
            outb(COM1 + 4, 0x0b);
        };
    }
}


#[cfg(target_arch = "aarch64")] 
mod arch {

    use alloc::vec::Vec;
    use crate::definitions::io::{UARCH0_DR, UARCH0_FR, RXFE, TXFF};
    
    pub unsafe fn io_read() -> u8
    {
        while (UARCH0_FR.read_volatile() & RXFE) != 0 {}
        (UARCH0_DR.read_volatile()  & 0xFF) as u8 
    }

    pub unsafe fn io_write(data: u8)
    {
        while (UARCH0_FR.read_volatile() & TXFF) != 0 {}
        UARCH0_DR.write_volatile(data as u32);
    }

    pub unsafe fn init() {}

}



pub struct Scancode {
    pub CODE: u8,
    pub RELEASE_MASK: u8,
    pub PRESS: bool,
}

impl Scancode {
    pub const fn new(code: u8, release_mask: u8) -> Self {
        Self { CODE: code, RELEASE_MASK: release_mask, PRESS: false }
    }
}

pub struct InputOutputSistem {
    pub KEYBOARD_STATUS: u16,
    pub KEYBOARD_DATA: u16,
    pub SHIFT_RIGHT: Scancode,
    pub SHIFT_LEFT: Scancode,
}


impl InputOutputSistem {
    pub const fn new() -> Self {
        Self {
            KEYBOARD_STATUS: 0x64,
            KEYBOARD_DATA: 0x60,
            SHIFT_LEFT: Scancode::new(0x2A, 0x80),
            SHIFT_RIGHT: Scancode::new(0x36, 0x80),
        }
    }

    pub fn init(&self){
        use crate::debuglog::dprint;

        unsafe {arch::init()};
        dprint(Some("IO.1"), "serial inicializada");
    }
   

    pub fn read_char(&mut self) -> char {
        unsafe {arch::io_read() as char}
    }

    #[inline(always)]
    pub fn write_char(&self, c: u8) 
    {
        unsafe { arch::io_write(c); }
    }

    #[inline(always)]
    pub fn write_str(&self, s: &str) 
    {
        for byte in s.bytes() 
        { self.write_char(byte);}
    }
}