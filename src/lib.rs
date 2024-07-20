use std::{thread, time::Duration};

pub fn infinite_loop() -> ! {
    loop {
        thread::sleep(Duration::from_secs(u64::MAX));
    }
}
