/// Basic error type for the library.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FreeRtosError {
    OutOfMemory,
    QueueSendTimeout,
    QueueReceiveTimeout,
    MutexTimeout,
    Timeout,
    QueueFull,
    StringConversionError,
    TaskNotFound,
    InvalidQueueSize,
    ProcessorHasShutDown,
}

use core::ptr;

pub use chlorine::{c_char, c_void};

pub use sys::BaseType_t as BaseType;
pub use sys::StackType_t as StackType;
pub use sys::TickType_t as TickType;
pub use sys::UBaseType_t as UBaseType;

pub type TaskHandle = *mut c_void;
pub type MaybeTaskHandle = Option<ptr::NonNull<c_void>>;
pub type QueueHandle = *mut c_void;
pub type MaybeQueueHandle = Option<ptr::NonNull<c_void>>;
pub type TimerHandle = *mut c_void;
pub type MaybeTimerHandle = Option<ptr::NonNull<c_void>>;

pub use sys::TaskStatus_t as TaskStatusFfi;

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum TaskState {
    /// A task is querying the state of itself, so must be running.
    Running = 0,
    /// The task being queried is in a read or pending ready list.
    Ready = 1,
    /// The task being queried is in the Blocked state.
    Blocked = 2,
    /// The task being queried is in the Suspended state, or is in the Blocked state with an infinite time out.
    Suspended = 3,
    /// The task being queried has been deleted, but its TCB has not yet been freed.
    Deleted = 4,
}
