extern crate alloc;

use alloc::vec::Vec;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

/// The number of tasks can fit into a type usize.
pub type TaskNumberType = usize;

// TODO: rewrite with cfg!
#[cfg(not(feature = "c-library"))]
/// Type of setup function, that is called once at the beginning of task.
type TaskSetupFunctionType = fn() -> ();
#[cfg(feature = "c-library")]
/// Type of setup function, that is called once at the beginning of task.
type TaskSetupFunctionType = extern "C" fn() -> ();
#[cfg(not(feature = "c-library"))]
/// Type of loop function, that is called in loop.
type TaskLoopFunctionType = fn() -> ();
#[cfg(feature = "c-library")]
/// Type of loop function, that is called in loop.
type TaskLoopFunctionType = extern "C" fn() -> ();
#[cfg(not(feature = "c-library"))]
/// Type of condition function for stopping loop function execution.
type TaskStopConditionFunctionType = fn() -> bool;
#[cfg(feature = "c-library")]
/// Type of condition function for stopping loop function execution.
type TaskStopConditionFunctionType = extern "C" fn() -> bool;

#[repr(C)]
/// Task representation for task manager.
struct Task {
    /// Setup function, that is called once at the beginning of task.
    setup_fn: TaskSetupFunctionType,
    /// Loop function, that is called in loop.
    loop_fn: TaskLoopFunctionType,
    /// Condition function for stopping loop function execution.
    stop_condition_fn: TaskStopConditionFunctionType,
}

#[repr(C)]
/// Future shell for task for execution.
struct FutureTask {
    /// Task to execute in task manager.
    task: Task,
    /// Marker for setup function completion.
    is_setup_completed: bool,
}

impl Future for FutureTask {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut array: [usize; 8] = core::array::from_fn(|i| i);
        array[0] = 5;
        if (self.task.stop_condition_fn)() {
            Poll::Ready(())
        } else {
            if !self.is_setup_completed {
                (self.task.setup_fn)();
                self.is_setup_completed = true;
            } else {
                (self.task.loop_fn)();
            }
            Poll::Pending
        }
    }
}

/// Creates simple task waker. May be more difficult in perspective.
fn task_waker() -> Waker {
    fn raw_clone(_: *const ()) -> RawWaker {
        RawWaker::new(core::ptr::null::<()>(), &NOOP_WAKER_VTABLE)
    }

    fn raw_wake(_: *const ()) {}

    fn raw_wake_by_ref(_: *const ()) {}

    fn raw_drop(_: *const ()) {}

    static NOOP_WAKER_VTABLE: RawWakerVTable =
        RawWakerVTable::new(raw_clone, raw_wake, raw_wake_by_ref, raw_drop);

    let raw_waker = RawWaker::new(core::ptr::null::<()>(), &NOOP_WAKER_VTABLE);
    unsafe { Waker::from_raw(raw_waker) }
}

#[repr(C)]
/// Task manager representation. Based on round-robin scheduling without priorities.
pub struct TaskManager {
    /// Vector of tasks to execute.
    tasks: Vec<FutureTask>,
    /// Index of task, that should be executed.
    task_to_execute_index: TaskNumberType,
}

/// Operating system task manager.
static mut TASK_MANAGER: TaskManager = TaskManager::new();

impl TaskManager {
    /// Creates new task manager.
    const fn new() -> TaskManager {
        TaskManager {
            tasks: Vec::new(),
            task_to_execute_index: 0,
        }
    }

    /// Add task to task manager. You should pass setup, loop and condition functions.
    pub fn add_task(
        setup_fn: TaskSetupFunctionType,
        loop_fn: TaskLoopFunctionType,
        stop_condition_fn: TaskStopConditionFunctionType,
    ) {
        let task = Task {
            setup_fn,
            loop_fn,
            stop_condition_fn,
        };
        let future_task = FutureTask {
            task,
            is_setup_completed: false,
        };
        unsafe {
            TASK_MANAGER.tasks.push(future_task);
        }
    }

    /// One step of task manager's work.
    // TODO: Support priorities.
    // TODO: Delete tasks from task vector if they are pending?
    fn task_manager_step() {
        if unsafe { !TASK_MANAGER.tasks.is_empty() } {
            let waker = task_waker();

            let task = unsafe { &mut TASK_MANAGER.tasks[TASK_MANAGER.task_to_execute_index] };
            let mut task_future_pin = Pin::new(task);
            let _ = task_future_pin
                .as_mut()
                .poll(&mut Context::from_waker(&waker));

            unsafe {
                if TASK_MANAGER.task_to_execute_index + 1 < TASK_MANAGER.tasks.len() {
                    TASK_MANAGER.task_to_execute_index += 1;
                } else {
                    TASK_MANAGER.task_to_execute_index = 0;
                }
            }
        }
    }

    /// Starts task manager work.
    pub fn start_task_manager() -> ! {
        loop {
            TaskManager::task_manager_step();
        }
    }

    /// Starts task manager work. Returns after 1000 steps only for testing task_manager_step.
    pub fn test_start_task_manager() {
        for _n in 1..=1000 {
            TaskManager::task_manager_step();
        }
    }
}