use std::error::Error;

/// Ошибки, которые могут произойти во время инициализации визуализатора.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RendererInitError {
    LibraryInitError(String),
    InstanceExtNotSupported,
    InstanceInitError(String),
}

impl Error for RendererInitError { }
impl std::fmt::Display for RendererInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RendererInitError::LibraryInitError(message)
                => write!(f, "Ошибка при поиске библиотеки Vulkan: {}", message),
            RendererInitError::InstanceExtNotSupported
                => write!(f, "Минимальные расширения, подключения к библиотеке \
                    Vulkan, не поддерживаются!"),
            RendererInitError::InstanceInitError(message)
                => write!(f, "Ошибка при инициализации подключения к библиотеке \
                    Vulkan: {}", message),

        }
        
    }
}
