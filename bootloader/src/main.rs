#![no_main]
#![no_std]

use bootinfo::BootInfo;
use r_efi::efi;
use uefi::SystemTableWrapper;
use x86_64_hardware::{com1_println, memory::PAGE_SIZE};
mod uefi;

#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    loop {};
}

#[no_mangle]
pub extern "C" fn efi_main(image_handle: efi::Handle, system_table: *const efi::SystemTable) -> efi::Status {
    let system_table = unsafe { SystemTableWrapper::new(system_table) };
    
    main(image_handle, system_table);

    efi::Status::SUCCESS
}

fn main(_image_handle: efi::Handle, mut system_table: SystemTableWrapper) -> Result<(), efi::Status> {
    com1_println!("Bootloader loaded");
    let bootinfo_size_pages = (core::mem::size_of::<BootInfo>() + PAGE_SIZE as usize - 1) / PAGE_SIZE as usize;
    let bootinfo = system_table.boot_services().allocate_pages::<BootInfo>(r_efi::system::LOADER_DATA, bootinfo_size_pages)?;
    let bootinfo = unsafe { &mut *bootinfo };
    (*bootinfo) = BootInfo::default();
    bootinfo.framebuffer = initialize_gop(system_table)?;

    loop {}

    // Status::SUCCESS
}

fn initialize_gop(system_table: uefi::SystemTableWrapper) -> Result<bootinfo::FrameBuffer, efi::Status>{
    let gop = match system_table.boot_services().get_graphics_output_protocol() {
        Ok(gop) => gop,
        Err(status) => {
            return Err(status)
        }
    };

    return Ok(gop.get_framebuffer());
}