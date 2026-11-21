
use core::str;
use alloc::string::{String, ToString};
use alloc::vec::{Vec};
use alloc::vec;
use na::DMatrix;
use rhai::Engine;
use crate::debuglog::{dprint, dwrite};
use crate::io::InputOutputSistem;
use crate::video::{MAX_HEIGHT, MAX_WIDTH, OFFWHITE_COLOR, Vga};


const MAXCOLS: usize = MAX_WIDTH / 8; 
const MAXROWS: usize = MAX_HEIGHT / 8;




///  eu sei.. ta muito seguro B]
pub struct TermUI
{
    font_size: usize,
    padding: usize,
    size: (usize, usize),
    cursor_pos: (usize, usize),
    view_pos: (usize, usize),
    buffer: Option<DMatrix<u8>>
}

impl TermUI 
{
    #[inline(always)]
    pub const fn new() -> Self
    { 
        Self {
            font_size: 8,
            padding: 10,
            size: (MAXROWS, MAXCOLS),
            cursor_pos: (0, 0),
            view_pos: (0, 0),
            buffer: None
        } 
    }


    #[inline(always)]
    pub fn init(&mut self)
    {
        self.buffer = Some(DMatrix::<u8>::zeros(MAXROWS, MAXCOLS));
        self.clear();
    }
    
    #[inline(always)]
    pub fn clear(&mut self)
    { for c in self.buffer.as_mut().unwrap() {*c = b' '} }


    #[inline(always)]
    pub fn scrollx(&mut self, dx: usize)
    { self.view_pos.1 += dx; }

    
    #[inline(always)]
    pub fn scrolly(&mut self, dy: usize)
    { self.view_pos.0 += dy; }


    #[inline(always)]
    pub fn writechar(&mut self, c: u8)
    {
        
    }

    pub fn draw(&mut self, video: &mut Vga)
    {
        let Some(buf) = self.buffer.as_mut() else { return; };

        let _ = buf
            .view_mut(self.view_pos, self.size)
            .map_with_location(
                | r, c, v| 
                { 
                    let (x, y) = 
                        (
                            c*self.font_size + self.padding,
                            r*self.font_size + self.padding
                        );
                    video.draw_char(y - self.view_pos.0, x - self.view_pos.1, v as char, OFFWHITE_COLOR); }
            );
    }
}




pub struct Shell
{
    UI: TermUI,
    VM: Option<Engine>,
    io: InputOutputSistem,

    historic: Option<Vec<String>>,
    buffer: Option<Vec<String>>,
    input_line: Option<String>,
    current_line_pos: usize,
    current_line_lenght: usize,

    status: u8, // 0. inicializando a linha; 1. tempo de requisção; 2. tempo de resposta
}


impl Shell 
{
    #[inline(always)]
    pub const fn new()->Self
    {Self {
        UI: TermUI::new(),
        VM: None,
        io: InputOutputSistem::new(),
        historic: None,
        buffer: None,
        input_line: None,
        current_line_pos: 0,
        current_line_lenght: 0,
        status: 0

    }}

    pub fn init(&mut self)
    {
        self.UI.init();
        self.VM = Some(Engine::new());
        self.io.init();

        self.historic = Some(vec![]);
        self.buffer = Some(vec![]);
        self.input_line = Some(String::new());
        self.status = 0;
    }

    #[inline(always)]
    fn clear_input_line(&mut self)
    {
        unsafe {
            for l in self.input_line.as_mut().unwrap().as_bytes_mut()
            { *l = b' ' }
        }
        self.current_line_lenght = 0
    }

    fn save_in_historic_command(&mut self, s: &String)
    {
        if let Some(hist) = self.historic.as_mut() 
        { (*hist).insert(self.current_line_pos, s.clone()); }
    }

    fn write_char(&mut self, c: char) 
    {
        match c as u8 {

            b'\n' => 
            {
                if let Some(line) = self.input_line.as_ref()
                {
                    let input = line.clone();
                    let result = self.execute(&input);

                    self.save_in_historic_command(&input);

                    if let Some(buff) = self.buffer.as_mut() 
                    {
                        (*buff).insert(self.current_line_pos, result);
                    }
                }
                self.clear_input_line();
            }

            b'\t' => 
            { self.write_str("  "); }
            
            0x08 =>
            {
                self.current_line_lenght -= if self.current_line_lenght == 0 {0} else {1};
                
                if let Some(line) = self.input_line.as_mut() {line.pop();}
            }

            _ => 
            {
                if let Some(line) = self.input_line.as_mut() {
                    line.insert(self.current_line_lenght, c as char);
                    self.current_line_lenght += 1;
                }
            }
            
        }

    }


    #[inline(always)]
    fn write_str(&mut self, s: &str)
    {
        for b in s.bytes()
        { self.write_char(b as char);}       
    }


    pub fn execute(&mut self, script: &String) -> String
    {
        let Some(vm) = self.VM.as_mut() else { return "VM não inicializada".to_string(); };
        let output = vm.eval::<String>(&script.clone());

        if let Ok(out) = output
        { out }
        else 
        { "erro".to_string() }
    }


    pub fn render(&mut self, video: &mut Vga)
    {  
        self.UI.draw(video);

        
        match self.status {
            0 => 
            {
                self.write_str(&"[motosserakernel] ".to_string());
                self.status = 1;
            }
            1 => 
            {
                
                let ch = self.io.read_char();
                self.write_char(ch);
                
            }
            2 => 
            {
                //self.UI.(self.VMoutput.to_vec());
                self.write_char('\n');
                self.status = 0
            }
            _ => todo!()
        };

        
    }

}




// abaixo o codigo antigo



// pub struct Terminal
// {
//     FONTSIZE: usize,
//     PANDDING: usize,

//     historic:

//     pos_cursor: (usize, usize), // .0 = x, .1 = y
//     aspect: (usize, usize), // .0 = row, .1 == cols
//     fg_color: u32,
//     bg_color: u32,
//     buffer: Mutex<MaybeUninit<Wrapperbuff<MAXROWS, MAXCOLS>>>,
//     VM: ToSafe<Engine>
// }


// impl Terminal {
//     pub const fn new() -> Self 
//     {Self { 
//         FONTSIZE: 0, 
//         PANDDING: 0, 
//         pos_cursor: (0, 0), 
//         aspect: (0, 0), 
//         fg_color: 0x0, 
//         bg_color: 0x0,
//         stdin: StdIn::new(),
//         buffer: Mutex::new(MaybeUninit::zeroed()),
//         VM: Mutex::new(MaybeUninit::zeroed())
//     }}
    
//     pub fn clear(&mut self) {
//         self.pos_cursor = (0, 0);
        
//         let mut guard = self.buffer.lock();
//         let buf = unsafe { guard.assume_init_mut() };

//         for cell in (&mut *(*buf).mat.get_mut()).iter_mut() { *cell = Cell::zeros(); }

//         drop(guard);
//     }

//     pub fn init(&mut self, vga: &Vga) -> bool
//     {

//         dprint(Some("Term.0"), "terminal::init começou");

//         self.PANDDING = 8;
//         self.FONTSIZE = 8;


//         dprint(Some("Term.1"), "ajeitou o tamanho da fonte e o pandding");

//         self.aspect = (
//             (vga.get_height() - self.PANDDING*2) / self.FONTSIZE,
//             (vga.get_width() - self.PANDDING*2) / self.FONTSIZE
//         );


//         dprint(Some("Term.2"), "ajeitou o aspecto do terminal");
        
//         self.bg_color = VirtualColor::new_hex(OCEANDARK_COLOR).unpack();
//         self.fg_color = VirtualColor::new_hex(OFFWHITE_COLOR).unpack();


//         dprint(Some("Term.3"), "ajeitou as cores");

//         self.clear();
        
//         true
//     }
//     pub fn update(&mut self, io: &mut InputOutputSistem)
//     {
//         let c = io.get_keyboard_input();

//         match c {
//             b'\r' | b'\n' => {
//                 // let 
//             }

//             _ => {}
            
//         }
//     }

//     pub fn render(&mut self,  vga: &mut Vga)
//     {   
//         vga.clear(self.bg_color);
        
        
//         let mut guard = self.buffer.lock();
//         let buf_ptr = unsafe { guard.assume_init_mut() };
//         let buff =  (*buf_ptr).mat.get_mut();

//         let _ = buff.map_with_location(
//             |r, c, val| 
//             {
//                 let pos = (
//                     c*self.FONTSIZE + self.PANDDING,
//                     r*self.FONTSIZE + self.PANDDING
//                 );

//                 if pos == self.pos_cursor 
//                 {
//                     vga.draw_char(pos.0, pos.1, val[(0,0)] as char, self.bg_color);
//                 }else {
//                     vga.draw_char(pos.0, pos.1, val[(0,0)] as char, self.fg_color);
//                 }

//             }
//         );

//         drop(guard);
//     }
// }


