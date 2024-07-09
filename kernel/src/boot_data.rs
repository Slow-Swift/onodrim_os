#[repr(C)]
pub enum PixelFormat {
    Rgb = 0,
    Bgr = 1,
    Bitmask = 2,
    BltOnly = 3,
}

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