use core::{ffi::c_void, ptr::null_mut};

use r_efi::{efi, protocols::graphics_output};
use super::graphics_output_protocol::GraphicsOutputProtocol;

pub struct BootServices {
    boot_services_ptr: *mut efi::BootServices,
}

impl BootServices {
    
    pub fn new(boot_services_ptr: *mut efi::BootServices) -> BootServices {
        BootServices { boot_services_ptr }
    }

    pub fn allocate_pages<T>(&self, mem_type: r_efi::system::MemoryType, pages: usize) -> Result<*mut T, efi::Status>{
        let (status, address) = self.allocate_pages_raw(r_efi::system::ALLOCATE_ANY_PAGES, mem_type, pages);

        match status {
            efi::Status::SUCCESS => Ok(address as *mut T),
            _ => Err(status)
        }
    }

    pub fn allocate_pages_raw(&self, alloc_type: r_efi::system::AllocateType, mem_type: r_efi::system::MemoryType, pages: usize) -> (efi::Status, u64) {
        let mut address: u64 = 0;

        let status = unsafe {
            ((*self.boot_services_ptr).allocate_pages)(alloc_type, mem_type, pages, &mut address)
        };

        return (status, address);
    }

    pub fn get_graphics_output_protocol(&self) -> Result<GraphicsOutputProtocol, efi::Status> {
        let mut guid = graphics_output::PROTOCOL_GUID;
        let registration: *mut core::ffi::c_void = null_mut();
        let protocol_ptr = self.locate_protocol(&mut guid, registration)? as *mut graphics_output::Protocol;
        Ok(GraphicsOutputProtocol::new(protocol_ptr))
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