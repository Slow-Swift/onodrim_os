use core::ptr::null_mut;

use r_efi::{efi, protocols::simple_file_system};

use super::file_protocol::FileProtocol;

pub struct SimpleFileSystemProtocol {
    file_system_ptr: *mut simple_file_system::Protocol,
}

impl SimpleFileSystemProtocol {
    pub fn new(file_system_ptr: *mut simple_file_system::Protocol) -> SimpleFileSystemProtocol {
        SimpleFileSystemProtocol { file_system_ptr }
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