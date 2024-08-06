use volatile::Volatile;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

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

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Paint {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15
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
    buffer: &'static mut Buffer
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
        return self;
    }

    pub fn set_foreground(&mut self, foreground: Paint) -> &mut Writer{
        self.colour.set_foreground(foreground);
        return self;
    }

    pub fn set_background(&mut self, background: Paint) -> &mut Writer{
        self.colour.set_background(background);
        return self;
    }

    fn new_line(&mut self){
        self.column = 0;

        if self.row + 1 < BUFFER_HEIGHT {
            self.row += 1;
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
            colour: self.colour
        };

        for col in 0..BUFFER_WIDTH{
            self.buffer.chars[row][col].write(blank)
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
    });
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments){
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}