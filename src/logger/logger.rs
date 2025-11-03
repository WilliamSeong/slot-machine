use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use chrono::Local;
// Define log levels
pub enum LogLevel {
    INFO,
    WARNING,
    ERROR,
    SECURITY,
    TRANSACTION,
    CRITICAL,
}

impl LogLevel {
    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::INFO => "INFO",
            LogLevel::WARNING => "WARNING",
            LogLevel::ERROR => "ERROR",
            LogLevel::SECURITY => "SECURITY",
            LogLevel::TRANSACTION => "TRANSACTION",
            LogLevel::CRITICAL => "CRITICAL",
        }
    }
}

pub struct Logger {
    file: File,
}

impl Logger {
    // Create a new logger that writes to the specified file
    pub fn new(log_path: &str) -> Result<Self, std::io::Error> {
        let path = Path::new(log_path);
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(path)?;
        
        Ok(Logger { file })
    }

    // Write a log entry with the given level and message
    pub fn log(&mut self, level: LogLevel, message: &str) -> Result<(), std::io::Error> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let log_entry = format!("[{}] [{}] {}\n", timestamp, level.as_str(), message);
        
        self.file.write_all(log_entry.as_bytes())?;
        self.file.flush()?;
        
        Ok(())
    }
}

// Helper function to create a logger instance
fn get_logger() -> Result<Logger, std::io::Error> {
    Logger::new("casino_logs.log")
    }

// Public functions to log at different levels
pub fn info(message: &str) {
    if let Ok(mut logger) = get_logger() {
        let _ = logger.log(LogLevel::INFO, message);
    }
}

pub fn warning(message: &str) {
    if let Ok(mut logger) = get_logger() {
        let _ = logger.log(LogLevel::WARNING, message);
    }
}

pub fn error(message: &str) {
    if let Ok(mut logger) = get_logger() {
        let _ = logger.log(LogLevel::ERROR, message);
    }
}

pub fn security(message: &str) {
    if let Ok(mut logger) = get_logger() {
        let _ = logger.log(LogLevel::SECURITY, message);
    }
}

pub fn transaction(message: &str) {
    if let Ok(mut logger) = get_logger() {
        let _ = logger.log(LogLevel::TRANSACTION, message);
    }
}

pub fn critical(message: &str) {
    if let Ok(mut logger) = get_logger() {
        let _ = logger.log(LogLevel::CRITICAL, message);
    }
}

// Log verification functions
pub fn verify_login_attempts(username: &str, time_window_minutes: u32) -> Result<(u32, u32), std::io::Error> {
    let log_content = std::fs::read_to_string("casino_logs.log")?;
    let lines: Vec<&str> = log_content.lines().collect();
    
    let now = Local::now();
    let window_start = now - chrono::Duration::minutes(time_window_minutes as i64);
    
    let mut successful_attempts = 0;
    let mut failed_attempts = 0;
    
    for line in lines {
        if !line.contains(username) {
            continue;
        }
        
        // Parse the timestamp
        if let Some(timestamp_str) = line.get(1..20) {
            if let Ok(timestamp) = chrono::NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S") {
                let log_time = chrono::DateTime::<chrono::Local>::from_naive_utc_and_offset(
                    timestamp, *chrono::Local::now().offset()
                );
                
                if log_time >= window_start {
                    if line.contains("Successful login") {
                        successful_attempts += 1;
                    } else if line.contains("Failed login") {
                        failed_attempts += 1;
                    }
                }
            }
        }
    }
    
    Ok((successful_attempts, failed_attempts))
}

pub fn verify_transactions(user_id: i32, time_window_minutes: u32) -> Result<Vec<String>, std::io::Error> {
    let log_content = std::fs::read_to_string("casino_logs.log")?;
    let lines: Vec<&str> = log_content.lines().collect();
    
    let now = Local::now();
    let window_start = now - chrono::Duration::minutes(time_window_minutes as i64);
    
    let mut transactions = Vec::new();
    
    for line in lines {
        if !line.contains(&format!("User ID: {}", user_id)) || !line.contains("[TRANSACTION]") {
            continue;
        }
        
        // Parse the timestamp
        if let Some(timestamp_str) = line.get(1..20) {
            if let Ok(timestamp) = chrono::NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S") {
                let log_time = chrono::DateTime::<chrono::Local>::from_naive_utc_and_offset(
                    timestamp, *chrono::Local::now().offset()
                );
                
                if log_time >= window_start {
                    transactions.push(line.to_string());
                }
            }
        }
    }
    
    Ok(transactions)
}

pub fn verify_security_events(time_window_minutes: u32) -> Result<Vec<String>, std::io::Error> {
    let log_content = std::fs::read_to_string("casino_logs.log")?;
    let lines: Vec<&str> = log_content.lines().collect();
    
    let now = Local::now();
    let window_start = now - chrono::Duration::minutes(time_window_minutes as i64);
    
    let mut security_events = Vec::new();
    
    for line in lines {
        if !line.contains("[SECURITY]") {
            continue;
        }
        
        // Parse the timestamp
        if let Some(timestamp_str) = line.get(1..20) {
            if let Ok(timestamp) = chrono::NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S") {
                let log_time = chrono::DateTime::<chrono::Local>::from_naive_utc_and_offset(
                    timestamp, *chrono::Local::now().offset()
                );
                
                if log_time >= window_start {
                    security_events.push(line.to_string());
                }
            }
        }
    }
    
    Ok(security_events)
}