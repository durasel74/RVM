#![windows_subsystem = "console"]

mod rvm;

use rvm::ApplicationBuilder;

fn main() {
    let app_builder = ApplicationBuilder::new();

    let application = match app_builder.build() {
        Ok(app) => app,
        Err(err) => return,
    };

    application.run();

    //rvm::main_old::main_old();
}
