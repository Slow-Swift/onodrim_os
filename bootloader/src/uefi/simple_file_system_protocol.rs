use core::ptr::null_mut;

use r_efi::{efi, protocols::simple_file_system};

use crate::uefi::BootServices;

use super::file_protocol::FileProtocol;

pub struct SimpleFileSystemProtocol {
    device_handle: efi::Handle,
    agent_handle: efi::Handle,
    file_system_ptr: *mut simple_file_system::Protocol,
}

impl SimpleFileSystemProtocol {
    pub fn new(
        device_handle: efi::Handle, 
        agent_handle: efi::Handle, 
        protocol_ptr: *mut simple_file_system::Protocol
    ) -> SimpleFileSystemProtocol {
        SimpleFileSystemProtocol { 
            device_handle,
            agent_handle,
            file_system_ptr: protocol_ptr 
        }
    }

    /// Close the FileSystemProtocol
    /// 
    /// I believe it is fine to close this and still have the FileProtocol open
    pub fn close(self, boot_services: &BootServices) -> Result<(), efi::Status> {
        boot_services.close_protocol(self.device_handle, self.agent_handle, simple_file_system::PROTOCOL_GUID)
    }

    pub fn open_volume(&self) -> Result<FileProtocol, efi::Status>{
        let mut volume = null_mut();
        let status = unsafe {
            ((*self.file_system_ptr).open_volume)(self.file_system_ptr, &mut volume)
        };

        match status {
            efi::Status::SUCCESS => Ok(FileProtocol::new(volume)),
            _ => Err(status),
        }
    }
}