use core::fmt;
use core::mem;
use core::ptr;

use crate::base::*;
use crate::glue;
use crate::isr::*;
use crate::prelude::*;
use crate::units::*;
use crate::utils::*;

unsafe impl Send for Task {}

/// Handle for a FreeRTOS task
#[derive(Debug, Clone)]
pub struct Task {
    task_handle: TaskHandle,
}

/// Task's execution priority. Low priority numbers denote low priority tasks.
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct TaskPriority(pub u8);

/// Task notify actions that can be performed
///
/// Corresponds to the same variants in [`TaskNotification`]
#[derive(Clone, Copy)]
#[repr(C)]
enum NotifyAction {
    NoAction = 0,
    SetBits,
    Increment,
    SetValueWithOverwrite,
    SetValueWithoutOverwrite,
}

/// Notification to be sent to a task.
#[derive(Debug, Copy, Clone)]
pub enum TaskNotification {
    /// Send the event, unblock the task, the task's notification value isn't changed.
    NoAction,
    /// Perform a logical or with the task's notification value.
    SetBits(u32),
    /// Increment the task's notification value by one.
    Increment,
    /// Set the task's notification value to this value.
    OverwriteValue(u32),
    /// Try to set the task's notification value to this value. Succeeds
    /// only if the task has no pending notifications. Otherwise, the
    /// notification call will fail.
    SetValue(u32),
}

impl TaskNotification {
    fn to_freertos(&self) -> (u32, NotifyAction) {
        match *self {
            TaskNotification::NoAction => (0, NotifyAction::NoAction),
            TaskNotification::SetBits(v) => (v, NotifyAction::SetBits),
            TaskNotification::Increment => (0, NotifyAction::Increment),
            TaskNotification::OverwriteValue(v) => (v, NotifyAction::SetValueWithOverwrite),
            TaskNotification::SetValue(v) => (v, NotifyAction::SetValueWithoutOverwrite),
        }
    }
}

impl TaskPriority {
    fn to_freertos(&self) -> UBaseType {
        self.0 as UBaseType
    }
}

/// Helper for spawning a new task. Instantiate with [`Task::new()`].
///
/// [`Task::new()`]: struct.Task.html#method.new
pub struct TaskBuilder {
    task_name: String,
    task_stack_size: u16,
    task_priority: TaskPriority,
}

impl TaskBuilder {
    /// Set the task's name.
    pub fn name(&mut self, name: &str) -> &mut Self {
        self.task_name = name.into();
        self
    }

    /// Set the stack size, in words.
    pub fn stack_size(&mut self, stack_size: u16) -> &mut Self {
        self.task_stack_size = stack_size;
        self
    }

    /// Set the task's priority.
    pub fn priority(&mut self, priority: TaskPriority) -> &mut Self {
        self.task_priority = priority;
        self
    }

    /// Start a new task that can't return a value.
    pub fn start<F>(&self, func: F) -> Result<Task, FreeRtosError>
    where
        F: FnOnce(Task) -> (),
        F: Send + 'static,
    {
        Task::spawn(
            &self.task_name,
            self.task_stack_size,
            self.task_priority,
            func,
        )
    }
}

impl Task {
    /// Prepare a builder object for the new task.
    pub fn new() -> TaskBuilder {
        TaskBuilder {
            task_name: "rust_task".into(),
            task_stack_size: 1024,
            task_priority: TaskPriority(1),
        }
    }

    fn spawn<F>(
        name: &str,
        stack_size: u16,
        priority: TaskPriority,
        f: F,
    ) -> Result<Task, FreeRtosError>
    where
        F: FnOnce(Task) + Send + 'static,
    {
        unsafe {
            let mut f = Box::new(f);
            let param_ptr = f.as_mut() as *mut F as *mut _;

            let (success, task_handle) = {
                let mut task_handle = mem::zeroed::<TaskHandle>();

                let ret = glue::create_task(
                    Self::thread_start::<F>,
                    param_ptr,
                    name,
                    stack_size,
                    priority.to_freertos(),
                    &mut task_handle,
                );

                (ret, task_handle)
            };

            if success {
                mem::forget(f);
                Ok(Task { task_handle })
            } else {
                Err(FreeRtosError::OutOfMemory)
            }
        }
    }

    extern "C" fn thread_start<F: FnOnce(Task)>(arg: *mut c_void) {
        unsafe {
            let b = Box::from_raw(arg as *mut F);
            b(Task::current());
            glue::delete_task(ptr::null_mut());
        }
    }

    /// Get the name of the current task.
    pub fn get_name(&self) -> String {
        unsafe { str_from_c_string(&glue::task_get_name(self.task_handle)).to_owned() }
    }

    /// Try to find the task of the current execution context.
    pub fn current() -> Task {
        unsafe {
            Task {
                task_handle: glue::get_current_task().unwrap().as_ptr(),
            }
        }
    }

    /// Forcibly set the notification value for this task.
    pub fn set_notification_value(&self, val: u32) {
        self.notify(TaskNotification::OverwriteValue(val))
    }

    /// Notify this task.
    pub fn notify(&self, notification: TaskNotification) {
        unsafe {
            let n = notification.to_freertos();
            glue::task_notify(self.task_handle, n.0, n.1 as _);
        }
    }

    /// Notify this task from an interrupt.
    pub fn notify_from_isr(
        &self,
        context: &InterruptContext,
        notification: TaskNotification,
    ) -> Result<(), FreeRtosError> {
        unsafe {
            let (value, action) = notification.to_freertos();
            if glue::task_notify_isr(
                self.task_handle,
                value,
                action as _,
                context.get_task_field_mut(),
            ) {
                Ok(())
            } else {
                Err(FreeRtosError::QueueFull)
            }
        }
    }

    /// Take the notification and either clear the notification value or decrement it by one.
    pub fn take_notification(&self, clear: bool, wait_for: impl Into<Ticks>) -> u32 {
        unsafe { glue::task_notify_take(clear, wait_for.into().ticks) }
    }

    /// Wait for a notification to be posted.
    pub fn wait_for_notification(
        &self,
        clear_bits_enter: u32,
        clear_bits_exit: u32,
        wait_for: impl Into<Ticks>,
    ) -> Result<u32, FreeRtosError> {
        let mut val = 0;
        if unsafe {
            glue::task_notify_wait(
                clear_bits_enter,
                clear_bits_exit,
                &mut val as *mut _,
                wait_for.into().ticks,
            )
        } {
            Ok(val)
        } else {
            Err(FreeRtosError::Timeout)
        }
    }

    /// Get the minimum amount of stack that was ever left on this task.
    pub fn get_stack_high_water_mark(&self) -> u32 {
        unsafe { glue::get_stack_high_water_mark(self.task_handle) as u32 }
    }
}

/// Helper methods to be performed on the task that is currently executing.
pub struct CurrentTask;

impl CurrentTask {
    /// Delay the execution of the current task.
    pub fn delay(delay: impl Into<Ticks>) {
        unsafe {
            glue::task_delay(delay.into().ticks);
        }
    }

    /// Get the minimum amount of stack that was ever left on the current task.
    pub fn get_stack_high_water_mark() -> u32 {
        unsafe { glue::get_stack_high_water_mark(0 as TaskHandle) as u32 }
    }
}

#[derive(Debug)]
pub struct SchedulerState {
    pub tasks: Vec<TaskStatus>,
    pub total_run_time: u32,
}

impl fmt::Display for SchedulerState {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt.write_str("FreeRTOS tasks\r\n")?;

        write!(fmt, "{id: <6} | {name: <16} | {state: <9} | {priority: <8} | {stack: >10} | {cpu_abs: >10} | {cpu_rel: >4}\r\n",
               id = "ID",
               name = "Name",
               state = "State",
               priority = "Priority",
               stack = "Stack left",
               cpu_abs = "CPU",
               cpu_rel = "%"
        )?;

        for task in &self.tasks {
            write!(fmt, "{id: <6} | {name: <16} | {state: <9} | {priority: <8} | {stack: >10} | {cpu_abs: >10} | {cpu_rel: >4}\r\n",
                   id = task.task_number,
                   name = task.name,
                   state = format!("{:?}", task.task_state),
                   priority = task.current_priority.0,
                   stack = task.stack_high_water_mark,
                   cpu_abs = task.run_time_counter,
                   cpu_rel = if self.total_run_time > 0 && task.run_time_counter <= self.total_run_time {
                       let p = (((task.run_time_counter as u64) * 100) / self.total_run_time as u64) as u32;
                       let ps = if p == 0 && task.run_time_counter > 0 {
                           "<1".to_string()
                       } else {
                           p.to_string()
                       };
                       format!("{: >3}%", ps)
                   } else {
                       "-".to_string()
                   }
            )?;
        }

        if self.total_run_time > 0 {
            write!(fmt, "Total run time: {}\r\n", self.total_run_time)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct TaskStatus {
    pub task: Task,
    pub name: String,
    pub task_number: UBaseType,
    pub task_state: TaskState,
    pub current_priority: TaskPriority,
    pub base_priority: TaskPriority,
    pub run_time_counter: u32,
    pub stack_high_water_mark: StackType,
}

pub fn start_scheduler() -> ! {
    unsafe {
        glue::start_scheduler();
    }
}

pub fn get_tick_count() -> TickType {
    unsafe { glue::task_get_tick_count() }
}

pub fn get_tick_count_duration() -> Ticks {
    Ticks::new(get_tick_count())
}

pub fn get_number_of_tasks() -> usize {
    unsafe { glue::get_number_of_tasks() as usize }
}

pub fn get_all_tasks(tasks_len: Option<usize>) -> SchedulerState {
    let tasks_len = tasks_len.unwrap_or(get_number_of_tasks());
    let mut tasks = Vec::with_capacity(tasks_len as usize);
    let mut total_run_time = 0;

    unsafe {
        let filled = glue::get_system_state(
            tasks.as_mut_ptr(),
            tasks_len as UBaseType,
            &mut total_run_time,
        );
        tasks.set_len(filled as usize);
    }

    let tasks = tasks
        .into_iter()
        .map(|t| TaskStatus {
            task: Task {
                task_handle: t.xHandle as _,
            },
            name: unsafe { str_from_c_string(&t.pcTaskName) }.to_owned(),
            task_number: t.xTaskNumber,
            task_state: unsafe { mem::transmute(t.eCurrentState as u8) },
            current_priority: TaskPriority(t.uxCurrentPriority as u8),
            base_priority: TaskPriority(t.uxBasePriority as u8),
            run_time_counter: t.ulRunTimeCounter,
            stack_high_water_mark: t.usStackHighWaterMark as u32,
        })
        .collect();

    SchedulerState {
        tasks: tasks,
        total_run_time: total_run_time,
    }
}
