pub type Callback = Box<dyn Fn(u64) + Send + Sync>;
pub type CallbackOnce = Box<dyn FnMut() + Send + Sync>;

pub struct Task {
    pub id: u64,
    pub next_run: u64,
    pub period_ticks: u32,
    pub is_repeating: bool,
    pub callback: Callback,
    pub callback_once: CallbackOnce,
}

impl Task {
    pub fn new_repeating(id: u64, delay_ticks: u32, period_ticks: u32, callback: Callback) -> Self {
        Self {
            id,
            next_run: delay_ticks as u64,
            period_ticks,
            is_repeating: true,
            callback,
            callback_once: Box::new(|| {}),
        }
    }

    pub fn new_once(id: u64, delay_ticks: u32, callback: CallbackOnce) -> Self {
        Self {
            id,
            next_run: delay_ticks as u64,
            period_ticks: 0,
            is_repeating: false,
            callback: Box::new(|_| {}),
            callback_once: callback,
        }
    }
}
