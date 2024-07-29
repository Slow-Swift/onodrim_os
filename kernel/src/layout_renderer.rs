use core::fmt;

use crate::font_renderer::FontRenderer;


pub struct LayoutRenderer {
    font_renderer: FontRenderer,
    cols: usize,
    rows: usize,
    cell_width: usize,
    cell_height: usize,
    x: usize,
    y: usize
}

impl LayoutRenderer {
    
    pub fn new(font_renderer: FontRenderer) -> LayoutRenderer {
        let frame_buffer = font_renderer.get_graphics_renderer();
        let (width, height) = frame_buffer.get_resolution();
        let cell_width = font_renderer.get_glyph_width();
        let cell_height = font_renderer.get_glyph_height();
        let cols = width / cell_width;
        let rows = height / cell_height;

        LayoutRenderer { font_renderer, cols, rows, cell_width, cell_height, x: 0, y:0 }
    }

    pub fn print_char(&mut self, char: u8) {
        match char {
            b'\n' => self.newline(),
            b'\r' => {
                self.x = 0;
            },
            byte => {
                if self.x >= self.cols {
                    self.newline()
                }

                self.font_renderer.draw_glyph(byte, self.x * self.cell_width, self.y * self.cell_height);
                self.x += 1;
            }
        }
    }

    pub fn print_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' | b'\r' => self.print_char(byte),
                _ => self.print_char(0x04),
            }
        }
    }

    fn newline(&mut self) {
        self.x = 0;
        self.y += 1;
        
        if self.y >= self.rows { 
            self.y = 0; 
        }
    }
}

impl fmt::Write for LayoutRenderer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.print_string(s);
        Ok(())
    }
}