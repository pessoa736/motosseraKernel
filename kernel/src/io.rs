use core::{arch::asm, cell::UnsafeCell, mem::MaybeUninit};
use spin::mutex::Mutex;
use crate::{debuglog::dprint, utils::ToSafe}; // Assuming this is defined in your crate

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
    pub COM1: u16,
    pub KEYBOARD_STATUS: u16,
    pub KEYBOARD_DATA: u16,
    pub KEYBOARD_MAP_DEFALT: ToSafe<[[u8; 128]; 2]>,
    pub SHIFT_RIGHT: Scancode,
    pub SHIFT_LEFT: Scancode,
}

/// Scancode Set 1 → ASCII (US QWERTY, sem shift)
static KEYBOARD_MAP_NORMAL: [u8; 128] = {
    let mut map = [0u8; 128];
    // Linha 0
    map[0x02] = b'1'; map[0x03] = b'2'; map[0x04] = b'3';
    map[0x05] = b'4'; map[0x06] = b'5'; map[0x07] = b'6';
    map[0x08] = b'7'; map[0x09] = b'8'; map[0x0A] = b'9';
    map[0x0B] = b'0'; map[0x0C] = b'-'; map[0x0D] = b'=';
    map[0x0E] = b'\x08'; // Backspace
    map[0x0F] = b'\t';   // Tab
    // Linha 1
    map[0x10] = b'q'; map[0x11] = b'w'; map[0x12] = b'e';
    map[0x13] = b'r'; map[0x14] = b't'; map[0x15] = b'y';
    map[0x16] = b'u'; map[0x17] = b'i'; map[0x18] = b'o';
    map[0x19] = b'p'; map[0x1A] = b'['; map[0x1B] = b']';
    map[0x1C] = b'\n';   // Enter
    // Linha 2
    map[0x1E] = b'a'; map[0x1F] = b's'; map[0x20] = b'd';
    map[0x21] = b'f'; map[0x22] = b'g'; map[0x23] = b'h';
    map[0x24] = b'j'; map[0x25] = b'k'; map[0x26] = b'l';
    map[0x27] = b';'; map[0x28] = b'\'';
    // Linha 3
    map[0x2C] = b'z'; map[0x2D] = b'x'; map[0x2E] = b'c';
    map[0x2F] = b'v'; map[0x30] = b'b'; map[0x31] = b'n';
    map[0x32] = b'm'; map[0x33] = b','; map[0x34] = b'.';
    map[0x35] = b'/';
    // Espaço e outros
    map[0x39] = b' ';
    map
};

/// Scancode Set 1 → ASCII (US QWERTY, com shift)
static KEYBOARD_MAP_SHIFT: [u8; 128] = {
    let mut map = [0u8; 128];
    map[0x02] = b'!'; map[0x03] = b'@'; map[0x04] = b'#';
    map[0x05] = b'$'; map[0x06] = b'%'; map[0x07] = b'^';
    map[0x08] = b'&'; map[0x09] = b'*'; map[0x0A] = b'(';
    map[0x0B] = b')'; map[0x0C] = b'_'; map[0x0D] = b'+';
    map[0x0E] = b'\x08'; // Backspace
    map[0x0F] = b'\t';   // Tab
    map[0x10] = b'Q'; map[0x11] = b'W'; map[0x12] = b'E';
    map[0x13] = b'R'; map[0x14] = b'T'; map[0x15] = b'Y';
    map[0x16] = b'U'; map[0x17] = b'I'; map[0x18] = b'O';
    map[0x19] = b'P'; map[0x1A] = b'{'; map[0x1B] = b'}';
    map[0x1C] = b'\n';   // Enter
    map[0x1E] = b'A'; map[0x1F] = b'S'; map[0x20] = b'D';
    map[0x21] = b'F'; map[0x22] = b'G'; map[0x23] = b'H';
    map[0x24] = b'J'; map[0x25] = b'K'; map[0x26] = b'L';
    map[0x27] = b':'; map[0x28] = b'"';
    map[0x2C] = b'Z'; map[0x2D] = b'X'; map[0x2E] = b'C';
    map[0x2F] = b'V'; map[0x30] = b'B'; map[0x31] = b'N';
    map[0x32] = b'M'; map[0x33] = b'<'; map[0x34] = b'>';
    map[0x35] = b'?';
    map[0x39] = b' ';
    map
};

impl InputOutputSistem {
    pub const fn new() -> Self {
        let map_ = [KEYBOARD_MAP_NORMAL, KEYBOARD_MAP_SHIFT];
        Self {
            COM1: 0x3f8,
            KEYBOARD_STATUS: 0x64,
            KEYBOARD_DATA: 0x60,
            KEYBOARD_MAP_DEFALT: Mutex::new(MaybeUninit::new(UnsafeCell::new(map_))),
            SHIFT_LEFT: Scancode::new(0x2A, 0x80),
            SHIFT_RIGHT: Scancode::new(0x36, 0x80),
        }
    }

    #[inline(always)]
    pub unsafe fn inb(port: u16) -> u8 {
        let ret: u8;
        asm!(
            "in al, dx",
            in("dx") port,
            out("al") ret,
            options(nomem, nostack, preserves_flags)
        );
        ret
    }
    
    #[inline(always)]
    pub unsafe fn outb(port: u16, data: u8) {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") data,
            options(nomem, nostack, preserves_flags)
        );
    }

    pub fn init(&self) {
        unsafe {
            dprint(Some("IO.1"), "interrupições dessativadas");
            Self::outb(self.COM1 + 1, 0x00);
            dprint(Some("IO.2"), "ativou o DLAB");
            Self::outb(self.COM1 + 3, 0x80);
            
            Self::outb(self.COM1 + 0, 0x03);
            Self::outb(self.COM1 + 1, 0x00);
            dprint(Some("IO.3"), "Definiu a velocidade");
            dprint(Some("IO.4"), "desativou o DLAB");

            Self::outb(self.COM1 + 3, 0x03);
            dprint(Some("IO.5"), "Configurou os buffers FIFO");
            Self::outb(self.COM1 + 2, 0x07);
            dprint(Some("IO.6"), "Preparar os sinais de controle");
            Self::outb(self.COM1 + 4, 0x0b);
        };
    }

    pub fn get_keyboard_input(&mut self) -> u8 {
        unsafe {
            let status = Self::inb(self.KEYBOARD_STATUS);

            // Verifica se o buffer de saída tem dados
            if (status & 0x01) == 0 {
                return 0;
            }

            let scancode = Self::inb(self.KEYBOARD_DATA);
            let release_mask = self.SHIFT_LEFT.RELEASE_MASK; // Assumindo que 0x80 é padrão para ambos

            // Tecla solta (release): bit 7 = 1
            if (scancode & release_mask) != 0 {
                let base = scancode & !release_mask;
                if base == self.SHIFT_LEFT.CODE {
                    self.SHIFT_LEFT.PRESS = false;
                } else if base == self.SHIFT_RIGHT.CODE {
                    self.SHIFT_RIGHT.PRESS = false;
                }
                return 0;
            }

            // Shift pressionado
            if scancode == self.SHIFT_LEFT.CODE {
                self.SHIFT_LEFT.PRESS = true;
                return 0;
            } else if scancode == self.SHIFT_RIGHT.CODE {
                self.SHIFT_RIGHT.PRESS = true;
                return 0;
            }

            // Impede acesso fora dos limites do array
            if scancode as usize >= KEYBOARD_MAP_NORMAL.len() {
                return 0; 
            }

            // Traduz scancode → ASCII
            if self.SHIFT_LEFT.PRESS || self.SHIFT_RIGHT.PRESS {
                KEYBOARD_MAP_SHIFT[scancode as usize]
            } else {
                KEYBOARD_MAP_NORMAL[scancode as usize]
            }
        }
    }

    #[inline(always)]
    fn is_transmit_empty(&self) -> bool {
        unsafe { (Self::inb(self.COM1 + 5) & 0x20) != 0 }
    }


    pub fn put_char_serial(&self, c: u8) {
        while !self.is_transmit_empty() {}
        unsafe {
            Self::outb(self.COM1, c);
        }
    }

    pub fn put_string_serial(&self, s: &str) {
        for byte in s.bytes() {
            self.put_char_serial(byte);
        }
    }
}