use crate::InterruptContext;
use crate::base::*;
use crate::glue;
use crate::units::*;

/// A counting or binary semaphore
#[repr(transparent)]
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
                Some(semaphore) => Ok(Semaphore {
                    semaphore,
                }),
                None => Err(FreeRtosError::OutOfMemory),
            }
        }
    }

    /// Create a new counting semaphore
    pub fn new_counting(max: u32, initial: u32) -> Result<Semaphore, FreeRtosError> {
        unsafe {
            match glue::create_counting_semaphore(max, initial) {
                Some(semaphore) => Ok(Semaphore {
                    semaphore,
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
    
    /// Give this semaphore
    pub fn give(&self) -> Result<(), FreeRtosError> {
        unsafe {
            if glue::give_mutex(self.semaphore) {
                Ok(())
            }
            else {
                Err(FreeRtosError::QueueFull)
            }
        }
    }

    /// Take this semaphore
    pub fn take(&self, max_wait: impl Into<Ticks>) -> Result<(), FreeRtosError> {
        unsafe {
            if glue::take_mutex(self.semaphore, max_wait.into().ticks) {
                Ok(())
            } else {
                Err(FreeRtosError::Timeout)
            }
        }
    }

    /// Give this semaphore from an interrupt context
    pub fn give_from_isr(&self, interrupt_context: &mut InterruptContext) -> Result<(), FreeRtosError> {
        unsafe {
            if glue::give_mutex_isr(self.semaphore, interrupt_context.get_task_field_mut()) {
                Ok(())
            }
            else {
                Err(FreeRtosError::QueueFull)
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
