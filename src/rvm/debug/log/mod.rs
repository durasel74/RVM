mod logger;
pub use logger::Logger;
use super::errors::LogError;

// Статическая ссылка на логгер
static LOGGER: std::sync::Mutex<Option<Logger>> = std::sync::Mutex::new(None);

pub fn init_log() -> Result<(), LogError> {
    if let Ok(mut guard) = LOGGER.lock() {
        if let None = *guard {
            let mut new_logger = Logger::new();
            new_logger.log_start_info();
            *guard = Some(new_logger);
            Ok(())
        } else { Ok(()) }
    } else { Err(LogError::InitializationFailed) }
}

pub fn log_info<T: std::fmt::Display>(message: T) {
    if let Ok(mut guard) = LOGGER.lock() {
        if let Some(logger) = &mut *guard {
            logger.log_info(message);
        }
    }
}

pub fn log_warn<T: std::fmt::Display>(message: T) {
    if let Ok(mut guard) = LOGGER.lock() {
        if let Some(logger) = &mut *guard {
            logger.log_warn(message);
        }
    }
}

pub fn log_error<T: std::fmt::Display>(error: T) {
    if let Ok(mut guard) = LOGGER.lock() {
        if let Some(logger) = &mut *guard {
            logger.log_error(error);
        }
    }
}

pub fn last_message() -> Result<String, LogError> {
    if let Ok(guard) = LOGGER.lock() {
        if let Some(logger) = &*guard {
            return Ok(logger.last_message())
        } else { return Err(LogError::NotInitialized) }
    }
    Err(LogError::GuardLocked)
}
