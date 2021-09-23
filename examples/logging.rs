use emerald::*;

pub fn main() {
    emerald::start(LoggingExample {}, GameSettings::default())
}

pub struct LoggingExample;
impl Game for LoggingExample {
    fn update(&mut self, mut emd: Emerald) {
        emd.logger().info("hey we're logging in emerald").unwrap();
    }
}
