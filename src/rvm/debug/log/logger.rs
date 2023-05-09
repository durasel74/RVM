const PROJECT_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Logger {

    last_message: String
}

impl Logger {
    pub fn new() -> Self {
        Logger {
            last_message: String::new(),
        }
    }

    pub fn last_message(&self) -> String {
        self.last_message.clone()
    }

    pub fn log_start_info(&mut self) {
        println!("\n___________________{} {}_____________________", PROJECT_NAME, VERSION);
    }

    pub fn log_info<T: std::fmt::Display>(&mut self, message: T) {
        self.last_message = message.to_string();
        println!("[Info] {}", message);
    }

    pub fn log_warn<T: std::fmt::Display>(&mut self, message: T) {
        self.last_message = message.to_string();
        println!("[Warning] {}", message);
    }

    pub fn log_error<T: std::fmt::Display>(&mut self, error: T) {
        self.last_message = error.to_string();
        println!("[Error] {}", error);
    }
}
