use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

use crate::base::*;
use crate::glue;

pub struct CriticalSection(UBaseType);
impl CriticalSection {
    pub fn enter() -> Self {
        let previous_state = unsafe { glue::enter_critical() };

        CriticalSection(previous_state)
    }
}

impl Drop for CriticalSection {
    fn drop(&mut self) {
        unsafe {
            glue::exit_critical(self.0);
        }
    }
}

unsafe impl<T: Sync + Send> Send for ExclusiveData<T> {}
unsafe impl<T: Sync + Send> Sync for ExclusiveData<T> {}

/// Data protected with a critical region. Lightweight version of a mutex,
/// intended for simple data structures.
pub struct ExclusiveData<T: ?Sized> {
    data: UnsafeCell<T>,
}

impl<T> ExclusiveData<T> {
    pub fn new(data: T) -> Self {
        ExclusiveData {
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> Result<ExclusiveDataGuard<T>, FreeRtosError> {
        Ok(ExclusiveDataGuard {
            data: &self.data,
            _lock: CriticalSection::enter(),
        })
    }

    pub fn lock_from_isr(
        &self,
        _context: &mut crate::isr::InterruptContext,
    ) -> Result<ExclusiveDataGuardIsr<T>, FreeRtosError> {
        Ok(ExclusiveDataGuardIsr { data: &self.data })
    }
}

/// Holds the mutex until we are dropped
pub struct ExclusiveDataGuard<'a, T: ?Sized + 'a> {
    data: &'a UnsafeCell<T>,
    _lock: CriticalSection,
}

impl<'mutex, T: ?Sized> Deref for ExclusiveDataGuard<'mutex, T> {
    type Target = T;

    fn deref<'a>(&'a self) -> &'a T {
        unsafe { &*self.data.get() }
    }
}

impl<'mutex, T: ?Sized> DerefMut for ExclusiveDataGuard<'mutex, T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        unsafe { &mut *self.data.get() }
    }
}

pub struct ExclusiveDataGuardIsr<'a, T: ?Sized + 'a> {
    data: &'a UnsafeCell<T>,
}

impl<'mutex, T: ?Sized> Deref for ExclusiveDataGuardIsr<'mutex, T> {
    type Target = T;

    fn deref<'a>(&'a self) -> &'a T {
        unsafe { &*self.data.get() }
    }
}

impl<'mutex, T: ?Sized> DerefMut for ExclusiveDataGuardIsr<'mutex, T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        unsafe { &mut *self.data.get() }
    }
}
