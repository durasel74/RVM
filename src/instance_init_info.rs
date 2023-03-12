use vulkano::instance::{ InstanceExtensions, InstanceCreationError };

pub struct InstanceInitInfo {
    pub required_extensions: InstanceExtensions,
    pub preferred_extensions: InstanceExtensions,
}

impl InstanceInitInfo {
    pub fn confirm_extensions(&self, supported_extensions: &InstanceExtensions) 
    -> Result<InstanceExtensions, InstanceCreationError> {
        if supported_extensions.contains(&self.required_extensions) == false {
            return Err(InstanceCreationError::ExtensionNotPresent)
        }
        Ok(supported_extensions
            .intersection(&self.preferred_extensions)
            .union(&self.required_extensions)
        )
    }
}

impl Default for InstanceInitInfo {
    fn default() -> Self {
        let required_extensions = InstanceExtensions {
            khr_surface: true,
            khr_win32_surface: true,
            khr_get_surface_capabilities2: true,
            ..InstanceExtensions::empty()
        };
        let preferred_extensions = InstanceExtensions {
            ..InstanceExtensions::empty()
        };
        InstanceInitInfo { required_extensions, preferred_extensions }
    }
}
