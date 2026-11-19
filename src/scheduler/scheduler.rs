use std::sync::atomic::{AtomicU64, Ordering};
use super::Task;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub struct Scheduler {
    tasks: parking_lot::RwLock<Vec<Task>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self { tasks: parking_lot::RwLock::new(Vec::new()) }
    }

    pub fn schedule_repeating<F>(&self, delay_ticks: u32, period_ticks: u32, f: F) -> u64
    where
        F: Fn(u64) + Send + Sync + 'static,
    {
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        let task = Task::new_repeating(id, delay_ticks, period_ticks, Box::new(f));
        self.tasks.write().push(task);
        id
    }

    pub fn schedule<F>(&self, delay_ticks: u32, f: F) -> u64
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        let mut f = Some(f);
        let task = Task::new_once(id, delay_ticks, Box::new(move || {
            if let Some(callback) = f.take() {
                callback();
            }
        }));
        self.tasks.write().push(task);
        id
    }

    pub fn tick(&self, current_tick: u64) {
        let mut tasks = self.tasks.write();
        let mut to_remove = Vec::new();
        for (i, task) in tasks.iter_mut().enumerate() {
            if current_tick >= task.next_run {
                if task.is_repeating {
                    (task.callback)(current_tick);
                    task.next_run = current_tick + task.period_ticks as u64;
                } else {
                    (task.callback_once)();
                    to_remove.push(i);
                }
            }
        }
        for i in to_remove.into_iter().rev() {
            tasks.remove(i);
        }
    }

    pub fn cancel(&self, id: u64) -> bool {
        let mut tasks = self.tasks.write();
        if let Some(pos) = tasks.iter().position(|t| t.id == id) {
            tasks.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn count(&self) -> usize {
        self.tasks.read().len()
    }
}

impl Default for Scheduler {
    fn default() -> Self { Self::new() }
}
