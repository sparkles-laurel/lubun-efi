#![no_main]
#![no_std]

mod cfg_table_type;
mod identity_acpi_handler;
mod kernel_args;
// Use the abstracted log interface for console output
use log::{error, info, warn};

// Import a bunch of commonly-used UEFI symbols exported by the crate
use uefi::prelude::*;

use acpi::AcpiTables;
use acpi::mcfg::PciConfigRegions;

use crate::identity_acpi_handler::IdentityAcpiHandler;
use crate::kernel_args::KernelArgs;

fn wait_for_keypress(st: &mut SystemTable<Boot>) -> uefi::Result {
    info!("Press a key to contine...");
    st.stdin().reset(true)?;

    let mut key_press_event = unsafe { [st.stdin().wait_for_key_event().unsafe_clone()] };
    st.boot_services()
        .wait_for_event(&mut key_press_event)
        .unwrap();
    Ok(())
}

// Tell the uefi crate that this function will be our entrypoint
#[entry]
// Declare "hello_main" to accept two arguments, and use the type definitions provided by uefi
fn hello_main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    // In order to use any of the services (input, output, etc...), they need to be manually
    // initialized by the UEFI program
    uefi_services::init(&mut system_table).unwrap();

    info!("Image Handle: {:#018x}", image_handle.as_ptr() as usize);
    info!(
        "System Table: {:#018x}",
        core::ptr::addr_of!(system_table) as usize
    );
    info!(
        "UEFI Revision: {}.{}",
        system_table.uefi_revision().major(),
        system_table.uefi_revision().minor()
    );

    let mut karg = KernelArgs::default();
    info!("Empty karg: {:?}", karg);
    karg.populate_from_cfg_table(system_table.config_table());
    info!("Populated karg: {:?}", karg);

    let ih = IdentityAcpiHandler;
    let acpi_tables = unsafe { AcpiTables::from_rsdp(ih, karg.get_acpi().0 as usize) }.unwrap();

    info!("ACPI Revision: {}", acpi_tables.revision);

    let pcie_cfg = PciConfigRegions::new(&acpi_tables).unwrap();
    let pcie_first_addr = pcie_cfg.physical_address(0, 0, 0, 0).unwrap();

    info!("PCIe(0, 0, 0, 0): {:#018x}", pcie_first_addr);

    wait_for_keypress(&mut system_table).unwrap();

    // Tell the UEFI firmware we exited without error
    Status::SUCCESS
}
