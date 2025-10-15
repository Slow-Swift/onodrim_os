use r_efi::efi;
use x86_64_hardware::memory::{PhysicalAddress, VirtualAddress, PAGE_SIZE};

use crate::uefi::BootServices;


/// Represents an asset loaded for an ELF program
#[derive(Copy, Clone)]
pub struct LoadedAsset {
    pub physical_address: PhysicalAddress,
    pub num_pages: usize,
    pub virtual_address: VirtualAddress,
}

/// Holds a list of assets loaded for en ELF program
pub struct LoadedAssetList {
    list_ptr: *mut LoadedAsset,
    num_pages: usize,
    num_items: usize,
}

impl LoadedAsset {
    pub fn new(physical_address: PhysicalAddress, num_pages: usize, virtual_address: VirtualAddress) -> LoadedAsset {
        LoadedAsset { physical_address, num_pages, virtual_address }
    }
}

impl LoadedAssetList {
    /// Create a new empty list of assets capable of holding at least [max_items]
    pub fn new(max_items: usize, boot_services: &BootServices) -> Result<LoadedAssetList, efi::Status> {
        let min_mem_size = size_of::<LoadedAsset>() * max_items;
        let num_pages = (min_mem_size + PAGE_SIZE as usize - 1) / PAGE_SIZE as usize;
        let list_ptr = boot_services.allocate_pages::<LoadedAsset>(
            r_efi::system::LOADER_DATA, num_pages
        )?;

        return Ok(LoadedAssetList {
            list_ptr,
            num_pages,
            num_items: 0,
        })
    }

    /// Get the maximum number of items that can be stored in this list
    pub fn max_items(&self) -> usize {
        return (PAGE_SIZE as usize * self.num_pages) / size_of::<LoadedAsset>();
    }

    /// Add an asset to the list
    /// 
    /// Returns None if the list is already full
    pub fn add_asset(&mut self, asset: LoadedAsset) -> Option<usize> {
        // Ensure the item can be added to the list
        if self.max_items() == self.num_items { return None; }

        let index = self.num_items;

        // Safety: This should be safe because the memory has been reserved and we checked that it fits
        unsafe {
            *(self.list_ptr.offset(index as isize)) = asset;
        }
        self.num_items += 1;

        return Some(index);
    }

    /// Get the asset at position [index] from the list
    /// 
    /// Returns None if the provided index is bigger than the highest index in the list
    pub fn get_asset(&self, index: usize) -> Option<LoadedAsset> {
        if index >= self.num_items {
            return None;
        }

        // Safety: This should be safe because we checked that the index is valid
        //         which means that an asset has already been stored at the given location
        unsafe { 
            return Some(*(self.list_ptr.offset(index as isize)));
        }
    }

    /// Iterate over the assets in the list
    pub fn iter(&'_ self) -> LoadedAssetListIterator<'_> {
        LoadedAssetListIterator {
            asset_list: self,
            current_index: 0,
            max_index: self.num_items
        }
    }
}

/// An iterator for a LoadedAssetList
/// 
/// Lifetime: The LoadedAssetList must have 
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