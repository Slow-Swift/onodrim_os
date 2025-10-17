use core::{ffi::c_void, ptr::{self, null_mut}};

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

        // Safety: This should be safe assuming that boot_service_ptr is valid
        let status = unsafe {
            ((*self.boot_services_ptr).allocate_pages)(alloc_type, mem_type, pages, &mut address)
        };

        return (status, address);
    }

    pub fn allocate_pool<T>(&self, mem_type: r_efi::system::MemoryType, count: usize) -> Result<*mut T, efi::Status> {
        let size = count * size_of::<T>();
        let buffer = self.allocate_pool_raw( mem_type, size)?;

        Ok(buffer as *mut T)
    }

    fn allocate_pool_raw(
        &self, mem_type: r_efi::system::MemoryType, size: usize
    ) -> Result<*mut c_void, efi::Status> {
        let mut buffer = ptr::null_mut();

        // Safety: This should be safe assuming that boot_service_ptr is valid
        let status = unsafe {
            ((*self.boot_services_ptr).allocate_pool)(
                mem_type, 
                size, 
                &mut buffer
            )
        };

        match status {
            efi::Status::SUCCESS => Ok(buffer),
            _ => Err(status)
        }
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

    /// ## Safety
    /// The caller must ensure that no references to the freed memory remains
    unsafe fn free_pool(&self, buffer: *mut c_void) -> Result<(), efi::Status> {
        let status = unsafe {
            ((*self.boot_services_ptr).free_pool)(buffer)
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
        let guid = loaded_image::PROTOCOL_GUID;
        let protocol_ptr = self.open_protocol(h, h, guid, )? as *mut loaded_image::Protocol;
        return Ok(LoadedImageProtocol::new(protocol_ptr));
    }

    pub fn get_simple_file_protocol(&self, h: efi::Handle) -> Result<SimpleFileSystemProtocol, efi::Status> {
        let guid = simple_file_system::PROTOCOL_GUID;
        let protocol_ptr = self.open_protocol(h, h, guid)? as *mut simple_file_system::Protocol;
        return Ok(SimpleFileSystemProtocol::new(protocol_ptr));
    }

    pub fn get_graphics_output_protocol(&self, agent_handle: efi::Handle) -> Result<GraphicsOutputProtocol, efi::Status> {
        let guid = graphics_output::PROTOCOL_GUID;
        let (handle, protocol) = self.find_and_open_protocol(agent_handle, guid)?;

        // Safety: This should be safe because the protocol pointer just got returned from Boot Services
        unsafe {
            Ok(GraphicsOutputProtocol::new(
                handle, 
                agent_handle,
                protocol as *mut graphics_output::Protocol
            ))
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

    /// Queries a handle to determine if it supports a specified protocol.
    /// 
    /// Checks if the handle supports the protocol indicated by the provided GUID. If the protocol is supported
    /// a pointer to the protocol interface is returned.
    /// 
    /// Note: UEFI Specification states that as of EFI 1.10 open_protocol should be used instead
    fn handle_protocol(&self, h: efi::Handle, mut guid: efi::Guid) -> Result<*mut c_void, efi::Status> {
        // The firmware implementation of handle protocol will return the address to this variable
        let mut interface = null_mut();

        // This should be safe assuming that boot_services_ptr actually points to a valid boot_services
        let status = unsafe {
            ((*self.boot_services_ptr).handle_protocol)(h, &mut guid, &mut interface)
        };

        match status {
            efi::Status::SUCCESS => Ok(interface),
            _ => Err(status),
        }
    }

    /// Queries a handle to determine if it supports a specified protocol.
    /// 
    /// Checks if the handle supports the protocol indicated by the provided GUID. If the protocol is supported
    /// a pointer to the protocol interface is returned.
    /// 
    /// Note: Open Protocol is only supported on EFI 1.10+. If the function is not supported, handle_protocol is automatically 
    /// used instead.
    fn open_protocol(&self, handle: efi::Handle, agent_handle: efi::Handle, mut guid: efi::Guid) -> Result<*mut c_void, efi::Status> {
        // This should be safe assuming that boot_services_ptr actually points to a valid boot_services
        let open_protocol_fn_ptr = unsafe { (*self.boot_services_ptr).open_protocol };
        
        // If open_protocol is not supported use handle_protocol instead 
        let is_open_protocol_supported = !ptr::eq(open_protocol_fn_ptr as *const c_void, ptr::null());
        if !is_open_protocol_supported {
            return self.handle_protocol(handle, guid);
        }
        
        // The firmware implementation of handle protocol will return the address to this variable
        let mut interface = null_mut();

        let status = open_protocol_fn_ptr(
            handle, 
            &mut guid,
            &mut interface,
            agent_handle,
            null_mut(),
            efi::OPEN_PROTOCOL_GET_PROTOCOL
        );

        match status {
            efi::Status::SUCCESS => Ok(interface),
            _ => Err(status),
        }
    }

    /// Locate a handle that supports the given protocol and open it
    fn find_and_open_protocol(&self, agent_handle: efi::Handle, guid: efi::Guid) -> Result<(efi::Handle, *mut c_void), efi::Status>{
        // Find a handle that supports the protocol
        let handles = self.locate_handle_buffer(guid)?;
        let protocol_handle = handles[0];

        // Should be safe because we are no longer using the handles list
        unsafe { self.free_pool(handles.as_ptr() as *mut c_void)?; }

        let interface = self.open_protocol(protocol_handle, agent_handle, guid)?;

        Ok((protocol_handle, interface))
    }

    /// Tells the firmware that the given protocol is no longer required by the agent
    /// 
    /// If close protocol is not supported then Err(UNSUPPORTED) is returned.
    pub fn close_protocol(&self, handle: efi::Handle, agent_handle: efi::Handle, mut guid: efi::Guid) -> Result<(), efi::Status>{
        // This should be safe assuming that boot_services_ptr actually points to a valid boot_services
        let close_protocol = unsafe { (*self.boot_services_ptr).close_protocol };
        
        // If open_protocol is not supported use handle_protocol instead 
        let is_close_protocol_supported = !ptr::eq(close_protocol as *const c_void, ptr::null());
        if !is_close_protocol_supported { return Err(efi::Status::UNSUPPORTED); }

        let status = close_protocol(handle, &mut guid, agent_handle, null_mut());
        match status {
            efi::Status::SUCCESS => Ok(()),
            _ => Err(status),
        }
    }

    /// Finds the first handle which supports the indicated protocol and returns the protocol interface
    /// 
    /// Warning: This bypasses protocol access control. Prefer using open_protocol
    fn _locate_protocol(&self, guid: *mut efi::Guid, registration: *mut c_void) -> Result<*mut c_void, efi::Status> {
        let mut output: *mut c_void = null_mut();
        let status = unsafe {
            ((*self.boot_services_ptr).locate_protocol)(guid, registration, &mut output)
        };

        match status {
            efi::Status::SUCCESS => Ok(output),
            _ => Err(status),
        }
    }

    /// Get a list of handles that support the given protocol
    /// 
    /// If the function is not supported it defaults to locate_handle instead
    fn locate_handle_buffer(&self, mut guid: efi::Guid) -> Result<&'static [efi::Handle], efi::Status>{
        let locate_handle_buffer = unsafe { ((*self.boot_services_ptr)).locate_handle_buffer };

        // If the function isn't supported (older than EFI 1.10) fall back on locate_handle
        let is_function_supported = !ptr::eq(locate_handle_buffer as *const c_void, ptr::null());
        if !is_function_supported {
            return self.locate_handle(guid);
        }

        let mut handle_buffer: *mut efi::Handle = ptr::null_mut();
        let mut num_handles = 0;

        let status = locate_handle_buffer(
            efi::BY_PROTOCOL,
            &mut guid,
            ptr::null_mut(),
            &mut num_handles,
            &mut handle_buffer
        );

        if status.is_error() {
            return Err(status);
        }


        Ok(unsafe { core::slice::from_raw_parts(handle_buffer, num_handles)} )
    }

    /// Get a list of handles that support the given protocol
    /// 
    /// Prefer using locate_handle_buffer
    fn locate_handle(&self, mut guid: efi::Guid) -> Result<&'static [efi::Handle], efi::Status> {
        let locate_handle = unsafe { ((*self.boot_services_ptr)).locate_handle };

        let mut buffer_size = 0;

        // Calling locate handle without a buffer will give us the buffer size we need
        let status = locate_handle(
            efi::BY_PROTOCOL,
            &mut guid,
            ptr::null_mut(),
            &mut buffer_size, 
            ptr::null_mut()
        );

        if status != efi::Status::BUFFER_TOO_SMALL { return Err(status); }

        // Now we can allocate a buffer for the handles
        let mut handle_buffer = self.allocate_pool_raw(
            efi::LOADER_DATA, 
            buffer_size
        )?;

        // Now actually get the list of handles
        let status = locate_handle(
            efi::BY_PROTOCOL,
            &mut guid,
            ptr::null_mut(),
            &mut buffer_size, 
            &mut handle_buffer
        );

        if status.is_error() { return Err(status); }

        let count = buffer_size / size_of::<efi::Handle>();

        // Safety: This should be safe since we just calculated the number of handles
        // that can fit in the array
        Ok(unsafe { core::slice::from_raw_parts(&mut handle_buffer, count)} )

    }
    
}