use volatile::Volatile;
use core::fmt::{self, Write};
use lazy_static::lazy_static;
use spin::Mutex;

use crate::values::Paint;

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print_w_colour {
    ($fg: expr, $bg: expr, $($arg:tt)*) => ($crate::vga_buffer::_print_retaining_colour($fg, $bg, format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println_w_colour {
    ($fg: expr, $bg: expr, $($arg:tt)*) => ($crate::print_w_colour!($fg, $bg, "{}\n", format_args!($($arg)*)));
}

pub fn set_terminal_colour(foreground: Paint, background: Paint) {
    WRITER.lock().set_colour(foreground, background);
}

pub fn set_terminal_fg(foreground: Paint){
    WRITER.lock().set_foreground(foreground);
}

pub fn set_terminal_bg(background: Paint){
    WRITER.lock().set_background(background);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct Colour(u8);

impl Colour {
    fn new(foreground: Paint, background: Paint) -> Colour {
        Colour((background as u8) << 4 | (foreground as u8))
    }

    fn set_background(&mut self, foreground: Paint){
        let mut colour: u8 = self.0;
        colour = colour & 0x0F;
        self.0 = colour | ((foreground as u8) << 4)
    }

    fn set_foreground(&mut self, background: Paint){
        let mut colour: u8 = self.0;
        colour = colour & 0xF0;
        self.0 = colour | (background as u8)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct TerminalChar {
    character: u8,
    colour: Colour
}

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<TerminalChar>; BUFFER_WIDTH]; BUFFER_HEIGHT]
}

pub struct Writer {
    row: usize,
    column: usize,
    colour: Colour,
    buffer: &'static mut Buffer,
    is_line_initialised: bool
}

impl Writer {
    pub fn write_string(&mut self, s: &str){
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe)
            }
        }
    }

    pub fn write_byte(&mut self, byte: u8){
        if !self.is_line_initialised {
            self.update_row_colour();
            self.is_line_initialised = true;
        }

        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column >= BUFFER_WIDTH {
                    self.new_line();
                }
                
                self.buffer.chars[self.row][self.column].write(TerminalChar {
                    character: byte,
                    colour: self.colour
                });

                self.column += 1;
            }
        }
    }

    pub fn set_colour(&mut self, foreground: Paint, background: Paint) -> &mut Writer {
        self.colour = Colour::new(foreground, background);
        self.update_row_colour();
        return self;
    }

    pub fn set_foreground(&mut self, foreground: Paint) -> &mut Writer{
        self.colour.set_foreground(foreground);
        self.update_row_colour();
        return self;
    }

    pub fn set_background(&mut self, background: Paint) -> &mut Writer{
        self.colour.set_background(background);
        self.update_row_colour();
        return self;
    }

    fn new_line(&mut self){
        self.column = 0;
        self.is_line_initialised = false;

        if self.row + 1 < BUFFER_HEIGHT {
            self.row += 1;
            self.clear_row(self.row);
            return;
        }

        for row in 1..BUFFER_HEIGHT{
            for col in 0..BUFFER_WIDTH{
                let char = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(char);
            }
        }

        self.clear_row(BUFFER_HEIGHT - 1);
    }

    fn clear_row(&mut self, row: usize){
        let blank = TerminalChar {
            character: b' ',
            colour: Colour::new(Paint::White, Paint::Black)
        };

        for col in 0..BUFFER_WIDTH{
            self.buffer.chars[row][col].write(blank)
        }
    }

    fn update_row_colour(&mut self){
        for col in 0..BUFFER_WIDTH {
            let mut char = self.buffer.chars[self.row][col].read();
            char.colour = self.colour;
            self.buffer.chars[self.row][col].write(char);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column: 0,
        row: 0,
        colour: Colour::new(Paint::Green, Paint::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        is_line_initialised: false
    });
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments){
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[doc(hidden)]
pub fn _print_retaining_colour(fg: Paint, bg: Paint, args: fmt::Arguments){
    let mut writer = WRITER.lock();
    let orig_colour = writer.colour;

    writer.set_colour(fg, bg);
    writer.write_fmt(args).unwrap();
    writer.colour = orig_colour;
}


//---- tests
#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    let s = "Some test string that fits on a single line";
    println!("{}", s);

    for (i, c) in s.chars().enumerate() {
        let writer = WRITER.lock();
        let screen_char = writer.buffer.chars[writer.row - 1][i].read();
        drop(writer);
        
        assert_eq!(char::from(screen_char.character), c);
    }
}