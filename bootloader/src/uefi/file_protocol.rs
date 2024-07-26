use core::{ffi::c_void, ptr::{addr_of, null_mut}};

use r_efi::{efi, protocols::file};

use crate::unicode::{str_utf8_to_utf16, EncodeStatus};

pub struct FileProtocol {
    file_ptr: *mut file::Protocol,
    is_open: bool
}

impl FileProtocol {
    pub fn new(file_ptr: *mut file::Protocol) -> FileProtocol {
        FileProtocol { file_ptr, is_open: true }
    }

    pub fn open_path(&self, path: &str, open_mode: u64, attributes: u64) -> Result<FileProtocol, efi::Status> {
        let mut path_parts = path.split('/');
        let path_count = path_parts.clone().count();

        let mut file;
        
        if path_count == 0 {
            return Err(efi::Status::INVALID_PARAMETER);
        } else if path_count == 1 {
            return self.open(path, open_mode, attributes);
        } else {
            file = self.open(path_parts.next().unwrap(), efi::protocols::file::MODE_READ, efi::protocols::file::READ_ONLY)?;
        }

        for (i, part) in path_parts.enumerate() {
            if i == path_count - 1 {
                file = file.open(part, open_mode, attributes)?;
            } else {
                file = file.open(part, efi::protocols::file::MODE_READ, efi::protocols::file::READ_ONLY)?;
            }
        }

        Ok(file)
    }

    pub fn open(&self, path: &str, open_mode: u64, attributes: u64) -> Result<FileProtocol, efi::Status> {
        let mut output_ptr = null_mut();
        let mut path_buffer = [0;1024];
        let input_len = path.as_bytes().len();

        let EncodeStatus {input_read, ..} = str_utf8_to_utf16(&path, &mut path_buffer);
        
        if input_read < input_len { return Err(efi::Status::BUFFER_TOO_SMALL) }

        let status = unsafe {
            ((*self.file_ptr).open)(self.file_ptr, &mut output_ptr, path_buffer.as_mut_ptr(), open_mode, attributes)
        };

        match status {
            efi::Status::SUCCESS => Ok(FileProtocol::new(output_ptr)),
            _ => Err(status),
        }
    }

    pub fn read(&self, buffer_size: &mut usize, buffer: *mut c_void) -> Result<(), efi::Status> {
        let status = unsafe {
            ((*self.file_ptr).read)(self.file_ptr, buffer_size, buffer)
        };

        match status {
            efi::Status::SUCCESS => Ok(()),
            _ => Err(status)
        }
    }

    pub fn close(&mut self) -> efi::Status {
        if self.is_open {
            let status = unsafe {
                ((*self.file_ptr).close)(self.file_ptr)
            };
            self.is_open = false;

            return status;
        } else {
            return efi::Status::SUCCESS;
        }
    }

    pub fn read_struct<T: Default>(&self) -> Result<T, efi::Status> {
        let mut size = core::mem::size_of::<T>();
        let output = Default::default();
        self.read(&mut size, addr_of!(output) as *mut c_void)?;
        return Ok(output);
    }

    pub fn set_position(&self, pos: u64) -> Result<(), efi::Status> {
        let status = unsafe {
            ((*self.file_ptr).set_position)(self.file_ptr, pos)
        };

        match status {
            efi::Status::SUCCESS => Ok(()),
            _ => Err(status),
        }
    }
}

impl Drop for FileProtocol {
    fn drop(&mut self) {
        self.close();
    }
}