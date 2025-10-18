use core::ffi::c_void;

use r_efi::efi;
use x86_64_hardware::{com1_println, memory::{PhysicalAddress, VirtualAddress, PAGE_SIZE}};

use crate::{elf_section_list::ElfSectionList, loaded_asset_list::{LoadedAsset, LoadedAssetList}, uefi::{file_protocol::FileProtocol, BootServices}};

/// Load the kernel into memory
/// 
/// Finds the kernel located at kernel/kernel.elf and loads it into memory
/// Returns a list of loaded program sections that later need to memory mapped
pub fn load_kernel(
    image_handle: efi::Handle, boot_services: &BootServices
) -> Result<(LoadedAssetList, VirtualAddress), efi::Status> {
    // Open the kernel file
    let kernel_file = boot_services.open_file(image_handle, "kernel/kernel.elf")?;
    com1_println!("Opened kernel file");

    let elf_common = kernel_file.read_struct::<elf::ElfHeaderCommon>()?;
    validate_kernel_elf(&elf_common)?;
    com1_println!("Kernel header verified successfully!");

    // Read the ELF header to get information about the ELF file
    kernel_file.set_position(0)?;
    let elf_header = kernel_file.read_struct::<elf::ElfHeader64>()?;

    let section_list = get_kernel_sections(
        &kernel_file, &elf_header, boot_services
    )?;

    let asset_list = load_kernel_sections(
        &kernel_file, &elf_header, &section_list, boot_services
    )?;

    Ok((asset_list, VirtualAddress::new(elf_header.e_entry)))
}

/// Get a list of all program sections that need to be loaded
/// 
/// All loadable sections are collected into an ElfSectionList and are sorted with
/// overlapping and adjacient sections being merged together
fn get_kernel_sections(
    kernel_file: &FileProtocol, elf_header: &elf::ElfHeader64, boot_services: &BootServices
) -> Result<ElfSectionList, efi::Status> {
    com1_println!("File has {} program sections", elf_header.e_phnum);

    // Read all program headers and collect loadable ones into the list
    let mut section_list = ElfSectionList::new(elf_header.e_phnum as usize, &boot_services)?;
    for header_index in 0..elf_header.e_phnum {
        let entry_position = elf_header.e_phoff + (u64::from(header_index) * u64::from(elf_header.e_phentsize));
        kernel_file.set_position(entry_position)?;
        let program_header = kernel_file.read_struct::<elf::ElfPhysicalHeader64>()?;
        
        // Add loadable sections to the list
        match program_header.p_type() {
            elf::ElfPhysicalType::Load => {
                section_list.add_section(&program_header)
                    .expect("This should not happen because there should be enough space in the ELF Section List");
                com1_println!(
                    "  Loadable section {}: \tms({:#X}), \tfs({:#X}), \tvaddr({:#X}) \tfaddr({:#X})", 
                    header_index, program_header.p_memsz, program_header.p_filesz, program_header.p_vaddr, program_header.p_offset
                );
            },
            _ => {},
        }
    }
    // Merge overlapping sections
    section_list.merge_sections();
    com1_println!("Merged down to {} sections", section_list.size());

    Ok(section_list)
}

/// Load a list of sections from the ELF file into memory
/// 
/// The list of sections should already be merged
fn load_kernel_sections(
    kernel_file: &FileProtocol, 
    elf_header: &elf::ElfHeader64, 
    section_list: &ElfSectionList,
    boot_services: &BootServices
) -> Result<LoadedAssetList, efi::Status> {
    let mut kernel_asset_list = LoadedAssetList::new(elf_header.e_phnum as usize, &boot_services)?;
    
    // Load each section in the section_list
    for section in section_list.iter() {
        // Allocate pages for the section
        let section_buffer = boot_services.allocate_pages::<c_void>(
            r_efi::system::LOADER_DATA, 
            section.num_mem_pages as usize
        )?;
        
        // Read the program section into memory
        kernel_file.set_position(section.file_address.as_u64())?;
        let mut program_size = (section.num_file_pages * PAGE_SIZE) as usize;
        kernel_file.read(&mut program_size, section_buffer)?;

        // Add the program section to the list of loaded assets
        kernel_asset_list.add_asset(
            LoadedAsset::new(
                PhysicalAddress::new(section_buffer as u64), 
                section.num_mem_pages as usize, 
                section.virtual_address
            )
        );
        com1_println!("  Loaded section: \tvaddr({:#X}), \tmp({}), \tfp({})", section.virtual_address.as_u64(), section.num_mem_pages, section.num_file_pages);
    }

    Ok(kernel_asset_list)
}

/// Check that the kernel ELF file is built for the correct system
fn validate_kernel_elf(header: &elf::ElfHeaderCommon) -> Result<(), efi::Status> {
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
