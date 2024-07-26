use x86_64_hardware::memory::PhysicalAddress;

#[repr(C)]
pub struct FrameBuffer {
    pub base_address: PhysicalAddress,
    pub buffer_size: usize,
    pub width: u32,
    pub height: u32,
    pub pixels_per_scan_line: u32,
}

impl FrameBuffer {
    pub fn new(base_address: PhysicalAddress, buffer_size: usize, width: u32, height: u32, pixels_per_scan_line: u32) -> FrameBuffer {
        FrameBuffer {
            base_address,
            buffer_size,
            width,
            height,
            pixels_per_scan_line,
        }
    }

    pub unsafe fn fill(&self, color: u32, memory_offset: u64) {
        let virt_addr = self.base_address.get_virtual_address_at_offset(memory_offset);
        let base_pixel = virt_addr.get_mut_ptr::<u32>();
        for x_pos in 0..self.width {
            for y_pos in 0..self.height {
                unsafe {
                    let pixel = base_pixel.add((y_pos * self.pixels_per_scan_line + x_pos) as usize);
                    (*pixel) = color;
                }
            }
        }
    }
}

impl Default for FrameBuffer {
    fn default() -> Self {
        FrameBuffer {
            base_address: PhysicalAddress::new(0),
            buffer_size: 0,
            width: 0,
            height: 0,
            pixels_per_scan_line: 0,
        }
    }
}