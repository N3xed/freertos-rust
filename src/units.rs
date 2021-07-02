use core::fmt::Debug;

use crate::base::TickType;
use crate::glue;

pub use glue::MAX_DELAY;
pub use glue::TICK_PERIOD_MS;
pub use glue::TICK_RATE_HZ;

/// Time unit used by FreeRTOS, passed to the scheduler as ticks.
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Ticks {
    pub ticks: TickType,
}

impl Debug for Ticks {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} ms ({} ticks)", self.to_milliseconds(), self.ticks)
    }
}

impl Ticks {
    pub const fn new(ticks: TickType) -> Ticks {
        Ticks { ticks }
    }

    pub const fn milliseconds(ms: u32) -> Ticks {
        Self::new((ms / TICK_PERIOD_MS) as TickType)
    }

    pub const fn seconds(secs: u32) -> Ticks {
        Self::new((secs * 1000 / TICK_PERIOD_MS) as TickType)
    }

    pub const fn infinite() -> Ticks {
        Self::new(MAX_DELAY)
    }

    pub const fn zero() -> Ticks {
        Self::new(0)
    }

    pub const fn to_milliseconds(&self) -> u32 {
        self.ticks * TICK_PERIOD_MS
    }
}
