
use core::{cell::UnsafeCell, default, mem::MaybeUninit, str};
use alloc::vec::{self, Vec};
use na::{DMatrix, RowDVector, SMatrix};
use rhai::Engine;
use spin::{Mutex, mutex::Mutex};
use crate::{debuglog::{dprint, dwrite}, io::InputOutputSistem, utils::{SafeMatrix, ToSafe}, video::{MAX_HEIGHT, MAX_WIDTH, OCEANDARK_COLOR, OFFWHITE_COLOR, Vga, VirtualColor}};


const MAXCOLS: usize = MAX_WIDTH / 8; 
const MAXROWS: usize = MAX_HEIGHT / 8;






///  eu sei.. ta muito seguro B]
struct TermUI
{
    font_size: ToSafe<usize>,
    padding: ToSafe<usize>,
    size: ToSafe<(usize, usize)>,
    cursor_pos: ToSafe<(usize, usize)>,
    view_pos: ToSafe<(usize, usize)>,
    buffer: SafeMatrix<u8, MAXCOLS, MAXROWS>
}







impl TermUI {


    #[inline(always)]
    pub fn new()->Self
    { Self::const_new() } /// so um wrapper
    
    #[inline(always)]
    pub const fn const_new() -> Self
    { Self {
        font_size: Mutex::new(MaybeUninit::zeroed()),
        padding: Mutex::new(MaybeUninit::zeroed()),
        size: Mutex::new(MaybeUninit::zeroed()),
        cursor_pos: Mutex::new(MaybeUninit::zeroed()),
        view_pos: Mutex::new(MaybeUninit::zeroed()),
        buffer: Mutex::new(MaybeUninit::zeroed())
    } }




    pub fn set_font_size(&mut self, fs: usize)
    {
        let mut guard = self.font_size.lock();
        let pt_fs = unsafe { guard.assume_init_mut() };

        *pt_fs = fs;

        drop(guard);
    }

    pub fn get_font_size_value(&mut self) -> usize
    {
        let guard = self.font_size.lock();
        let fs = unsafe { guard.assume_init_read() };
        drop(guard);
        fs
    }




    pub fn set_padding(&mut self, pd: usize)
    {
        let mut guard = self.padding.lock();
        let pt_pd = unsafe { guard.assume_init_mut() };


        *pt_pd = pd;

        drop(guard);
    }

    pub fn get_padding_value(&mut self) -> usize
    {
        let guard = self.padding.lock();
        let padding = unsafe { guard.assume_init_read() };
        drop(guard);
        padding
    }





    pub fn set_size(&mut self, size: (usize, usize))
    {
        let mut guard = self.size.lock();
        let pt_size= unsafe { guard.assume_init_mut() };

        *pt_size = size;

        drop(guard);
    }
    
    pub fn get_size_value(&mut self) -> (usize, usize)
    {
        let guard = self.size.lock();
        let size = unsafe { guard.assume_init_read() };
        drop(guard);
        size
    }





    pub fn set_view_pos(&mut self, pos: (usize, usize))
    {
        let mut guard = self.view_pos.lock();
        let pt_vpos= unsafe { guard.assume_init_mut() };

        *pt_vpos = pos;

        drop(guard);
    }
    
    pub fn get_view_pos_value(&mut self) -> (usize, usize)
    {
        let guard = self.view_pos.lock();
        let vp = unsafe { guard.assume_init_read() };
        drop(guard);
        vp
    }




    pub fn draw(&mut self, _shell: ())
    {
        
        let mut guard = self.buffer.lock();
        let pt_buff= unsafe { guard.assume_init_mut() };

        // let (vpx, vpy) = self.get_view_pos_value();
        // let (h, w) = self.get_size_value();

        // let _ = (*pt_buff).map_with_location(
        //     | r, c, v| 
        //     {

        //     }
        // );
    }
}

struct Shell
{
    UI: ToSafe<TermUI>,
    VM: ToSafe<Engine>,

    historic: ToSafe<DMatrix<u8>>,
    input_line: ToSafe<Vec<u8>>,
    current_line_pos:  ToSafe<usize>,
    current_line_lenght:  ToSafe<usize>
}

impl Shell 
{

    #[inline(always)]
    pub fn new() -> Self 
    { Self::const_new() }

    #[inline(always)]
    pub const fn const_new()->Self
    {Self {
        UI: Mutex::new(MaybeUninit::zeroed()),
        VM: Mutex::new(MaybeUninit::new(Engine::new_raw())),
        historic: Mutex::new(MaybeUninit::zeroed()),
        current_line_pos:  Mutex::new(MaybeUninit::zeroed()),
        current_line_lenght:  Mutex::new(MaybeUninit::zeroed()),
        input_line: Mutex::new(MaybeUninit::zeroed())
    }}


    pub fn write_char(&mut self, c: u8, is_output: bool) 
    {
        // let mut guardcl = self.current_line_pos.lock();
        // let pt_cl= unsafe { guardcl.assume_init_mut() }.get_mut();


        let script: &Vec<u8> = {
            let mut guardinp = self.input_line.lock();
            let pt_inpl = unsafe { guardinp.assume_init_mut() };
            
            &(pt_inpl).as_slice().to_vec()
        };

        match c {
            b'\n' => {
                self.execute(&script);

                let mut guardhis = self.historic.lock();
                let pt_his= unsafe { guardhis.assume_init_mut() };

                
            }
            b'\t' => {
                let mut guardinp = self.input_line.lock();
                let pt_inpl = unsafe { guardinp.assume_init_mut() };

                let mut guardlen = self.current_line_lenght.lock();
                let pt_len = unsafe { guardlen.assume_init_mut() };

                for _ in 0..1 
                {
                    (*pt_inpl)[*pt_len] = b' ';
                }

                drop(guardinp);
                drop(guardlen);
            }
            _ => {
                let mut guardinp = self.input_line.lock();
                let pt_inpl = unsafe { guardinp.assume_init_mut() };

                let mut guardlen = self.current_line_lenght.lock();
                let pt_len = unsafe { guardlen.assume_init_mut() };

                (*pt_inpl)[*pt_len] = c;
                (*pt_len) += 1;

                drop(guardinp);
                drop(guardlen);
            }
            
        }

    }

    pub fn write_str(&mut self, s: Vec<u8>, is_output: bool)
    {
        for b in s
        { self.write_char(b, is_output); }       
    }

    pub fn execute(&mut self, script: &Vec<u8>) 
    {
        let mut guard_vm = self.VM.lock();
        let pt_vm= unsafe { guard_vm.assume_init_mut() };

        // let mut guard_his = self.historic.lock();
        // let pt_his= unsafe { guard_his.assume_init_mut() };
        
        let scr = str::from_utf8(script).unwrap();
        let output = (*pt_vm).eval::<Vec<u8>>(scr).unwrap();

        self.write_str(output, true);
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


