use crate::base::*;
use crate::glue;
use crate::units::*;
use core::cell::UnsafeCell;
use core::fmt;
use core::mem;
use core::ops::{Deref, DerefMut};
use core::ptr;

pub type Mutex<T> = BasicMutex<T, Normal>;
pub type RecursiveMutex<T> = BasicMutex<T, Recursive>;

unsafe impl<T: Sync + Send, M> Send for BasicMutex<T, M> {}

unsafe impl<T: Sync + Send, M> Sync for BasicMutex<T, M> {}

/// Mutual exclusion access to a contained value. Can be recursive -
/// the current owner of a lock can re-lock it.
pub struct BasicMutex<T: ?Sized, M> {
    mutex: M,
    data: UnsafeCell<T>,
}

impl<T: ?Sized, M> fmt::Debug for BasicMutex<T, M>
where
    M: Lockable + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Mutex address: {:?}", self.mutex)
    }
}

impl<T> BasicMutex<T, Normal> {
    /// Create a new mutex with the given inner value
    pub fn new(t: T) -> Result<Self, FreeRtosError> {
        Ok(BasicMutex {
            mutex: Normal::create()?,
            data: UnsafeCell::new(t),
        })
    }
}

impl<T> BasicMutex<T, Recursive> {
    /// Create a new recursive mutex with the given inner value
    pub fn new(t: T) -> Result<Self, FreeRtosError> {
        Ok(BasicMutex {
            mutex: Recursive::create()?,
            data: UnsafeCell::new(t),
        })
    }
}

impl<T, M> BasicMutex<T, M>
where
    M: Lockable,
{
    /// Try to obtain a lock and mutable access to our inner value
    pub fn lock(&self, max_wait: impl Into<Ticks>) -> Result<MutexGuard<T, M>, FreeRtosError> {
        self.mutex.take(max_wait)?;

        Ok(MutexGuard {
            mutex: &self.mutex,
            data: &self.data,
        })
    }

    /// Consume the mutex and return its inner value
    pub fn into_inner(self) -> T {
        // Manually deconstruct the structure, because it implements Drop
        // and we cannot move the data value out of it.
        unsafe {
            let (mutex, data) = {
                let Self {
                    ref mutex,
                    ref data,
                } = self;
                (ptr::read(mutex), ptr::read(data))
            };
            mem::forget(self);

            drop(mutex);

            data.into_inner()
        }
    }
}

/// Holds the mutex until we are dropped
pub struct MutexGuard<'a, T: ?Sized + 'a, M: 'a>
where
    M: Lockable,
{
    mutex: &'a M,
    data: &'a UnsafeCell<T>,
}

impl<'mutex, T: ?Sized, M> Deref for MutexGuard<'mutex, T, M>
where
    M: Lockable,
{
    type Target = T;

    fn deref<'a>(&'a self) -> &'a T {
        unsafe { &*self.data.get() }
    }
}

impl<'mutex, T: ?Sized, M> DerefMut for MutexGuard<'mutex, T, M>
where
    M: Lockable,
{
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        unsafe { &mut *self.data.get() }
    }
}

impl<'a, T: ?Sized, M> Drop for MutexGuard<'a, T, M>
where
    M: Lockable,
{
    fn drop(&mut self) {
        self.mutex.give();
    }
}

pub trait Lockable
where
    Self: Sized,
{
    fn create() -> Result<Self, FreeRtosError>;
    fn take(&self, max_wait: impl Into<Ticks>) -> Result<(), FreeRtosError>;
    fn give(&self);
}

pub struct Normal(QueueHandle);

impl Lockable for Normal {
    fn create() -> Result<Self, FreeRtosError> {
        match unsafe { glue::create_mutex() } {
            Some(h) => Ok(Normal(h.as_ptr())),
            None => Err(FreeRtosError::OutOfMemory),
        }
    }

    fn take(&self, max_wait: impl Into<Ticks>) -> Result<(), FreeRtosError> {
        if unsafe { glue::take_mutex(self.0, max_wait.into().ticks) } {
            Ok(())
        } else {
            Err(FreeRtosError::MutexTimeout)
        }
    }

    fn give(&self) {
        unsafe {
            glue::give_mutex(self.0);
        }
    }
}

impl Drop for Normal {
    fn drop(&mut self) {
        unsafe { glue::delete_semaphore(self.0) }
    }
}

impl fmt::Debug for Normal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

pub struct Recursive(QueueHandle);

impl Lockable for Recursive {
    fn create() -> Result<Self, FreeRtosError> {
        match unsafe { glue::create_recursive_mutex() } {
            Some(m) => Ok(Recursive(m.as_ptr())),
            None => Err(FreeRtosError::OutOfMemory),
        }
    }

    fn take(&self, max_wait: impl Into<Ticks>) -> Result<(), FreeRtosError> {
        if unsafe { glue::take_recursive_mutex(self.0, max_wait.into().ticks) } {
            Ok(())
        } else {
            Err(FreeRtosError::MutexTimeout)
        }
    }

    fn give(&self) {
        unsafe {
            glue::give_mutex(self.0);
        }
    }
}

impl Drop for Recursive {
    fn drop(&mut self) {
        unsafe { glue::delete_semaphore(self.0) }
    }
}

impl fmt::Debug for Recursive {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
