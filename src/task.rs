use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Task(Arc<AtomicUsize>);

impl Task {
    pub fn new<T> (f :T, delay :usize) -> Self
        where
            T: FnOnce() -> (),
            T: std::marker::Send + 'static
    {
        let d = Arc::new(AtomicUsize::new(delay));
        let task = Task(Arc::clone(&d));
        thread::spawn(move || {
            while d.load(Ordering::Relaxed) > 0 {
                thread::sleep(Duration::from_secs(1));
                d.fetch_sub(1, Ordering::SeqCst);
            }
            f();
        });
        task
    }

    pub fn delay(&self) -> usize {
        self.0.load(Ordering::Relaxed)
    }
}
