#![no_std]
#![no_main]

#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod vga_buffer;
mod values;

use core::panic::PanicInfo;
use values::QemuExitCode;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

#[no_mangle]
pub extern  "C" fn _start() -> ! {
    println!("Hello world!");

    #[cfg(test)]

    test_main();

    loop {}
}

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    use values::Paint;
    use vga_buffer::set_terminal_colour;

    set_terminal_colour(Paint::Black, Paint::White);
    println!("Running {} tests", tests.len());
    
    for test in tests {
        test();
    }

    exit_qemu(QemuExitCode::Success);
}

#[test_case]
fn trivial_assertion() {
    print!("Trivial assertion... ");
    assert_eq!(1, 1);
    println!("[ok]");
}