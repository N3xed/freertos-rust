use crate::base::*;
use crate::glue;
use crate::units::*;

/// A counting or binary semaphore
pub struct Semaphore {
    semaphore: QueueHandle,
}

unsafe impl Send for Semaphore {}
unsafe impl Sync for Semaphore {}

impl Semaphore {
    /// Create a new binary semaphore
    pub fn new_binary() -> Result<Semaphore, FreeRtosError> {
        unsafe {
            match glue::create_binary_semaphore() {
                Some(s) => Ok(Semaphore {
                    semaphore: s.as_ptr(),
                }),
                None => Err(FreeRtosError::OutOfMemory),
            }
        }
    }

    /// Create a new counting semaphore
    pub fn new_counting(max: u32, initial: u32) -> Result<Semaphore, FreeRtosError> {
        unsafe {
            match glue::create_counting_semaphore(max, initial) {
                Some(s) => Ok(Semaphore {
                    semaphore: s.as_ptr(),
                }),
                None => Err(FreeRtosError::OutOfMemory),
            }
        }
    }

    /// Lock this semaphore in a RAII fashion
    pub fn lock(&self, max_wait: impl Into<Ticks>) -> Result<SemaphoreGuard, FreeRtosError> {
        unsafe {
            if glue::take_mutex(self.semaphore, max_wait.into().ticks) {
                Ok(SemaphoreGuard {
                    semaphore: self.semaphore,
                })
            } else {
                Err(FreeRtosError::Timeout)
            }
        }
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe {
            glue::delete_semaphore(self.semaphore);
        }
    }
}

/// Holds the lock to the semaphore until we are dropped
pub struct SemaphoreGuard {
    semaphore: QueueHandle,
}

impl Drop for SemaphoreGuard {
    fn drop(&mut self) {
        unsafe {
            glue::give_mutex(self.semaphore);
        }
    }
}
