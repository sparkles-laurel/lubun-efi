# Copilot Instructions for Lubun UEFI Loader

## Build and Test

### Build for UEFI target
```bash
cargo build --target x86_64-unknown-uefi
```

Debug build (default):
```bash
cargo build --target x86_64-unknown-uefi --debug
```

### Run in QEMU
```bash
cd qemu-testing/
./runtest.sh
```

This script:
1. Copies the compiled `.efi` binary to the ESP (EFI System Partition)
2. Boots QEMU with OVMF firmware (UEFI emulation)
3. Connects serial output to stdio for console output

The binary is built as `target/x86_64-unknown-uefi/debug/uefiloader.efi` and copied to `esp/EFI/BOOT/BOOTX64.EFI`.

### No automated tests currently exist
The project is in early-stage development. Testing is manual via QEMU execution and serial console inspection.

## High-Level Architecture

**Lubun** is a lightweight, UEFI-native operating system kernel research project. This repository contains the **UEFI Loader** — a native UEFI application that runs immediately after firmware boot to initialize hardware.

### Boot Flow

1. **UEFI Firmware Boot** → Loads and executes `uefiloader.efi`
2. **UEFI Initialization** → Calls `uefi_services::init()` to set up console logging and UEFI services
3. **Hardware Discovery** → Uses UEFI configuration tables to locate:
   - **ACPI RSDP** (Root System Description Pointer) — Foundation for hardware discovery and power management
   - **SMBIOS** — System management information (hardware details)
   - **PCIe ECAM** — PCI Express Enhanced Configuration Address Mapping for device enumeration
4. **Memory Map Extraction** → Retrieves UEFI boot-time memory map and converts it to kernel-consumable format
5. **KernelArgs Construction** → Packages all discovered hardware pointers into a data structure for the next stage (kernel)
6. **Handoff** → Returns `Status::SUCCESS` to firmware; hardware initialization is ready for kernel execution

### Key Data Structure: `KernelArgs`

Located in `src/kernel_args.rs`, this structure is the primary interface between the loader and the kernel:

```rust
pub struct KernelArgs {
    acpi_ptr: *const c_void,      // Physical address of ACPI RSDP
    acpi_ver: u8,                  // ACPI version (1 or 2)
    smbios_ptr: *const c_void,    // Physical address of SMBIOS table
    smbios_ver: u8,                // SMBIOS version (1 or 3)
    pcie_ptr: *mut c_void,        // Physical address of PCIe ECAM space
    memmap_ptr: *mut OSMemEntry,  // Pointer to memory map array
    memmap_entries: usize,        // Count of entries in memory map
}
```

Methods like `populate_from_cfg_table()` and getters (`get_acpi()`, `get_smbios()`, etc.) populate this structure with discovered hardware state.

### ACPI & SMBIOS Version Preference

The loader **favors newer standards** when multiple versions are available:
- ACPI: Prefers ACPI2 over ACPI1
- SMBIOS: Prefers SMBIOS3 over SMBIOS1

This preference is implemented in `populate_from_cfg_table()` with version checking (`if self.acpi_ver < 2`).

### Memory Map Conversion

The `get_mm()` function in `kernel_args.rs` performs a critical transformation:

1. Requests UEFI memory map via boot services
2. Allocates UEFI boot pool memory (must be done while still in UEFI context)
3. Converts `MemoryDescriptor` (UEFI native format) to `OSMemEntry` (kernel format via `From` trait)
4. Returns both pointer and count for kernel use

This must happen **before exiting UEFI boot services**, as the kernel will use this data after services are unavailable.

## Key Conventions

### `no_std` Rust Environment
- **No standard library** — Uses `#![no_std]` and `#![no_main]`
- **Manual memory management** — Allocations via UEFI boot services, no heap allocator
- **Raw pointers throughout** — Kernel args use raw pointers for inter-stage communication; unsafe code is expected
- **Entry point is `hello_main`** (marked with `#[entry]`), not Rust's standard `main()`

### Logging and Console

- Uses the `log` crate abstraction (via `uefi_services::init()`)
- Available as `info!()`, `warn!()`, `error!()` macros after initialization
- Logs go to UEFI serial console
- Console output is **not guaranteed** until `uefi_services::init()` completes; be careful with early logging

### UEFI Table Abstractions

The `uefi` crate provides type-safe wrappers:
- `SystemTable<Boot>` — Main UEFI interface; obtained at entry point
- `ConfigTableEntry` — Entries in the firmware configuration table
- GUIDs like `ACPI_GUID`, `ACPI2_GUID`, `SMBIOS_GUID`, `SMBIOS3_GUID` — Used to identify configuration table entries

### ACPI Handling

- Uses the `acpi` crate for table parsing
- Requires custom ACPI handler implementation: `IdentityAcpiHandler` in `src/identity_acpi_handler.rs`
- The handler must implement the physical-to-virtual address mapping required by the ACPI parser
- PCIe configuration is discovered via ACPI's MCFG (PCI Memory-Mapped Configuration) table

### Raw Pointer Safety Pattern

Kernel args are passed as raw pointers because:
1. The kernel runs in a different execution context (no longer in UEFI services)
2. The struct may be passed via assembly or across module boundaries
3. Version compatibility requires fixed memory layout

When working with kernel args:
- **Treat pointers as volatile across execution context changes**
- Ensure the struct layout is preserved (no reordering fields)
- Document assumptions about pointer lifetime (e.g., memory allocated by UEFI persists for kernel use)

### Testing in QEMU

OVMF firmware provides UEFI 2.7+ with:
- Full boot services implementation
- Serial console via `-serial stdio`
- Support for configuration tables including ACPI, SMBIOS, and PCIe
- `-enable-kvm` and `-machine q35` provide modern x86_64 emulation with PCIe support

Output appears directly in the terminal; watch for `info!()` and `error!()` messages from the loader.

## Roadmap: Evolution to EFI Stub Microkernel

This roadmap outlines the path from a minimal boot-stage loader to a full UEFI-bootable microkernel architecture inspired by efistubs (like Linux's EFI stub), where the kernel itself can be booted directly by UEFI firmware with minimal hardware setup delegated to a thin abstraction layer.

### Phase 1: Kernel Initialization & Memory Management

**Goal:** Create the core kernel that consumes `KernelArgs` and establishes a safe runtime environment.

**Tasks:**
- Create a new `kernel/` module that processes `KernelArgs` at entry point
- Implement virtual memory initialization:
  - Parse UEFI-provided memory map
  - Set up page tables for identity-mapped physical memory (early boot requirement)
  - Implement `VirtualMemoryManager` abstraction
- Add `panic!` handler that prints diagnostics and halts (replaces current minimal panic handler)
- Establish kernel-safe logging (must work outside UEFI services)
- Document the boundary between boot-stage (UEFI context) and kernel-stage (post-UEFI) code

**Output:** Kernel boots, initializes paging, logs messages, and can run code in isolated virtual address space.

### Phase 2: Task & Scheduling Foundation

**Goal:** Implement basic preemptive multitasking; prepare for userspace abstractions.

**Tasks:**
- Implement CPU exception/interrupt handling (IDT setup for x86_64)
- Set up timer interrupt (PIT or APIC timer via ACPI discovery)
- Create `Task` abstraction:
  - Task context (registers, stack, memory space)
  - Task state machine (Ready, Running, Blocked, etc.)
- Implement simple round-robin scheduler
- Add `context_switch()` entry point (likely in assembly)
- Test with multiple kernel tasks co-running (kernel-only, no userspace yet)

**Output:** Kernel can spawn and switch between multiple kernel-space tasks via preemptive scheduling.

### Phase 3: Microkernel IPC & Capability Model

**Goal:** Establish the microkernel's core capability: inter-process communication with security isolation.

**Tasks:**
- Design and implement IPC mechanism:
  - Message passing model (mailbox-based or capability-based)
  - Asynchronous I/O or synchronous calls (choose based on design goals)
- Implement `Capability` abstraction for task isolation
- Create basic `CapabilityTable` (tracks which tasks can communicate)
- Add syscall entry point for IPC primitives (`send()`, `receive()`, `call()`)
- Implement `PortalServer` abstraction (task that provides a service)
- Test IPC between kernel tasks

**Output:** Kernel-space services communicate via secure IPC; foundation for moving drivers/services out of kernel.

### Phase 4: Platform Abstraction Layer (PAL)

**Goal:** Decouple hardware specifics from core kernel logic to enable efistub-like flexibility.

**Tasks:**
- Refactor hardware discovery into a `PlatformAbstraction` trait:
  - `discover_memory()` - abstracts memory map discovery
  - `discover_acpi()` - abstracts ACPI/SMBIOS/PCIe discovery
  - `get_timer()`, `get_console()` - device abstractions
- Move UEFI-specific code (UEFI loader) into `platforms/uefi/` backend
- Create `platforms/api.rs` that defines the PAL contract
- Document how alternative platforms (e.g., multiboot2, direct paging) could implement PAL
- Ensure `KernelArgs` becomes a platform-independent data structure

**Output:** Kernel logic is platform-agnostic; UEFI is one interchangeable backend.

### Phase 5: Efistub-Like Direct UEFI Boot

**Goal:** Enable the kernel binary itself to be bootable by UEFI firmware (efistub pattern).

**Tasks:**
- Add kernel EFI entry point (in addition to loader stage):
  - Kernel can recognize when booted directly by UEFI (vs. via loader)
  - Kernel constructs its own `KernelArgs` if booted from UEFI (call platform PAL discovery)
- Loader becomes optional: kernel can boot standalone from UEFI
- Build both `uefiloader.efi` (minimal) and `lubun-kernel.efi` (full kernel bootable)
- Document the two-stage flow and efistub-like single-stage flow
- Test both entry paths boot to identical kernel state

**Output:** Kernel binary is a self-contained EFI application that needs no separate loader for basic functionality.

### Phase 6: Modular Driver/Server Architecture

**Goal:** Move device drivers and services into isolated microkernel servers with pluggable architecture.

**Tasks:**
- Create driver abstraction layer (e.g., `DriverServer` trait)
- Implement example drivers as isolated tasks:
  - Serial console driver (communicates with platform via PAL)
  - ACPI service (parses tables, responds to queries)
  - Device enumeration service (PCIe/PCI discovery)
- Load drivers via a configuration manifest or discovery protocol
- Establish driver-to-kernel communication via IPC
- Drivers can be loaded by loader or kernel at boot time
- Support runtime driver loading (future enhancement)

**Output:** Hardware services are pluggable, testable in isolation, communicate via IPC.

### Phase 7: Userspace & Privilege Separation

**Goal:** Implement true isolation between kernel and userspace applications.

**Tasks:**
- Implement ring 3 privilege level support (x86_64)
- Create userspace task abstraction with:
  - Separate address space (user VAS)
  - Reduced CPU privileges
  - Syscall entry point for kernel service requests
- Add `syscall`/`sysret` handlers for key operations (IPC, memory management, I/O)
- Create example userspace application (test program)
- Implement capability-based authorization (userspace apps request capabilities from kernel)

**Output:** Kernel enforces CPU-level privilege separation; userspace tasks have limited, auditable access to kernel services.

### Architecture Goals by Phase

| Phase | Kernel Type | Boot Path | IPC | Hardware Drivers |
|-------|------------|-----------|-----|-----------------|
| 1 | Single-stage kernel | UEFI loader → KernelArgs → Kernel | N/A | In kernel |
| 2 | Multitasking kernel | Same | Kernel tasks only | In kernel |
| 3 | Microkernel foundation | Same | Message passing | In kernel |
| 4 | Platform-agnostic μ-kernel | Same | Message passing | In kernel |
| 5 | **Efistub-like** | Direct UEFI OR UEFI loader | Message passing | In kernel |
| 6 | Modular μ-kernel | Direct UEFI OR UEFI loader | IPC + drivers | **Isolated servers** |
| 7 | Full μ-kernel OS | Direct UEFI OR UEFI loader | Syscalls + IPC | Isolated servers + userspace |

### Key Design Principles

1. **Minimal Trusted Computing Base (TCB):** Each phase moves more functionality into isolated services, reducing the kernel's TCB.
2. **Capability-Based Security:** All inter-component communication is capability-gated; no implicit trust.
3. **Platform Independence:** Hardware-specific code is behind the PAL; kernel logic is portable.
4. **Incremental Validation:** Each phase is independently testable before moving to the next.
5. **UEFI-First but Not UEFI-Only:** Start with UEFI bootability, but design abstractions that don't couple to UEFI specifics.

### Development Milestones & Testing

Each phase should include:
- **Unit tests** for new abstractions (task scheduling, IPC, virtual memory)
- **Integration tests** in QEMU to verify boot flow and multi-component interaction
- **Documentation** of new APIs and architecture decisions
- **Example code** demonstrating the phase's capabilities (e.g., sample IPC between tasks)

Success criteria per phase:
- Phase 1: Kernel boots from `KernelArgs` and handles page faults safely
- Phase 2: Multiple kernel tasks run concurrently, preempted by timer
- Phase 3: Two kernel tasks exchange messages and verify isolation
- Phase 4: PAL abstraction compiles; UEFI backend functions identically to current code
- Phase 5: Kernel binary boots directly from UEFI firmware
- Phase 6: Device drivers run as isolated servers; kernel stable
- Phase 7: Userspace app boots, makes syscalls, runs safely isolated from kernel
