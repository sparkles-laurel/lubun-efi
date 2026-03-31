use acpi::{AcpiHandler, PhysicalMapping};

#[derive(Clone)]
/// Provides an `AcpiHandler` for an identity-mapped virtual memory.
pub struct IdentityAcpiHandler;

impl AcpiHandler for IdentityAcpiHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        unsafe {
            PhysicalMapping::new(
                physical_address,
                core::ptr::NonNull::<T>::new_unchecked(physical_address as *mut T),
                size,
                size,
                self.clone(),
            )
        }
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {}
}
