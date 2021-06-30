use crate::base::*;
use crate::shim::*;
use crate::task::*;
use crate::units::*;

/// Delay the current task by the given duration, minus the
/// time that was spent processing the last wakeup loop.
pub struct TaskDelay {
    last_wake_time: FreeRtosTickType,
}

impl TaskDelay {
    /// Create a new helper, marking the current time as the start of the
    /// next measurement.
    pub fn new() -> TaskDelay {
        TaskDelay {
            last_wake_time: FreeRtosUtils::get_tick_count(),
        }
    }

    /// Delay the execution of the current task by the given duration,
    /// minus the time spent in this task since the last delay.
    pub fn delay_until<D: DurationTicks>(&mut self, delay: D) {
        unsafe {
            freertos_rs_vTaskDelayUntil(
                &mut self.last_wake_time as *mut FreeRtosTickType,
                delay.to_ticks(),
            );
        }
    }
}

/// Periodic delay timer.
///
/// Use inside a polling loop, for example: the loop polls this instance every second.
/// The method `should_run` will return true once 30 seconds or more has elapsed
/// and it will then reset the timer for that period.
pub struct TaskDelayPeriodic {
    last_wake_time: FreeRtosTickType,
    period_ticks: FreeRtosTickType,
}

impl TaskDelayPeriodic {
    /// Create a new timer with the set period.
    pub fn new<D: DurationTicks>(period: D) -> TaskDelayPeriodic {
        let l = FreeRtosUtils::get_tick_count();

        TaskDelayPeriodic {
            last_wake_time: l,
            period_ticks: period.to_ticks(),
        }
    }

    /// Has the set period passed? If it has, resets the internal timer.
    pub fn should_run(&mut self) -> bool {
        let c = FreeRtosUtils::get_tick_count();
        if (c - self.last_wake_time) < (self.period_ticks) {
            false
        } else {
            self.last_wake_time = c;
            true
        }
    }

    /// Set a new delay period
    pub fn set_period<D: DurationTicks>(&mut self, period: D) {
        self.period_ticks = period.to_ticks();
    }

    /// Reset the internal timer to zero.
    pub fn reset(&mut self) {
        self.last_wake_time = FreeRtosUtils::get_tick_count();
    }
}
