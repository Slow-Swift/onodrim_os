#[repr(C)]
pub enum PixelFormat {
    Rgb = 0,
    Bgr = 1,
    Bitmask = 2,
    BltOnly = 3,
}


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
    pub memory_map_size: usize,
    pub memory_descriptor_size: usize,
    pub memory_map: *const u8
}