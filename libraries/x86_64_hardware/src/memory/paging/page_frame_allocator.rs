use bitmap::Bitmap;
use spin::mutex::Mutex;

use crate::memory::{PhysicalAddress, PAGE_SIZE};

#[derive(Debug)]
pub enum AllocError {
    OutOfMemory,
    DoubleFree,
    AlreadyUsed,
}

pub trait FrameAllocator {
    fn request_page(&self) -> Result<PhysicalAddress, AllocError>;
    fn free_page(&self, address: PhysicalAddress) -> Result<(), AllocError>;
}

struct PageFrameAllocatorInner {
    pub page_bitmap: Bitmap,
    free_memory: u64,
    used_memory: u64,
    last_allocated_page: usize,
}

impl PageFrameAllocatorInner {
    pub const unsafe fn new_uninitialized() -> PageFrameAllocatorInner {
        PageFrameAllocatorInner {
            page_bitmap: Bitmap::new_uninitialized(),
            free_memory: 0,
            used_memory: 0,
            last_allocated_page: 0,
        }
    }

    pub unsafe fn init(&mut self, page_bitmap: *mut Bitmap, free_memory: u64, used_memory: u64) {
        self.page_bitmap = *page_bitmap;
        self.free_memory = free_memory;
        self.used_memory = used_memory;
        self.last_allocated_page = 0;
    }

    pub fn lock_page(&mut self, address: PhysicalAddress) -> Result<(), AllocError>{
        let page_number = address.as_usize() / PAGE_SIZE as usize;
        if self.page_bitmap.get(page_number) { return Err(AllocError::AlreadyUsed); }

        if self.page_bitmap.set(page_number, true) {
            self.free_memory -= PAGE_SIZE;
            self.used_memory += PAGE_SIZE;
        }

        Ok(())
    }

    fn request_page(&mut self) -> Result<PhysicalAddress, AllocError> {
        for index in self.last_allocated_page..self.page_bitmap.size() * 8 {
            if !self.page_bitmap.get(index) {
                self.last_allocated_page = index;
                let addr = PhysicalAddress::new(index as u64 * PAGE_SIZE);
                self.lock_page(addr)?;
                return Ok(addr);
            }
        }

        Err(AllocError::OutOfMemory)
    }

    fn free_page(&mut self, address: PhysicalAddress) -> Result<(), AllocError> {
        let page_number = address.as_usize() / PAGE_SIZE as usize;
        
        if !self.page_bitmap.get(page_number) { return Err(AllocError::DoubleFree) }

        // This should always be true since self.page_bitmap.get returned false
        if self.page_bitmap.set(page_number, false) {
            self.free_memory += PAGE_SIZE;
            self.used_memory -= PAGE_SIZE;
            if self.last_allocated_page > page_number {
                self.last_allocated_page = page_number
            }
        }

        Ok(())
    }
}

pub struct PageFrameAllocator {
    lockable_allocator: Mutex<PageFrameAllocatorInner>,
}

impl PageFrameAllocator {
    pub unsafe fn new_from_bitmap(page_bitmap: *mut Bitmap, free_memory: u64, used_memory: u64) -> PageFrameAllocator {
        PageFrameAllocator::new(
            PageFrameAllocatorInner {
                page_bitmap: *page_bitmap,
                free_memory,
                used_memory,
                last_allocated_page: 0,
            }
        )
    }

    const fn new(inner: PageFrameAllocatorInner) -> PageFrameAllocator {
        PageFrameAllocator { lockable_allocator: Mutex::new(inner) }
    }

    pub const fn new_uninitialized() -> PageFrameAllocator {
        PageFrameAllocator::new(
            unsafe { PageFrameAllocatorInner::new_uninitialized() }
        )
    }

    pub unsafe fn init(&mut self, page_bitmap: *mut Bitmap, free_memory: u64, used_memory: u64) {
        self.lockable_allocator.lock().init(page_bitmap, free_memory, used_memory);
    }

    pub fn page_bitmap(&self) -> Bitmap {
        self.lockable_allocator.lock().page_bitmap
    }

    pub fn get_free_ram(&self) -> u64 {
        self.lockable_allocator.lock().free_memory
    }

    pub fn get_used_ram(&self) -> u64 {
        self.lockable_allocator.lock().used_memory
    }

    pub fn free_pages(&self, address: PhysicalAddress, page_count: usize) -> Result<(), AllocError> {
        let mut inner = self.lockable_allocator.lock();
        for i in 0..page_count {
            // ? May want to keep freeing even on error.
            inner.free_page(address.increment_pages(i as u64))?;
        }
        Ok(())
    }

    pub fn lock_page(&self, address: PhysicalAddress) -> Result<(), AllocError> {
        self.lockable_allocator.lock().lock_page(address)
    }

    pub fn lock_pages(&self, address: PhysicalAddress, page_count: usize) -> Result<(), AllocError> {
        let mut inner = self.lockable_allocator.lock();
        for i in 0..page_count {
            inner.lock_page(address.increment_pages(i as u64))?;
        }

        Ok(())
    }
}

impl FrameAllocator for PageFrameAllocator {
    fn request_page(&self) -> Result<PhysicalAddress, AllocError> {
        self.lockable_allocator.lock().request_page()
    }

    fn free_page(&self, address: PhysicalAddress) -> Result<(), AllocError> {
        self.lockable_allocator.lock().free_page(address)
    }
}