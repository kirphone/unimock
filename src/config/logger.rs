use log4rs::append::console::ConsoleAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::{Config, Handle};
use log4rs::config::{Appender, Root};
use log::LevelFilter;

const PRINT_PATTERN: &str = "{d(%Y-%m-%d %H:%M:%S%.3f)} {l} {M} {T} - {m}{n}";
const LOG_FILE_PATH: &str = "logs/current.log";
const MAX_LOG_FILE_SIZE: u64 = 10 * 1024 * 1024;
const LOG_ROLLING_ARCHIVE_PATH: &str = "logs/{}.log.zip";
const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::Debug;

pub fn init_logger_handler() -> Handle {
    log4rs::init_config(build_config(DEFAULT_LOG_LEVEL)).unwrap()
}

pub fn change_logger_level(handle: &Handle, log_level: LevelFilter){
    handle.set_config(build_config(log_level));
}

fn build_config(log_level: LevelFilter) -> Config {
    let console_logger = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(PRINT_PATTERN)))
        .build();
    let file_logger = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(PRINT_PATTERN)))
        .build(LOG_FILE_PATH, Box::new(CompoundPolicy::new(Box::new(SizeTrigger::new(MAX_LOG_FILE_SIZE)),
                                                           Box::new(FixedWindowRoller::builder().base(1).build(LOG_ROLLING_ARCHIVE_PATH, 5).unwrap())))).unwrap();

    Config::builder()
        .appender(Appender::builder().build("console", Box::new(console_logger)))
        .appender(Appender::builder().build("file-log", Box::new(file_logger)))
        .build(Root::builder()
            .appenders(["console".to_string(), "file-log".to_string()].iter())
            .build(log_level))
        .unwrap()
}