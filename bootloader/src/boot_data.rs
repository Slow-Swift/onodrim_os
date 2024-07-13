use core::ptr::{null, null_mut};

use uefi::proto::console::gop::PixelFormat;

#[repr(C)]
pub struct GraphicsMode {
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub frame_buffer: *mut u8,
    pub frame_buffer_size: usize,
    pub format: PixelFormat
}

#[repr(C)]
pub struct BootData {
    pub graphics_mode: GraphicsMode,
    pub font_file_address: *const u8,
    pub font_file_size: usize,
    pub memory_map_size: usize,
    pub memory_descriptor_size: usize,
    pub memory_map: *const u8
}

impl BootData {
    pub fn empty() -> BootData {
        BootData {
            graphics_mode: GraphicsMode { 
                width: 0, 
                height: 0, 
                stride: 0, 
                frame_buffer: null_mut(),
                frame_buffer_size: 0,
                format: PixelFormat::Bgr 
            },
            font_file_address: null(),
            font_file_size: 0,
            memory_map_size: 0,
            memory_descriptor_size: 0,
            memory_map: null()
        }
    }
}