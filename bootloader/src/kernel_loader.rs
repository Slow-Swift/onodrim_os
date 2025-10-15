use core::ffi::c_void;

use r_efi::efi;
use x86_64_hardware::{com1_println, memory::{PhysicalAddress, VirtualAddress, PAGE_SIZE}};

use crate::{elf_section_list::ElfSectionList, loaded_asset_list::{LoadedAsset, LoadedAssetList}, uefi::{BootServices}};

pub fn load_kernel(image_handle: efi::Handle, boot_services: &BootServices) -> Result<(LoadedAssetList, VirtualAddress), efi::Status> {
    // Open the kernel file
    let file_volume = boot_services.open_volume(image_handle)?;
    let kernel_file = file_volume.open_path(
        "kernel/kernel.elf", 
        efi::protocols::file::MODE_READ, 
        efi::protocols::file::READ_ONLY
    )?;
    com1_println!("Opened kernel file");

    let elf_common = kernel_file.read_struct::<elf::ElfHeaderCommon>()?;
    validate_kernel_elf(&elf_common)?;
    com1_println!("Kernel header verified successfully!");

    kernel_file.set_position(0)?;

    let elf_header = kernel_file.read_struct::<elf::ElfHeader64>()?;

    com1_println!("File has {} program sections", elf_header.e_phnum);

    let mut kernel_asset_list = LoadedAssetList::new(elf_header.e_phnum as usize, &boot_services)?;
    let mut kernel_section_list = ElfSectionList::new(elf_header.e_phnum as usize, &boot_services)?;
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
        let section_buffer = boot_services.allocate_pages::<c_void>(
            r_efi::system::LOADER_DATA, 
            section.num_mem_pages as usize
        )?;
        
        kernel_file.set_position(section.file_address.as_u64())?;
        let mut program_size = (section.num_file_pages * PAGE_SIZE) as usize;
        kernel_file.read(&mut program_size, section_buffer)?;
        kernel_asset_list.add_asset(
            LoadedAsset::new(
                PhysicalAddress::new(section_buffer as u64), 
                section.num_mem_pages as usize, 
                section.virtual_address
            )
        );
        com1_println!("  Loaded section: \tvaddr({:#X}), \tmp({}), \tfp({})", section.virtual_address.as_u64(), section.num_mem_pages, section.num_file_pages);
    }

    Ok((kernel_asset_list, VirtualAddress::new(elf_header.e_entry)))
}

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
