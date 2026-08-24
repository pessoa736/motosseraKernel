use core::{arch::asm};
use alloc::vec::Vec;

use crate::{debuglog::dprint};



/// locais da memoria responsaveis pela entrada e saida do usuario
#[cfg(target_arch = "aarch64")] pub const UARCH0_DR: *mut u32 = 0x0900_0000 as *mut u32; 
#[cfg(target_arch = "aarch64")] pub const UARCH0_FR: *const u32 = 0x0900_0018 as *const u32; 
#[cfg(target_arch = "aarch64")] pub const TXFF: u32 = 1<<5;
#[cfg(target_arch = "aarch64")] pub const RXFE: u32 = 1<<4;

#[cfg(target_arch = "x86_64")]  pub const COM1: u16 = 0x3f8 ;




#[inline(always)]
pub unsafe fn outb_x86_64(port: u16, data: u8) {
    #[cfg(target_arch = "x86_64")]
    {asm!(
        "out dx, al",
        in("dx") port,
        in("al") data,
        options(nomem, nostack, preserves_flags)
    );}
}

#[inline(always)]
pub unsafe fn inb_x86_64(port: u16) -> u8
{
    #[cfg(target_arch = "x86_64")]
    {
        let ret: u8;
        asm!(
            "in al, dx",
            in("dx") port,
            out("al") ret,
            options(nomem, nostack, preserves_flags)
        );
        return ret ;
    }

    #[cfg(not(target_arch = "x86_64"))] 0u8
}


pub unsafe fn io_read() -> u8
{
    #[cfg(target_arch = "x86_64")] 
    {
        while (inb_x86_64(COM1+5) & 0x01) == 0 {};
        inb_x86_64(COM1)
    }

    #[cfg(target_arch = "aarch64")]
    { 
        while (UARCH0_FR.read_volatile() & RXFE) != 0 {}
        (UARCH0_DR.read_volatile()  & 0xFF)as u8 
    }
}

pub unsafe fn io_write(data: u8)
{
    #[cfg(target_arch = "x86_64")] {
        while (inb_x86_64(COM1 + 5) & 0x20) == 0 {}
        outb_x86_64(COM1, data)
    }; 
    

     #[cfg(target_arch = "aarch64")]
     {
        while (UARCH0_FR.read_volatile() & TXFF) != 0 {}
        UARCH0_DR.write_volatile(data as u32);
     }
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

    
    

    pub fn init(&self) {
        #[cfg(target_arch = "x86_64")]
        unsafe 
        {
            dprint(Some("IO.1"), "interrupições dessativadas");
            outb_x86_64(COM1 + 1, 0x00);
            dprint(Some("IO.2"), "ativou o DLAB");
            outb_x86_64(COM1 + 3, 0x80);
            
            outb_x86_64(COM1 + 0, 0x03);
            outb_x86_64(COM1 + 1, 0x00);
            dprint(Some("IO.3"), "Definiu a velocidade");
            dprint(Some("IO.4"), "desativou o DLAB");

            outb_x86_64(COM1 + 3, 0x03);
            dprint(Some("IO.5"), "Configurou os buffers FIFO");
            outb_x86_64(COM1 + 2, 0x07);
            dprint(Some("IO.6"), "Preparar os sinais de controle");
            outb_x86_64(COM1 + 4, 0x0b);
        };
    }

    pub fn read_char(&mut self) -> char {
        unsafe {io_read() as char}
    }

    #[inline(always)]
    pub fn write_char(&self, c: u8) 
    {
        unsafe { io_write(c); }
    }

    #[inline(always)]
    pub fn write_str(&self, s: &str) 
    {
        for byte in s.bytes() 
        { self.write_char(byte);}
    }
}