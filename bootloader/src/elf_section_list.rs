use core::cmp::{max, min};

use r_efi::efi;
use x86_64_hardware::memory::{PhysicalAddress, VirtualAddress, PAGE_OFFSET_MASK, PAGE_SIZE};

use crate::uefi::{BootServices};

#[derive(Copy, Clone)]
pub struct ElfSection {
    pub file_address: PhysicalAddress,
    pub virtual_address: VirtualAddress,
    pub num_file_pages: u64,
    pub num_mem_pages: u64,
}

impl ElfSection {
    pub fn new(file_address: u64, virtual_address: u64, file_size: u64, memory_size: u64) -> ElfSection {
        let page_offset = file_address & PAGE_OFFSET_MASK;
        let num_file_pages = (file_size + page_offset + PAGE_SIZE - 1) / PAGE_SIZE;
        let num_mem_pages = (memory_size + page_offset + PAGE_SIZE - 1) / PAGE_SIZE;

        ElfSection {
            file_address: PhysicalAddress::new(file_address),
            // The reason virtual address is page aligned is that file_address is page aligned. Why are all PhysicalAddresses page aligned??? I don't know???
            virtual_address: VirtualAddress::new(virtual_address & !PAGE_OFFSET_MASK),
            num_file_pages,
            num_mem_pages,
        }
    }

    pub fn get_mem_end(&self) -> VirtualAddress {
        self.virtual_address.increment_pages(self.num_mem_pages)
    }

    pub fn get_file_end(&self) -> PhysicalAddress {
        self.file_address.increment_pages(self.num_file_pages)
    }

    /// Determines whether the virtual address spaces of these two sections overlap
    pub fn has_virtual_overlap(&self, other: ElfSection) -> bool {
        (self.virtual_address <= other.virtual_address) && (self.get_mem_end() > other.virtual_address) ||
        (other.virtual_address <= self.virtual_address) && (other.get_mem_end() > self.virtual_address)
    }

    /// Determine whether the offset from file to virtual address is the same for two sections
    pub fn has_same_offset(&self, other: ElfSection) -> bool {
        // Collect the values in variables for clarity and brevity
        let self_file_addr = self.file_address.as_u64();
        let self_virt_addr = self.virtual_address.as_u64();
        let other_file_addr = other.file_address.as_u64();
        let other_virt_addr = other.virtual_address.as_u64();

        // Make sure the absolute difference between the addresses are the same and check that they differ in the same direction
        let same_direction = (self_file_addr <= self_virt_addr) == (other_file_addr <= other_virt_addr);
        let offset_equal = self_virt_addr.abs_diff(self_file_addr) == other_virt_addr.abs_diff(other_file_addr);
        return same_direction && offset_equal;
    }

    /// Combine two ELF sections if they overlap or are contiguous in virtual space and in file space.
    /// 
    /// If it is possible to combine these sections, a new ElfSection which is the combination of both
    /// old ones is returned.
    /// If it is not possible, None is returned
    pub fn combine_if_possible(&self, other: ElfSection) -> Option<ElfSection> {
        // If these two sections overlap in virtual memory then we will try to merge them
        if !self.has_virtual_overlap(other) { return None }

        // In order to merge them they must be contiguous in the file. We can check this by checking
        // that they have the same offset between virtual and file address.
        // I think it is possible in theory for sections to overlap in virtual space but not be contiguous
        // in the file. According to the ELF specs the page alignment should be consistent but it is theoretically
        // possible for them to live in different physical pages. 
        // For now I will keep it like this and just error if they aren't contiguous.
        if !self.has_same_offset(other) { panic!("The ELF file has an unexpected format with misaligned virtual and file addressed"); } 
        
        // Get the range for the merged section
        let min_file_address = min(self.file_address, other.file_address);
        let max_file_address = max(self.get_file_end(), other.get_file_end());
        let min_virtual_address = min(self.virtual_address, other.virtual_address);
        let max_virtual_address = max(self.get_mem_end(), other.get_mem_end());

        return Some(ElfSection::new(
            min_file_address.as_u64(),
            min_virtual_address.as_u64(),
            max_file_address.as_u64() - min_file_address.as_u64(),
            max_virtual_address.as_u64() - min_virtual_address.as_u64()
        ));
    }
}

pub struct ElfSectionList {
    list_ptr: *mut ElfSection,
    num_pages: usize,
    num_items: usize,
}

impl ElfSectionList {
    /// Create a new list capable of storing at least [item_count] ElfSections
    pub fn new(item_count: usize, boot_services: &BootServices) -> Result<ElfSectionList, efi::Status> {
        let min_mem_size = size_of::<ElfSection>() * item_count;
        let num_pages = (min_mem_size + PAGE_SIZE as usize - 1) / PAGE_SIZE as usize;
        let list_ptr = boot_services.allocate_pages::<ElfSection>(r_efi::system::LOADER_DATA, num_pages)?;

        return Ok(ElfSectionList {
            list_ptr,
            num_pages,
            num_items: 0,
        })
    }

    /// Get the maximum number items that can be stored in the list
    pub fn max_items(&self) -> usize {
        return (PAGE_SIZE as usize * self.num_pages) / size_of::<ElfSectionList>();
    }

    /// Get the current number of items stored in the list
    pub fn size(&self) -> usize {
        return self.num_items;
    }

    /// Add an ElfSection to the list, ensuring the list remains sorted by virtual address
    pub fn add_section(&mut self, section_header: &elf::ElfPhysicalHeader64) -> Result<(), efi::Status>{      
        if self.max_items() == self.num_items { return Err(efi::Status::BUFFER_TOO_SMALL) } // Error: Could not add
        let section = ElfSection::new(
            section_header.p_offset,
            section_header.p_vaddr,
            section_header.p_filesz,
            section_header.p_memsz
        );
        self.set_section(section, self.num_items);

        for i in (0..self.num_items).rev() {
            let other_section = self.get_section(i).expect("Should be impossible since the index is valid");
            if other_section.virtual_address < section.virtual_address { break; }
            self.set_section(self.get_section(i).expect("Should be impossible since the index is valid"), i+1);
            self.set_section(section, i);
        }
        self.num_items += 1;

        Ok(())
    }

    /// Merge all ELF sections that can be merged
    pub fn merge_sections(&mut self) {
        let mut last_section_index = 0;
        for i in 1..self.num_items {
            let last_section = self.get_section(last_section_index)
                .expect("Should be impossible since the index is valid");
            let current_section = self.get_section(i)
                .expect("Should be impossible since the index is valid");

            match last_section.combine_if_possible(current_section) { 
                Some(combined) => self.set_section(combined, last_section_index),
                None => {
                    last_section_index += 1;
                    self.set_section(current_section, last_section_index);
                }
            }
        }
        self.num_items = last_section_index + 1;
    }

    /// Get the section stored at the given index if it exists
    pub fn get_section(&self, index: usize) -> Option<ElfSection> {
        if index >= self.num_items {
            return None;
        }

        unsafe { return Some(*(self.list_ptr.offset(index as isize))) }
    }

    /// Set the section at the given index
    /// If the index given is too large, the code panics
    fn set_section(&self, section: ElfSection, index: usize) {
        if index >= self.max_items() { panic!("Not enough space in ELF List"); }

        // Safety: Should be safe because we checked to make sure the index is valid
        unsafe { *(self.list_ptr.offset(index as isize)) = section };
    }

    pub fn iter(&'_ self) -> ElfSectionListIterator<'_> {
        ElfSectionListIterator {
            asset_list: self,
            current_index: 0,
            max_index: self.num_items
        }
    }
}

pub struct ElfSectionListIterator<'a> {
    asset_list: &'a ElfSectionList,
    current_index: usize,
    max_index: usize,
}

impl<'a> Iterator for ElfSectionListIterator<'a> {
    type Item = ElfSection;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index == self.max_index {
            return None;
        } else {
            let output = self.asset_list.get_section(self.current_index);
            if output.is_some() {
                self.current_index += 1;
            }
            return output;
        }
    }
}