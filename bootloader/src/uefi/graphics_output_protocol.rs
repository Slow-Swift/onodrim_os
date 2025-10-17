use bootinfo::FrameBuffer;
use r_efi::{efi, protocols::graphics_output};
use x86_64_hardware::memory::PhysicalAddress;

use crate::uefi::BootServices;

pub struct GraphicsOutputProtocol {
    device_handle: efi::Handle,
    agent_handle: efi::Handle,
    graphics_output_protocol_ptr: *mut graphics_output::Protocol,
}

impl GraphicsOutputProtocol {
    /// Creates a new wrapper around a GraphicsOutputProtocol pointer
    /// 
    /// Safety: It is up to the caller to ensure that the GraphicsOutputProtocol pointer
    /// actually points to a valid UEFI graphics output protocal
    pub unsafe fn new(
        device_handle: efi::Handle, 
        agent_handle: efi::Handle,
        graphics_output_protocol_ptr: *mut graphics_output::Protocol
    )  -> GraphicsOutputProtocol {
        GraphicsOutputProtocol { 
            device_handle, 
            agent_handle,
            graphics_output_protocol_ptr 
        }
    }

    pub fn close(self, boot_services: &BootServices) -> Result<(), efi::Status> {
        boot_services.close_protocol(
            self.device_handle, 
            self.agent_handle, 
            graphics_output::PROTOCOL_GUID
        )
    }

    /// Get the framebuffer from the GraphicsOutputProtocol
    pub fn get_framebuffer(&self) -> FrameBuffer {
        let mode = self.mode();

        match FrameBuffer::new(
            mode.frame_buffer_base(),
            mode.frame_buffer_size(),
            mode.info().horizontal_resolution(),
            mode.info().vertical_resolution(),
            mode.info().pixels_per_scan_line(),
        ) {
            Ok(buffer) => buffer,
            Err(e) => panic!(
                "Could not get Graphics Output Protocol FrameBuffer for some reason. This should not happen and indicates an issue with the UEFI. Error: {:?}",
                e
            )
        }
    }

    fn mode(&self) -> GopMode {
        // Safety: This is safe as long as the mode pointer is actually a mode pointer
        unsafe {
            GopMode::new(
                (*self.graphics_output_protocol_ptr).mode
            )
        }
    }
}

struct GopMode {
    mode_ptr: *mut graphics_output::Mode,
}

impl GopMode {
    /// Creates a new GraphicsOutputProtocol Mode from a mode pointer
    /// 
    /// Safety: It is up to the caller to ensure that the mode pointer 
    /// actually points to a valid mode
    pub unsafe fn new(mode_ptr: *mut graphics_output::Mode) -> GopMode {
        GopMode { mode_ptr }
    }

    pub fn frame_buffer_base(&self) -> PhysicalAddress {
        // Safety: This is safe as long as the mode pointer is actually a mode pointer
        unsafe {
            PhysicalAddress::new((*self.mode_ptr).frame_buffer_base)
        }
    }

    pub fn frame_buffer_size(&self) -> usize {
        // Safety: This is safe as long as the mode pointer is actually a mode pointer
        unsafe {
            (*self.mode_ptr).frame_buffer_size
        }
    }

    pub fn info(&self) -> GopModeInfo {
        // Safety: This is safe as long as the mode pointer actually points to a GOP mode
        unsafe {
            GopModeInfo::new(
                (*self.mode_ptr).info
            )
        }
    }
}

struct GopModeInfo {
    info_ptr: *mut graphics_output::ModeInformation,
}

impl GopModeInfo {
    /// Creates a new GraphicsOutputProtocol Mode Info from a mode info pointer
    /// 
    /// Safety: It is up to the caller to ensure that the mode info pointer 
    /// actually points to valid mode info
    pub unsafe fn new(info_ptr: *mut graphics_output::ModeInformation) -> GopModeInfo {
        GopModeInfo { info_ptr }
    }

    pub fn horizontal_resolution(&self) -> u32 {
        // Safety: This is safe as long as the info pointer actuall points to mode info
        unsafe {
            (*self.info_ptr).horizontal_resolution
        }
    }

    pub fn vertical_resolution(&self) -> u32 {
        // Safety: This is safe as long as the info pointer actuall points to mode info
        unsafe {
            (*self.info_ptr).vertical_resolution
        }
    }

    pub fn pixels_per_scan_line(&self) -> u32 {
        // Safety: This is safe as long as the info pointer actuall points to mode info
        unsafe {
            (*self.info_ptr).pixels_per_scan_line
        }
    }
}