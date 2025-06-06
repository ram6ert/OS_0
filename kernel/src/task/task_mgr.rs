#![allow(dead_code)]
use alloc::{collections::LinkedList, vec::Vec};

use crate::{
    sync::{RwLock, SpinLock},
    trace,
};

use super::task::Task;

pub struct TaskManager {
    tasks: RwLock<LinkedList<Task>>,
    ids: SpinLock<IdentifierGenerator>,
}

struct IdentifierGenerator {
    top: usize,
    stack: Vec<usize>,
}

impl TaskManager {
    pub const fn new() -> Self {
        Self {
            tasks: RwLock::new(LinkedList::new()),
            ids: SpinLock::new(IdentifierGenerator::new(1)),
        }
    }

    pub fn add_task(&self, entry: usize) {
        let id = self.ids.lock().alloc();
        let task = Task::new(id, entry);
        self.tasks.exclusive_access().push_back(task);
    }

    pub fn access_current_task<F: FnOnce(Option<&Task>)>(&self, callback: F) {
        callback(self.tasks.shared_access().front())
    }

    pub unsafe fn schedule_next_task(&self) -> ! {
        let mut lock = self.tasks.exclusive_access();
        let tasks = &mut *lock;
        let task_option = tasks.pop_back();
        if let Some(task) = task_option {
            tasks.push_front(task);
            trace!("After");
            unsafe { tasks.front().unwrap().jump_to() }
        } else {
            panic!("No more task to schedule.")
        }
    }
}

impl IdentifierGenerator {
    pub const fn new(top: usize) -> Self {
        Self {
            top,
            stack: Vec::new(),
        }
    }

    pub fn alloc(&mut self) -> usize {
        if !self.stack.is_empty() {
            self.stack.pop().unwrap()
        } else {
            let result = self.top;
            self.top += 1;
            result
        }
    }

    pub fn free(&mut self, id: usize) {
        self.stack.push(id);
    }
}

pub static TASK_MANAGER: SpinLock<TaskManager> = SpinLock::new(TaskManager::new());
