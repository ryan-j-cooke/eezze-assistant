// here
use std::time::Instant;

pub struct Timer {
    start_time: Instant,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }

    pub fn reset(&mut self) {
        self.start_time = Instant::now();
    }

    pub fn elapsed_ms(&self) -> u128 {
        self.start_time.elapsed().as_millis()
    }
}

pub async fn measure<T, F, Fut>(f: F) -> (T, u128)
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    let mut timer = Timer::new();
    let result = f().await;
    let duration = timer.elapsed_ms();
    (result, duration)
}