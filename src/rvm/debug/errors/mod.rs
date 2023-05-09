use std::error::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogError {
    InitializationFailed,
    NotInitialized,
    GuardLocked,
}

impl Error for LogError { }
impl std::fmt::Display for LogError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LogError::InitializationFailed => write!(f, "Не удалось инициализировать логи."),
            LogError::NotInitialized => write!(f, "Логи не были инициализированы, запись не возможна!"),
            LogError::GuardLocked => write!(f, "Логи находятся в блокировке другим потоком."),
        }
        
    }
}
