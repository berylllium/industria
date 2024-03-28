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

pub mod fs {
    use std::{io::Cursor, path::Path};

    pub fn load<P: AsRef<Path>>(path: P) -> Cursor<Vec<u8>> {
        use std::fs::File;
        use std::io::Read;
        
        let mut buf = Vec::new();
        let fullpath = &Path::new("assets").join(&path);
        let mut file = File::open(&fullpath).unwrap();
        file.read_to_end(&mut buf).unwrap();

        Cursor::new(buf)
    }
}
