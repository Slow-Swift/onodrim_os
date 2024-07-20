use core::ptr::null_mut;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Bitmap {
    size: usize,
    buffer: *mut BitmapByte
}

impl Bitmap {
    pub const unsafe fn new_uninitialized() -> Bitmap {
        Bitmap {
            size: 0,
            buffer: null_mut(),
        }
    }

    /// Create a new bitmap by passing in a pointer to the bitmap data
    /// 
    /// ## Safety
    /// 
    /// The caller must ensure that the buffer is:
    /// 1. Only used by this Bitmap
    /// 2. Is a valid pointer
    /// 3. Points to a memory block large enough for the bitmap
    pub unsafe fn new(size: usize, buffer: *mut u8) -> Bitmap {
        Bitmap {
            size,
            buffer: buffer as *mut BitmapByte,
        }
    }

    pub unsafe fn new_init_zero(size: usize, buffer: *mut u8) -> Bitmap {
        Self::new_init(size, buffer, 0)
    }

    pub unsafe fn new_init(size: usize, buffer: *mut u8, default: u8) -> Bitmap {
        let mut bitmap = Self::new(size, buffer);

        for i in 0..size {
            bitmap.set_byte(i, default);
        }

        bitmap
    }

    pub fn get(&self, index: usize) -> bool {
        let buffer_index = (index / 8) as usize;
        let bit_index = index % 8;

        if buffer_index > self.size {
            return false;
        } else {
            unsafe {
                return (*self.buffer.offset(buffer_index as isize)).get(bit_index)
            }
        }
    }

    pub fn set(&self, index: usize, value: bool) -> bool {
        let buffer_index = (index / 8) as usize;
        let bit_index = index % 8;

        if buffer_index > self.size {
            return false;
        } else {
            unsafe {
                (*self.buffer.offset(buffer_index as isize)).set(bit_index, value);
            }
            return true;
        }
    }

    /// Returns the buffer as a u8 pointer
    /// 
    /// ## Safety
    /// 
    /// The caller must ensure that either the pointer is not stored
    /// or that the buffer is discarded so that both cannot be used at the same time
    /// 
    /// Essentially, calling this function should act as a borrow of the bitmap
    pub unsafe fn get_buffer(&self) -> *mut u8 {
        self.buffer as *mut u8
    }

    /// Set the buffer
    /// 
    /// ## Safety
    /// 
    /// The caller must ensure that the buffer is:
    /// 1. Only used by this Bitmap
    /// 2. Is a valid pointer
    /// 3. Points to a memory block large enough for the bitmap
    pub unsafe fn set_buffer(&mut self, buffer: *mut u8) {
        self.buffer = buffer as *mut BitmapByte;
    }

    pub fn size(&self) -> usize {
        self.size
    }

    fn set_byte(&mut self, index: usize, byte: u8) {
        if index > self.size { return }

        unsafe {
            (*self.buffer.offset(index as isize)).set_byte(byte);
        }
    }

}

#[repr(transparent)]
struct BitmapByte {
    pub byte: u8,
}

impl BitmapByte {
    pub fn get(&self, index: usize) -> bool {
        if index > 7 {
            false
        } else {
            (self.byte & 0b1000_0000 >> index) > 0
        }
    }

    pub fn set(&mut self, index: usize, value: bool) {
        if index > 7 { return; }

        let byte_indexer = 0b1000_0000 >> index;

        if value {
            self.byte |= byte_indexer;
        } else {
            self.byte &= !byte_indexer;
        }
    }

    pub fn set_byte(&mut self, byte: u8) {
        self.byte = byte;
    }
}