// Adapted from https://github.com/GRMorgan/rust-os/blob/master/libraries/bootinfo/src/bootinfo.rs

use x86_64_hardware::memory::VirtualAddress;

use crate::framebuffer::FrameBuffer;
use crate::meminfo::MemInfo;

// This could be changed to something more significant like the bootloader name
const BOOTINFO_MAGIC: [u8; 4] = [b'B', b'O', b'O', b'T'];

#[repr(C)]
pub struct BootInfo {
    magic: [u8; 4],
    pub framebuffer: FrameBuffer,
    pub page_table_memory_offset: u64,
    pub next_availiable_kernel_page: VirtualAddress,
    pub meminfo: MemInfo,
}

impl BootInfo {
    pub fn has_valid_magic(&self) -> bool { self.magic == BOOTINFO_MAGIC }
}

impl Default for BootInfo {
    fn default() -> BootInfo {
        BootInfo {
            magic: BOOTINFO_MAGIC,
            framebuffer: FrameBuffer::default(),
            page_table_memory_offset: 0,
            next_availiable_kernel_page: VirtualAddress::new(0),
            meminfo: MemInfo::default(),
        }
    }
}