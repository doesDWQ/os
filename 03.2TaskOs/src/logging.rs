
use log::{Level, Log};

use crate::println;

struct SimpleLogger;

impl Log for SimpleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let color = match record.level() {
            Level::Error => 31, // 红色
            Level::Warn => 93,  // 亮黄色
            Level::Info => 34,  // 蓝色
            Level::Debug => 32, // 绿色
            Level::Trace => 90, // 亮黑色
        };

        println!("\u{1B}[{}m[{:>5}] {}\u{1B}[0m", color, record.level(), record.args() )
    }

    fn flush(&self) {}
}

pub fn init() {
    static LOGGER: SimpleLogger = SimpleLogger;
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(match option_env!("LOG") {
        Some("Error") => log::LevelFilter::Error,
        Some("Warn") => log::LevelFilter::Warn,
        Some("Info") => log::LevelFilter::Info,
        Some("Debug") => log::LevelFilter::Debug,
        Some("Trace") => log::LevelFilter::Trace,
        // _ => log::LevelFilter::Off,
        _ => log::LevelFilter::Trace,
    })
}