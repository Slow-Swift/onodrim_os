use x86_64_hardware::memory::PhysicalAddress;
use bitmap::Bitmap;
use core::ptr::null_mut;

pub struct MemInfo {
    pub bitmap: bitmap::Bitmap,
    pub free_memory: u64,
    pub reserved_memory: u64,
    pub used_memory: u64,
    pub max_physical_address: PhysicalAddress,
}

impl MemInfo {
    pub fn new(bitmap: bitmap::Bitmap, free_memory: u64, reserved_memory: u64, used_memory: u64, max_physical_address: PhysicalAddress) -> MemInfo {
        MemInfo {
            bitmap,
            free_memory,
            reserved_memory,
            used_memory,
            max_physical_address,
        }
    }

    pub fn move_out(&mut self) -> MemInfo {
        let output = MemInfo {
            bitmap: self.bitmap,
            free_memory: self.free_memory,
            reserved_memory: self.reserved_memory,
            used_memory: self.used_memory,
            max_physical_address: self.max_physical_address,
        };

        *self = MemInfo::default();

        return output;
    }
}

impl Default for MemInfo {
    fn default() -> MemInfo {
        MemInfo {
            bitmap: unsafe { Bitmap::new(0, null_mut()) },
            free_memory: 0,
            reserved_memory: 0,
            used_memory: 0,
            max_physical_address: PhysicalAddress::new(0),
        }
    }
}