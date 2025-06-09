#![allow(dead_code)]

use alloc::{collections::LinkedList, sync::Arc, vec::Vec};

use crate::{
    INIT_PROGRAM,
    mm::utils::free_initial_page_table,
    sync::SpinLockNoIrq,
    task::elf::{MemoryReader, Readable},
    trace,
};

use super::task::Task;

pub struct TaskManager {
    tasks: LinkedList<Task>,
    current_task: Option<Task>,
    ids: IdentifierGenerator,
}

struct IdentifierGenerator {
    top: usize,
    stack: Vec<usize>,
}

impl TaskManager {
    pub const fn new() -> Self {
        Self {
            tasks: LinkedList::new(),
            current_task: None,
            ids: IdentifierGenerator::new(1),
        }
    }

    pub fn add_task<R: Readable>(&mut self, elf_file: R) {
        let id = self.ids.alloc();
        let task = Task::new(id, elf_file);
        self.tasks.push_back(task);
    }

    pub fn current_task(&self) -> Option<&Task> {
        self.current_task.as_ref()
    }

    pub fn rotate_tasks(&mut self) -> Option<&Task> {
        if self.tasks.is_empty() {
            self.current_task.as_ref()
        } else {
            let next_task = self.tasks.pop_back();
            if self.current_task().is_some() {
                self.tasks.push_front(self.current_task.take().unwrap());
            }
            self.current_task = next_task;
            self.current_task.as_ref()
        }
    }

    pub fn remove_task(&mut self, id: usize) {
        if let Some(ref current) = self.current_task {
            if current.id() == id {
                self.current_task = None;
                self.rotate_tasks();
                return;
            }
        }

        let mut cursor = self.tasks.cursor_front_mut();
        while let Some(task) = cursor.current() {
            if task.id() == id {
                cursor.remove_current();
                break;
            }
            cursor.move_next();
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

pub static TASK_MANAGER: SpinLockNoIrq<TaskManager> = SpinLockNoIrq::new(TaskManager::new());

pub fn init_first_process_and_jump_to() -> ! {
    trace!("Preparing for init task...");
    let task = {
        let mut lock = TASK_MANAGER.lock();
        lock.add_task(MemoryReader::new(INIT_PROGRAM.as_ptr(), INIT_PROGRAM.len()));
        lock.rotate_tasks();
        lock.current_task().expect("Failure create init.") as *const Task
    };
    free_initial_page_table();
    trace!("Init starts.");
    unsafe { (*task).jump_to() }
}

pub unsafe fn schedule_next_task() -> ! {
    TASK_MANAGER.lock().rotate_tasks();
    let task = TASK_MANAGER.lock().current_task().expect("No task to run.") as *const Task;
    unsafe { (*task).jump_to() }
}
