



/// locais da memoria responsaveis pela entrada e saida do usuario

//arm
#[cfg(target_arch = "aarch64")] pub const UARCH0_DR: *mut u32 = 0x0900_0000 as *mut u32; 
#[cfg(target_arch = "aarch64")] pub const UARCH0_FR: *const u32 = 0x0900_0018 as *const u32; 
#[cfg(target_arch = "aarch64")] pub const TXFF: u32 = 1<<5;
#[cfg(target_arch = "aarch64")] pub const RXFE: u32 = 1<<4;

//x86_64
#[cfg(target_arch = "x86_64")]  pub const COM1: u16 = 0x3f8;