use std::error::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppInitError {
    OSNotSupported(&'static str),
    WindowInitError(String),
}

impl Error for AppInitError { }
impl std::fmt::Display for AppInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AppInitError::OSNotSupported(os_name) => write!(f, "{} platform not supported!", os_name),
            AppInitError::WindowInitError(message) => write!(f, "Window initialization error: {}", message),
        }
        
    }
}
