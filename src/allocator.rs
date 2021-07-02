use crate::base::*;
use crate::glue;
use core::alloc::{GlobalAlloc, Layout};

/// A global allocator that uses FreeRTOS's memory management
///
/// Use with:
/// ```ignore
/// #[global_allocator]
/// static GLOBAL: FreeRtosAllocator = FreeRtosAllocator;
/// ```
pub struct FreeRtosAllocator;

unsafe impl GlobalAlloc for FreeRtosAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let res = glue::port_malloc(layout.size() as usize);
        return res as *mut u8;
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        glue::port_free(ptr as *mut c_void)
    }
}
