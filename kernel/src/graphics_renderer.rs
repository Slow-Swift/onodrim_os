
use crate::{boot_data::BootData, errors::{Error, ErrorStatus}};

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

    pub fn set_pixel(&mut self, color: Color, x: usize, y: usize) {
        let x = if x >= self.width { self.width - 1} else {x};
        let y = if y >= self.width { self.height - 1} else {y};
        let pixel_index = y * self.stride + x;
        self.pixels[pixel_index] = BgrPixel::new(color);
    }
}

pub struct GraphicsRenderer {
    frame_buffer: FrameBuffer
}

impl GraphicsRenderer {

    pub fn from_boot_data(boot_data: &BootData) -> Result<GraphicsRenderer, Error> {
        let frame_buffer_size = boot_data.graphics_mode.frame_buffer_size;
        let pixel_count = frame_buffer_size / core::mem::size_of::<BgrPixel>();

        let pixels: &mut [BgrPixel] = unsafe {
            match boot_data.graphics_mode.format {
                crate::boot_data::PixelFormat::Bgr => core::slice::from_raw_parts_mut(
                    boot_data.graphics_mode.frame_buffer as *mut BgrPixel, 
                    pixel_count
                ),
                _ => return Err(Error::new(ErrorStatus::GraphicsPixelFormatNotSupported))
            }
        };

        let frame_buffer = FrameBuffer {
            pixels,
            width: boot_data.graphics_mode.width,
            height: boot_data.graphics_mode.height,
            stride: boot_data.graphics_mode.stride
        };

        Ok(
            GraphicsRenderer {
                frame_buffer 
            }
        )
    }

    pub fn fill(&mut self, color: Color) {
        for y in 0..self.frame_buffer.height {
            for x in 0..self.frame_buffer.width {
                self.frame_buffer.set_pixel(color, x, y);
            }
        }
    }

}