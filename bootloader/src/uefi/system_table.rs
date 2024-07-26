
use r_efi::efi;
use super::boot_services::BootServices;
use super::configuration_table::{ConfigurationTable, ConfigurationTableEntry};
use super::simple_text_output_protocol::SimpleTextOutputProtocol;

#[derive(Copy, Clone)]
pub struct SystemTableWrapper {
    system_table_ptr: *const efi::SystemTable,
}

impl SystemTableWrapper {

    pub unsafe fn new(system_table_ptr: *const efi::SystemTable) -> SystemTableWrapper {
        SystemTableWrapper { system_table_ptr }
    }

    pub fn boot_services(&self) -> BootServices {
        unsafe {
            return BootServices::new((*self.system_table_ptr).boot_services)
        }
    }

    pub fn con_out(&self) -> SimpleTextOutputProtocol {
        unsafe {
            return SimpleTextOutputProtocol::new((*self.system_table_ptr).con_out)
        }
    }

    pub fn get_configuration_table(&self) -> ConfigurationTable {
        unsafe {
            let num_entries = (*self.system_table_ptr).number_of_table_entries;
            let table_entries = (*self.system_table_ptr).configuration_table as *mut ConfigurationTableEntry;
            ConfigurationTable::new(num_entries, table_entries)
        }
    }
}