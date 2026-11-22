use limine::{request::FramebufferRequest};
use na::DMatrix;
// use naive_timer::Timer;
use crate::{db_font, debuglog::dprint, timer::TimeInteration};


pub use crate::definitions::video::*;

/// matriz de cor virtual:
/// 
/// ```
/// | color | shift | size |
/// | red   |   r1  |  r2  |
/// | green |   g1  |  g2  |
/// | blue  |   b1  |  b2  |
/// | alfa  |   a1  |  a2  |
/// ```
/// 

#[derive(Copy, Clone, Debug)]
pub struct VirtualColor 
{ 
    r: u8,
    g: u8,
    b: u8,
    a: u8
}


impl VirtualColor 
{
    #[inline(always)]
    pub const fn new_hex(color : u32) -> Self
    {
        Self{
            r: ((color >> 24) & 0xFF) as u8,
            g: ((color >> 16) & 0xFF) as u8,
            b: ((color >> 8) & 0xFF) as u8,
            a: (color & 0xFF) as u8,
        }
    }


    #[inline(always)]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self
    { Self{ r: r, g: g, b: b, a: a } }    

    pub const fn unpack(&self) -> u32 
    {
        ((self.a as u32) << 24) | 
        ((self.r as u32) << 16) | 
        ((self.g as u32) << 8) | 
        (self.b as u32)
    }

    pub const fn lerp(&self, color2: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        let um_menos_t = 1.0 - t;

        Self {
            r: ((self.r as f32 * um_menos_t) + (color2.r as f32 * t)) as u8,
            g: ((self.g as f32 * um_menos_t) + (color2.g as f32 * t)) as u8,
            b: ((self.b as f32 * um_menos_t) + (color2.b as f32 * t)) as u8,
            a: ((self.a as f32 * um_menos_t) + (color2.a as f32 * t)) as u8,
        }
    }
}

impl PartialEq for VirtualColor {
    fn eq(&self, other: &Self) -> bool {
        if 
            self.a == other.a &&
            self.r == other.r &&
            self.g == other.g &&
            self.b == other.b 
        {true} else {false}
    }
}



pub const MAX_WIDTH: usize = 1920;
pub const MAX_HEIGHT: usize = 1080;


pub struct Vga 
{
    framebuffer: Option<DMatrix<u32>>,
    framebufferpixels: *mut u32,
    picthpixels: usize,
    width: usize, 
    height: usize
}

impl Vga {
    pub fn new
    () -> Self
    {Self {
        framebuffer: None,
        framebufferpixels: core::ptr::null_mut(),
        picthpixels: 0,
        width: 0,
        height: 0
    }}

    pub fn init
    (
        &mut self, 
        fb_request: &FramebufferRequest
    ) -> bool 
    {
        let mut timeout = TimeInteration::new(Some(100u64));
        
        self.framebuffer = Some(DMatrix::<u32>::zeros(MAX_HEIGHT, MAX_WIDTH));

        let mut initialized: bool = false;
        dprint(Some("[V.0]"), "video::init começou");

        timeout.loop_(|_time| {
            dprint(Some("[V.1]"), "esperando request");
            if let Some(framebuffer_response) = fb_request.response() 
            {
                dprint(Some("[V.2]"), "recebeu a resposta da requisção");
                if let Some(framebuffer) = framebuffer_response.framebuffers().first() 
                {
                    dprint(Some("[V.3]"), "Recebeu o frame buffer");

                    self.framebufferpixels = framebuffer.address() as *mut u32;
                    self.picthpixels       = (framebuffer.pitch as usize) / 4;
                    self.width             = framebuffer.width as usize;
                    self.height            = framebuffer.height as usize;

                    if self.width > MAX_WIDTH { self.width = MAX_WIDTH; }
                    if self.height > MAX_HEIGHT { self.height = MAX_HEIGHT; }
                    dprint(Some("[V.4]"), "organizou as dimenções");

                    
                    // Limpa o backbuffer estático com preto no boot
                    self.clear(TRANSPARENT);

                    initialized = true;
                    dprint(Some("[V.5]"), "iniciou");
                }
            }
            
            if initialized {_time.close();}
        });
        
        initialized
    }




    pub fn set_pixel(&mut self, x: usize, y: usize, color: u32) 
    {

        if x >= self.width || y >= self.height 
        { return; }

        let src = VirtualColor::new_hex(color);
        if src.a == 0 
        { return; }


        if src.a == 255 
        { *unsafe {self.framebuffer.as_mut().unwrap().get_unchecked_mut((y, x))}  = color;  () }


        let dest = unsafe {VirtualColor::new_hex(*self.framebuffer.as_mut().unwrap().get_unchecked((y, x)))};

        let a = src.a as u128;
        let r_res = ((src.r as u128 * a + dest.r as u128 * (255 - a)) / 255) as u8;
        let g_res = ((src.g as u128 * a + dest.g as u128 * (255 - a)) / 255) as u8;
        let b_res = ((src.b as u128 * a + dest.b as u128 * (255 - a)) / 255) as u8;

        let nc = VirtualColor::new(r_res, g_res, b_res, 0xff);

        unsafe 
        { 
            *self
                .framebuffer
                .as_mut()
                .unwrap()
                .get_unchecked_mut((y, x)) = nc.unpack() 
        };
    }




    pub fn swap_buffers(&mut self) 
    {
        unsafe 
        {
            if self.framebufferpixels.is_null() 
            { return; }


            for y in 0..self.height 
            { 
                for x in 0..self.width 
                {
                    // Recupera a cor do nosso backbuffer nalgebra
                    let cor = 
                        *self
                        .framebuffer
                        .as_mut()
                        .unwrap()
                        .get_unchecked((y, x));


                    // Escrita volátil direta na memória de vídeo mapeada pelo Limine
                    self.framebufferpixels
                        .add(y * self.picthpixels + x)
                        .write_volatile(cor);
                }
            }
        }
    }

    pub fn clear(&mut self, color: u32) 
    {
        for c in self.framebuffer.as_mut().unwrap().iter_mut() 
        {*c = color};
    }

    pub fn draw_char(&mut self, x: usize, y: usize, c: char, color: u32) 
    {
        let glyph: [u16; 16];

        let ci = c as usize;
        if ci >= 128 {
            return;
        }
        glyph = db_font::FONT_TEXT[ci];
        
        for row in 0..16 
        {
            let bits = glyph[row];
            for col in 0..16 
            {
                if (bits & (1 << (15 - col))) != 0 
                {
                    self.set_pixel(x + col, y + row, color);
                }
            }
        }
    }
    
    pub fn draw_string(&mut self,mut x: usize, y: usize, s: &str, color: u32) 
    {
        for c in s.chars() 
        {
            self.draw_char(x, y, c, color);
            x += 16;
        }
    }


    pub fn draw_rect(&mut self, pos: (usize, usize), size: (usize, usize), color: u32)
    {
        for pixel in self.framebuffer
            .as_mut()
            .unwrap()
            .view_mut((pos.1, pos.0), (size.1, size.0))
            .iter_mut()
        {
            *pixel = color;
        }
    }


    pub fn get_width(&self) -> usize 
    { self.width }

    pub fn get_height(&self) -> usize 
    { self.height }

}




