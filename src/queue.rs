use core::marker::PhantomData;
use core::mem;

use crate::base::*;
use crate::glue;
use crate::isr::*;
use crate::units::*;

unsafe impl<T: Sized + Copy> Send for Queue<T> {}
unsafe impl<T: Sized + Copy> Sync for Queue<T> {}

/// A queue with a finite size. The items are owned by the queue and are
/// copied.
#[derive(Debug)]
#[repr(transparent)]
pub struct Queue<T: Sized + Copy> {
    queue: QueueHandle,
    item_type: PhantomData<T>,
}

impl<T: Sized + Copy> Queue<T> {
    pub fn new(max_size: usize) -> Result<Queue<T>, FreeRtosError> {
        match unsafe { glue::queue_create(max_size as u32, mem::size_of::<T>() as u32) } {
            Some(queue) => Ok(Queue {
                queue,
                item_type: PhantomData,
            }),
            None => Err(FreeRtosError::OutOfMemory),
        }
    }

    /// Send an item to the end of the queue. Wait for the queue to have empty space for it.
    pub fn send(&self, item: T, max_wait: impl Into<Ticks>) -> Result<(), FreeRtosError> {
        unsafe {
            if glue::queue_send(
                self.queue,
                &item as *const _ as *const _,
                max_wait.into().ticks,
            ) {
                Ok(())
            } else {
                Err(FreeRtosError::QueueSendTimeout)
            }
        }
    }

    /// Send an item to the end of the queue, from an interrupt.
    pub fn send_from_isr(
        &self,
        context: &mut InterruptContext,
        item: T,
    ) -> Result<(), FreeRtosError> {
        unsafe {
            if glue::queue_send_isr(
                self.queue,
                &item as *const _ as *const _,
                context.get_task_field_mut(),
            ) {
                Ok(())
            } else {
                Err(FreeRtosError::QueueFull)
            }
        }
    }

    /// Wait for an item to be available on the queue.
    pub fn receive(&self, max_wait: impl Into<Ticks>) -> Result<T, FreeRtosError> {
        unsafe {
            let mut buff = mem::zeroed::<T>();
            if glue::queue_receive(
                self.queue,
                &mut buff as *mut _ as *mut _,
                max_wait.into().ticks,
            ) {
                Ok(buff)
            } else {
                Err(FreeRtosError::QueueReceiveTimeout)
            }
        }
    }
}

impl<T: Sized + Copy> Drop for Queue<T> {
    fn drop(&mut self) {
        unsafe {
            glue::queue_delete(self.queue);
        }
    }
}
