use std::{sync::Mutex, thread};

use rand::Rng;

pub struct ThreadPool {
    resources: Vec<Mutex<bool>>,
}

impl ThreadPool {
    pub fn new(_size: Option<u32>) -> ThreadPool {
        let size = _size.unwrap_or(10);
        ThreadPool {
            resources: (0..size).map(|_| Mutex::new(true)).collect(),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let round_robin_idx = rand::thread_rng().gen_range(0..=self.resources.len());
        match self.resources.get(round_robin_idx) {
            Some(resource) => {
                let _ = resource.lock();
                thread::spawn(f);
            }
            None => return,
        }
    }
}
