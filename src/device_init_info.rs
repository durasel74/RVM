use vulkano::device::{ DeviceExtensions, Features, DeviceCreationError  };

pub struct DeviceInitInfo {
    pub required_extensions: DeviceExtensions,
    pub preferred_extensions: DeviceExtensions,
    pub required_features: Features,
    pub preferred_features: Features,
}

impl DeviceInitInfo {
    pub fn confirm_extensions(&self, supported_extensions: &DeviceExtensions) 
    -> Result<DeviceExtensions, DeviceCreationError> {
        if supported_extensions.contains(&self.required_extensions) == false {
            return Err(DeviceCreationError::ExtensionNotPresent)
        }
        Ok(supported_extensions
            .intersection(&self.preferred_extensions)
            .union(&self.required_extensions)
        )
    }

    pub fn confirm_features(&self, supported_features: &Features) 
    -> Result<Features, DeviceCreationError> {
        if supported_features.contains(&self.required_features) == false {
            return Err(DeviceCreationError::FeatureNotPresent)
        }
        Ok(supported_features
            .intersection(&self.preferred_features)
            .union(&self.required_features)
        )
    }
}

impl Default for DeviceInitInfo {
    fn default() -> Self {
        let required_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };
        let preferred_extensions = DeviceExtensions {
            khr_dedicated_allocation: true,
            khr_device_group: true,
            ..DeviceExtensions::empty()
        };

        let required_features = Features {
            ..Features::empty()
        };
        let preferred_features = Features {
            buffer_device_address: true,
            ..Features::empty()
        };
        
        DeviceInitInfo { 
            required_extensions, 
            preferred_extensions, 
            required_features, 
            preferred_features 
        }
    }
}
