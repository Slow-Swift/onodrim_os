use r_efi::{efi, protocols::loaded_image};

use crate::uefi::BootServices;

pub struct LoadedImageProtocol {
    device_handle: efi::Handle,
    agent_handle: efi::Handle,
    protocol_ptr: *mut loaded_image::Protocol,
}

impl LoadedImageProtocol {
    pub fn new(
        device_handle: efi::Handle, 
        agent_handle: efi::Handle,
        protocol_ptr: *mut loaded_image::Protocol
    ) -> LoadedImageProtocol {
        LoadedImageProtocol { device_handle, agent_handle, protocol_ptr }
    }

    /// Close the LoadedImageProtocol
    pub fn close(self, boot_services: &BootServices) -> Result<(), efi::Status> {
        boot_services.close_protocol(
            self.device_handle, 
            self.agent_handle, 
            loaded_image::PROTOCOL_GUID
        )
    }

    /// Get the handle of the device where this image has been loaded from (eg. hard drive)
    /// 
    /// Note that this is different than the image handle passed by EFI
    pub fn device_handle(&self) -> efi::Handle {
        // Should be safe assuming the protocol_ptr is valid
        unsafe { (*self.protocol_ptr).device_handle }
    }
}