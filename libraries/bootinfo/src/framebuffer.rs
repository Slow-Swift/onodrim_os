use x86_64_hardware::memory::PhysicalAddress;

/// Represents a frame buffer which can be drawn to
#[repr(C)]
pub struct FrameBuffer {
    pub base_address: PhysicalAddress,
    pub buffer_size: usize,
    pub width: u32,
    pub height: u32,
    pub pixels_per_scan_line: u32,
}

impl FrameBuffer {
    /// Creates a new FrameBuffer at the given address with the given sizes.
    /// 
    /// Assumptions: The memory from base_address -> base_address + buffer_size has been
    /// reserved for this frame buffer and will not be reallocated while this frame buffer
    /// is used.
    /// 
    /// The buffer size must be at least as big as pixels_per_scan_line * height * 4 (size of u32) or 
    /// a FrameBufferError will be returned
    pub fn new(
        base_address: PhysicalAddress, 
        buffer_size: usize, 
        width: u32, height: u32, 
        pixels_per_scan_line: u32
    ) -> Result<FrameBuffer, FrameBufferError> {
        // Check to ensure that the buffer is big enough to avoid errors later
        let required_size = pixels_per_scan_line as usize * height as usize * size_of::<u32>();
        if buffer_size < required_size {
            return Err(FrameBufferError::SizeTooSmall { required: required_size, available: buffer_size });
        }

        Ok(FrameBuffer {
            base_address,
            buffer_size,
            width,
            height,
            pixels_per_scan_line,
        })
    }

    /// Fill the buffer with the given color
    /// 
    /// Safety Conditions: The caller must ensure the frame buffer memory is reserved for the frame buffer alone
    pub unsafe fn fill(&self, color: u32, memory_offset: u64) {
        let virt_addr = self.base_address.get_virtual_address_at_offset(memory_offset);
        let first_pixel = virt_addr.get_mut_ptr::<u32>();
        for x_pos in 0..self.width {
            for y_pos in 0..self.height {
                // Assumption 1: width * height <= buffer_size
                // - This should be ensured by the constructor
                // Assumption 2: The memory for the frame buffer must be reserved for the frame buffer alone
                // - The caller must ensure this
                unsafe {
                    let pixel = first_pixel.add(
                        (y_pos * self.pixels_per_scan_line + x_pos) as usize
                    );
                    (*pixel) = color;
                }
            }
        }
    }
}

impl Default for FrameBuffer {
    /// Create a new frame buffer of size 0.
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

#[derive(Debug)]
pub enum FrameBufferError {
    SizeTooSmall { required: usize, available: usize },
}