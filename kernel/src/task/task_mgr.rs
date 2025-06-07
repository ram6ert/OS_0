#![allow(dead_code)]

use alloc::{collections::LinkedList, sync::Arc, vec::Vec};

use crate::{
    INIT_PROGRAM,
    mm::utils::free_initial_page_table,
    sync::{RwLock, SpinLock},
    task::elf::{MemoryReader, Readable},
};

use super::task::Task;

pub struct TaskManager {
    tasks: RwLock<LinkedList<Arc<Task>>>,
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

    pub fn add_task<R: Readable>(&self, elf_file: R) {
        let id = self.ids.lock().alloc();
        let task = Task::new(id, elf_file);
        let arc = Arc::new(task);
        self.tasks.exclusive_access().push_back(arc);
    }

    pub fn current_task(&self) -> Option<Arc<Task>> {
        self.tasks.shared_access().front().cloned()
    }

    pub fn rotate_tasks(&self) -> Option<Arc<Task>> {
        let mut lock = self.tasks.exclusive_access();
        let tasks = &mut *lock;
        let task_option = tasks.pop_back();
        if let Some(task) = task_option {
            tasks.push_front(task);
            tasks.front().cloned()
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

pub fn init_first_process_and_jump_to() -> ! {
    let task = {
        let lock = TASK_MANAGER.lock();
        lock.add_task(MemoryReader::new(INIT_PROGRAM.as_ptr(), INIT_PROGRAM.len()));
        lock.current_task()
    };
    free_initial_page_table();
    unsafe { task.unwrap().jump_to() }
}

pub fn schedule_next_task() -> ! {
    let task = TASK_MANAGER.lock().rotate_tasks();
    if let Some(task) = task {
        unsafe { task.jump_to() }
    } else {
        panic!("No task to run.");
    }
}
