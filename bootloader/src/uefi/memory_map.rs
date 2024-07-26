use core::{ffi::c_void, ptr::null_mut};

use x86_64_hardware::{com1_println, memory::{AllocError, PageFrameAllocator, PhysicalAddress, PAGE_SIZE}};

#[derive(PartialEq, Debug)]
pub enum DescriptorType {
    EfiReservedMemoryType,
    EfiLoaderCode,
    EfiLoaderData,
    EfiBootServicesCode,
    EfiBootServicesData,
    EfiRuntimeServicesCode,
    EfiRuntimeServicesData,
    EfiConventionalMemory,
    EfiUnusableMemory,
    EfiACPIReclaimableMemory,
    EfiACPIMemoryNVS,
    EfiMemoryMappedIO,
    EfiMemoryMappedIOPortSpace,
    EfiPalCode,
    EfiPersistentMemory,
    EfiUnkown,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct EfiMemoryDescriptor {
    pub mem_type: u32,
    pub phys_addr: PhysicalAddress,
    pub virt_addr: *mut c_void,
    pub num_pages: u64,
    pub attribs: u64,
}

impl EfiMemoryDescriptor {
    pub fn num_bytes(&self) -> u64 {
        self.num_pages * PAGE_SIZE
    }

    pub fn max_physical_address(&self) -> PhysicalAddress {
        self.phys_addr.increment_pages(self.num_pages)
    }

    fn is_usable_memory(&self) -> bool {
        match self.mem_type() {
            DescriptorType::EfiLoaderCode |
            DescriptorType::EfiLoaderData |
            DescriptorType::EfiBootServicesCode |
            DescriptorType::EfiBootServicesData |
            DescriptorType::EfiConventionalMemory |
            DescriptorType::EfiACPIReclaimableMemory => true,
            _ => false,
        }
    }

    pub fn mem_type(&self) -> DescriptorType {
        match self.mem_type {
            0 => DescriptorType::EfiReservedMemoryType,
            1 => DescriptorType::EfiLoaderCode,
            2 => DescriptorType::EfiLoaderData,
            3 => DescriptorType::EfiBootServicesCode,
            4 => DescriptorType::EfiBootServicesData,
            5 => DescriptorType::EfiRuntimeServicesCode,
            6 => DescriptorType::EfiRuntimeServicesData,
            7 => DescriptorType::EfiConventionalMemory,
            8 => DescriptorType::EfiUnusableMemory,
            9 => DescriptorType::EfiACPIReclaimableMemory,
            10 => DescriptorType::EfiACPIMemoryNVS,
            11 => DescriptorType::EfiMemoryMappedIO,
            12 => DescriptorType::EfiMemoryMappedIOPortSpace,
            13 => DescriptorType::EfiPalCode,
            14 => DescriptorType::EfiPersistentMemory,
            _ => DescriptorType::EfiUnkown,
        }
    }
}

#[repr(C)]
pub struct EfiMemoryMap {
    pub descriptors: *mut EfiMemoryDescriptor,
    pub map_size: usize,
    pub descriptor_size: usize,
    pub num_pages: usize,
}

impl EfiMemoryMap {
    pub fn get_descriptor(&self, index: usize) -> Result<EfiMemoryDescriptor, ()> {
        if index > self.entry_count() { return Err(()) }

        // In case the descriptor size is not the same as the size of the rust struct
        let raw_ptr = self.descriptors as *mut u8;
            unsafe {Ok(*(raw_ptr.offset(self.descriptor_size as isize * index as isize) as *mut EfiMemoryDescriptor))
        }
    }

    pub fn entry_count(&self) -> usize {
        self.map_size / self.descriptor_size
    }

    pub fn iter(&self) -> EfiMemoryMapIterator {
        EfiMemoryMapIterator { memory_map: self, current_index: 0, max_index: self.entry_count() }
    }

    //? Note: I am not sure why this assumes everything is usable. This is from https://github.com/GRMorgan/rust-os/blob/master/bootloader_uefi/src/uefi/memory_map.rs
    //? Consider replacing with a loop to add each memory descriptor
    pub fn get_usable_memory_size_pages(&self) -> u64 {
        self.max_usable_physical_address().as_u64() / PAGE_SIZE
    }

    //? Note: I am not sure why not use self.max_usable_physical_address() instead.
    //? This is from https://github.com/GRMorgan/rust-os/blob/master/bootloader_uefi/src/uefi/memory_map.rs
    //? Maybe this is in case self.max_usable_physical_address() is not a multiple of PAGE_SIZE
    pub fn get_usable_memory_size_bytes(&self) -> u64 {
        self.get_usable_memory_size_pages() * PAGE_SIZE
    }

    pub fn max_usable_physical_address(&self) -> PhysicalAddress {
        let mut highest_address = PhysicalAddress::new(0);

        for descriptor in self.iter() {
            if descriptor.is_usable_memory() &&  descriptor.max_physical_address() > highest_address {
                highest_address = descriptor.max_physical_address();
            }
        }

        highest_address
    }

    pub fn max_physical_address(&self) -> PhysicalAddress {
        let mut highest_address = PhysicalAddress::new(0);

        for descriptor in self.iter() {
            if descriptor.max_physical_address() > highest_address {
                highest_address = descriptor.max_physical_address();
            }
        }

        highest_address
    }

    pub fn init_frame_allocator(&self) -> PageFrameAllocator {
        let mut largest_free_mem_seg_size = 0;
        let mut largest_free_mem_seg = PhysicalAddress::new(0);

        for current_descriptor in self.iter() {
            if current_descriptor.mem_type() == DescriptorType::EfiConventionalMemory && current_descriptor.num_bytes() > largest_free_mem_seg_size {
                largest_free_mem_seg_size = current_descriptor.num_bytes();
                largest_free_mem_seg = current_descriptor.phys_addr;
            }
        }


        let memory_size_pages = self.get_usable_memory_size_pages();
        com1_println!("Memory size in pages: {:#X}", memory_size_pages);
        let bitmap_size = (memory_size_pages + (8-1)) / 8;
        if bitmap_size > largest_free_mem_seg_size {
            panic!("Not enough space for allocator bitmap");
        }
        

        let mut bitmap = unsafe { bitmap::Bitmap::new_init(bitmap_size as usize, largest_free_mem_seg.as_u64() as *mut u8, 0xFF) };
        let output = unsafe { PageFrameAllocator::new_from_bitmap(&mut bitmap, 0, self.get_usable_memory_size_bytes()) };

        for current_descriptor in self.iter() {
            if current_descriptor.mem_type() == DescriptorType::EfiConventionalMemory {
                // If there is somehow a double free that means that there were overlapping segments
                // This should not happen but if it does that is ok
                // Except that if there are overlapping segments then should also make sure that no reserved segment overlaps 
                // a free segment. This is not currently done.
                let _ = output.free_pages(current_descriptor.phys_addr, current_descriptor.num_pages as usize);
            }
        }

        // Allocate the bitmap pages
        let bitmap_size_in_pages = (bitmap_size + (PAGE_SIZE - 1)) / PAGE_SIZE;
        com1_println!("Bitmap size pages: {bitmap_size_in_pages}");
        output.lock_pages(largest_free_mem_seg, bitmap_size_in_pages as usize)
            .expect("This should be impossible since we just unreserved this page.");

        output
    }

    /// If there is an error here then an extra free occurred elsewhere
    pub fn free_pages(mut self, allocator: &mut PageFrameAllocator) -> Result<(), AllocError>{
        allocator.free_pages(PhysicalAddress::new(self.descriptors as u64), self.num_pages)?;
        self.num_pages = 0;
        self.map_size = 0;
        self.descriptors = null_mut();

        Ok(())
    }
}

impl Default for EfiMemoryMap {
    fn default() -> Self {
        EfiMemoryMap {
            descriptors: null_mut(),
            map_size: 0,
            descriptor_size: 0,
            num_pages: 0,
        }
    }
}

pub struct EfiMemoryMapIterator<'a> {
    memory_map: &'a EfiMemoryMap,
    current_index: usize,
    max_index: usize,
}

impl<'a> Iterator for EfiMemoryMapIterator<'a> {
    type Item = EfiMemoryDescriptor;

    fn next(&mut self) -> Option<EfiMemoryDescriptor> {
        if self.current_index >= self.max_index { return None }

        match self.memory_map.get_descriptor(self.current_index) {
            Ok(descriptor) => {
                self.current_index += 1;
                Some(descriptor)
            },
            Err(()) => None,
        }
    }
}

pub struct GetMemoryMapOutput {
    pub map: EfiMemoryMap,
    pub map_key: usize,
    pub descriptor_version: u32,
}

impl Default for GetMemoryMapOutput {
    fn default() -> Self {
        GetMemoryMapOutput {
            map: EfiMemoryMap::default(),
            map_key: 0,
            descriptor_version: 0,
        }
    }
}