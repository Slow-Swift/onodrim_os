use crate::memory::{PhysicalAddress, PHYSICAL_ADDRESS_MASK};

pub const PAGE_TABLE_MAX_INDEX: usize = 511;
const PRESENT_FLAG: u64 = 1 << 0;
const READ_WRITE_FLAG: u64 = 1 << 1;
const _USER_SUPERVISION_FLAG: u64 = 1 << 2;
const PAGE_SIZE_FLAG: u64 = 1 << 7;
const _EXECUTE_DISABLE_FLAG: u64 = 1 << 63;

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct PageTableEntry {
    entry: u64,
}

impl PageTableEntry {
    #[inline]
    pub fn is_unused(&self) -> bool { self.entry == 0 }

    #[inline]
    pub fn make_unused(&mut self) { self.entry = 0 }

    #[inline]
    pub fn present(&self) -> bool { self.are_flag_set(PRESENT_FLAG) }

    #[inline]
    pub fn set_present(&mut self, value: bool) {
        self.set_flags(PRESENT_FLAG, value);
    }

    #[inline]
    pub fn read_write(&self) -> bool { self.are_flag_set(READ_WRITE_FLAG) }

    #[inline]
    pub fn set_read_write(&mut self, value: bool) {
        self.set_flags(READ_WRITE_FLAG, value);
    }

    #[inline]
    pub fn page_size(&self) -> bool { self.are_flag_set(PAGE_SIZE_FLAG) }

    #[inline]
    pub fn set_page_size(&mut self, value: bool) {
        self.set_flags(PAGE_SIZE_FLAG, value);
    }

    #[inline]
    pub fn address(&self) -> PhysicalAddress {
        PhysicalAddress::new(self.entry & PHYSICAL_ADDRESS_MASK)
    }

    pub fn set_address(&mut self, addr: PhysicalAddress) {
        self.entry = (self.entry & !PHYSICAL_ADDRESS_MASK) | addr.as_u64();
    }

    #[inline]
    fn are_flag_set(&self, flags: u64) -> bool { (self.entry & flags) == flags }

    #[inline]
    fn set_flags(&mut self, flags: u64, value: bool) {
        if value {
            self.entry |= flags;
        } else {
            self.entry &= !flags;
        }
    }
}

impl Default for PageTableEntry {
    fn default() -> Self {
        PageTableEntry { entry: 0 }
    }
}

#[repr(align(4096))]
#[repr(C)]
pub struct PageTable {
    pub table: [PageTableEntry; 512],
}

impl PageTable {
    pub fn make_unused(&mut self) {
        for i in 0..self.table.len() {
            self.table[i].make_unused();
        }
    }

    pub fn copy_from(&mut self, other_page_table: &PageTable) {
        self.table = other_page_table.table;
    }

    pub fn get_entry(&self, index: usize) -> PageTableEntry {
        self.table[index]
    }

    /// ## Safety
    /// 
    /// The caller must ensure that this entry is valid
    pub unsafe fn set_entry(&mut self, index: usize, entry: PageTableEntry) {
        self.table[index] = entry;
    }
}

impl Default for PageTable {
    fn default() -> Self {
        let blank_entry = PageTableEntry::default();
        PageTable { table: [blank_entry; 512] }
    }
}