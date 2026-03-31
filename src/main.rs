#![no_std]
#![no_main]

use log::info;
use uefi::prelude::*;

#[entry]
fn hello_main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    system_table.stdout().clear().unwrap();

    info!("Hello from the UEFI loader!");
    info!("Display initialized successfully.");

    loop {
        system_table.boot_services().stall(1_000_000);
    }
}
