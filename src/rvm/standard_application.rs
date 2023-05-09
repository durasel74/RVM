use winit::event_loop::{ EventLoop, ControlFlow };
use winit::window::{ WindowBuilder, Window };
use winit::event;
use super::errors::AppInitError;
use super::debug;
use super::render::Renderer;

const PROJECT_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Стандартное приложение со всеми возможностями.
#[derive(Debug)]
pub struct StandardApplication {
    event_loop: EventLoop<()>,
    app_window: Window,
    renderer: Renderer,
}

impl StandardApplication {
    /// Создает приложение для платформы Windows.
    pub fn new_windows() -> Result<Self, AppInitError> {
        Self::init_log()?;
        debug::log::log_info("Запуск на платформе Windows...");

        let event_loop = EventLoop::new();
        let app_window = Self::init_app_window(&event_loop)?;
        let renderer = Self::init_app_renderer()?;




        debug::log::log_info("Инициализация приложения завершена.");
        Ok(StandardApplication { 
            event_loop,
            app_window,
            renderer,
        })
    }

    pub fn new_android() -> Result<Self, AppInitError> {
        Err(AppInitError::OSNotSupported("Android"))
    }

    /// Инициализирует логи приложения.
    pub fn init_log() -> Result<(), AppInitError> {
        if let Err(err) = debug::log::init_log() {
            Err(AppInitError::LogInitError(err.to_string()))
        } else { Ok(()) }
    }

    /// Инициализирует главное окно приложения.
    pub fn init_app_window(event_loop: &EventLoop<()>) -> Result<Window, AppInitError> {
        let window_builder = WindowBuilder::new()
            .with_title(format!("{} {}", PROJECT_NAME, VERSION))
            .with_inner_size(winit::dpi::PhysicalSize::new(1000, 800))
            .with_visible(false);

        match window_builder.build(event_loop) {
            Ok(win) => {
                debug::log::log_info("Окно приложения инициализировано.");
                Ok(win)
            },
            Err(err) => {
                let error = AppInitError::WindowInitError(err.to_string());
                debug::log::log_error(error.clone());
                Err(error)
            }
        }
    }

    /// Инициализирует рендер приложения.
    pub fn init_app_renderer() -> Result<Renderer, AppInitError> {
        match Renderer::new_windows() {
            Ok(renderer) => Ok(renderer),
            Err(err) => {
                let error = AppInitError::RendererInitError(err.to_string());
                debug::log::log_error(error.clone());
                Err(error)
            }
        }
    }
}

impl StandardApplication {
    pub fn run(self) {
        self.app_window.set_visible(true);

        self.event_loop.run(move |event, _, control_flow| {
            match event {
                event::Event::WindowEvent { event, window_id } if window_id == self.app_window.id() => {
                    match event {
                        event::WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                        },
                        _ => ()
                    }
                },
                _ => (),
            }
        });
        
    }
}
