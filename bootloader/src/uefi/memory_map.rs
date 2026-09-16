use core::{ffi::c_void, fmt::Display, ptr::{self, null_mut}};

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
        // Technically the max physical address is this -1.
        // TODO: Check where this is called and whether it matters
        self.phys_addr.increment_pages(self.num_pages)
    }

    /// Determines whether or not this memory section is usable
    /// 
    /// See UEFI specification section 7.2 for more information about each type
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

    /// Get the type of memory that this descriptor is. 
    /// 
    /// See UEFI spec 7.2.1 for a description of these types.
    pub fn mem_type(&self) -> DescriptorType {
        // mem_type is represented as a u32 so that any extra types get mapped to UNKOWN
        // instead of causing errors.
        // That means we need this function to convert to a descriptor type.
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

impl Default for EfiMemoryDescriptor {
    fn default() -> Self {
        Self { 
            mem_type: DescriptorType::EfiUnkown as u32, 
            phys_addr: PhysicalAddress::new(0), 
            virt_addr: ptr::null_mut(), 
            num_pages: 0, 
            attribs: 0 
        }
    }
}

impl Display for EfiMemoryDescriptor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "  {:30?}: {:#X} -> {:#X} ({} pages) ({} bytes)", 
            self.mem_type(), 
            self.phys_addr.as_u64(), 
            self.max_physical_address().as_u64(), 
            self.num_pages,
            self.num_bytes()
        )
    }
}

#[repr(C)]
pub struct EfiMemoryMap {
    pub descriptors: *mut EfiMemoryDescriptor,
    pub size_bytes: usize,
    pub descriptor_size: usize,
    pub num_pages: usize,
    pub map_key: usize,
    pub descriptor_version: u32,
}

impl EfiMemoryMap {
    /// Get the descriptor at the given index
    pub fn get_descriptor(&self, index: usize) -> Result<EfiMemoryDescriptor, ()> {
        if index > self.entry_count() { return Err(()) }

        // In case the descriptor size is not the same as the size of the rust struct
        let raw_ptr = self.descriptors as *mut u8;
            unsafe {Ok(*(raw_ptr.offset(self.descriptor_size as isize * index as isize) as *mut EfiMemoryDescriptor))
        }
    }

    /// Get the number of descriptor entries
    pub fn entry_count(&self) -> usize {
        self.size_bytes / self.descriptor_size
    }

    pub fn iter(&'_ self) -> EfiMemoryMapIterator<'_> {
        EfiMemoryMapIterator { memory_map: self, current_index: 0, max_index: self.entry_count() }
    }

    //? Note: I am not sure why this assumes everything is usable. This is from https://github.com/GRMorgan/rust-os/blob/master/bootloader_uefi/src/uefi/memory_map.rs
    //? Consider replacing with a loop to add each memory descriptor
    /// Get the physical index of the highest page of usable memory
    /// 
    /// Note, this is technically the index of the next page which would be unusable.
    pub fn get_highest_usable_page(&self) -> u64 {
        self.max_usable_physical_address().as_u64() / PAGE_SIZE
    }

    /// Get the physical address of the highest usable memory
    /// 
    /// Note, this is technical 1 higher than the max. I don't know if this is needed somewhere
    pub fn max_usable_physical_address(&self) -> PhysicalAddress {
        let mut highest_address = PhysicalAddress::new(0);

        for descriptor in self.iter() {
            if descriptor.is_usable_memory() &&  descriptor.max_physical_address() > highest_address {
                highest_address = descriptor.max_physical_address();
            }
        }

        highest_address
    }

    /// Get the physical address of the highest memory
    /// 
    /// Note, this is technical 1 higher than the max. I don't know if this is needed somewhere
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
        // Determine size of bitmap needed
        // We don't need to manage any pages that are unusable so we only need
        // enough space to manage up to the highest usable page
        let memory_size_pages = self.get_highest_usable_page();

        // One byte in the bitmap can represent 8 pages
        let bitmap_size = (memory_size_pages + (8-1)) / 8;
        com1_println!("Memory size in pages: {:#X}", memory_size_pages);
        com1_println!("Bitmap size in bytes: {:#X}", bitmap_size);

        // Find a memory segment large enough for the bitmap
        // Currently this just uses the first one that works
        let mut bitmap_memory_segment = EfiMemoryDescriptor::default();
        for current_descriptor in self.iter() {
            if current_descriptor.mem_type() != DescriptorType::EfiConventionalMemory { continue; }
            if current_descriptor.num_bytes() < bitmap_size { continue; }

            bitmap_memory_segment = current_descriptor;
            break;
        }

        // Make sure we actually found some valid memory
        if bitmap_memory_segment.num_bytes() < bitmap_size {
            panic!("Could not find memory for the bitmap allocator");
        }
        
        // Create a new bitmap starting from the beginning of the free memory
        // All pages will be marked as reserved by default
        let mut bitmap = unsafe { 
            bitmap::Bitmap::new_init(
                bitmap_size as usize, 
                bitmap_memory_segment.phys_addr.as_u64() as *mut u8, 
                0xFF
            ) 
        };

        // Create a new page frame allocator using the bitmap
        let output = unsafe { 
            PageFrameAllocator::new_from_bitmap(
                &mut bitmap, 
                memory_size_pages, 
                0
            ) 
        };

        // Mark all memory in that should be free as available. This includes
        // - EfiConventionalMemory (regular free memory)
        // - EfiBootServicesCode (was used by boot services but no longer)
        // - EfiBootServicesData (was used by boot services but no longer)
        // Note that this means the pages that contain the bitmap will be marked as free
        // We will lock those pages again right away
        for current_descriptor in self.iter() {
            match current_descriptor.mem_type() {
                // TODO: For some reason freeing this now causes a Double Free later.
                //       Work out why? I think it is because we still use the firmware page table
                // DescriptorType::EfiBootServicesCode |
                // DescriptorType::EfiBootServicesData | 
                DescriptorType::EfiConventionalMemory => {
                    // If there is somehow a double free that means that there were overlapping segments
                    // This should not happen but if it does that is ok
                    // Except that if there are overlapping segments then should also make sure that no reserved segment overlaps 
                    // a free segment. This is not currently done.
                    let _ = output.free_pages(
                        current_descriptor.phys_addr, 
                        current_descriptor.num_pages as usize
                    );
                },
                _ => {}
            }
        }

        // Lock the pages containing the bitmap to ensure that they don't get used
        let bitmap_size_in_pages = (bitmap_size + (PAGE_SIZE - 1)) / PAGE_SIZE;
        com1_println!("Bitmap size pages: {bitmap_size_in_pages}");
        output.lock_pages(
            bitmap_memory_segment.phys_addr, 
            bitmap_size_in_pages as usize
        ).expect("This should be impossible since we just unreserved this page.");

        output
    }

    /// If there is an error here then an extra free occurred elsewhere
    pub fn free_pages(mut self, allocator: &mut PageFrameAllocator) -> Result<(), AllocError>{
        allocator.free_pages(PhysicalAddress::new(self.descriptors as u64), self.num_pages)?;
        self.num_pages = 0;
        self.size_bytes = 0;
        self.descriptors = null_mut();

        Ok(())
    }
}

impl Display for EfiMemoryMap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Memory Map with {} descriptors:", self.entry_count())?;
        for descriptor in self.iter() {
            writeln!(f, "{}", descriptor)?;
        }
        Ok(())
    }
}

impl Default for EfiMemoryMap {
    fn default() -> Self {
        EfiMemoryMap {
            descriptors: null_mut(),
            size_bytes: 0,
            descriptor_size: 0,
            num_pages: 0,
            map_key: 0,
            descriptor_version: 0,
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