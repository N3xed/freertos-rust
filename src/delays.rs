use crate::base::*;
use crate::glue;
use crate::task::*;
use crate::units::*;

/// Delay the current task by the given duration, minus the
/// time that was spent processing the last wakeup loop.
pub struct TaskDelay {
    last_wake_time: TickType,
}

impl TaskDelay {
    /// Create a new helper, marking the current time as the start of the
    /// next measurement.
    pub fn new() -> TaskDelay {
        TaskDelay {
            last_wake_time: get_tick_count(),
        }
    }

    /// Delay the execution of the current task by the given duration,
    /// minus the time spent in this task since the last delay.
    pub fn delay_until(&mut self, delay: impl Into<Ticks>) {
        unsafe {
            glue::task_delay_until(&mut self.last_wake_time as *mut _, delay.into().ticks);
        }
    }
}

/// Periodic delay timer.
///
/// Use inside a polling loop, for example: the loop polls this instance every second.
/// The method `should_run` will return true once 30 seconds or more has elapsed
/// and it will then reset the timer for that period.
pub struct TaskDelayPeriodic {
    last_wake_time: TickType,
    period_ticks: TickType,
}

impl TaskDelayPeriodic {
    /// Create a new timer with the set period.
    pub fn new(period: impl Into<Ticks>) -> TaskDelayPeriodic {
        let l = get_tick_count();

        TaskDelayPeriodic {
            last_wake_time: l,
            period_ticks: period.into().ticks,
        }
    }

    /// Has the set period passed? If it has, resets the internal timer.
    pub fn should_run(&mut self) -> bool {
        let c = get_tick_count();
        if (c - self.last_wake_time) < (self.period_ticks) {
            false
        } else {
            self.last_wake_time = c;
            true
        }
    }

    /// Set a new delay period
    pub fn set_period(&mut self, period: impl Into<Ticks>) {
        self.period_ticks = period.into().ticks;
    }

    /// Reset the internal timer to zero.
    pub fn reset(&mut self) {
        self.last_wake_time = get_tick_count();
    }
}
