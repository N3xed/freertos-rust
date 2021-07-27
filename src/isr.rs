use crate::base::*;
use crate::glue;

/// Keep track of whether we need to yield the execution to a different
/// task at the end of the interrupt.
///
/// Should be dropped as the last thing inside a interrupt.
pub struct InterruptContext {
    x_higher_priority_task_woken: BaseType,
}

impl InterruptContext {
    /// Instantiate a new context.
    pub fn new() -> InterruptContext {
        InterruptContext {
            x_higher_priority_task_woken: 0,
        }
    }

    pub unsafe fn get_task_field_mut(&mut self) -> *mut BaseType {
        &mut self.x_higher_priority_task_woken as *mut _
    }
}

impl Drop for InterruptContext {
    fn drop(&mut self) {
        if self.x_higher_priority_task_woken == 1 {
            unsafe {
                glue::task_yield_from_isr();
            }
        }
    }
}
