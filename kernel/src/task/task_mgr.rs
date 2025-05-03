#![allow(dead_code)]
use alloc::{collections::BTreeMap, vec::Vec};

use crate::sync::{RwLock, SpinLock};

use super::task::Task;

struct TaskManager {
    tasks: RwLock<BTreeMap<usize, Task>>,
    ids: SpinLock<IdentifierGenerator>,
}

struct IdentifierGenerator {
    top: usize,
    stack: Vec<usize>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks: RwLock::new(BTreeMap::new()),
            ids: SpinLock::new(IdentifierGenerator::new(1)),
        }
    }

    pub fn access_task<F>(&self, key: usize, callback: F)
    where
        F: FnOnce(Option<&Task>) -> (),
    {
        callback(self.tasks.shared_access().get(&key));
    }

    pub fn add_task(&self, task: Task) {
        self.tasks
            .exclusive_access()
            .insert(self.ids.lock().alloc(), task);
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
