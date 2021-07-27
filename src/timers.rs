use core::mem;

use crate::base::*;
use crate::glue;
use crate::prelude::*;
use crate::units::*;

unsafe impl Send for Timer {}
unsafe impl Sync for Timer {}

/// A FreeRTOS software timer.
///
/// Note that all operations on a timer are processed by a FreeRTOS internal task
/// that receives messages in a queue. Every operation has an associated waiting time
/// for that queue to get unblocked.
pub struct Timer {
    handle: TimerHandle,
    destructor: Option<fn(usize)>,
}

/// Helper builder for a new software timer.
pub struct TimerBuilder {
    name: String,
    period: Ticks,
    auto_reload: bool,
}

impl TimerBuilder {
    /// Set the name of the timer.
    pub fn set_name(&mut self, name: &str) -> &mut Self {
        self.name = name.into();
        self
    }

    /// Set the period of the timer.
    pub fn set_period(&mut self, period: impl Into<Ticks>) -> &mut Self {
        self.period = period.into();
        self
    }

    /// Should the timer be automatically reloaded?
    pub fn set_auto_reload(&mut self, auto_reload: bool) -> &mut Self {
        self.auto_reload = auto_reload;
        self
    }

    /// Try to create the new timer.
    ///
    /// Note that the newly created timer must be started.
    pub fn create<F>(&self, callback: F) -> Result<Timer, FreeRtosError>
    where
        F: Fn(Timer) -> (),
        F: Send + 'static,
    {
        Timer::spawn(
            self.name.as_str(),
            self.period.ticks,
            self.auto_reload,
            callback,
        )
    }
}

impl Timer {
    /// Create a new timer builder.
    pub fn new(period: impl Into<Ticks>) -> TimerBuilder {
        TimerBuilder {
            name: "timer".into(),
            period: period.into(),
            auto_reload: true,
        }
    }

    fn spawn<F>(
        name: &str,
        period_ticks: TickType,
        auto_reload: bool,
        callback: F,
    ) -> Result<Timer, FreeRtosError>
    where
        F: FnMut(Timer),
        F: Send + 'static,
    {
        unsafe {
            let mut f = Box::new(callback);
            let param_ptr = f.as_mut() as *mut _ as usize;

            match glue::timer_create(
                name,
                period_ticks,
                auto_reload,
                param_ptr,
                Self::timer_callback::<F>,
            ) {
                Some(h) => {
                    mem::forget(f);
                    Ok(Timer {
                        handle: h,
                        destructor: Some(Self::timer_destructor::<F>),
                    })
                }
                None => Err(FreeRtosError::OutOfMemory),
            }
        }
    }

    extern "C" fn timer_callback<F: FnMut(Timer)>(handle: TimerHandle) {
        unsafe {
            let timer = Timer {
                handle,
                destructor: None,
            };

            let callback_ptr: *mut F = mem::transmute(timer.get_id().unwrap());
            (*callback_ptr)(timer);
        }
    }

    fn timer_destructor<F: FnMut(Timer)>(callback_ptr: usize) {
        unsafe { drop(Box::from_raw(callback_ptr as *mut F)) }
    }

    /// Start the timer.
    pub fn start(&self, block_time: impl Into<Ticks>) -> Result<(), FreeRtosError> {
        unsafe {
            if glue::timer_start(self.handle, block_time.into().ticks) {
                Ok(())
            } else {
                Err(FreeRtosError::Timeout)
            }
        }
    }

    /// Stop the timer.
    pub fn stop(&self, block_time: impl Into<Ticks>) -> Result<(), FreeRtosError> {
        unsafe {
            if glue::timer_stop(self.handle, block_time.into().ticks) {
                Ok(())
            } else {
                Err(FreeRtosError::Timeout)
            }
        }
    }

    /// Change the period of the timer.
    pub fn change_period(
        &self,
        block_time: impl Into<Ticks>,
        new_period: impl Into<Ticks>,
    ) -> Result<(), FreeRtosError> {
        unsafe {
            if glue::timer_change_period(
                self.handle,
                block_time.into().ticks,
                new_period.into().ticks,
            ) {
                Ok(())
            } else {
                Err(FreeRtosError::Timeout)
            }
        }
    }

    /// Detach this timer from Rust's memory management. The timer will still be active and
    /// will consume the memory.
    ///
    /// Can be used for timers that will never be changed and don't need to stay in scope.
    pub unsafe fn detach(mut self) {
        self.destructor = None;
    }

    fn get_id(&self) -> Result<usize, FreeRtosError> {
        unsafe { Ok(glue::timer_get_id(self.handle)) }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        if let Some(destructor) = self.destructor {
            unsafe {
                if let Ok(callback_ptr) = self.get_id() {
                    // free the memory
                    destructor(callback_ptr);
                }

                // todo: configurable timeout?
                glue::timer_delete(self.handle, Ticks::milliseconds(1000).ticks);
            }
        }
    }
}
