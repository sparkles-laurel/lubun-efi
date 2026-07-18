// Stack manager for kernel tasks
// Provides a small fixed pool of stacks for task contexts.

use spin::Mutex;
use core::ptr;

const MAX_STACKS: usize = 8;
pub const STACK_SIZE: usize = 16 * 1024; // 16 KiB per stack

// Static stack pool
static mut STACK_POOL: [[u8; STACK_SIZE]; MAX_STACKS] = [[0u8; STACK_SIZE]; MAX_STACKS];

static STACK_BITMAP: Mutex<[u8; MAX_STACKS]> = Mutex::new([0u8; MAX_STACKS]);

pub struct StackManager;

impl StackManager {
    /// Initialize stack manager (no-op for static pool)
    pub fn init() -> Self {
        StackManager
    }

    /// Allocate a stack and return pointer to top of stack (stack grows down).
    /// Returns None if no stacks available.
    pub fn allocate_stack(&self) -> Option<*mut u8> {
        let mut bm = STACK_BITMAP.lock();
        for i in 0..MAX_STACKS {
            if bm[i] == 0 {
                bm[i] = 1;
                unsafe {
                    let base = STACK_POOL[i].as_mut_ptr();
                    return Some(base.add(STACK_SIZE));
                }
            }
        }
        None
    }

    /// Deallocate a previously allocated stack top pointer. Returns true on success.
    pub fn deallocate_stack(&self, top_ptr: *mut u8) -> bool {
        let mut bm = STACK_BITMAP.lock();
        for i in 0..MAX_STACKS {
            unsafe {
                let base = STACK_POOL[i].as_mut_ptr();
                if base.add(STACK_SIZE) == top_ptr {
                    if bm[i] == 1 {
                        bm[i] = 0;
                        // zero the stack memory for hygiene
                        for j in 0..STACK_SIZE {
                            ptr::write(base.add(j), 0u8);
                        }
                        return true;
                    } else {
                        return false;
                    }
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_and_free() {
        let mgr = StackManager::init();
        let s1 = mgr.allocate_stack();
        assert!(s1.is_some());
        assert!(mgr.deallocate_stack(s1.unwrap()));
    }
}
