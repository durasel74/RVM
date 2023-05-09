use std::sync::Arc;
use vulkano::library::VulkanLibrary;
use super::debug;
use super::errors::RendererInitError;
use super::VulkanInstance;

/// Выполняет всю работу по визуализации графики приложения.
#[derive(Debug, PartialEq, Eq)]
pub struct Renderer {
    vulkan_instance: VulkanInstance,

}

impl Renderer {
    /// Создает визуализатор графики для платформы Windows.
    pub fn new_windows() -> Result<Self, RendererInitError> {
        debug::log::log_info("Инициализация рендера...");

        let vulkan_lib = Self::init_vulkan_library()?;
        let vulkan_instance = VulkanInstance::new_windows(vulkan_lib)?;


        debug::log::log_info("Инициализация рендера завершена.");
        Ok(Renderer {
            vulkan_instance
        })
    }

    // pub fn new_android() -> Result<Self, RendererInitError> {
    //     Err(RendererInitError::LibraryInitError("Не поддерживается Android.".to_string()))
    // }

    // Проверяет наличие библиотеки Vulkan и возвращает информацию о ней
    fn init_vulkan_library() -> Result<Arc<VulkanLibrary>, RendererInitError> {
        match VulkanLibrary::new() {
            Ok(lib) => {
                debug::log::log_info(
                    format!("Библиотека Vulkan {} найдена на устройстве.",
                    lib.api_version())
                );
                Ok(lib)
            },
            Err(err) => {
                let error = RendererInitError::LibraryInitError(err.to_string());
                debug::log::log_error(error.clone());
                Err(error)
            }
        }
    }
}
