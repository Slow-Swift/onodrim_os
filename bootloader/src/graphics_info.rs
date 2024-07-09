use uefi::{println, proto::console::gop::GraphicsOutput, table::boot::{BootServices, ScopedProtocol}};

use crate::boot_data::BootData;


pub struct GraphicsInfo<'a> {
    gop: ScopedProtocol<'a, GraphicsOutput>,
    boot_services: &'a BootServices,
}

impl GraphicsInfo<'_> {

    pub fn new<'a>(boot_services: &BootServices) -> GraphicsInfo {
        let gop_handle = boot_services
            .get_handle_for_protocol::<GraphicsOutput>()
            .expect("Could not get GraphicsOutputProtocol handle");

        let gop = boot_services.open_protocol_exclusive(gop_handle).expect("Could not open protocol");

        GraphicsInfo { gop, boot_services }
    }

    pub fn print_modes(&self) {
        println!("Graphics Modes:");
        for (i, mode) in self.gop.modes(self.boot_services).enumerate() {
            let (x,y) = mode.info().resolution();
            let format = mode.info().pixel_format();
            println!("  Mode {i}: {x}x{y} Format: {format:?}");
        }

        let current_mode = self.gop.current_mode_info();
        let (width, height) = current_mode.resolution();
        let format = self.gop.current_mode_info().pixel_format();
        println!("Current Graphics Mode: {width}x{height}, Format: {:?}", format);
    }

    pub fn fill_boot_data(&mut self, boot_data: &mut BootData) {
        let current_mode = self.gop.current_mode_info();
        let (width, height) = current_mode.resolution();
        let format = self.gop.current_mode_info().pixel_format();

        boot_data.graphics_mode.width = width;
        boot_data.graphics_mode.height = height;
        boot_data.graphics_mode.format = format;
        boot_data.graphics_mode.frame_buffer = self.gop.frame_buffer().as_mut_ptr();
        boot_data.graphics_mode.frame_buffer_size = self.gop.frame_buffer().size();
        boot_data.graphics_mode.stride = current_mode.stride();
    }

}