



/// locais da memoria responsaveis pela entrada e saida do usuario

//arm

#[cfg(target_arch = "aarch64")] pub const UART0_PHYS: usize = 0x0900_0000;
#[cfg(target_arch = "aarch64")] pub const UARCH0_DR: *mut u32 = 0x0900_0000 as *mut u32; 
#[cfg(target_arch = "aarch64")] pub const UARCH0_FR: *const u32 = 0x0900_0018 as *const u32; 
#[cfg(target_arch = "aarch64")] pub const TXFF: u32 = 1<<5;
#[cfg(target_arch = "aarch64")] pub const RXFE: u32 = 1<<4;

//x86_64
#[cfg(target_arch = "x86_64")]  pub const COM1: u16 = 0x3f8;


pub const FRAME_SIZE: usize = 4096;


pub const MAXCOLS: usize = 1920 / 8; 
pub const MAXROWS: usize = 1080 / 8;

pub const RED_COLOR: u32 = 0xF01010FF;
pub const OFFWHITE_COLOR: u32 = 0xece8baFF;
pub const OCEANDARK_COLOR: u32 = 0x003747FF;

pub const BLACK_COLOR: u32 = 0x000000FF;
pub const WHITE_COLOR: u32 = 0xFFF;
pub const TRANSPARENT: u32 = 0x0;