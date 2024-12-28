use std::fmt;
use colored::*;
use chrono::Local;

#[derive(PartialOrd, PartialEq, Debug)]
pub enum LogLevel {
    ERROR,
    WARN,
    INFO,
    DEBUG,
    TRACE,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LogLevel::ERROR => write!(f, "{}", "ERROR".red()),
            LogLevel::WARN => write!(f, "{}", "WARN".yellow()),
            LogLevel::INFO => write!(f, "{}", "INFOS".green()),
            LogLevel::DEBUG => write!(f, "{}", "DEBUG".blue()),
            LogLevel::TRACE => write!(f, "{}", "TRACE".magenta()),
        }
    }
}


#[derive(Debug)]
pub struct Logger {
    level: LogLevel,
}

impl Logger {
    pub fn new(level: LogLevel) -> Self {
        Logger {
            level
        }
    }

    pub fn log(&self, level: LogLevel, message: &str, module: &str) {
        if level <= self.level {
            let now = Local::now();
            let log_entry = format!(
                "[{}] [{}] [{}] - {}", 
                level, 
                now.format("%Y-%m-%d %H:%M:%S"), 
                module, 
                message
            );

            println!("{}", log_entry);
        }
    }

    pub fn error(&self, message: &str, module: &str) {
        self.log(LogLevel::ERROR, message, module);
    }

    pub fn warn(&self, message: &str, module: &str) {
        self.log(LogLevel::WARN, message, module);
    }

    pub fn info(&self, message: &str, module: &str) {
        self.log(LogLevel::INFO, message, module);
    }

    pub fn debug(&self, message: &str, module: &str) {
        self.log(LogLevel::DEBUG, message, module);
    }

    pub fn trace(&self, message: &str, module: &str) {
        self.log(LogLevel::TRACE, message, module);
    }
}