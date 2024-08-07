#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use os::values::Paint;
use os::vga_buffer::set_terminal_colour;
use os::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World!");
    os::init();

    #[cfg(test)]
    test_main();

    set_terminal_colour(Paint::White, Paint::Green);
    println!("We made it!");

    loop {}
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic_handler(info)
}