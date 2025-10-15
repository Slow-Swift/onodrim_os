use r_efi::efi;
use crate::uefi::memory_map::GetMemoryMapOutput;

use super::boot_services::BootServices;
use super::configuration_table::{ConfigurationTable, ConfigurationTableEntry};
use super::simple_text_output_protocol::SimpleTextOutputProtocol;

/// Provides a wrapper around the system table while boot services are running
pub struct BootSystemTable {
    system_table_ptr: *const efi::SystemTable,
    pub boot_services: BootServices
}

impl BootSystemTable {

    /// Create a new wrapper around the EFI System table.
    /// 
    /// ## Safety
    /// It is up to the caller to ensure that the system table pointer actually
    /// points to a system table and that exit_boot_services has not yet been called
    pub unsafe fn new(system_table_ptr: *const efi::SystemTable) -> BootSystemTable {
        BootSystemTable { 
            system_table_ptr,
            boot_services: BootServices::new((*system_table_ptr).boot_services)
        }
    }

    pub fn con_out(&self) -> SimpleTextOutputProtocol {
        // Safety: This should be safe assuming this is a valid system table
        unsafe {
            return SimpleTextOutputProtocol::new((*self.system_table_ptr).con_out)
        }
    }

    pub fn get_configuration_table(&self) -> ConfigurationTable {
        // Safety: This should be safe assuming this is a valid system table
        unsafe {
            let num_entries = (*self.system_table_ptr).number_of_table_entries;
            let table_entries = (*self.system_table_ptr).configuration_table as *mut ConfigurationTableEntry;
            ConfigurationTable::new(num_entries, table_entries)
        }
    }

    /// Exits boot services and switches the boot system table for a runtime system table
    /// Also collects a map of the current memory layout
    /// 
    /// ## Safety
    /// It is up to the caller to ensure the memory map is properly used so that no allocated memory is overwritten
    pub unsafe fn exit_boot_services(mut self, image_handle: efi::Handle) -> 
        Result<(RuntimeSystemTable, GetMemoryMapOutput), efi::Status>{
        let mem_info = self.boot_services.get_memory_map()?;
        self.boot_services.exit_boot_services(image_handle, mem_info.map_key)?;
        let runtime_table = RuntimeSystemTable::new(self.system_table_ptr);
        Ok((runtime_table, mem_info))
    }
}

/// Provides a wrapper around the system table after boot services are stopped
pub struct RuntimeSystemTable {
    system_table_ptr: *const efi::SystemTable
}

impl RuntimeSystemTable {

    /// Create a new wrapper around the EFI System table.
    /// 
    /// ## Safety
    /// It is up to the caller to ensure that the system table pointer actually
    /// points to a system table
    pub unsafe fn new(system_table_ptr: *const efi::SystemTable) -> RuntimeSystemTable {
        RuntimeSystemTable { 
            system_table_ptr
        }
    }

    pub fn get_configuration_table(&self) -> ConfigurationTable {
        // Safety: This should be safe assuming this is a valid system table
        unsafe {
            let num_entries = (*self.system_table_ptr).number_of_table_entries;
            let table_entries = (*self.system_table_ptr).configuration_table as *mut ConfigurationTableEntry;
            ConfigurationTable::new(num_entries, table_entries)
        }
    }
}