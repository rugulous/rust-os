#![no_std]
#![no_main]

mod vga_buffer;
mod values;

use core::panic::PanicInfo;

use vga_buffer::{set_terminal_bg, set_terminal_colour, set_terminal_fg};
use crate::values::Paint;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle]
pub extern  "C" fn _start() -> ! {
    let colours = [Paint::Black, Paint:: Blue, Paint::Brown, Paint::Green, Paint::White];

    for foreground in colours{
        set_terminal_fg(foreground as Paint);

        for background in colours{
            set_terminal_bg(background as Paint);

            println!("Testing colour {}, {}", (foreground as u8), (background as u8));
        }
    }

    set_terminal_colour(Paint::Green, Paint::Black);
    println!("Green text!");

    set_terminal_fg(Paint::White);
    set_terminal_bg(Paint::Brown);
    println!("poopy bum bum");

    set_terminal_colour(Paint::LightBlue, Paint::White);
    println!("Different colours?");

    loop {}
}