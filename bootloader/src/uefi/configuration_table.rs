use core::ffi::c_void;

use acpi_system_tables::{RsdpV1, RsdpV2};
use r_efi::efi::Guid;

pub struct ConfigurationTable {
    num_entries: usize,
    configuration_table: *mut ConfigurationTableEntry,
}

impl ConfigurationTable {
    pub unsafe fn new(num_entries: usize, configuration_table: *mut ConfigurationTableEntry) -> ConfigurationTable {
        ConfigurationTable { num_entries, configuration_table }
    }

    pub fn get_rsdp_v1(&self) -> Option<RsdpV1> {
        for entry in self.iter() {
            if entry.get_type() == TableType::AcpiV1_0 {
                return entry.get_rsdp_v1();
            }
        }

        None
    }

    pub fn get_rsdp_v2(&self) -> Option<RsdpV2> {
        for entry in self.iter() {
            if entry.get_type() == TableType::AcpiV2_0 {
                return entry.get_rsdp_v2();
            }
        }

        None
    }

    pub fn get_entry(&self, index: usize) -> Option<ConfigurationTableEntry> {
        if index >= self.num_entries { return None; }

        unsafe { return Some(*(self.configuration_table.offset(index as isize))); }
    }

    pub fn iter(&self) -> ConfigurationTableIterator {
        ConfigurationTableIterator {
            asset_list: &self,
            current_index: 0,
            max_index: self.num_entries,
        }
    }
}

pub const ACPI_V1_0_RSDP_GUID: Guid = Guid::from_fields(
    0xEB9D2D30,
    0x2D88,
    0x11D3,
    0x9A,
    0x16,
    &[0x00, 0x90, 0x27, 0x3F, 0xC1, 0x4D]
);

pub const ACPI_V2_0_RSDP_GUID: Guid = Guid::from_fields(
    0x8868E871,
    0xE4F1,
    0x11D3,
    0xBC,
    0x22,
    &[0x00, 0x80, 0xC7, 0x3C, 0x88, 0x81]
);

#[derive(PartialEq, Debug)]
pub enum TableType {
    AcpiV1_0,
    AcpiV2_0,
    Unknown,
}

#[derive(Clone, Copy)]
pub struct ConfigurationTableEntry {
    vendor_guid: Guid,
    vendor_table: *mut c_void,
}

impl ConfigurationTableEntry {
    pub fn get_type(&self) -> TableType {
        match self.vendor_guid {
            ACPI_V1_0_RSDP_GUID => TableType::AcpiV1_0,
            ACPI_V2_0_RSDP_GUID => TableType::AcpiV2_0,
            _ => TableType::Unknown,
        }
    }

    pub fn get_rsdp_v1(&self) -> Option<RsdpV1> {
        if self.get_type() == TableType::AcpiV1_0 {
            let rsdp_ptr = self.vendor_table as *mut RsdpV1;
            unsafe { return Some(*rsdp_ptr) }
        }
        return None;
    }

    pub fn get_rsdp_v2(&self) -> Option<RsdpV2> {
        if self.get_type() == TableType::AcpiV2_0 {
            let rsdp_ptr = self.vendor_table as *mut RsdpV2;
            unsafe { return Some(*rsdp_ptr) }
        }
        return None;
    }
}

pub struct ConfigurationTableIterator<'a> {
    asset_list: &'a ConfigurationTable,
    current_index: usize,
    max_index: usize,
}

impl<'a> Iterator for ConfigurationTableIterator<'a> {
    type Item = ConfigurationTableEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index == self.max_index {
            return None;
        } else {
            let output = self.asset_list.get_entry(self.current_index);
            if output.is_some() {
                self.current_index += 1;
            }
            return output;
        }
    }
    
}