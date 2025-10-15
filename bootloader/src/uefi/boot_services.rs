use core::{ffi::c_void, ptr::null_mut};

use r_efi::{efi, protocols::{graphics_output, loaded_image, simple_file_system}};
use x86_64_hardware::memory::PAGE_SIZE;
use super::{file_protocol::FileProtocol, graphics_output_protocol::GraphicsOutputProtocol, loaded_image_protocol::LoadedImageProtocol, memory_map::{self, GetMemoryMapOutput}, simple_file_system_protocol::SimpleFileSystemProtocol};

pub struct BootServices {
    boot_services_ptr: *mut efi::BootServices,
}

impl BootServices {
    /// Create a new wrapper around the EFI Boot Services
    /// 
    /// ## Safety
    /// It is up to the caller to ensure this actually points to a valid boot services 
    /// that has not yet exited and that no other BootServices is created with the same pointer
    pub unsafe fn new(boot_services_ptr: *mut efi::BootServices) -> BootServices {
        BootServices { boot_services_ptr }
    }

    pub fn allocate_pages<T>(&self, mem_type: r_efi::system::MemoryType, pages: usize) -> Result<*mut T, efi::Status>{
        let (status, address) = self.allocate_pages_raw(r_efi::system::ALLOCATE_ANY_PAGES, mem_type, pages);

        match status {
            efi::Status::SUCCESS => Ok(address as *mut T),
            _ => Err(status)
        }
    }

    fn allocate_pages_raw(&self, alloc_type: r_efi::system::AllocateType, mem_type: r_efi::system::MemoryType, pages: usize) -> (efi::Status, u64) {
        let mut address: u64 = 0;

        let status = unsafe {
            ((*self.boot_services_ptr).allocate_pages)(alloc_type, mem_type, pages, &mut address)
        };

        return (status, address);
    }

    /// ## Safety
    /// The caller must ensure that no references to the freed memory remains
    pub unsafe fn free_pages<T>(&self, mem: *mut T, num_pages: usize) -> Result<(), efi::Status> {
        return self.free_pages_raw(mem as u64, num_pages)
    }

    /// ## Safety
    /// The caller must ensure that no references to the freed memory remains
    unsafe fn free_pages_raw(&self, mem_addr: u64, num_pages: usize) -> Result<(), efi::Status> {
        let status = unsafe {
            ((*self.boot_services_ptr).free_pages)(mem_addr, num_pages)
        };

        match status {
            efi::Status::SUCCESS => Ok(()),
            _ => Err(status),
        }
    }

    pub fn open_volume(&self, h: efi::Handle) -> Result<FileProtocol, efi::Status>{
        let loaded_image = self.get_loaded_image_protocol(h)?;
        let file_system = self.get_simple_file_protocol(loaded_image.device_handle())?;
        file_system.open_volume()
    }

    pub fn get_loaded_image_protocol(&self, h: efi::Handle) -> Result<LoadedImageProtocol, efi::Status>{
        let mut guid = loaded_image::PROTOCOL_GUID;
        let protocol_ptr = self.handle_protocol(h, &mut guid)? as *mut loaded_image::Protocol;
        return Ok(LoadedImageProtocol::new(protocol_ptr));
    }

    pub fn get_simple_file_protocol(&self, h: efi::Handle) -> Result<SimpleFileSystemProtocol, efi::Status> {
        let mut guid = simple_file_system::PROTOCOL_GUID;
        let protocol_ptr = self.handle_protocol(h, &mut guid)? as *mut simple_file_system::Protocol;
        return Ok(SimpleFileSystemProtocol::new(protocol_ptr));
    }

    pub fn get_graphics_output_protocol(&self) -> Result<GraphicsOutputProtocol, efi::Status> {
        let mut guid = graphics_output::PROTOCOL_GUID;
        let registration: *mut core::ffi::c_void = null_mut();
        let protocol_ptr = self.locate_protocol(&mut guid, registration)? as *mut graphics_output::Protocol;

        // Safety: This should be safe because the protocol pointer just got returned from Boot Services
        unsafe {
            Ok(GraphicsOutputProtocol::new(protocol_ptr))
        }
    }

    pub fn get_memory_map(&self) -> Result<GetMemoryMapOutput, efi::Status> {
        let mut output = GetMemoryMapOutput::default();
        let mut mem_ptr = null_mut();

        // If the buffer is too small (which it will be since it is 0)
        // this modifies output.map.map_size to tell us how big it needs to be
        let mut status = unsafe {
            ((*self.boot_services_ptr).get_memory_map)(
                &mut output.map.map_size,
                mem_ptr as *mut efi::MemoryDescriptor,
                &mut output.map_key,
                &mut output.map.descriptor_size,
                &mut output.descriptor_version,
            )
        };

        while status == efi::Status::BUFFER_TOO_SMALL {
            let num_pages = (output.map.map_size + PAGE_SIZE as usize - 1) / PAGE_SIZE as usize;
            mem_ptr = self.allocate_pages(efi::LOADER_DATA, num_pages)?;

            status = unsafe {
                ((*self.boot_services_ptr).get_memory_map)(
                    &mut output.map.map_size,
                    mem_ptr as *mut efi::MemoryDescriptor,
                    &mut output.map_key,
                    &mut output.map.descriptor_size,
                    &mut output.descriptor_version,
                )
            };

            if status != efi::Status::SUCCESS {
                // This is safe since we are not going to refer to mem_ptr before allocating it again
                unsafe { self.free_pages(mem_ptr, num_pages) }?;
                if status != efi::Status::BUFFER_TOO_SMALL { return Err(status) }
            } else {
                output.map.num_pages = num_pages;
            }
        }

        output.map.descriptors = mem_ptr as *mut memory_map::EfiMemoryDescriptor;

        Ok(output)
    }

    /// Exit the boot services
    /// 
    /// ## Safety
    /// The caller must ensure the boot services are not used again and must ensure that all previously allocated
    /// memory remains allocated by whatever memory allocator takes over for the boot services
    pub unsafe fn exit_boot_services(&mut self, handle: efi::Handle, map_key: usize) -> Result<(), efi::Status> {
        // Safety: This should be safe assuming that boot services
        let status = unsafe {
            ((*self.boot_services_ptr).exit_boot_services)(handle, map_key)
        };

        match status {
            efi::Status::SUCCESS => Ok(()),
            _ => Err(status),
        }
    }

    fn handle_protocol(&self, h: efi::Handle, guid: *mut efi::Guid) -> Result<*mut c_void, efi::Status> {
        let mut output = null_mut();
        let status = unsafe {
            ((*self.boot_services_ptr).handle_protocol)(h, guid, &mut output)
        };

        match status {
            efi::Status::SUCCESS => Ok(output),
            _ => Err(status),
        }
    }

    fn locate_protocol(&self, guid: *mut efi::Guid, registration: *mut c_void) -> Result<*mut c_void, efi::Status> {
        let mut output: *mut c_void = null_mut();
        let status = unsafe {
            ((*self.boot_services_ptr).locate_protocol)(guid, registration, &mut output)
        };

        match status {
            efi::Status::SUCCESS => Ok(output),
            _ => Err(status),
        }
    }
    
}