use r_efi::efi;
use x86_64_hardware::memory::{PhysicalAddress, VirtualAddress, PAGE_SIZE};

use crate::uefi;

#[derive(Copy, Clone)]
pub struct LoadedAsset {
    pub physical_address: PhysicalAddress,
    pub num_pages: usize,
    pub virtual_address: VirtualAddress,
}

pub struct LoadedAssetList {
    list_ptr: *mut LoadedAsset,
    num_pages: usize,
    num_items: usize,
}

impl LoadedAssetList {
    pub fn new(item_count: usize, system_table: uefi::SystemTableWrapper) -> Result<LoadedAssetList, efi::Status> {
        let min_mem_size = size_of::<LoadedAsset>() * item_count;
        let num_pages = (min_mem_size + PAGE_SIZE as usize - 1) / PAGE_SIZE as usize;
        let list_ptr = system_table.boot_services().allocate_pages::<LoadedAsset>(r_efi::system::LOADER_DATA, num_pages)?;

        return Ok(LoadedAssetList {
            list_ptr,
            num_pages,
            num_items: 0,
        })
    }

    pub fn max_items(&self) -> usize {
        return (PAGE_SIZE as usize * self.num_pages) / size_of::<LoadedAsset>();
    }

    pub fn add_asset(&mut self, physical_address: PhysicalAddress, num_pages: usize, virtual_address: VirtualAddress) -> Option<usize> {
        if self.max_items() == self.num_items { return None; }

        let index = self.num_items;

        unsafe {
            *(self.list_ptr.offset(index as isize)) = LoadedAsset {
                physical_address,
                num_pages,
                virtual_address
            };
        }
        self.num_items += 1;

        return Some(index);
    }

    pub fn get_asset(&self, index: usize) -> Option<LoadedAsset> {
        if index >= self.num_items {
            return None;
        }

        unsafe { return Some(*(self.list_ptr.offset(index as isize))) }
    }

    pub fn iter(&self) -> LoadedAssetListIterator {
        LoadedAssetListIterator {
            asset_list: self,
            current_index: 0,
            max_index: self.num_items
        }
    }
}

pub struct LoadedAssetListIterator<'a> {
    asset_list: &'a LoadedAssetList,
    current_index: usize,
    max_index: usize,
}

impl<'a> Iterator for LoadedAssetListIterator<'a> {
    type Item = LoadedAsset;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index == self.max_index {
            return None;
        } else {
            let output = self.asset_list.get_asset(self.current_index);
            if output.is_some() {
                self.current_index += 1;
            }
            return output;
        }
    }
}