use crate::{SystemDescriptionTable, SystemDescriptionTableHeader};

#[repr(C)]
struct ExtendedSystemDescriptionTableInternal {
    pub header: SystemDescriptionTableHeader,
    pub first_entry: u32,
}

pub struct ExtendedSystemDescriptionTable {
    xdst_ptr: *mut ExtendedSystemDescriptionTableInternal,
    mem_offset: u64,
}

impl ExtendedSystemDescriptionTable {
    pub unsafe fn new(physical_address: u64, offset: u64) -> ExtendedSystemDescriptionTable {
        let virtual_address = physical_address + offset;

        ExtendedSystemDescriptionTable {
            xdst_ptr: virtual_address as *mut ExtendedSystemDescriptionTableInternal,
            mem_offset: offset,
        }
    }

    pub fn num_entries(&self) -> usize {
        let table_len = unsafe { (*self.xdst_ptr).header.length() as usize };
        let size_of_entries = table_len - size_of::<SystemDescriptionTableHeader>();
        size_of_entries / size_of::<u64>()
    }

    pub fn get_entry(&self, index: usize) -> Option<SystemDescriptionTable> {
        if index >= self.num_entries() { return None; }

        let xsdt_ptr_u8 = self.xdst_ptr as *mut u8;
        let entry_ptr = unsafe {xsdt_ptr_u8.offset(size_of::<SystemDescriptionTableHeader>() as isize) } as *mut u64;
        let entry = unsafe { *(entry_ptr.offset(index as isize)) };
        unsafe {
            Some(SystemDescriptionTable::new(entry as u64, self.mem_offset))
        }
    }

    pub fn iter(&self) -> ExtendedSystemDescriptionTableIterator {
        ExtendedSystemDescriptionTableIterator {
            xsdt: self,
            current_index: 0,
            max_index: self.num_entries(),
        }
    }
}

pub struct ExtendedSystemDescriptionTableIterator<'a> {
    xsdt: &'a ExtendedSystemDescriptionTable,
    current_index: usize,
    max_index: usize,
}

impl<'a> Iterator for ExtendedSystemDescriptionTableIterator<'a> {
    type Item = SystemDescriptionTable;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index == self.max_index { return None; }

        let output = self.xsdt.get_entry(self.current_index);
        if output.is_some() {
            self.current_index += 1;
        }
        
        output
    }
}