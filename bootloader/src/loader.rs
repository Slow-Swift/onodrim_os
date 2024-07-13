use core::u64::MAX;

use uefi::{data_types::PhysicalAddress, print, println, proto::media::file::{Directory, File, FileAttribute, FileInfo, FileMode, RegularFile}, table::boot::{AllocateType, BootServices, MemoryType}, CStr16, Error};

use crate::elf::{ElfClass, ElfHeader, ElfIdentifier, ElfInstructionSet, Endianness, ProgramHeader, ProgramHeaderSectionType};

pub fn load_kernel(boot_services: &BootServices, root: Directory, base_address: u64) -> u64 {
    println!("Loading Kernel...");
    let mut kernel_file = open_file(root, "kernel/kernel.elf").expect("Could not open kernel file.");
    println!("  -> found kernel file");

    let header = unsafe { read_and_allocate::<ElfHeader>(boot_services, &mut kernel_file) };

    if !validate_elf_identifier(&header.identifier)  || !validate_elf_header(&header) {
        println!("Could not validate kernel file.");
        return 0;
    }
    
    println!("  -> validated kernel file");
    println!("  -> elf version: {}", header.elf_version);
    println!("  -> type: {}", header.elf_type);
    
    println!("  -> found entry point {:#X}", header.entry_offset);
    
    
    assert!( header.program_header_entry_size as usize == core::mem::size_of::<ProgramHeader>(), "Invalid Program Header Size");
    
    kernel_file.set_position(header.program_header_table_offset).expect("Could not seek to program headers");
    let program_headers = unsafe { 
        read_and_allocate_array::<ProgramHeader>(boot_services, header.program_header_entry_count as usize, &mut kernel_file) 
    };
    
    println!("  -> read {} program headers", header.program_header_entry_count);

    let entry_point = load_program_segments(boot_services, &mut kernel_file, header, program_headers, base_address);
    
    unsafe { boot_services.free_pool(header as *mut ElfHeader as *mut u8).expect("Could not free memory"); }
    unsafe { boot_services.free_pool(program_headers as *mut [ProgramHeader] as *mut u8).expect("Could not free memory"); }

    return entry_point;
}

pub fn load_font(boot_services: &BootServices, root: Directory) -> (u64, usize) {
    // Allocations: info_buffer, file_pages

    println!("Loading font...");

    // Open the font file
    let mut font_file = open_file(root, "kernel/fonts/ascii.psf").expect("Could not open font file.");
    
    // Determine the size of the file
    let mut info_buffer = allocate_u8_ref(boot_services, 128);
    let info: &FileInfo = match font_file.get_info(info_buffer) {
        Ok(info) => info,
        Err(size) => {
            unsafe { boot_services.free_pool(info_buffer.as_mut_ptr()).expect("Could not free memory"); }
            info_buffer = allocate_u8_ref(boot_services, size.data().expect("Could not get file info size."));
            font_file.get_info(info_buffer).expect("Could not get file info")
        }
    };
    let file_size = info.file_size();
    let page_count = size_to_pages(file_size);
    println!("Font File Size: {file_size} p({page_count})");

    // Allocate pages for the font file
    let file_pages: PhysicalAddress = boot_services.allocate_pages(
        AllocateType::AnyPages, 
        MemoryType::LOADER_DATA, 
        page_count as usize
    )
        .expect("Could not allocate pages");
    let file_buffer = unsafe { core::slice::from_raw_parts_mut(file_pages as *mut u8, file_size as usize) };

    // Read the font file
    font_file.read(file_buffer).expect("Could not read file");

    // Frees: buffer
    unsafe { boot_services.free_pool(info_buffer.as_mut_ptr()).expect("Could not free memory"); }

    return (file_pages, file_size as usize);

    // Does not free: file_pages (this is needed by the kernel)
}

fn open_file(mut root: Directory, path: &str) -> Result<RegularFile, Error>{
    let mut path_parts = path.split('/');
    let mut path_buffer = [0; 64];

    // Open the first level of the path
    let first = path_parts.next().expect("Kernel path was empty.");
    let part = CStr16::from_str_with_buf(first, &mut path_buffer)
        .expect("Could not convert path to CStr16");
    let mut file_handle = root.open(part, FileMode::Read, FileAttribute::READ_ONLY)?;

    // Open each part of the path
    for part in path_parts {
        if file_handle.is_regular_file().unwrap() {
            file_handle.close();
            panic!("File is not a directory");
        }

        let part = 
            CStr16::from_str_with_buf(part, &mut path_buffer)
            .expect("Could not convert path to CStr16");
        let new_file_handle = file_handle.open(part, FileMode::Read, FileAttribute::READ_ONLY)?;
        file_handle.close();
        file_handle = new_file_handle;
    }

    if !file_handle.is_regular_file()? {
        panic!("The file is not a regular file!");
    }

    return Ok(file_handle.into_regular_file().unwrap());
}

fn validate_elf_identifier(identifier: &ElfIdentifier) -> bool {
    if identifier.magic != [0x7F, 0x45, 0x4C, 0x46] {
        println!("ELF Magic is invalid");
        return false;
    }

    if identifier.class != ElfClass::Bit64 as u8 {
        println!("ELF is not 64-Bit");
        return false;
    }

    if identifier.endianness != Endianness::Little as u8 {
        println!("ELF is not in Little Endian Format");
        return false;
    }

    return true;
}

fn validate_elf_header(header: &ElfHeader)  -> bool {
    if header.instruction_set != ElfInstructionSet::X86_64 as u16 {
        println!("ELF Instruction Set is invalid");
        return false;
    }
    return true;
}

fn allocate_u8_ref(boot_services: &BootServices, size: usize)  -> &mut [u8] {
    let buffer = boot_services.allocate_pool(MemoryType::BOOT_SERVICES_DATA, size)
        .expect("Could not allocate Memory.");
    return unsafe {
        core::slice::from_raw_parts_mut(buffer.as_ptr(), size)
    };
}

///
/// Read from a file and return a struct reference to that file
/// 
/// SAFETY
/// The struct must eventually be freed
/// 
/// The data read into the struct must be valid or the results will be undefined.
/// 
unsafe fn read_and_allocate<'a, T: Sized>(boot_services: &BootServices, file: &mut RegularFile) -> &'a mut T{ 
    let size = core::mem::size_of::<T>();
    let buffer = boot_services.allocate_pool(MemoryType::BOOT_SERVICES_DATA, size)
        .expect("Could not allocate Memory.");
    let buffer = core::slice::from_raw_parts_mut(
        buffer.as_ptr(), 
        size
    );

    let bytes_read = file.read(buffer).unwrap();
    assert!(bytes_read == size, "Attempted to read {} bytes, only read {}", size, bytes_read);

    return &mut *buffer.as_mut_ptr().cast::<T>();
}

unsafe fn read_and_allocate_array<'a, T: Sized>(boot_services: &'a BootServices, array_length: usize, file: &mut RegularFile) -> &'a mut [T] {
    let size = array_length * core::mem::size_of::<T>();
    let buffer = boot_services.allocate_pool(MemoryType::BOOT_SERVICES_DATA, size)
        .expect("Could not allocate Memory.");
    let buffer = core::slice::from_raw_parts_mut(
        buffer.as_ptr(), 
        size
    );
    let bytes_read = file.read(buffer).unwrap();
    assert!(bytes_read == size, "Attempted to read {} bytes, only read {}", size, bytes_read);


    core::slice::from_raw_parts_mut(buffer.as_ptr() as *mut T, array_length)
}

fn load_program_segments(boot_services: &BootServices, file: &mut RegularFile, header: &ElfHeader, program_headers: &[ProgramHeader], base_address: u64) -> u64 {
    if header.program_header_entry_count == 0 {
        println!("  -> no program segments found!");
        return 0;
    }

    
    let mut start_addr: u64 = MAX;
    let mut end_addr: u64 = 0;
    
    for program_header in program_headers {
        if program_header.segment_type != ProgramHeaderSectionType::Load as u32 { continue; }

        let start = program_header.virtual_address;
        let end = start + program_header.mem_size;
        if start < start_addr { start_addr = start };
        if end > end_addr { end_addr = end };
    }
    
    let total_size = end_addr - start_addr;
    let page_count = size_to_pages(total_size);
    println!("  -> total size in memory: {:#x} ({}) pages", total_size, page_count);
    
    let buffer: PhysicalAddress = boot_services.allocate_pages(AllocateType::Address(base_address), MemoryType::LOADER_DATA, page_count as usize)
        .expect("Could not allocate pages");
    let buffer = unsafe { core::slice::from_raw_parts_mut(buffer as *mut u8, total_size as usize) };

    let mut loaded_count = 0;

    println!("  -> loading {} program segments", header.program_header_entry_count);

    for (i, program_header) in program_headers.iter().enumerate() {
        if program_header.segment_type != ProgramHeaderSectionType::Load as u32 { continue; }

        let mem_size = program_header.mem_size;
        let virtual_address = program_header.virtual_address - start_addr + base_address;
        let address_offset = (program_header.virtual_address - start_addr) as usize;
        let address_offset_end = address_offset + program_header.file_size as usize;
        print!("     [{} m({:#X}) va({:#X})]", i, mem_size, virtual_address);

        file.set_position(program_header.data_offset).expect("Could not seek file.");
        file.read(&mut buffer[address_offset..address_offset_end]).expect("Could not read segment.");


        println!("]");
        loaded_count += 1;
    }

    println!("  -> loaded {loaded_count} sections.");

    return header.entry_offset - start_addr + base_address;
    
}

fn size_to_pages(bytes: u64) -> u64 {
    if (bytes & 0xFFF) > 0 {
        return (bytes >> 12) + 1;
    } else {
        return bytes >> 12;
    }
}