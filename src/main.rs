#![no_std]
#![no_main]

mod vga_buffer;
use core::panic::PanicInfo;

use vga_buffer::Paint;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle]
pub extern  "C" fn _start() -> ! {
    vga_buffer::WRITER.lock()
        .set_foreground(Paint::White)
        .set_background(Paint::Brown)
        .write_string("ZOO WEE MAMA\n");

    vga_buffer::WRITER.lock()
        .set_colour(Paint::LightBlue, Paint::White)
        .write_string("Different colours?");

    loop {}
}