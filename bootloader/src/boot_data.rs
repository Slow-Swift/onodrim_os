use core::ptr::null;

use uefi::proto::console::gop::PixelFormat;

#[repr(C)]
pub struct OutputMode {
    pub output_width: usize,
    pub output_height: usize
}

#[repr(C)]
pub struct GraphicsMode {
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub format: PixelFormat
}

#[repr(C)]
pub struct BootData {
    pub output_mode: OutputMode,
    pub graphics_mode: GraphicsMode,
    pub memory_map_size: usize,
    pub memory_descriptor_size: usize,
    pub memory_map: *const u8
}

impl BootData {
    pub fn empty() -> BootData {
        BootData {
            output_mode: OutputMode { output_width: 0, output_height: 0},
            graphics_mode: GraphicsMode { width: 0, height: 0, stride: 0, format: PixelFormat::Bgr },
            memory_map_size: 0,
            memory_descriptor_size: 0,
            memory_map: null()
        }
    }
}