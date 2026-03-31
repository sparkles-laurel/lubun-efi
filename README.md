# Lubun Operating System

**Lubun** is a lightweight, modern kernel research project implemented in Rust, designed with a focus on modern UEFI-based x86_64 hardware.

## Overview

The Lubun project aims to explore systems programming in Safely-Typed Rust while prioritizing modern hardware standards (UEFI, ACPI, PCIe) over legacy BIOS-era interfaces. This repository currently focuses on the **UEFI Loader**, which serves as the primary gateway for preparing the hardware environment before handing control to the Lubun kernel.

### The UEFI Loader (`uefiloader`)

The `uefiloader` is a native UEFI application that performs early-stage initialization:
- **Environment Setup**: Initializes UEFI services and early serial console logging.
- **RSDP Discovery**: Locates the ACPI Root System Description Pointer (RSDP) via UEFI configuration tables.
- **SMBIOS Parsing**: Identifies system management table addresses for hardware enumeration.
- **PCIe Mapping**: Early detection of PCI Express base memory-mapped configuration regions.
- **Kernel Argument Passing**: Constructs the `KernelArgs` structure to provide essential hardware pointers to the next stage.

## Features

- **Pure Rust**: Built using `no_std` and `no_main` for maximum control and safety.
- **UEFI Native**: Leverages modern UEFI protocols instead of legacy BIOS interrupts.
- **ACPI & SMBIOS**: Robust table discovery to support modern power management and hardware info.
- **PCIe Support**: Initial mappings for PCI Express address spaces.

## Getting Started

### Prerequisites

To build and run Lubun, you'll need:
- **Rust Toolchain**: `cargo` with `x86_64-unknown-uefi` target.
- **QEMU**: For system emulation during development.
- **OVMF**: UEFI firmware for execution within QEMU.

### Compilation

Build the loader as a UEFI application:

```bash
cargo build --target x86_64-unknown-uefi
```

### Local Execution (QEMU)

A testing environment is provided in the `qemu-testing` directory. To launch the UEFI loader in QEMU:

```bash
cd qemu-testing/
./runtest.sh
```

This will boot a virtual machine using the provided OVMF firmware and execute the freshly compiled loader.

## Project Structure

- `src/main.rs`: The UEFI entry point and main initialization logic.
- `src/kernel_args.rs`: Definitions for hardware pointers passed from loader to kernel.
- `src/cfg_table_type.rs`: Utilities for navigating UEFI configuration tables.
- `qemu-testing/`: Test bench containing boot scripts and firmware files.

## Development Status

Lubun is currently in an early **Initial Boot Stage**. The loader is capable of discovering fundamental ACPI resources and preparing the environment for a future kernel payload.

## License

See the [LICENSE](license) file for project-specific licensing information.
