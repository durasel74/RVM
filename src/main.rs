//#![allow(non_snake_case)]

use winit::event::{ Event, WindowEvent, StartCause };
use winit::event_loop::{ ControlFlow, EventLoop, DeviceEventFilter };
use winit::window::WindowBuilder;

use vulkano::{ VulkanLibrary };
use vulkano::instance::{ Instance, InstanceCreateInfo, InstanceExtensions };

use egui_winit_vulkano::Gui;
use vulkano_util::context::{ VulkanoConfig, VulkanoContext };
use vulkano_util::window::{ VulkanoWindows, WindowDescriptor };
//use egui::{ ScrollArea, TextEdit, TextStyle, Label };

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn sized_text(ui: &mut egui::Ui, text: impl Into<String>, size: f32) {
    ui.label(egui::RichText::new(text).size(size).color(egui::Color32::BLACK));
}

fn show_lib_info(ui: &mut egui::Ui, library: std::sync::Arc<VulkanLibrary>) {
    egui::Frame::none()
    .fill(egui::Color32::from_rgb(180, 180, 180))
    .outer_margin(egui::style::Margin::same(5.0))
    .inner_margin(egui::style::Margin::same(10.0))
    .show(ui, |ui| {
        ui.set_max_size(egui::vec2(550.0, 370.0));
        ui.vertical_centered(|ui| {
            sized_text(ui, 
                format!("Vulkan library version: {}", 
                    library.api_version()), 30.0
            );
        });

        sized_text(ui, format!("Extensions:"), 18.0);
        egui::Frame::none()
        .fill(egui::Color32::from_rgb(160, 160, 160))
        .inner_margin(egui::style::Margin::same(10.0))
        .show(ui, |ui| {
            egui::ScrollArea::vertical().id_source("lib_ext_scroll")
            .max_height(120.0)
            .show(ui, |ui| {
                ui.set_min_width(530.0);
                for i in library.extension_properties() {
                    sized_text(
                        ui, 
                        format!("{} v{}",
                        i.extension_name, i.spec_version), 20.0
                    );
                }
            });
        });

        sized_text(ui, format!("Layers:"), 18.0);
        egui::Frame::none()
        .fill(egui::Color32::from_rgb(160, 160, 160))
        .inner_margin(egui::style::Margin::same(10.0))
        .show(ui, |ui| {
            egui::ScrollArea::vertical().id_source("lib_layers_scroll")
            .max_height(120.0)
            .show(ui, |ui| {
                ui.set_min_width(530.0);
                for i in library.layer_properties().unwrap() {
                    sized_text(
                        ui, 
                        format!("{} v{}",
                        i.name(), i.implementation_version()), 20.0
                    );
                }
            });
        });
    });
}

fn show_instance_info(ui: &mut egui::Ui, instance: std::sync::Arc<Instance>) {
    egui::Frame::none()
    .fill(egui::Color32::from_rgb(180, 180, 180))
    .outer_margin(egui::style::Margin::same(5.0))
    .inner_margin(egui::style::Margin::same(10.0))
    .show(ui, |ui| {
        ui.set_max_size(egui::vec2(550.0, 370.0));
        ui.vertical_centered(|ui| {
            sized_text(ui, 
                format!("Instance api version: {}", 
                    instance.api_version()), 30.0
            );
        });

        sized_text(ui, format!("Enabled extensions:"), 18.0);
        egui::Frame::none()
        .fill(egui::Color32::from_rgb(160, 160, 160))
        .inner_margin(egui::style::Margin::same(10.0))
        .show(ui, |ui| {
            egui::ScrollArea::vertical().id_source("instance_ext_scroll")
            .max_height(120.0)
            .show(ui, |ui| {
                ui.set_min_width(530.0);
                sized_text(ui, format!("{:?}", instance.enabled_extensions()), 20.0);
            });
        });

        sized_text(ui, format!("Enabled layers:"), 18.0);
        egui::Frame::none()
        .fill(egui::Color32::from_rgb(160, 160, 160))
        .inner_margin(egui::style::Margin::same(10.0))
        .show(ui, |ui| {
            egui::ScrollArea::vertical().id_source("instance_layers_scroll")
            .max_height(120.0)
            .show(ui, |ui| {
                ui.set_min_width(530.0);
                for i in instance.enabled_layers() {
                    sized_text(ui, i, 20.0);
                }
            });
        });
    });
}

fn main() {
    let library = VulkanLibrary::new().expect("no local Vulkan library/DLL");
    let lib_extensions = library.supported_extensions();

    let extensions = InstanceExtensions {
        khr_surface: lib_extensions.khr_surface,
        khr_win32_surface: lib_extensions.khr_win32_surface,
        .. InstanceExtensions::empty()
    };

    let layers: Vec<_> = library.layer_properties().unwrap()
        .filter(|l| l.name().contains("VK_LAYER_LUNARG_monitor"))
        .collect();

    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions: extensions,
            enabled_layers: layers.iter().map(|l| l.name().to_owned()).collect(),
            ..InstanceCreateInfo::application_from_cargo_toml()
        },
    ).expect("Couldn't create instance");


    let event_loop = EventLoop::new();
    // let window = WindowBuilder::new()
    //     .with_title("RVM")
    //     .build(&event_loop)
    //     .unwrap();


    // Vulkano context
    let context = VulkanoContext::new(VulkanoConfig::default());
    // Vulkano windows (create one)
    let mut windows = VulkanoWindows::default();
    windows.create_window(&event_loop, &context, &WindowDescriptor::default(), |ci| {
        ci.image_format = Some(vulkano::format::Format::B8G8R8A8_SRGB)
    });
    // Create gui as main render pass (no overlay means it clears the image each frame)
    let mut gui = {
        let renderer = windows.get_primary_renderer_mut().unwrap();
        Gui::new(
            &event_loop,
            renderer.surface(),
            Some(vulkano::format::Format::B8G8R8A8_SRGB),
            renderer.graphics_queue(),
            false,
        )
    };

    event_loop.run(move |event, _, control_flow| {
        //*control_flow = ControlFlow::Wait;

        // match event {
        //     Event::NewEvents(start_cause) => {},
        //     Event::WindowEvent {
        //         event: WindowEvent::CloseRequested,
        //         window_id,
        //     } if window_id == window.id() => control_flow.set_exit(),

        //     Event::DeviceEvent { device_id, event } => {
        //         // println!("{:?}", event)
        //     }

        //     Event::MainEventsCleared => {},
        //     //Event::RedrawRequested(windowId) => {},
        //     Event::RedrawEventsCleared => {},
        //     Event::LoopDestroyed => {},
        //     _ => (),
        // }


        let renderer = windows.get_primary_renderer_mut().unwrap();
        match event {
            Event::WindowEvent { event, window_id } if window_id == renderer.window().id() => {
                // Update Egui integration so the UI works!
                let _pass_events_to_game = !gui.update(&event);
                match event {
                    WindowEvent::Resized(_) => {
                        renderer.resize();
                    }
                    WindowEvent::ScaleFactorChanged { .. } => {
                        renderer.resize();
                    }
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => (),
                }
            }

            Event::RedrawRequested(window_id) if window_id == window_id => {
                // Set immediate UI in redraw here
                gui.immediate_ui(|gui| {
                    let ctx = gui.context();
                    let frame = egui::Frame::none().fill(egui::Color32::from_rgb(80, 80, 80));
                    egui::CentralPanel::default().frame(frame).show(&ctx, |ui| {

                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                show_lib_info(ui, instance.library().clone());
                            });
                            ui.vertical(|ui| {
                                show_instance_info(ui, instance.clone());
                            });
                        });
                        
                        
                        // ui.separator();
                        // ui.columns(2, |columns| {
                        //     ScrollArea::vertical().id_source("source").show(
                        //         &mut columns[0],
                        //         |ui| {
                        //             // ui.add(
                        //             //     TextEdit::multiline(&mut code).font(TextStyle::Monospace),
                        //             // );
                        //         },
                        //     );
                        //     ScrollArea::vertical().id_source("rendered").show(
                        //         &mut columns[1],
                        //         |ui| {
                        //             //egui_demo_lib::easy_mark::easy_mark(ui, &code);
                        //         },
                        //     );
                        // });
                    });
                });
                // Render UI
                // Acquire swapchain future
                let before_future = renderer.acquire().unwrap();
                // Render gui
                let after_future =
                    gui.draw_on_image(before_future, renderer.swapchain_image_view());
                // Present swapchain
                renderer.present(after_future, true);
            }

            Event::MainEventsCleared => {
                renderer.window().request_redraw();
            }
            _ => (),
        }
    });
}
