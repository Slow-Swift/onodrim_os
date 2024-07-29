#![no_main]
#![no_std]

use core::ffi::c_void;

use bootinfo::{BootInfo, MemInfo};
use elf_section_list::ElfSectionList;
use loaded_asset_list::LoadedAssetList;
use r_efi::efi;
use uefi::SystemTableWrapper;
use x86_64_hardware::{com1_println, memory::{PageFrameAllocator, PageTableManager, PhysicalAddress, VirtualAddress, MAX_MEM_SIZE, MAX_VIRTUAL_ADDRESS, MEM_1G, PAGE_SIZE}};

mod uefi;
mod unicode;
mod loaded_asset_list;
mod elf_section_list;

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    com1_println!("Panic:");
    com1_println!("  {}", info.message());

    match info.location() {
        Some(location) => com1_println!("  {location}"),
        None => {},
    }
    loop {};
}

#[no_mangle]
pub extern "C" fn efi_main(image_handle: efi::Handle, system_table: *const efi::SystemTable) -> efi::Status {
    let system_table = unsafe { SystemTableWrapper::new(system_table) };
    
    let result = main(image_handle, system_table);

    match result {
        Ok(()) => efi::Status::SUCCESS,
        Err(status) => {
            com1_println!("Bootloader Error: {status:#?}");
            status
        }
    }
}

fn main(image_handle: efi::Handle, system_table: SystemTableWrapper) -> Result<(), efi::Status> {
    com1_println!("Bootloader loaded");
    let bootinfo_size_pages = (core::mem::size_of::<BootInfo>() + PAGE_SIZE as usize - 1) / PAGE_SIZE as usize;
    let bootinfo = system_table.boot_services().allocate_pages::<BootInfo>(r_efi::system::LOADER_DATA, bootinfo_size_pages)?;
    let bootinfo = unsafe { &mut *bootinfo };
    (*bootinfo) = BootInfo::default();
    bootinfo.framebuffer = initialize_gop(system_table)?;

    let (kernel_asset_list, entry_point) = load_kernel(image_handle, system_table)?;

    let (font_file_address, font_file_page_count, font_file_size) = load_font(image_handle, &system_table)?;

    let configuration_table = system_table.get_configuration_table();
    let mem_info = system_table.boot_services().get_memory_map()?;
    com1_println!("Got Memory Map");

    system_table.boot_services().exit_boot_services(image_handle, mem_info.map_key)?;
    com1_println!("Exited Boot Services");

    unsafe {
        // Memory offset is 0 since we haven't set up paging yet
        (*bootinfo).framebuffer.fill(0, 0);
    }


    com1_println!("Memory Map:");
    for descriptor in mem_info.map.iter() {
        com1_println!("  {:?}: {:#X} -> {:#X} p({})", descriptor.mem_type(), descriptor.phys_addr.as_u64(), descriptor.max_physical_address().as_u64(), descriptor.num_pages)
    }

    let mut allocator = mem_info.map.init_frame_allocator();
    let max_physical_address = mem_info.map.max_physical_address();
    let max_usable_address = mem_info.map.max_usable_physical_address();
    mem_info.map.free_pages(&mut allocator).expect("Alloc error on free memory map. This should be impossible.");
    com1_println!("Availiable Memory: {:#X} p({})", allocator.get_free_ram(), (allocator.get_free_ram() + PAGE_SIZE - 1) / PAGE_SIZE);
    com1_println!("Used Memory: {:#X} p({})", allocator.get_used_ram(), (allocator.get_used_ram() + PAGE_SIZE - 1) / PAGE_SIZE);
    com1_println!("Total Usable Memory: {:#X} p({})", max_usable_address.as_u64(), (max_usable_address.as_u64() + PAGE_SIZE - 1) / PAGE_SIZE);
    com1_println!("Total System Memory: {:#X} p({})", max_physical_address.as_u64(), (max_physical_address.as_u64() + PAGE_SIZE - 1) / PAGE_SIZE);

    let mut kernel_base_address = VirtualAddress::new(MAX_VIRTUAL_ADDRESS);
    for asset in kernel_asset_list.iter() {
        if asset.virtual_address < kernel_base_address {
            kernel_base_address = asset.virtual_address;
        }
    }

    let firmware_page_table_manager = PageTableManager::new_from_cr3(0);
    let (mut page_table_manager, offset) = match init_page_table_manager(&mut allocator, max_physical_address, kernel_base_address) {
        Some(ptm) => ptm,
        None => {
            com1_println!("Memsize too large");
            return Err(efi::Status::ABORTED);
        }
    };
    com1_println!("Created page table");

    (*bootinfo).page_table_memory_offset = offset;

    unsafe {
        page_table_manager.activate_page_table();
        page_table_manager.set_offset(offset);
    }

    firmware_page_table_manager.release_tables(&mut allocator)
        .expect("Could not release firmware page table.");

    for asset in kernel_asset_list.iter() {
        com1_println!(
            "Mapping kernel asset. Phys: {:#X} -> {:#X}, Virt: {:#X} -> {:#X}", 
            asset.physical_address.as_u64(), asset.physical_address.increment_pages(asset.num_pages as u64).as_u64(),
            asset.virtual_address.as_u64(), asset.virtual_address.increment_pages(asset.num_pages as u64).as_u64(),
        );
        page_table_manager.map_memory_pages(asset.virtual_address, asset.physical_address, asset.num_pages as u64, &mut allocator)
            .expect("Could not map kernel virtual memory");
        let max_address = asset.virtual_address.increment_pages(asset.num_pages as u64);
        if max_address > bootinfo.next_availiable_kernel_page {
            bootinfo.next_availiable_kernel_page = max_address;
        }
    }

    // Map bootinfo into kernel space
    let bootinfo_virtual_address = bootinfo.next_availiable_kernel_page;
    let bootinfo_physical_address = PhysicalAddress::new(bootinfo as *mut BootInfo as u64);
    com1_println!("Mapping bootinfo from {:#X} to {:#X}", bootinfo_physical_address.as_u64(), bootinfo_virtual_address.as_u64());
    page_table_manager.map_memory_pages(bootinfo_virtual_address, bootinfo_physical_address, bootinfo_size_pages as u64, &mut allocator)
        .expect("Could not map boot info virtual memory");
    bootinfo.next_availiable_kernel_page = bootinfo_virtual_address.increment_pages(bootinfo_size_pages as u64);

    unsafe { page_table_manager.activate_page_table(); }

    // Update boot info pointer to point to the kernel mapped address
    let bootinfo = unsafe { &mut *(bootinfo_virtual_address.get_mut_ptr::<BootInfo>()) };

    if !bootinfo.has_valid_magic() { panic!("Could not correctly map bootinfo into kernel space. BootInfo Magic incorrect") }
    
    // Map allocator bitmap into kernel space
    let num_bitmap_pages = (allocator.page_bitmap().size() as u64 + PAGE_SIZE - 1) / PAGE_SIZE;
    let bitmap_buffer_physical_addr = PhysicalAddress::new(unsafe { allocator.page_bitmap().get_buffer() as u64 });
    let bitmap_buffer_virtual_addr = bootinfo.next_availiable_kernel_page;
    page_table_manager.map_memory_pages(bitmap_buffer_virtual_addr, bitmap_buffer_physical_addr, num_bitmap_pages, &mut allocator)
        .expect("Could not map allocator bitmap into virtual memory");
    bootinfo.next_availiable_kernel_page = bitmap_buffer_virtual_addr.increment_pages(num_bitmap_pages);

    unsafe { page_table_manager.activate_page_table(); }

    let output_bitmap = unsafe { bitmap::Bitmap::new(allocator.page_bitmap().size(), bitmap_buffer_virtual_addr.get_mut_ptr::<u8>()) };
    bootinfo.meminfo = MemInfo::new(output_bitmap, allocator.get_free_ram(), 0, allocator.get_used_ram(), max_physical_address);

    // Map font file into kernel space
    let font_file_virtual_addr = bootinfo.next_availiable_kernel_page;
    page_table_manager.map_memory_pages(font_file_virtual_addr, font_file_address, font_file_page_count as u64, &mut allocator)
        .expect("Could not map font file into virtual memory");
    bootinfo.next_availiable_kernel_page = font_file_virtual_addr.increment_pages(font_file_page_count as u64);
    bootinfo.font_file_address = font_file_virtual_addr;
    bootinfo.font_file_size = font_file_size;

    com1_println!("Starting Kernel");
    let kernel_start: unsafe extern "sysv64" fn(*mut BootInfo) = unsafe { core::mem::transmute(entry_point.get_mut_ptr::<c_void>()) };
    unsafe { kernel_start(bootinfo) };

    return Ok(())
}

fn load_kernel(image_handle: efi::Handle, system_table: uefi::SystemTableWrapper) -> Result<(LoadedAssetList, VirtualAddress), efi::Status> {
    let file_volume = system_table.boot_services().open_volume(image_handle)?;
    let kernel_file = file_volume.open_path(
        "kernel/kernel.elf", 
        efi::protocols::file::MODE_READ, 
        efi::protocols::file::READ_ONLY
    )?;
    com1_println!("Opened kernel file");

    let elf_common = kernel_file.read_struct::<elf::ElfHeaderCommon>()?;
    validate_elf(&elf_common)?;
    com1_println!("Kernel header verified successfully!");

    kernel_file.set_position(0)?;

    let elf_header = kernel_file.read_struct::<elf::ElfHeader64>()?;

    com1_println!("File has {} program sections", elf_header.e_phnum);

    let mut kernel_asset_list = LoadedAssetList::new(elf_header.e_phnum as usize, system_table)?;
    let mut kernel_section_list = ElfSectionList::new(elf_header.e_phnum as usize, system_table)?;
    for header_index in 0..elf_header.e_phnum {
        let entry_position = elf_header.e_phoff + (u64::from(header_index) * u64::from(elf_header.e_phentsize));
        kernel_file.set_position(entry_position)?;
        let program_header = kernel_file.read_struct::<elf::ElfPhysicalHeader64>()?;

        match program_header.p_type() {
            elf::ElfPhysicalType::Load => {
                kernel_section_list.add_section(&program_header);
                com1_println!("  Loadable section {}: \tms({:#X}), \tfs({:#X}), \tvaddr({:#X})", header_index, program_header.p_memsz, program_header.p_filesz, program_header.p_vaddr);
            },
            _ => {},
        }
    }

    for section in kernel_section_list.iter() {
        let section_buffer = system_table
            .boot_services()
            .allocate_pages
            ::<c_void>(r_efi::system::LOADER_DATA, section.num_mem_pages as usize)?;
        
        
        kernel_file.set_position(section.file_address.as_u64())?;
        let mut program_size = (section.num_file_pages * PAGE_SIZE) as usize;
        kernel_file.read(&mut program_size, section_buffer)?;
        kernel_asset_list.add_asset(PhysicalAddress::new(section_buffer as u64), section.num_mem_pages as usize, section.virtual_address);
        com1_println!("  Loaded section: \tvaddr({:#X}), \tmp({}), \tfp({})", section.virtual_address.as_u64(), section.num_mem_pages, section.num_file_pages);
    }

    Ok((kernel_asset_list, VirtualAddress::new(elf_header.e_entry)))
}

fn load_font(image_handle: efi::Handle, system_table: &uefi::SystemTableWrapper) -> Result<(PhysicalAddress, usize, usize), efi::Status>{
    com1_println!("Loading Font");
    let file_volume = system_table.boot_services().open_volume(image_handle)?;
    let font_file = file_volume.open_path(
        "kernel/fonts/ascii.psf", 
        efi::protocols::file::MODE_READ, 
        efi::protocols::file::READ_ONLY
    )?;

    let file_info = font_file.get_info(system_table)?;
    let page_count = ((file_info.file_size + PAGE_SIZE - 1) / PAGE_SIZE) as usize;
    com1_println!("Font File Size: {:#X}, p({:#X})", file_info.file_size, page_count);

    let pages = system_table.boot_services().allocate_pages(r_efi::system::LOADER_DATA, page_count)?;
    let mut buffer_size = page_count * PAGE_SIZE as usize;
    font_file.set_position(0)?;
    font_file.read(&mut buffer_size, pages)?;

    Ok((PhysicalAddress::new(pages as u64), page_count, file_info.file_size as usize))
}

fn init_page_table_manager(
    allocator: &mut PageFrameAllocator, 
    max_physical_address: PhysicalAddress, 
    kernel_base_address: VirtualAddress
) -> Option<(PageTableManager, u64)> {
    if max_physical_address.as_u64() > MAX_MEM_SIZE {
        return None;
    }

    let page_table_manager = PageTableManager::new_from_allocator(allocator, 0);

    // Identitiy map the entire range
    let num_mem_pages = max_physical_address.as_u64() / PAGE_SIZE;
    page_table_manager.map_memory_pages(VirtualAddress::new(0), PhysicalAddress::new(0), num_mem_pages, allocator)
        .expect("Could not map memory pages");

    // Size of address space set aside in GB
    let num_gb = (max_physical_address.as_u64() + MEM_1G - 1) / MEM_1G;

    let offset;
    if num_gb * MEM_1G < kernel_base_address.as_u64() {
        offset = kernel_base_address.as_u64() - num_gb * MEM_1G;
        page_table_manager.map_memory_pages(VirtualAddress::new(offset), PhysicalAddress::new(0), num_mem_pages, allocator)
            .expect("Could not map memory pages.");
    } else {
        offset = 0;
    }
    
    
    Some((page_table_manager, offset))
}

fn validate_elf(header: &elf::ElfHeaderCommon) -> Result<(), efi::Status> {
    if !header.has_valid_magic() {
        com1_println!("Invalid magic");
        return Err(efi::Status::LOAD_ERROR);
    }

    if header.class() != elf::ElfClass::Bits64 {
        com1_println!("Invalid class: {:?}", header.class());
        return Err(efi::Status::LOAD_ERROR);
    }

    if header.endianness() != elf::ElfEndianness::Little {
        com1_println!("Invalid endianness: {:?}", header.endianness());
        return Err(efi::Status::LOAD_ERROR);
    }

    if header.e_type() != elf::ElfType::Exec {
        com1_println!("Invalid type: {:?}", header.e_type());
        return Err(efi::Status::LOAD_ERROR);
    }

    if header.e_machine() != elf::ElfMachine::ElfMachineX8664 {
        com1_println!("Invalid machine: {:?}", header.e_machine());
        return Err(efi::Status::LOAD_ERROR);
    }

    if header.e_version() != elf::ElfVersion::Current {
        com1_println!("Invalid version: {:?}", header.e_version());
        return Err(efi::Status::LOAD_ERROR);
    }

    Ok(())
}

fn initialize_gop(system_table: uefi::SystemTableWrapper) -> Result<bootinfo::FrameBuffer, efi::Status>{
    let gop = match system_table.boot_services().get_graphics_output_protocol() {
        Ok(gop) => gop,
        Err(status) => {
            com1_println!("Cannot load GOP. Status: {status:#?}");
            return Err(status)
        }
    };
    com1_println!("Loaded GOP");

    return Ok(gop.get_framebuffer());
}