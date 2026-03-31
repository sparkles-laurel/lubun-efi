#![no_std]
#![no_main]

use log::{error, info, warn};
use uefi::prelude::*;
use uefi_services::system_table;
mod cfg_table_type;
use crate::cfg_table_type::CfgTableType;

#[entry]
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

    for cfg in system_table.config_table() {
        let cfg_table_name: CfgTableType = cfg.guid.into();
        info!(
            "Ptr: {:#018x}, GUID: {}",
            cfg.address as usize, cfg_table_name
        );
    }

    wait_for_keypress(&mut system_table).unwrap();

    // Tell the UEFI firmware we exited without error
    Status::SUCCESS
}

fn wait_for_keypress(st: &mut SystemTable<Boot>) -> uefi::Result {
    info!("Press a key to continue...");
    _ = st.stdin().reset(true);

    let mut key_press_event = unsafe { [st.stdin().wait_for_key_event().unsafe_clone()] };
    st.boot_services()
        .wait_for_event(&mut key_press_event)
        .unwrap();

    Ok(())
}
