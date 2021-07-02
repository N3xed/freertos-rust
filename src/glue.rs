use core::mem;
use core::ptr;

use crate::base::*;

pub use sys::configTICK_RATE_HZ as TICK_RATE_HZ;
pub use sys::portMAX_DELAY as MAX_DELAY;
pub const TICK_PERIOD_MS: u32 = 1000 / TICK_RATE_HZ;

#[cfg(feature = "use-platform-strlen")]
#[inline]
pub unsafe fn strlen(c_str: *const c_char) -> usize {
    sys::strlen(c_str) as usize
}
#[cfg(not(feature = "use-platform-strlen"))]
pub unsafe fn strlen(c_str: *const c_char) -> usize {
    if c_str.is_null() {
        return 0;
    }

    let mut iter = c_str as usize;
    while *(iter as *const u8) != 0 {
        iter += 1;
    }

    iter - (c_str as usize)
}

pub unsafe fn start_scheduler() -> ! {
    sys::vTaskStartScheduler();
    unreachable!();
}

pub unsafe fn port_malloc(xWantedSize: usize) -> *mut c_void {
    sys::pvPortMalloc(xWantedSize)
}
pub unsafe fn port_free(pv: *mut c_void) {
    sys::vPortFree(pv)
}

pub unsafe fn task_delay_until(
    pxPreviousWakeTime: *mut TickType,
    xTimeIncrement: TickType,
) -> bool {
    sys::xTaskDelayUntil(pxPreviousWakeTime, xTimeIncrement) == sys::pdTRUE
}
pub unsafe fn task_delay(xTicksToDelay: TickType) {
    sys::vTaskDelay(xTicksToDelay)
}

pub unsafe fn get_number_of_tasks() -> UBaseType {
    sys::uxTaskGetNumberOfTasks()
}

pub unsafe fn task_get_tick_count() -> TickType {
    sys::xTaskGetTickCount()
}

pub unsafe fn task_get_tick_count_from_isr() -> TickType {
    sys::xTaskGetTickCountFromISR()
}

pub unsafe fn create_recursive_mutex() -> MaybeQueueHandle {
    mem::transmute(sys::xQueueCreateMutex(sys::queueQUEUE_TYPE_RECURSIVE_MUTEX))
}
pub unsafe fn create_mutex() -> MaybeQueueHandle {
    mem::transmute(sys::xQueueCreateMutex(sys::queueQUEUE_TYPE_MUTEX))
}

pub unsafe fn take_mutex(mutex: QueueHandle, max: TickType) -> bool {
    sys::xQueueSemaphoreTake(mutex as *mut _, max) == sys::pdTRUE
}
pub unsafe fn give_mutex(mutex: QueueHandle) -> bool {
    sys::xQueueGenericSend(mutex as *mut _, ptr::null(), 0, 0) == sys::pdTRUE
}
pub unsafe fn give_recursive_mutex(mutex: QueueHandle) -> bool {
    sys::xQueueGiveMutexRecursive(mutex as *mut _) == sys::pdTRUE
}
pub unsafe fn take_recursive_mutex(mutex: QueueHandle, max: TickType) -> bool {
    sys::xQueueTakeMutexRecursive(mutex as *mut _, max) == sys::pdTRUE
}

pub unsafe fn delete_semaphore(mutex: QueueHandle) {
    sys::vQueueDelete(mutex as *mut _)
}

pub unsafe fn create_binary_semaphore() -> MaybeQueueHandle {
    mem::transmute(sys::xQueueGenericCreate(
        1,
        0,
        sys::queueQUEUE_TYPE_BINARY_SEMAPHORE,
    ))
}
pub unsafe fn create_counting_semaphore(max: UBaseType, initial: UBaseType) -> MaybeQueueHandle {
    mem::transmute(sys::xQueueCreateCountingSemaphore(max, initial))
}

pub unsafe fn queue_create(length: UBaseType, item_size: UBaseType) -> MaybeQueueHandle {
    mem::transmute(sys::xQueueGenericCreate(
        length,
        item_size,
        sys::queueQUEUE_TYPE_BASE,
    ))
}
pub unsafe fn queue_delete(queue: QueueHandle) {
    sys::vQueueDelete(queue as *mut _)
}
pub unsafe fn queue_send(queue: QueueHandle, item: *const c_void, max_wait: TickType) -> bool {
    sys::xQueueGenericSend(queue as *mut _, item, max_wait, 0) == sys::pdTRUE
}
pub unsafe fn queue_receive(queue: QueueHandle, item: *mut c_void, max_wait: TickType) -> bool {
    sys::xQueueReceive(queue as *mut _, item, max_wait) == sys::pdTRUE
}

pub unsafe fn queue_send_isr(
    queue: QueueHandle,
    item: *const c_void,
    xHigherPriorityTaskWoken: *mut BaseType,
) -> bool {
    sys::xQueueGenericSendFromISR(queue as *mut _, item, xHigherPriorityTaskWoken, 0) == sys::pdTRUE
}
pub unsafe fn isr_yield() {
    sys::vPortYield()
}

pub unsafe fn task_notify_take(clear_count: bool, wait: TickType) -> u32 {
    sys::ulTaskGenericNotifyTake(
        0,
        if clear_count {
            sys::pdTRUE
        } else {
            sys::pdFALSE
        },
        wait,
    )
}
pub unsafe fn task_notify_wait(
    ulBitsToClearOnEntry: u32,
    ulBitsToClearOnExit: u32,
    pulNotificationValue: *mut u32,
    xTicksToWait: TickType,
) -> bool {
    sys::xTaskGenericNotifyWait(
        0,
        ulBitsToClearOnEntry,
        ulBitsToClearOnExit,
        pulNotificationValue,
        xTicksToWait,
    ) == sys::pdPASS
}

pub unsafe fn task_notify(task: TaskHandle, value: u32, action: sys::eNotifyAction) -> bool {
    sys::xTaskGenericNotify(task as *mut _, 0, value, action, ptr::null_mut()) == sys::pdPASS
}
pub unsafe fn task_notify_isr(
    task: TaskHandle,
    value: u32,
    action: sys::eNotifyAction,
    xHigherPriorityTaskWoken: *mut BaseType,
) -> bool {
    sys::xTaskGenericNotifyFromISR(
        task as *mut _,
        0,
        value,
        action,
        ptr::null_mut(),
        xHigherPriorityTaskWoken,
    ) == sys::pdPASS
}

pub unsafe fn create_task(
    f: extern "C" fn(*mut c_void),
    value: *mut c_void,
    name: &str,
    stack_size: u16,
    priority: UBaseType,
    task_handle: *mut TaskHandle,
) -> bool {
    let mut buf = [0u8; sys::configMAX_TASK_NAME_LEN as usize];
    let name_bytes = name.as_bytes();
    let size = (sys::configMAX_TASK_NAME_LEN as usize - 1).min(name_bytes.len());
    buf[..size].copy_from_slice(&name_bytes[..size]);

    sys::xTaskCreate(
        Some(f),
        buf.as_ptr() as _,
        stack_size,
        value,
        priority,
        task_handle as _,
    ) == sys::pdPASS
}
pub unsafe fn delete_task(task: TaskHandle) {
    sys::vTaskDelete(task as _)
}
pub unsafe fn task_get_name(task: TaskHandle) -> *const c_char {
    sys::pcTaskGetName(task as _)
}
pub unsafe fn get_stack_high_water_mark(task: TaskHandle) -> UBaseType {
    sys::uxTaskGetStackHighWaterMark(task as _)
}

pub unsafe fn get_current_task() -> MaybeTaskHandle {
    mem::transmute(sys::xTaskGetCurrentTaskHandle())
}
pub unsafe fn get_system_state(
    tasks: *mut TaskStatusFfi,
    tasks_len: UBaseType,
    total_run_time: *mut u32,
) -> UBaseType {
    sys::uxTaskGetSystemState(tasks as _, tasks_len, total_run_time)
}

pub unsafe fn timer_create(
    name: &str,
    period: TickType,
    auto_reload: bool,
    timer_id: usize,
    callback: extern "C" fn(TimerHandle),
) -> MaybeTimerHandle {
    let mut buf = [0u8; sys::configMAX_TASK_NAME_LEN as usize];
    let name_bytes = name.as_bytes();
    let size = (sys::configMAX_TASK_NAME_LEN as usize - 1).min(name_bytes.len());
    buf[..size].copy_from_slice(&name_bytes[..size]);

    mem::transmute(sys::xTimerCreate(
        buf.as_ptr() as _,
        period,
        if auto_reload {
            sys::pdTRUE as _
        } else {
            sys::pdFALSE as _
        },
        timer_id as _,
        Some(mem::transmute(callback)),
    ))
}
pub unsafe fn timer_start(timer: TimerHandle, block_time: TickType) -> bool {
    sys::xTimerGenericCommand(
        timer as _,
        1,
        sys::xTaskGetTickCount(),
        ptr::null_mut(),
        block_time,
    ) == sys::pdPASS
}
pub unsafe fn timer_stop(timer: TimerHandle, block_time: TickType) -> bool {
    sys::xTimerGenericCommand(timer as _, 3, 0, ptr::null_mut(), block_time) == sys::pdPASS
}
pub unsafe fn timer_delete(timer: TimerHandle, block_time: TickType) -> bool {
    sys::xTimerGenericCommand(timer as _, 5, 0, ptr::null_mut(), block_time) == sys::pdPASS
}
pub unsafe fn timer_change_period(
    timer: TimerHandle,
    block_time: TickType,
    new_period: TickType,
) -> bool {
    sys::xTimerGenericCommand(timer as _, 4, new_period, ptr::null_mut(), block_time) == sys::pdPASS
}
pub unsafe fn timer_reset(timer: TimerHandle, block_time: TickType) -> bool {
    sys::xTimerGenericCommand(
        timer as _,
        2,
        sys::xTaskGetTickCount(),
        ptr::null_mut(),
        block_time,
    ) == sys::pdPASS
}
pub unsafe fn timer_get_id(timer: TimerHandle) -> usize {
    sys::pvTimerGetTimerID(timer as _) as usize
}

pub unsafe fn enter_critical() -> UBaseType {
    sys::ulTaskEnterCriticalFromISR()
}
pub unsafe fn exit_critical(previous_state: UBaseType) {
    sys::vTaskExitCriticalFromISR(previous_state)
}
