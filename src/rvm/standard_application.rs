use winit::event_loop::{ EventLoop, ControlFlow };
use winit::window::Window;
use winit::event;
use super::errors::AppInitError;
use super::debug;
use super::window;

pub struct StandardApplication {
    event_loop: EventLoop<()>,
    app_window: Window,
}

// Конструкторы
impl StandardApplication {
    pub fn new_windows() -> Result<Self, AppInitError> {
        debug::log::init_log().unwrap();
        debug::log::log_info("Запуск на платформе Windows.");

        let event_loop = EventLoop::new();
        let window = match window::init_app_window(&event_loop) {
            Ok(win) => win,
            Err(err) => {
                let error = AppInitError::WindowInitError(err);
                debug::log::log_error(error.clone());
                return Err(error)
            }
        };
        debug::log::log_info("Окно приложения инициализировано.");



        debug::log::log_info("Тестовое сообщение");
        println!("{}", debug::log::last_message().unwrap());
        debug::log::log_error(AppInitError::WindowInitError("Тестовая ошибка".to_string()));
        println!("{}", debug::log::last_message().unwrap());



        debug::log::log_info("Инициализация приложения завершена.");
        Ok(StandardApplication { 
            event_loop,
            app_window: window,
        })
    }

    pub fn new_android() -> Result<Self, AppInitError> {
        Err(AppInitError::OSNotSupported("Android"))
    }
}

//
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
