use crate::{errors::{Error, ErrorStatus}, graphics_renderer::{Color, FrameBuffer}};

const GLYPH_START_OFFSET: usize = 4;

pub struct FontRenderer {
    font_data: &'static [u8],
    glyph_height: usize,
    foreground_color: Color,
    background_color: Color,
    scale: usize,
    frame_buffer: FrameBuffer
}

impl FontRenderer {

    pub fn create(font_address: *const u8, font_size: usize, frame_buffer: FrameBuffer) -> Result<FontRenderer, Error> {
        let font_data = unsafe {
            core::slice::from_raw_parts(font_address, font_size)
        };

        if !is_valid_font_header(font_data) { return Err(Error::new(ErrorStatus::InvalidFileFormat)) }

        let glyph_height = font_data[3] as usize;

        Ok( FontRenderer { 
            font_data, 
            glyph_height,
            foreground_color: Color::new(0x00B000),
            background_color: Color::new(0x000000),
            scale: 2,
            frame_buffer
        } )
    }

    pub fn get_graphics_renderer(&self) -> &FrameBuffer { &self.frame_buffer }

    pub fn draw_glyph(&mut self, glyph: u8, x: usize, y: usize) {
        let glyph_start = (glyph as usize) * (self.glyph_height as usize) + GLYPH_START_OFFSET;
        let glyph_end = glyph_start + (self.glyph_height as usize);
        let glyph_bytes = &self.font_data[glyph_start..glyph_end];

        for row in 0..self.glyph_height {
            let byte = glyph_bytes[row];
            for col in 0..8 {
                let fill_pixel = byte & (1 << (7 - col)) != 0;
                let color = if fill_pixel { self.foreground_color } else { self.background_color };

                for py in 0..self.scale {
                    for px in 0..self.scale {
                        self.frame_buffer.set_pixel(x + col * self.scale + px, y + row * self.scale + py, color);
                    }
                }
            }
        }
    }

    pub fn get_glyph_width(&self) -> usize { 8 * self.scale }

    pub fn get_glyph_height(&self) -> usize { self.glyph_height * self.scale }

    pub fn set_colors(&mut self, foreground: Color, background: Color) {
        self.foreground_color = foreground;
        self.background_color = background;
    }

}

fn is_valid_font_header(font_data: &[u8]) -> bool { (font_data[0] == 0x36) && (font_data[1] == 0x04) }