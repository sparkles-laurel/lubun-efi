/// Kernel module: handles post-UEFI execution context and hardware abstraction
/// 
/// The kernel module is responsible for:
/// - Processing KernelArgs passed from the loader
/// - Setting up virtual memory and paging
/// - Establishing kernel-safe logging (no UEFI services)
/// - Managing the transition from UEFI boot context to kernel context
/// - Coordinating kernel subsystems (future phases: scheduling, IPC, drivers)

pub mod memory;
pub mod log;
pub mod stack;
pub mod timer;

use crate::kernel_args::KernelArgs;
use self::stack::StackManager;
use self::timer::Timer;

/// The main Kernel struct that orchestrates all kernel subsystems
pub struct Kernel {
    memory_manager: memory::VirtualMemoryManager,
    stack_manager: StackManager,
    timer: Timer,
}

impl Kernel {
    /// Initialize the kernel from KernelArgs passed by the loader
    /// 
    /// This is called immediately after the loader hands off to the kernel.
    /// Must set up:
    /// - Virtual memory management and paging
    /// - Kernel logging (outside UEFI context)
    /// - Panic handler
    /// 
    /// # Safety
    /// This function performs low-level memory operations (page table setup).
    /// The caller must ensure:
    /// - KernelArgs points to valid hardware structures
    /// - UEFI services have been exited
    /// - IRQs are disabled during initialization
    pub unsafe fn init(kargs: KernelArgs) -> Self {
        // Initialize kernel-safe logging first so we can report errors
        log::init_kernel_logging();
        
        log::info!("Kernel initializing from KernelArgs...");
        log::info!("  ACPI: {:#x} (v{})", kargs.get_acpi().0 as usize, kargs.get_acpi().1);
        log::info!("  SMBIOS: {:#x} (v{})", kargs.get_smbios().0 as usize, kargs.get_smbios().1);
        log::info!("  PCIe: {:#x}", kargs.get_pcie() as usize);
        
        // Initialize virtual memory manager from memory map
        let memory_manager = unsafe {
            memory::VirtualMemoryManager::init_from_memmap(
                kargs.get_memmap(),
                kargs.get_memmap_entries(),
            )
        };
        
        log::info!("Virtual memory initialized: {} entries in memory map", kargs.get_memmap_entries());
        
        // Initialize stack manager and timer (required for task scheduling)
        let stack_manager = StackManager::init();
        let timer = Timer::init();
        
        Kernel {
            memory_manager,
            stack_manager,
            timer,
        }
    }
    
    /// Main kernel loop (placeholder for future phases)
    /// Will coordinate scheduling, IPC, driver management, etc.
    pub fn run(&mut self) -> ! {
        log::info!("Kernel running. System ready.");
        
        // TODO Phase 2: Implement task scheduling loop
        // TODO Phase 3: Implement IPC message pump
        // For now, just halt
        loop {
            unsafe { 
                core::arch::asm!("hlt");
            }
        }
    }
    
    /// Get reference to the virtual memory manager
    pub fn memory_manager(&self) -> &memory::VirtualMemoryManager {
        &self.memory_manager
    }
    
    /// Get mutable reference to the virtual memory manager
    pub fn memory_manager_mut(&mut self) -> &mut memory::VirtualMemoryManager {
        &mut self.memory_manager
    }

    /// Get reference to the stack manager
    pub fn stack_manager(&self) -> &StackManager {
        &self.stack_manager
    }

    /// Get mutable reference to the stack manager
    pub fn stack_manager_mut(&mut self) -> &mut StackManager {
        &mut self.stack_manager
    }

    /// Get reference to the timer
    pub fn timer(&self) -> &Timer {
        &self.timer
    }

    /// Get mutable reference to the timer
    pub fn timer_mut(&mut self) -> &mut Timer {
        &mut self.timer
    }
}

/// Initialize and run the kernel
/// Called by the loader after hardware discovery
pub fn start_kernel(kargs: crate::kernel_args::KernelArgs) -> ! {
    unsafe {
        let mut kernel = Kernel::init(kargs);
        kernel.run()
    }
}
