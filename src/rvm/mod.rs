// Модули проекта
mod debug;
mod render;
mod ui;

// Файлы главного модуля
mod errors;
mod application_builder;
mod standard_application;

pub use application_builder::ApplicationBuilder;
pub use standard_application::StandardApplication;

// Старая реализация фракталов
mod old_module;
pub use old_module::main_old;