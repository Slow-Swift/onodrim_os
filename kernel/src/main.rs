#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use core::panic::PanicInfo;

use boot_data::BootData;
use vga_buffer::print_something;

mod boot_data;
mod vga_buffer;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}


#[no_mangle] // don't mangle the name of this function
pub extern "sysv64" fn _start(boot_data: BootData) -> u32 {
    print_something();
    return boot_data.output_mode.output_width as u32;
}