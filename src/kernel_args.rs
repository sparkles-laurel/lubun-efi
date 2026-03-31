use core::ffi::c_void;
use uefi::table::cfg::{ACPI_GUID, ACPI2_GUID, ConfigTableEntry, SMBIOS_GUID, SMBIOS3_GUID};

#[derive(Copy, Clone, Debug)]
pub struct KernelArgs {
    /// The physical address of the ACPI RSDP
    acpi_ptr: *const c_void,
    /// The physical address of the SMBIOS table
    smbios_ptr: *const c_void,
    /// The version of the ACPI RSDP pointed at by `self.acpi_ptr`
    acpi_ver: u8,
    /// The version of the SMBIOS table pointed at by `self.smbios_ptr`
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
    pub fn populate_from_cfg_table(&mut self, cfg_tables: &[ConfigTableEntry]) {
        // Iterate across the Config Tables, find the SMBIOS and ACPI tables, and populate their
        // pointers. Multiple versions of the standards could exist in memory, so this process will
        // search the entire table space and favor the highest-version implementation of the ACPI
        // or SMBIO standards, where they are present, and reflect this choice in a separate version
        // field.

        for cfg in cfg_tables {
            match cfg.guid {
                ACPI2_GUID => {
                    if self.acpi_ver < 2 {
                        self.acpi_ver = 2;
                        self.acpi_ptr = cfg.address;
                    }
                }
                ACPI_GUID => {
                    if self.acpi_ver < 1 {
                        self.acpi_ver = 1;
                        self.acpi_ptr = cfg.address;
                    }
                }
                SMBIOS3_GUID => {
                    if self.smbios_ver < 3 {
                        self.smbios_ver = 3;
                        self.smbios_ptr = cfg.address;
                    }
                }
                SMBIOS_GUID => {
                    if self.smbios_ver < 1 {
                        self.smbios_ver = 1;
                        self.smbios_ptr = cfg.address;
                    }
                }
                _ => {}
            }
        }
    }

    /// Returns the ACPI pointer and version as a pair
    pub fn get_acpi(&self) -> (*const c_void, u8) {
        (self.acpi_ptr, self.acpi_ver)
    }

    #[allow(unused)]
    /// Returns the SMBIOS pointer and version as a pair
    pub fn get_smbios(&self) -> (*const c_void, u8) {
        (self.smbios_ptr, self.smbios_ver)
    }
}
