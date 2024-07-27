#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use core::panic::PanicInfo;

use bootinfo::BootInfo;
use x86_64_hardware::{com1_println, devices::uart::COM1};
// use core::fmt::Write;

// use boot_data::BootData;
// use font_renderer::FontRenderer;
// use graphics_renderer::{Color, FrameBuffer};
// use layout_renderer::LayoutRenderer;

// mod errors;
// mod boot_data; 
// mod graphics_renderer;
// mod font_renderer;
// mod layout_renderer;

// static mut lr: Option<LayoutRenderer> = None;
// static mut bd: Option<BootData> = None;

/// This function is called on panic. 
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // let boot_data = unsafe { bd.unwrap() };
    // let mut frame_buffer = match FrameBuffer::from_boot_data(&boot_data) {
    //     Ok(gr) => gr,
    //     Err(_) => loop {}
    // };

    // // Clear the screen
    // frame_buffer.fill(Color::new(0x0000FF));

    // let font_renderer = match FontRenderer::create(
    //     boot_data.font_file_address, 
    //     boot_data.font_file_size, 
    //     frame_buffer
    // ) {
    //     Ok(renderer) => renderer,
    //     Err(_) => {
    //         loop {}
    //     }
    // };

    // let mut layout_renderer = LayoutRenderer::new(font_renderer);
    // layout_renderer.print_string("Kernel Panicked\n");

    loop {}
}

// #[no_mangle] // don't mangle the name of this function
// pub extern "sysv64" fn _start(boot_data: BootData) -> u32 {
//     unsafe { bd = Some(boot_data); }

//     let mut frame_buffer = match FrameBuffer::from_boot_data(&boot_data) {
//         Ok(gr) => gr,
//         Err(_) => return 2
//     };


//     // Clear the screen
//     frame_buffer.fill(Color::new(0x000000));

//     let font_renderer = match FontRenderer::create(
//         boot_data.font_file_address, 
//         boot_data.font_file_size, 
//         frame_buffer
//     ) {
//         Ok(renderer) => renderer,
//         Err(_) => {
//             return 3;
//         }
//     };

//     let mut layout_renderer = LayoutRenderer::new(font_renderer);
//     layout_renderer.print_string("Hello World from the kernel!\n");
//     layout_renderer.print_string("Kernel Finished\n");

//     // loop {}

//     loop {
//         layout_renderer.print_string("\r|");
//         delay(10000000);
//         layout_renderer.print_string("\r/");
//         delay(10000000);
//         layout_renderer.print_string("\r-");
//         delay(10000000);
//         layout_renderer.print_string("\r\\");
//         delay(10000000);
//     }
// } 

// fn delay(amount: u64) {
//     let mut _n=0;
//     for _ in 0..amount {
//         _n += 1;
//     }
// }

#[no_mangle]
pub extern "C" fn kernel_main(bootinfo: *mut BootInfo) {
    let bootinfo = unsafe { &*bootinfo };
    COM1.lock().initialize();

    com1_println!("Hello world from the kernel!");
    com1_println!("Bootinfo valid: {}", bootinfo.has_valid_magic());

    loop {}
}