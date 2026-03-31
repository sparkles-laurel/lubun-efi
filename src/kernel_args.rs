use core::ffi::c_void;
use uefi::table::cfg::{ACPI_GUID, ACPI2_GUID, ConfigTableEntry, SMBIOS_GUID, SMBIOS3_GUID};

#[derive(Copy, Clone, Debug)]
pub struct KernelArgs {
    acpi_ptr: *const c_void,
    smbios_prt: *const c_void,
    acpi_ver: u8,
    smbios_ver: u8,
}

impl Default for KernelArgs {
    fn default() -> Self {
        Self {
            acpi_ptr: 0 as *const c_void,
            smbios_ptr: 0 as *const c_void,
            acpi_ver: 0,
            smbios_ver: 0,
        }
    }
}

impl KernelArgs {
    // pub fn populate_from_cfg_table
}
