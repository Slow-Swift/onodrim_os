use crate::{com1_println, memory::{PhysicalAddress, VirtualAddress}};

use super::{AllocError, FrameAllocator, PageTable, PageTableEntry, PAGE_TABLE_MAX_INDEX};

pub const MEM_1G: u64 = 1024 * 1024 * 1024;
pub const MAX_MEM_SIZE: u64 = 512 * MEM_1G;

pub struct PageTableManager {
    p4: PhysicalAddress,
    offset: u64,
}

impl PageTableManager {
    pub fn new_from_allocator(allocator: &mut impl FrameAllocator, offset: u64) -> PageTableManager {
        // TODO: Make this return a result
        let p4_paddr = allocator.request_page().expect("Could not allocate memory");
        let p4_vaddr = p4_paddr.get_virtual_address_at_offset(offset);

        // TODO: Should really check that this is not larger than a page
        let p4_table = unsafe { p4_vaddr.get_mut_ptr::<PageTable>() };
        unsafe { (*p4_table).make_unused(); }
        PageTableManager::new(p4_paddr, offset)
    }

    pub fn new_from_cr3(offset: u64) -> PageTableManager {
        let p4_addr: u64;

        unsafe {
            core::arch::asm!("mov {}, cr3", out(reg) p4_addr, options(nomem, nostack, preserves_flags))
        }

        com1_println!("Firmware page tabel address: {:#X}", p4_addr);

        PageTableManager::new(PhysicalAddress::new(p4_addr), offset)
    }

    pub fn new(p4: PhysicalAddress, offset: u64) -> PageTableManager {
        PageTableManager { p4, offset }
    }

    pub fn get_p4(&self) -> &PageTable {
        let virt_address: VirtualAddress = self.translate_address(self.p4);
        unsafe { &*(virt_address.get_mut_ptr::<PageTable>()) }
    }

    pub fn get_p4_address(&self) -> PhysicalAddress { self.p4 }

    pub fn release_tables(&self, allocator: &mut impl FrameAllocator) -> Result<(), AllocError>{
        for index in 0..511 {
            self.unmap_p4_index(index, allocator)?;
        }

        allocator.free_page(self.p4)?;

        Ok(())
    }

    /// Sets the offset this PageTableManager uses to handle page
    /// virtual address lookup
    /// 
    /// ## Safety
    /// 
    /// The caller must be sure the current live page table has
    /// the system memory offset mapped at the given offset
    pub unsafe fn set_offset(&mut self, offset: u64) {
        self.offset = offset;
    }

    fn translate_address(&self, physical_address: PhysicalAddress) -> VirtualAddress {
        return physical_address.get_virtual_address_at_offset(self.offset);
    }

    pub unsafe fn activate_page_table(&self) {
        core::arch::asm!("mov cr3, {}", in(reg) self.p4.as_u64(), options(nostack, preserves_flags));
    }

    pub fn map_memory_pages(
        &self, virtual_address: VirtualAddress, 
        physical_address: PhysicalAddress, 
        num_pages: u64, 
        allocator: &mut impl FrameAllocator
    ) -> Result<(), AllocError>{
        for page in 0..num_pages {
            let cur_paddr = physical_address.increment_pages(page);
            let cur_vaddr = virtual_address.increment_pages(page);
            self.map_memory(cur_vaddr, cur_paddr, allocator)?;
        }

        Ok(())
    }

    pub fn map_memory(
        &self, virtual_address: VirtualAddress, physical_address: PhysicalAddress, allocator: &mut impl FrameAllocator
    ) -> Result<(), AllocError> {
        let p4_ptr = unsafe { self.translate_address(self.p4).get_mut_ptr::<PageTable>() };
        let mut p4_table_entry = unsafe { (*p4_ptr).table[virtual_address.p4_index()] };
        if !p4_table_entry.present() {
            let p3_addr = self.create_and_map_p3(virtual_address, physical_address, allocator)?;
            p4_table_entry.make_unused();
            p4_table_entry.set_address(p3_addr);
            p4_table_entry.set_present(true);
            p4_table_entry.set_read_write(true);
            unsafe { (*p4_ptr).table[virtual_address.p4_index()] = p4_table_entry; }
        } else {
            let p3_ptr = unsafe { self.translate_address(p4_table_entry.address()).get_mut_ptr::<PageTable>() };
            self.map_p3(p3_ptr, virtual_address, physical_address, allocator)?;
        }

        Ok(())
    }

    fn create_and_map_p3(&self, virtual_address: VirtualAddress, physical_address: PhysicalAddress, allocator: &mut impl FrameAllocator) -> Result<PhysicalAddress, AllocError> {
        let output = allocator.request_page()?;
        let p3_ptr = unsafe { self.translate_address(output).get_mut_ptr::<PageTable>() };
        unsafe { (*p3_ptr).make_unused() }
        self.map_p3(p3_ptr, virtual_address, physical_address, allocator)?;

        Ok(output)
    }

    fn map_p3(
        &self, p3_ptr: *mut PageTable, virtual_address: VirtualAddress, physical_address: PhysicalAddress, allocator: &mut impl FrameAllocator
    ) -> Result<(), AllocError> {
        let mut p3_table_entry = unsafe { (*p3_ptr).table[virtual_address.p3_index()] };
        if !p3_table_entry.present() {
            let p2_addr = self.create_and_map_p2(virtual_address, physical_address, allocator)?;
            p3_table_entry.set_address(p2_addr);
            p3_table_entry.set_present(true);
            p3_table_entry.set_read_write(true);
            unsafe { (*p3_ptr).table[virtual_address.p3_index()] = p3_table_entry; }
        } else {
            let p2_ptr = unsafe { self.translate_address(p3_table_entry.address()).get_mut_ptr::<PageTable>() };
            self.map_p2(p2_ptr, virtual_address, physical_address, allocator)?;
        }

        Ok(())
    }

    fn create_and_map_p2(&self, virtual_address: VirtualAddress, physical_address: PhysicalAddress, allocator: &mut impl FrameAllocator) -> Result<PhysicalAddress, AllocError> {
        let output = allocator.request_page()?;
        let p2_ptr = unsafe { self.translate_address(output).get_mut_ptr::<PageTable>() };
        unsafe { (*p2_ptr).make_unused() }
        self.map_p2(p2_ptr, virtual_address, physical_address, allocator)?;

        Ok(output)
    }

    fn map_p2(
        &self, p2_ptr: *mut PageTable, virtual_address: VirtualAddress, physical_address: PhysicalAddress, allocator: &mut impl FrameAllocator
    ) -> Result<(), AllocError> {
        let mut p2_table_entry = unsafe { (*p2_ptr).table[virtual_address.p2_index()] };
        if !p2_table_entry.present() {
            let p1_addr = self.create_and_map_p1(virtual_address, physical_address, allocator)?;
            p2_table_entry.set_address(p1_addr);
            p2_table_entry.set_present(true);
            p2_table_entry.set_read_write(true);
            unsafe { (*p2_ptr).table[virtual_address.p2_index()] = p2_table_entry; }
        } else {
            let p1_ptr = unsafe { self.translate_address(p2_table_entry.address()).get_mut_ptr::<PageTable>() };
            self.map_p1(p1_ptr, virtual_address, physical_address);
        }

        Ok(())
    }

    fn create_and_map_p1(&self, virtual_address: VirtualAddress, physical_address: PhysicalAddress, allocator: &mut impl FrameAllocator) -> Result<PhysicalAddress, AllocError> {
        let output = allocator.request_page()?;
        let p1_ptr = unsafe { self.translate_address(output).get_mut_ptr::<PageTable>() };
        unsafe { (*p1_ptr).make_unused() }
        self.map_p1(p1_ptr, virtual_address, physical_address);

        Ok(output)
    }

    fn map_p1(
        &self, p1_ptr: *mut PageTable, virtual_address: VirtualAddress, physical_address: PhysicalAddress
    ) {
        let mut p1_table_entry = unsafe { (*p1_ptr).table[virtual_address.p1_index()] };
       
        p1_table_entry.set_address(physical_address);
        p1_table_entry.set_present(true);
        p1_table_entry.set_read_write(true);
        unsafe { (*p1_ptr).table[virtual_address.p1_index()] = p1_table_entry; }
    }

    fn get_page_table_entry(&self, virtual_address: VirtualAddress) -> Option<&mut PageTableEntry> {
        let p4_ptr = unsafe { self.translate_address(self.p4).get_mut_ptr::<PageTable>() };
        
        let p4_table_entry = unsafe { (*p4_ptr).table[virtual_address.p4_index()] };
        if !p4_table_entry.present() { return None }
        let p3_ptr = unsafe { self.translate_address(p4_table_entry.address()).get_mut_ptr::<PageTable>() };

        // TODO: Implement larger page sizes
        let p3_table_entry = unsafe { (*p3_ptr).table[virtual_address.p3_index()] };
        if !p3_table_entry.present() { return None }
        let p2_ptr = unsafe { self.translate_address(p3_table_entry.address()).get_mut_ptr::<PageTable>() };

        // TODO: Implement larger page sizes
        let p2_table_entry = unsafe { (*p2_ptr).table[virtual_address.p2_index()] };
        if !p2_table_entry.present() { return None }
        let p1_ptr = unsafe { self.translate_address(p2_table_entry.address()).get_mut_ptr::<PageTable>() };

        unsafe { Some(&mut (*p1_ptr).table[virtual_address.p1_index()]) }
    }

    pub fn get_page_physical_address(&self, virtual_address: VirtualAddress) -> Option<PhysicalAddress> {
        let page_table_entry = self.get_page_table_entry(virtual_address)?;

        Some(page_table_entry.address())
    }

    pub fn unmap_p4_index(&self, p4_index: usize, allocator: &mut impl FrameAllocator) -> Result<(), AllocError> {
        // TODO: Should probably error here
        if p4_index > PAGE_TABLE_MAX_INDEX { return Ok(()); }

        let p4_ptr = unsafe { self.translate_address(self.p4).get_mut_ptr::<PageTable>() };
        let mut p4_entry = unsafe { (*p4_ptr).get_entry(p4_index) };

        if p4_entry.present() {
            let p3_phys_address = p4_entry.address();
            let p3_ptr = unsafe { self.translate_address(p3_phys_address).get_mut_ptr::<PageTable>() };

            // ? Should maybe continue freeing
            self.unmap_p3(p3_ptr, allocator)?;
            p4_entry.make_unused();
            unsafe { (*p4_ptr).set_entry(p4_index, p4_entry); }

            // ? Should maybe continue freeing
            allocator.free_page(p3_phys_address)?;
        }

        Ok(())
    }

    fn unmap_p3(&self, p3_ptr: *mut PageTable, allocator: &mut impl FrameAllocator)  -> Result<(), AllocError> {
        for index in 0..PAGE_TABLE_MAX_INDEX {
            let mut p3_entry = unsafe { (*p3_ptr).get_entry(index) };

            if p3_entry.present() {
                if p3_entry.page_size() {
                    p3_entry.make_unused();
                    unsafe { (*p3_ptr).set_entry(index, p3_entry) };
                } else {
                    let p2_phys_address = p3_entry.address();
                    let p2_ptr = unsafe { self.translate_address(p2_phys_address).get_mut_ptr::<PageTable>() };

                    // ? Should maybe continue freeing
                    self.unmap_p2(p2_ptr, allocator)?;
                    p3_entry.make_unused();
                    unsafe { (*p3_ptr).set_entry(index, p3_entry); }

                    // ? Should maybe continue freeing
                    allocator.free_page(p2_phys_address)?;
                }
            }
        }

        Ok(())
    }

    fn unmap_p2(&self, p2_ptr: *mut PageTable, allocator: &mut impl FrameAllocator) -> Result<(), AllocError> {
        for index in 0..PAGE_TABLE_MAX_INDEX {
            let mut p2_entry = unsafe { (*p2_ptr).get_entry(index) };

            if p2_entry.present() {
                if p2_entry.page_size() {
                    p2_entry.make_unused();
                    unsafe { (*p2_ptr).set_entry(index, p2_entry); }
                } else {
                    let p1_phys_address = p2_entry.address();
                    let p1_ptr = unsafe { self.translate_address(p1_phys_address).get_mut_ptr::<PageTable>() };
                    unsafe { (*p1_ptr).make_unused(); }
                    p2_entry.make_unused();
                    unsafe { (*p2_ptr).set_entry(index, p2_entry); }

                    // ? Should maybe continue freeing
                    allocator.free_page(p1_phys_address)?;
                }
            }
        }

        Ok(())
    }

}