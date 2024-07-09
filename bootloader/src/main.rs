#![no_main]
#![no_std]

use core::mem::transmute;
use boot_data::BootData;
use loader::load_kernel;
use uefi::println;
use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::proto::console::text::Output;
use uefi::proto::media::file::Directory;
use uefi::table::boot::MemoryType;
use uefi::table::boot::OpenProtocolParams;
use uefi::Error;

mod elf;
mod loader;
mod boot_data;

#[entry]
fn main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {

    uefi::helpers::init(&mut system_table).unwrap();

    system_table.stdout().clear().expect("Could not clear display");
    println!("Bootloader started!");

    let mut boot_data = BootData::empty();

    setup_output_mode(system_table.stdout());
    println!("Activated output mode 0");

    let current_output_mode = system_table.stdout().current_mode().expect("Could not get current output mode").expect("No output mode");
    boot_data.output_mode.output_width = current_output_mode.columns();
    boot_data.output_mode.output_height = current_output_mode.rows();

    graphics(image_handle.clone(), system_table.boot_services(), &mut boot_data);
    let kernel_base = get_memory_map(system_table.boot_services()).expect("Could not get mm");

    let root = open_root_directory(image_handle, system_table.boot_services()).expect("Could not open root directory");
    let entry_point = load_kernel(system_table.boot_services(), root, kernel_base);

    // println!("Exiting boot services...");
    // let (_system_table, memory_map) = unsafe { 
    //     system_table.exit_boot_services(MemoryType::LOADER_DATA)
    // };
    // let (memory_map, memory_map_meta) = memory_map.as_raw();
    // boot_data.memory_descriptor_size = memory_map_meta.desc_size;
    // boot_data.memory_map_size = memory_map_meta.map_size;
    // boot_data.memory_map = Some(memory_map);

    println!("{}", boot_data.output_mode.output_height);
    println!("Starting kernel...");
    type KernelEntry = extern "sysv64" fn(BootData) -> u32;
    let kernel_entry: KernelEntry = unsafe { transmute(entry_point) };
    let exit_code = kernel_entry(boot_data);
    println!("Kernel quit with exit code {exit_code}");

    loop {}

    Status::SUCCESS
}

fn setup_output_mode(stdout: &mut Output) {
    for mode in stdout.modes() {
        if mode.index() == 0 {
            stdout
                .set_mode(mode)
                .expect("Could not set output mode");
            return;
        }
    }

    panic!("Output Mode 0 Not Availiable");
}

fn graphics(image_handle: Handle, boot_services: &BootServices, boot_data: &mut BootData) {
    println!("Checking Graphics");
    let gop_handle = 
        match boot_services.get_handle_for_protocol::<GraphicsOutput>() {
            Ok(handle) => handle,
            Err(_error) => {
                println!("Could not get GraphicsOutputProtocol handle");
                return;
            }
        };

    let gop;

    // Using open_protocol because open_protocol_exclusive stops printing from working
    unsafe { 
        gop = match boot_services.open_protocol::<GraphicsOutput>(
            OpenProtocolParams {
                handle: gop_handle,
                agent: image_handle,
                controller: None
            },
            uefi::table::boot::OpenProtocolAttributes::GetProtocol
        ) {
            Ok(gop) => gop,
            Err(_error) => {
                println!("Could not open GraphicsOutputProtocol");
                return;
            }
        };
    }
    
    println!("Graphics Modes:");
    for (i, mode) in gop.modes(boot_services).enumerate() {
        let (x,y) = mode.info().resolution();
        let format = mode.info().pixel_format();
        println!("  Mode {i}: {x}x{y} Format: {format:?}");
    }

    let current_mode = gop.current_mode_info();
    let (width, height) = current_mode.resolution();
    let format = gop.current_mode_info().pixel_format();

    boot_data.graphics_mode.width = width;
    boot_data.graphics_mode.height = height;
    boot_data.graphics_mode.format = format;
    boot_data.graphics_mode.stride = current_mode.stride();

    println!("Current Graphics Mode: {width}x{height}, Format: {:?}", format);
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