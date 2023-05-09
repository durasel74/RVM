use std::error::Error;

/// Ошибки, которые могут произойти во время инициализации приложения.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppInitError {
    OSNotSupported(&'static str),
    LogInitError(String),
    WindowInitError(String),
    RendererInitError(String),
}

impl Error for AppInitError { }
impl std::fmt::Display for AppInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AppInitError::OSNotSupported(os_name) => write!(f, 
                "Платформа {} не поддерживается!", os_name),
            AppInitError::LogInitError(message) => write!(f, 
                "Ошибка при инициализации логов: {}", message),
            AppInitError::WindowInitError(message) => write!(f, 
                "Ошибка при инициализации окна приложения: {}", message),
            AppInitError::RendererInitError(message) => write!(f, 
                "Ошибка при инициализации рендера приложения: {}", message),
        }
    }
}
