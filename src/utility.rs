use std::time::Instant;
pub struct Clock {
    start_time: Instant,
}

impl Clock {
    pub fn new() -> Self {
        Clock {
            start_time: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> u128 {
        self.start_time.elapsed().as_micros()
    }

    pub fn reset(&mut self) {
        self.start_time = Instant::now();
    }
}
