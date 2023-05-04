use std::env;
use super::errors::AppInitError;
use super::StandardApplication;

pub struct ApplicationBuilder {

}

impl ApplicationBuilder {
    pub fn new() -> Self {
        ApplicationBuilder {

        }
    }

    pub fn build(self) -> Result<StandardApplication, AppInitError> {
        if cfg!(windows) {
            StandardApplication::new_windows()
        } else if cfg!(android) {
            StandardApplication::new_android()
        } else {
            Err(AppInitError::OSNotSupported(env::consts::OS))
        }
    }

    // pub fn build_safe_mode() -> Result<StandardApplication, AppInitError> {
    //     if cfg!(windows) {
    //         StandardApplication::new_windows()
    //     } else if cfg!(android) {
    //         StandardApplication::new_android()
    //     } else {
    //         Err(AppInitError::OSNotSupported(env::consts::OS))
    //     }
    // }
}
