#![no_std]
#![no_main]

use core::panic::PanicInfo;

use bootinfo::BootInfo;
use x86_64_hardware::memory::PageFrameAllocator;

mod errors;
mod graphics_renderer;
mod font_renderer;
mod layout_renderer;
mod logger;

/// This function is called on panic. 
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    logger::initialize_com1();

    println!("Kernel Panicked:");
    println!("  {}", info.message());
    match info.location() {
        Some(location) => println!("  At: {location}"),
        None => {},
    }

    loop {}
}

#[no_mangle]
pub extern "C" fn kernel_main(bootinfo: *mut BootInfo) {
    let bootinfo = unsafe { &mut *bootinfo };

    logger::initialize_com1();
    logger::initialize_screen_output(bootinfo);

    println!("Hello World from the kernel");

    let allocator = unsafe { 
        PageFrameAllocator::new_from_bitmap(
            &mut bootinfo.meminfo.bitmap, 
            bootinfo.meminfo.free_memory, 
            bootinfo.meminfo.used_memory
        ) 
    };
    println!("Initialized Page Allocator:");
    println!(
        "  Free Memory: {:#X} ({} GB, {} MB, {} KB)", 
        allocator.get_free_ram(), 
        allocator.get_free_ram() / (1024 * 1024 * 1024), 
        allocator.get_free_ram() / (1024 * 1024) % 1024, 
        allocator.get_free_ram() / 1024 % 1024
    );
    println!(
        "  Used Memory: {:#X} ({} GB, {} MB, {} KB)", 
        allocator.get_used_ram(), allocator.get_used_ram() / (1024 * 1024 * 1024), 
        allocator.get_used_ram() / (1024 * 1024) % 1024, 
        allocator.get_used_ram() / 1024 % 1024);
    let total_memory = allocator.get_free_ram() + allocator.get_used_ram();
    println!(
        "  Total Usable Memory: {:#X} ({} GB, {} MB, {} KB)", 
        total_memory, total_memory / (1024 * 1024 * 1024), 
        total_memory / (1024 * 1024) % 1024, 
        total_memory / 1024 % 1024
    );

    println!("Kernel Finished");

    log_debug!("Kernel", "Debug Test");
    log_info!("Kernel", "Yes");
    log_warn!("Kernel", "Uh oh");
    log_error!("Kernel", "Oh no!");
    log_critical!("Kernel", "BOOM");

    loop {}
}