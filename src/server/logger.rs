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
        let padding = 5;
        match self {
            LogLevel::ERROR => write!(f, "{:padding$}", "ERROR".bold().red(), padding = padding),
            LogLevel::WARN => write!(f, "{:padding$}", "WARN".bold().yellow(), padding = padding),
            LogLevel::INFO => write!(f, "{:padding$}", "INFO".bold().green(), padding = padding),
            LogLevel::DEBUG => write!(f, "{:padding$}", "DEBUG".bold().blue(), padding = padding),
            LogLevel::TRACE => write!(f, "{:padding$}", "TRACE".bold().magenta(), padding = padding),
        }
    }
}

#[derive(Debug)]
pub struct Logger {
    level: LogLevel,
}

impl Logger {
    pub fn new(level: LogLevel) -> Self {
        Logger { level }
    }

    pub fn log(&self, level: LogLevel, message: &str, module: &str) {
        if level <= self.level {
            let now = Local::now();
            let timestamp = now.format("%Y-%m-%d %H:%M:%S%.3f").to_string().dimmed();
            let module_name = format!("{:>15}", module).cyan();
            
            let log_entry = format!(
                "{} {} {} │ {}", 
                timestamp,
                level,
                module_name,
                message
            );

            println!("{}", log_entry);
        }
    }

    pub fn error(&self, message: &str, module: &str) {
        self.log(LogLevel::ERROR, &message.red().to_string(), module);
    }

    pub fn warn(&self, message: &str, module: &str) {
        self.log(LogLevel::WARN, &message.yellow().to_string(), module);
    }

    pub fn info(&self, message: &str, module: &str) {
        self.log(LogLevel::INFO, message, module);
    }

    pub fn debug(&self, message: &str, module: &str) {
        self.log(LogLevel::DEBUG, &message.blue().to_string(), module);
    }

    pub fn trace(&self, message: &str, module: &str) {
        self.log(LogLevel::TRACE, &message.magenta().to_string(), module);
    }
}