use crate::{SystemDescriptionTable, SystemDescriptionTableHeader};

#[repr(C)]
struct RootSystemDescriptionTableInternal {
    pub header: SystemDescriptionTableHeader,
    pub first_entry: u32,
}

pub struct RootSystemDescriptionTable {
    rdst_ptr: *mut RootSystemDescriptionTableInternal,
    mem_offset: u64,
}

impl RootSystemDescriptionTable {
    pub unsafe fn new(physical_address: u32, offset: u64) -> RootSystemDescriptionTable {
        let virtual_address = physical_address as u64 + offset;

        RootSystemDescriptionTable {
            rdst_ptr: virtual_address as *mut RootSystemDescriptionTableInternal,
            mem_offset: offset,
        }
    }

    pub fn num_entries(&self) -> usize {
        let table_len = unsafe { (*self.rdst_ptr).header.length() as usize };
        let size_of_entries = table_len - size_of::<SystemDescriptionTableHeader>();
        size_of_entries / size_of::<u32>()
    }

    pub fn get_entry(&self, index: usize) -> Option<SystemDescriptionTable> {
        if index >= self.num_entries() { return None; }

        let rsdt_ptr_u8 = self.rdst_ptr as *mut u8;
        let entry_ptr = unsafe {rsdt_ptr_u8.offset(size_of::<SystemDescriptionTableHeader>() as isize) } as *mut u32;
        let entry = unsafe { *(entry_ptr.offset(index as isize)) };
        unsafe {
            Some(SystemDescriptionTable::new(entry as u64, self.mem_offset))
        }
    }

    pub fn iter(&self) -> RootSystemDescriptionTableIterator {
        RootSystemDescriptionTableIterator {
            rsdt: self,
            current_index: 0,
            max_index: self.num_entries(),
        }
    }
}

pub struct RootSystemDescriptionTableIterator<'a> {
    rsdt: &'a RootSystemDescriptionTable,
    current_index: usize,
    max_index: usize,
}

impl<'a> Iterator for RootSystemDescriptionTableIterator<'a> {
    type Item = SystemDescriptionTable;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index == self.max_index { return None; }

        let output = self.rsdt.get_entry(self.current_index);
        if output.is_some() {
            self.current_index += 1;
        }
        
        output
    }
}