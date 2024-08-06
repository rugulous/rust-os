#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod values;
mod serial;
pub mod vga_buffer;

use core::panic::PanicInfo;
use values::QemuExitCode;

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

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

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        dbgln!("{}...\t", core::any::type_name::<T>());
        self();
        dbgln!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}