#![allow(dead_code)]

/// Virtual memory management for x86_64
///
/// Handles:
/// - Page table allocation and management
/// - Identity mapping of physical memory
/// - Virtual address space abstraction
/// - Memory type tracking from UEFI
use crate::kernel_args::OSMemEntry;
use uefi::table::boot::MemoryType;

/// x86_64 page sizes
pub const PAGE_SIZE: usize = 4096;
pub const PAGE_SIZE_2M: usize = 2 * 1024 * 1024;
pub const PAGE_SIZE_1G: usize = 1024 * 1024 * 1024;

/// Page table entry flags (x86_64)
pub mod page_flags {
    pub const PRESENT: u64 = 1 << 0;
    pub const WRITE: u64 = 1 << 1;
    pub const USER: u64 = 1 << 2;
    pub const WRITE_THROUGH: u64 = 1 << 3;
    pub const CACHE_DISABLE: u64 = 1 << 4;
    pub const ACCESSED: u64 = 1 << 5;
    pub const DIRTY: u64 = 1 << 6;
    pub const HUGE_PAGE: u64 = 1 << 7;
    pub const GLOBAL: u64 = 1 << 8;
    pub const NO_EXECUTE: u64 = 1 << 63;
}

/// A page table entry
#[derive(Clone, Copy)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    /// Create a new page table entry pointing to the given physical address
    pub fn new(phys_addr: u64, flags: u64) -> Self {
        PageTableEntry((phys_addr & 0x000ffffffffff000) | (flags & 0xfff))
    }

    /// Check if this entry is present (valid)
    pub fn present(&self) -> bool {
        self.0 & page_flags::PRESENT != 0
    }

    /// Get the physical address pointed to by this entry
    pub fn address(&self) -> u64 {
        self.0 & 0x000ffffffffff000
    }

    /// Get the raw entry value
    pub fn raw(&self) -> u64 {
        self.0
    }
}

/// A page table (512 entries, 4096 bytes)
#[repr(align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; 512],
}

impl PageTable {
    /// Create a new page table with all entries zeroed
    pub fn new() -> Self {
        PageTable {
            entries: [PageTableEntry(0); 512],
        }
    }

    /// Set an entry in this page table
    pub fn set_entry(&mut self, index: usize, entry: PageTableEntry) {
        if index < 512 {
            self.entries[index] = entry;
        }
    }

    /// Get an entry from this page table
    pub fn get_entry(&self, index: usize) -> PageTableEntry {
        if index < 512 {
            self.entries[index]
        } else {
            PageTableEntry(0)
        }
    }
}

/// Virtual memory manager for the kernel
pub struct VirtualMemoryManager {
    /// Memory regions from UEFI memory map
    memory_regions: &'static [OSMemEntry],
    memory_region_count: usize,
}

impl VirtualMemoryManager {
    /// Initialize the virtual memory manager from UEFI memory map
    ///
    /// # Safety
    /// Caller must ensure:
    /// - memmap pointer is valid and points to OSMemEntry array
    /// - entry_count matches actual number of entries
    pub unsafe fn init_from_memmap(
        memmap_ptr: *mut crate::kernel_args::OSMemEntry,
        entry_count: usize,
    ) -> Self {
        let memory_regions = unsafe { core::slice::from_raw_parts(memmap_ptr, entry_count) };

        VirtualMemoryManager {
            memory_regions,
            memory_region_count: entry_count,
        }
    }

    /// Get the total physical memory available
    pub fn total_physical_memory(&self) -> usize {
        self.memory_regions
            .iter()
            .map(|region| region.pages * PAGE_SIZE)
            .sum()
    }

    /// Count usable memory (conventional RAM that can be allocated)
    pub fn usable_memory(&self) -> usize {
        self.memory_regions
            .iter()
            .filter(|region| region.ty == MemoryType::CONVENTIONAL)
            .map(|region| region.pages * PAGE_SIZE)
            .sum()
    }

    /// Find a region containing the given physical address
    pub fn find_region(&self, phys_addr: usize) -> Option<&OSMemEntry> {
        self.memory_regions.iter().find(|region| {
            let region_start = region.base;
            let region_end = region.base + (region.pages * PAGE_SIZE);
            phys_addr >= region_start && phys_addr < region_end
        })
    }

    /// Get memory regions slice
    pub fn regions(&self) -> &'static [OSMemEntry] {
        self.memory_regions
    }

    /// Get number of memory regions
    pub fn region_count(&self) -> usize {
        self.memory_region_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_table_entry_creation() {
        let addr = 0x1000;
        let flags = page_flags::PRESENT | page_flags::WRITE;
        let entry = PageTableEntry::new(addr, flags);

        assert!(entry.present());
        assert_eq!(entry.address(), addr);
    }

    #[test]
    fn page_table_entry_flags() {
        let entry = PageTableEntry::new(0x1000, page_flags::PRESENT);
        assert!(entry.present());

        let entry = PageTableEntry::new(0x1000, 0);
        assert!(!entry.present());
    }

    #[test]
    fn page_table_operations() {
        let mut pt = PageTable::new();
        let entry = PageTableEntry::new(0x1000, page_flags::PRESENT);

        pt.set_entry(0, entry);
        assert_eq!(pt.get_entry(0).address(), 0x1000);
        assert!(pt.get_entry(0).present());
    }

    #[test]
    fn page_size_constants() {
        assert_eq!(PAGE_SIZE, 4096);
        assert_eq!(PAGE_SIZE_2M, 2 * 1024 * 1024);
        assert_eq!(PAGE_SIZE_1G, 1024 * 1024 * 1024);
    }
}
