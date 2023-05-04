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
            LogError::InitializationFailed => write!(f, "Failed to initialize log"),
            LogError::NotInitialized => write!(f, "Log has not been initialized"),
            LogError::GuardLocked => write!(f, "Log guard was blocked"),
        }
        
    }
}
