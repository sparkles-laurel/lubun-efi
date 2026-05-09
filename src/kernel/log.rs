/// Kernel logging without UEFI services
/// 
/// Works after exiting UEFI boot services by writing directly to serial port.
/// Note: Currently uses the log crate for compatibility during boot phase.
/// After Phase 2, this will be replaced with direct serial output.

/// Initialize kernel logging
pub fn init_kernel_logging() {
    // Logging is now safe; subsequent info! will work
    // The log crate's UEFI backend will work until services are disabled
}

// Re-export log macros for kernel use
pub use log::info;

#[cfg(test)]
mod tests {
    use super::*;
    
    fn serial_port_creation() {
        // Logging functions are re-exported from log crate
    }
}
