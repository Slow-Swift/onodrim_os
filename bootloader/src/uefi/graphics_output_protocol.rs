use bootinfo::FrameBuffer;
use r_efi::protocols::graphics_output;
use x86_64_hardware::memory::PhysicalAddress;


pub struct GraphicsOutputProtocol {
    graphics_output_protocol_ptr: *mut graphics_output::Protocol,
}

impl GraphicsOutputProtocol {
    pub fn new(graphics_output_protocol_ptr: *mut graphics_output::Protocol)  -> GraphicsOutputProtocol {
        GraphicsOutputProtocol { graphics_output_protocol_ptr }
    }

    pub fn get_framebuffer(&self) -> FrameBuffer {
        FrameBuffer::new(
            self.mode().frame_buffer_base(),
            self.mode().frame_buffer_size(),
            self.mode().info().horizontal_resolution(),
            self.mode().info().vertical_resolution(),
            self.mode().info().pixels_per_scan_line(),
        ).unwrap()
    }

    fn mode(&self) -> GopMode {
        GopMode::new(
            unsafe {
                (*self.graphics_output_protocol_ptr).mode
            }
        )
    }
}

struct GopMode {
    mode_ptr: *mut graphics_output::Mode,
}

impl GopMode {
    pub fn new(mode_ptr: *mut graphics_output::Mode) -> GopMode {
        GopMode { mode_ptr }
    }

    pub fn frame_buffer_base(&self) -> PhysicalAddress {
        unsafe {
            PhysicalAddress::new((*self.mode_ptr).frame_buffer_base)
        }
    }

    pub fn frame_buffer_size(&self) -> usize {
        unsafe {
            (*self.mode_ptr).frame_buffer_size
        }
    }

    pub fn info(&self) -> GopModeInfo {
        GopModeInfo::new(
            unsafe {
                (*self.mode_ptr).info
            }
        )
    }
}

struct GopModeInfo {
    info_ptr: *mut graphics_output::ModeInformation,
}

impl GopModeInfo {
    pub fn new(info_ptr: *mut graphics_output::ModeInformation) -> GopModeInfo {
        GopModeInfo { info_ptr }
    }

    pub fn horizontal_resolution(&self) -> u32 {
        unsafe {
            (*self.info_ptr).horizontal_resolution
        }
    }

    pub fn vertical_resolution(&self) -> u32 {
        unsafe {
            (*self.info_ptr).vertical_resolution
        }
    }

    pub fn pixels_per_scan_line(&self) -> u32 {
        unsafe {
            (*self.info_ptr).pixels_per_scan_line
        }
    }
}