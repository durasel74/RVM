use winit::event_loop::EventLoop;
use winit::window::{ WindowBuilder, Window };

const PROJECT_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn init_app_window(event_loop: &EventLoop<()>) -> Result<Window, String> {
    let window_builder = WindowBuilder::new()
        .with_title(format!("{} {}", PROJECT_NAME, VERSION))
        .with_inner_size(winit::dpi::PhysicalSize::new(1000, 800))
        .with_visible(false);

    match window_builder.build(event_loop) {
        Ok(win) => Ok(win),
        Err(err) => Err(err.to_string())
    }
}
