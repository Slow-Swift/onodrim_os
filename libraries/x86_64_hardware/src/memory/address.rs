pub const PAGE_SIZE: u64 = 0x1000;
pub const PHYSICAL_ADDRESS_MASK: u64 = 0x000F_FFFF_FFFF_F000;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(transparent)]
pub struct PhysicalAddress(u64);

impl PhysicalAddress {
    #[inline]
    pub const fn new(addr: u64) -> PhysicalAddress {
        PhysicalAddress(addr & PHYSICAL_ADDRESS_MASK)
    }

    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    #[inline]
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }

    #[inline]
    pub const fn get_virtual_address_at_offset(&self, offset: u64) -> VirtualAddress {
        VirtualAddress::new(self.0 + offset)
    }
}

impl PartialOrd for PhysicalAddress {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

pub const VIRTUAL_ADDRESS_MASK: u64 = 0x0000_FFFF_FFFF_FFFF;
pub const VIRTUAL_ADDRESS_SIGN_EXTENSION_MASK: u64 = 0xFFFF_0000_0000_0000;
pub const MAX_VIRTUAL_ADDRESS: u64 = 0xFFFF_FFFF_FFFF_FFFF;
pub const PAGE_OFFSET_MASK: u64 = 0xFFF;
pub const VIRTUAL_ADDRESS_HIGH_BIT: u64 = 1 << 47;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(transparent)]
pub struct VirtualAddress(u64);

impl VirtualAddress {
    #[inline]
    pub const fn new(mut addr: u64) -> VirtualAddress {
        addr &= VIRTUAL_ADDRESS_MASK;

        if addr & VIRTUAL_ADDRESS_HIGH_BIT != 0 {
            addr |= VIRTUAL_ADDRESS_SIGN_EXTENSION_MASK;
        }

        VirtualAddress(addr)
    }
}