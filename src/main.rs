#![no_std]
#![no_main]

#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod vga_buffer;
mod values;
mod serial;

use core::panic::PanicInfo;
use values::QemuExitCode;

macro_rules! dbg {
    ($fmt:expr) => {
        serial_print!($fmt);
        print!($fmt);
    };
    ($fmt:expr, $($arg:tt)*) => {
        serial_print!($fmt, $($arg)*);
        print!($fmt, $($arg)*);
    };
}

macro_rules! dbgln {
    () => {
        serial_println!("\n");
        println!("\n");
    };
    ($fmt:expr) => {
        serial_println!($fmt);
        println!($fmt);
    };
    ($fmt:expr, $($arg:tt)*) => {
        serial_println!($fmt, $($arg)*);
        println!($fmt, $($arg)*);
    };
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    dbgln!("[failed]\n");
    dbgln!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
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
pub fn test_runner(tests: &[&dyn Testable]) {
    use values::Paint;
    use vga_buffer::set_terminal_colour;

    set_terminal_colour(Paint::Black, Paint::White);
    dbgln!("Running {} tests", tests.len());
    
    for test in tests {
        test.run();
    }

    exit_qemu(QemuExitCode::Success);
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where T: Fn(),
{
    fn run(&self) {
        dbgln!("{}...\t", core::any::type_name::<T>());
        self();
        dbgln!("[ok]");
    }
}