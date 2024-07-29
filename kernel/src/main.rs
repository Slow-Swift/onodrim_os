#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use core::panic::PanicInfo;

use bootinfo::BootInfo;
use layout_renderer::LayoutRenderer;
use x86_64_hardware::{com1_println, devices::uart::COM1};

use font_renderer::FontRenderer;
use graphics_renderer::{Color, FrameBuffer};

mod errors;
mod graphics_renderer;
mod font_renderer;
mod layout_renderer;

// static mut lr: Option<LayoutRenderer> = None;
static mut BOOT_INFO: Option<BootInfo> = None;

/// This function is called on panic. 
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    com1_println!("Panicked");

    loop {}
}

#[no_mangle]
pub extern "C" fn kernel_main(bootinfo: *mut BootInfo) {
    let bootinfo = unsafe { &*bootinfo };
    COM1.lock().initialize();

    com1_println!("Hello world from the kernel!");
    com1_println!("Bootinfo valid: {}", bootinfo.has_valid_magic());

    let mut frame_buffer = FrameBuffer::from_boot_data(&bootinfo)
        .expect("Could not create frame buffer.");
    frame_buffer.fill(Color::new(0x000000));

    let font_renderer = FontRenderer::create(
        bootinfo.font_file_address.as_u64() as *mut u8, 
        bootinfo.font_file_size, 
        frame_buffer
    ).expect("Could not create font renderer");
    
    let mut layout_renderer = LayoutRenderer::new(font_renderer);
    layout_renderer.print_string("Hello World from the kernel!\n");
    layout_renderer.print_string("Kernel Finished\n");

    loop {}
}