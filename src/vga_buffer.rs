use volatile::Volatile;
use core::fmt;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Colour {
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
struct ColourCode(u8);

impl ColourCode {
    fn new(foreground: Colour, background: Colour) -> ColourCode {
        ColourCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    colour_code: ColourCode
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT]
}

pub struct Writer {
    row: usize,
    column: usize,
    colour: ColourCode,
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
                
                self.buffer.chars[self.row][self.column].write(ScreenChar {
                    ascii_character: byte,
                    colour_code: self.colour
                });

                self.column += 1;
            }
        }
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
        let blank = ScreenChar {
            ascii_character: b' ',
            colour_code: self.colour
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


pub fn print_something() {
    use core::fmt::Write;
    let mut writer = Writer {
        row: 0,
        column: 0,
        colour: ColourCode::new(Colour::Yellow, Colour::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };

    for iter in 1..20 {

        write!(writer, "Iteration {}: ", iter).unwrap();
    writer.write_byte(b'H');
    writer.write_string("ello!\n");
    write!(writer, "The numbers are {} and {}", 42, 1.0/3.0).unwrap();
    writer.write_byte(b'\n')
    }
}