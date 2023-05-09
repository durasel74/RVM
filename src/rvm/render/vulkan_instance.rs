use std::sync::Arc;
use vulkano::library::VulkanLibrary;
use vulkano::instance::{ Instance, InstanceCreateInfo, InstanceExtensions };
use super::debug;
use super::errors::RendererInitError;
use super::{ InstanceExtensionsControl, InstExtControlCreateResult };

/// Управляет подключением к библиотеке Vulkan и ее расширениями.
#[derive(Debug, PartialEq, Eq)]
pub struct VulkanInstance {
    instance: Arc<Instance>,
    instance_extensions_control: InstanceExtensionsControl
}

impl VulkanInstance {
    /// Создает объект подключения к библиотеке Vulkan для платформы Windows.
    pub fn new_windows(library: Arc<VulkanLibrary>) 
    -> Result<VulkanInstance, RendererInitError> {
        debug::log::log_info("Создание подключения к библиотеке Vulkan...");

        let ext_control = match InstanceExtensionsControl
        ::new_windows(library.supported_extensions()) {
            InstExtControlCreateResult::Ok(iec) => {
                debug::log::log_info("Все расширения поддерживаются.");
                iec
            },
            InstExtControlCreateResult::PreferredExtNotSupported(iec) => {
                debug::log::log_warn(format!("Некоторые, некритичные расширения, \
                    не удалось подключить.\nСписок неподключенных расширений: {:?}", 
                    iec.missing_extensions()));
                iec
            },
            InstExtControlCreateResult::MinExtNotSupported => {
                debug::log::log_error(RendererInitError::InstanceExtNotSupported);
                return Err(RendererInitError::InstanceExtNotSupported)
            }
        };

        match Self::new(library, ext_control) {
            Ok(vi) => {
                debug::log::log_info(
                    format!("Подключение к библиотеке Vulkan создано, версия API \
                    подключения: {}.", vi.api_version())
                );
                Ok(vi)
            },
            Err(err) => {
                debug::log::log_error(err.clone());
                Err(err)
            }
        }
    }

    /// Создает объект подключения к библиотеке Vulkan для платформы Windows 
    /// с минимальными расширениями.
    pub fn new_windows_minimal(library: Arc<VulkanLibrary>) 
    -> Result<VulkanInstance, RendererInitError> {
        debug::log::log_info("Создание подключения к библиотеке Vulkan... \
            (Безопасный режим)");

        let ext_control = match InstanceExtensionsControl
        ::new_windows_minimal(library.supported_extensions()) {
            InstExtControlCreateResult::Ok(iec) => {
                debug::log::log_info("Все расширения поддерживаются.");
                iec
            },
            _ => {
                debug::log::log_error(RendererInitError::InstanceExtNotSupported);
                return Err(RendererInitError::InstanceExtNotSupported)
            },
        };

        match Self::new(library, ext_control) {
            Ok(vi) => {
                debug::log::log_info(
                    format!("Подключение к библиотеке Vulkan создано, версия API \
                    подключения: {}.", vi.api_version())
                );
                Ok(vi)
            },
            Err(err) => {
                debug::log::log_error(err.clone());
                Err(err)
            }
        }
    }

    /// Создает объект подключения к библиотеке Vulkan.
    pub fn new(library: Arc<VulkanLibrary>, instance_ext_control: InstanceExtensionsControl)
    -> Result<VulkanInstance, RendererInitError> {
        let new_instance = match Instance::new(
            library.clone(),
            InstanceCreateInfo {
                enabled_extensions: instance_ext_control.current_extensions().clone(),
                ..InstanceCreateInfo::application_from_cargo_toml()
            }
        ) {
            Ok(instance) => instance,
            Err(err) => return Err(RendererInitError::InstanceInitError(err.to_string()))
        };

        let current_ext = instance_ext_control.current_extensions();
        if !new_instance.enabled_extensions().contains(current_ext) {
            return Err(RendererInitError::InstanceExtNotSupported)
        }

        Ok(VulkanInstance {
            instance: new_instance.clone(),
            instance_extensions_control: instance_ext_control
        })
    }

    // pub fn new_android() {

    // }
}

impl VulkanInstance {
    /// Возвращает версию библиотеки, к которой создано подключение.
    pub fn api_version(&self) -> String {
        self.instance.api_version().to_string()
    }

    /// Возвращает поддерживаемые библиотекой Vulkan расширения, на этом устройстве.
    pub fn supported_extensions(&self) -> &InstanceExtensions {
        self.instance_extensions_control.supported_extensions()
    }

    /// Возвращает минимальный набор расширений необходимый для запуска.
    pub fn minimal_extensions(&self) -> &InstanceExtensions {
        self.instance_extensions_control.minimal_extensions()
    }

    /// Возвращает желаемые расширения, которые могут быть не подключены.
    pub fn preferred_extensions(&self) -> &InstanceExtensions {
        self.instance_extensions_control.preferred_extensions()
    }

    /// Возвращает все возможные расширения, используемые в приложении.
    pub fn all_extensions(&self) -> &InstanceExtensions {
        self.instance_extensions_control.all_extensions()
    }

    /// Возвращает расширения, проверенные на этом устройстве.
    pub fn current_extensions(&self) -> &InstanceExtensions {
        self.instance_extensions_control.current_extensions()
    }

    /// Возвращает необязательные расширения, которые не поддерживаются 
    /// библиотекой Vulkan на этом устройстве.
    pub fn missing_extensions(&self) -> InstanceExtensions {
        self.instance_extensions_control.missing_extensions()
    }
}
