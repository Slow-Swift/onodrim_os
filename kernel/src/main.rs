#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use core::panic::PanicInfo;

use boot_data::BootData;
use graphics_renderer::{Color, GraphicsRenderer};

mod errors;
mod boot_data; 
mod graphics_renderer;

/// This function is called on panic. 
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle] // don't mangle the name of this function
pub extern "sysv64" fn _start(boot_data: BootData) -> u32 {
    let mut graphics_renderer = match GraphicsRenderer::from_boot_data(&boot_data) {
        Ok(gr) => gr,
        Err(_) => return 2
    };

    loop {
        graphics_renderer.fill(Color(0xFF, 0x00, 0x00));
        graphics_renderer.fill(Color::new(0x00FF00));
        graphics_renderer.fill(Color::new(0x0000FF));
        graphics_renderer.fill(Color::new(0xFFFFFF));
        graphics_renderer.fill(Color::new(0xFF00FF));
        graphics_renderer.fill(Color::new(0xFFFF00));
    }
    return 1;
} 