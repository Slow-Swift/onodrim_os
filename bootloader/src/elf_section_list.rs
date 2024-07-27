use core::cmp::{max, min};

use r_efi::efi;
use x86_64_hardware::memory::{PhysicalAddress, VirtualAddress, PAGE_OFFSET_MASK, PAGE_SIZE};

use crate::uefi;

#[derive(Copy, Clone)]
pub struct ElfSection {
    pub file_address: PhysicalAddress,
    pub virtual_address: VirtualAddress,
    pub num_file_pages: u64,
    pub num_mem_pages: u64,
}

impl ElfSection {
    pub fn get_mem_end(&self) -> VirtualAddress {
        self.virtual_address.increment_pages(self.num_mem_pages)
    }

    pub fn get_file_end(&self) -> PhysicalAddress {
        self.file_address.increment_pages(self.num_file_pages)
    }

    // Determines whether ther virtual address spaces of these two sections overlap
    pub fn has_virtual_overlap(&self, other: ElfSection) -> bool {
        (self.virtual_address <= other.virtual_address) && (self.get_mem_end() > other.virtual_address) ||
        (other.virtual_address <= self.virtual_address) && (other.get_mem_end() > self.virtual_address)
    }

    pub fn has_same_offset(&self, other: ElfSection) -> bool {
        if self.file_address.as_u64() < self.virtual_address.as_u64() {
            if other.file_address.as_u64() >= other.virtual_address.as_u64() { return false; }
            let self_offset = self.virtual_address.as_u64() - self.file_address.as_u64();
            let other_offset = other.virtual_address.as_u64() - other.file_address.as_u64();
            return self_offset == other_offset;
        } else {
            if other.file_address.as_u64() < other.virtual_address.as_u64() { return false; }
            let self_offset = self.file_address.as_u64() - self.virtual_address.as_u64();
            let other_offset = other.file_address.as_u64() - other.virtual_address.as_u64();
            return self_offset == other_offset;
        }
    }

    pub fn combine(&mut self, other: ElfSection) -> bool {
        if !self.has_virtual_overlap(other) { return false }
        if !self.has_same_offset(other) { return false; } // TODO: Should error
        
        let min_file_address = min(self.file_address, other.file_address);
        let max_file_address = max(self.get_file_end(), other.get_file_end());
        let min_virtual_address = min(self.virtual_address, other.virtual_address);
        let max_virtual_address = max(self.get_mem_end(), other.get_mem_end());

        let file_pages = (max_file_address.as_u64() - min_file_address.as_u64()) / PAGE_SIZE;
        let memory_pages = (max_virtual_address.as_u64() - min_virtual_address.as_u64()) / PAGE_SIZE;

        self.file_address = min_file_address;
        self.virtual_address = min_virtual_address;
        self.num_file_pages = file_pages;
        self.num_mem_pages = memory_pages;

        return true;
    }
}

pub struct ElfSectionList {
    list_ptr: *mut ElfSection,
    num_pages: usize,
    num_items: usize,
}

impl ElfSectionList {
    pub fn new(item_count: usize, system_table: uefi::SystemTableWrapper) -> Result<ElfSectionList, efi::Status> {
        let min_mem_size = size_of::<ElfSection>() * item_count;
        let num_pages = (min_mem_size + PAGE_SIZE as usize - 1) / PAGE_SIZE as usize;
        let list_ptr = system_table.boot_services().allocate_pages::<ElfSection>(r_efi::system::LOADER_DATA, num_pages)?;

        return Ok(ElfSectionList {
            list_ptr,
            num_pages,
            num_items: 0,
        })
    }

    pub fn max_items(&self) -> usize {
        return (PAGE_SIZE as usize * self.num_pages) / size_of::<ElfSectionList>();
    }

    pub fn add_section(&mut self, section_header: &elf::ElfPhysicalHeader64) {        
        // Expand program section sizes to nearest pages
        let page_offset = section_header.p_offset & PAGE_OFFSET_MASK;
        let num_file_pages = (section_header.p_filesz + page_offset + PAGE_SIZE - 1) / PAGE_SIZE;
        let num_mem_pages = (section_header.p_memsz + page_offset + PAGE_SIZE - 1) / PAGE_SIZE;

        let section = ElfSection {
            file_address: PhysicalAddress::new(section_header.p_offset),
            virtual_address: VirtualAddress::new(section_header.p_vaddr & !PAGE_OFFSET_MASK),
            num_file_pages,
            num_mem_pages,
        };

        for i in 0..self.num_items {
            let mut other_section = self.get_section(i)
                .expect("Should be impossible since the index is valid");
            if other_section.combine(section) { 
                self.set_section(other_section, i);
                return; 
            }
        }

        if self.max_items() == self.num_items { return; } // Error: Could not add
        self.set_section(section, self.num_items);

        for i in (0..self.num_items).rev() {
            let other_section = self.get_section(i).expect("Should be impossible since the index is valid");
            if other_section.virtual_address < section.virtual_address { break; }
            self.set_section(self.get_section(i).expect("Should be impossible since the index is valid"), i+1);
            self.set_section(section, i);
        }
        self.num_items += 1;
    }

    pub fn get_section(&self, index: usize) -> Option<ElfSection> {
        if index >= self.num_items {
            return None;
        }

        unsafe { return Some(*(self.list_ptr.offset(index as isize))) }
    }

    fn set_section(&self, section: ElfSection, index: usize) {
        unsafe { *(self.list_ptr.offset(index as isize)) = section };
    }

    pub fn iter(&self) -> ElfSectionListIterator {
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