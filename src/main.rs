mod rvm;
mod ui;

fn main() {
    // let event_loop = EventLoop::new();
    // let window_builder = WindowBuilder::new()
    //     .with_title(format!("RVM {}", VERSION))
    //     .with_inner_size(PhysicalSize::new(1000, 800));
    // let window = match window_builder.build(&event_loop) {
    //     Ok(win) => Arc::new(win),
    //     Err(err) => { println!("Window creating error: {:?}", err); return; }
    // };

    rvm::main_old::main_old();
}
