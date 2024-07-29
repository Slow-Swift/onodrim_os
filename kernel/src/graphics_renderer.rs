
use bootinfo::BootInfo;

use crate::errors::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color(pub u8, pub u8, pub u8);

impl Color {
    pub fn new(hex_code: u32) -> Color {
        Color (
            ((hex_code >> 16) & 0xFF) as u8,
            ((hex_code >> 8) & 0xFF) as u8,
            (hex_code & 0xFF) as u8
        )
    }

    #[inline]
    pub fn red(&self) -> u8 { self.0 }

    #[inline]
    pub fn green(&self) -> u8 { self.1 }

    #[inline]
    pub fn blue(&self) -> u8 { self.2 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct BgrPixel(u32);

impl BgrPixel {
    fn new(color: Color) -> BgrPixel {
        BgrPixel((color.red() as u32) << 16 | (color.green() as u32) << 8 | color.blue() as u32)
    }
}

pub struct FrameBuffer {
    pub width: usize,
    pub height: usize,
    pixels: &'static mut [BgrPixel],
    stride: usize,
}

impl FrameBuffer {

    pub fn from_boot_data(bootinfo: &BootInfo) -> Result<FrameBuffer, Error> {
        let frame_buffer_size = bootinfo.framebuffer.buffer_size;
        let pixel_count = frame_buffer_size / core::mem::size_of::<BgrPixel>();

        let buffer_address = bootinfo.framebuffer.base_address.get_virtual_address_at_offset(bootinfo.page_table_memory_offset);

        let pixels: &mut [BgrPixel] = unsafe {
            core::slice::from_raw_parts_mut(
                buffer_address.as_u64() as *mut BgrPixel, 
                pixel_count
            )
        };

        Ok(
            FrameBuffer {
                pixels,
                width: bootinfo.framebuffer.width as usize,
                height: bootinfo.framebuffer.height as usize,
                stride: bootinfo.framebuffer.pixels_per_scan_line as usize,
            }
        )
    }

    pub fn fill(&mut self, color: Color) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.set_pixel(x, y, color);
            }
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        let x = if x >= self.width { self.width - 1} else {x};
        let y = if y >= self.width { self.height - 1} else {y};
        let pixel_index = y * self.stride + x;
        self.pixels[pixel_index] = BgrPixel::new(color);
    }

    pub fn get_resolution(&self) -> (usize, usize) { (self.width, self.height) }
}