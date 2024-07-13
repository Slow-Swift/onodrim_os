#![no_main]
#![no_std]

use core::mem::transmute;
use boot_data::BootData;
use graphics_info::GraphicsInfo;
use loader::load_font;
use loader::load_kernel;
use uefi::println;
use uefi::prelude::*;
use uefi::proto::media::file::Directory;
use uefi::table::boot::MemoryDescriptor;
use uefi::table::boot::MemoryType;
use uefi::Error;

mod elf;
mod loader;
mod boot_data;
mod graphics_info;

#[entry]
fn main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {

    uefi::helpers::init(&mut system_table).unwrap();

    system_table.stdout().clear().expect("Could not clear display");
    println!("Bootloader started!");

    let mut boot_data = BootData::empty();

    // Load the kernel
    let kernel_base = get_memory_map(system_table.boot_services()).expect("Could not get mm");
    let root = open_root_directory(image_handle, system_table.boot_services()).expect("Could not open root directory");
    let entry_point = load_kernel(system_table.boot_services(), root, kernel_base);

    // Load the font
    let root = open_root_directory(image_handle, system_table.boot_services()).expect("Could not open root directory");
    let (font_file_address, font_file_size) = load_font(system_table.boot_services(), root);
    boot_data.font_file_address = font_file_address as *const u8;
    boot_data.font_file_size = font_file_size;

    println!("Activating Graphics. Console will be deactivated.");
    let mut graphics_info = GraphicsInfo::new(system_table.boot_services());
    // graphics_info.print_modes();
    graphics_info.fill_boot_data(&mut boot_data);
    drop(graphics_info);

    system_table.boot_services().stall(1_000_000);

    // println!("Starting kernel...");
    let (system_table, _memory_map) = unsafe { 
        system_table.exit_boot_services(MemoryType::LOADER_DATA)
    };

    // let (memory_map, memory_map_meta) = _memory_map.as_raw();
    // let memory_map: &mut [MemoryDescriptor] = unsafe {
    //     core::slice::from_raw_parts_mut(memory_map.as_mut_ptr() as *mut MemoryDescriptor, memory_map_meta.map_size / memory_map_meta.desc_size)
    // };
    
    // unsafe { system_table.set_virtual_address_map(memory_map, 0); }

    // boot_data.memory_descriptor_size = memory_map_meta.desc_size;
    // boot_data.memory_map_size = memory_map_meta.map_size;
    // boot_data.memory_map = Some(memory_map);

    type KernelEntry = extern "sysv64" fn(BootData) -> u32;
    let kernel_entry: KernelEntry = unsafe { transmute(entry_point) };
    let _exit_code = kernel_entry(boot_data);

    loop {}

    // Status::SUCCESS
}

fn get_memory_map(boot_services: &BootServices) -> Result<u64, Error> {
    // Get the memory map
    let memory_map = match boot_services.memory_map(MemoryType::BOOT_SERVICES_DATA) {
        Ok(map) => map,
        Err(error) => {
            println!("Could not get memory map!\nError: {error}");
            return Err(error);
        }
    };

    // Loop over the memory map to find space to load the kernel into
    let mut base_address = 0;
    let mut num_pages = 0;
    println!("Searching memory for space for kernel...");
    for descriptor in memory_map.entries() {
        if descriptor.ty == MemoryType::CONVENTIONAL {
            if base_address <= descriptor.phys_start {
                base_address = descriptor.phys_start;
                num_pages = descriptor.page_count;
            } 
        }
    }
    println!("Found {num_pages} pages at {base_address:#X}.");

    Ok(base_address)
}

fn open_root_directory(image_handle: Handle, boot_services: &BootServices) -> Result<Directory, Error>{
    boot_services
        .get_image_file_system(image_handle)?
        .open_volume()
}