//#![allow(non_snake_case)]

mod instance_init_info;
mod device_init_info;
use instance_init_info::InstanceInitInfo;
use device_init_info::DeviceInitInfo;

use std::error::Error;
use std::mem::{ size_of, size_of_val };
use std::sync::Arc;
use bytesize::ByteSize;

use winit::event::{ Event, WindowEvent, StartCause };
use winit::event_loop::{ ControlFlow, EventLoop, DeviceEventFilter };
use winit::window::WindowBuilder;

use vulkano::{ VulkanLibrary, VulkanError };
use vulkano::instance::{ Instance, InstanceCreateInfo };
use vulkano::device::physical::{ PhysicalDevice, PhysicalDeviceType };
use vulkano::device::{ Device, DeviceCreateInfo, DeviceExtensions, Features, DeviceCreationError, QueueCreateInfo };
use vulkano::memory::allocator::{ GenericMemoryAllocator, GenericMemoryAllocatorCreateInfo, AllocationType, MemoryUsage };
use vulkano::memory::allocator::suballocator::{ FreeListAllocator, BumpAllocator, PoolAllocator, BuddyAllocator };
use vulkano::buffer::{ BufferUsage, BufferAccess, CpuAccessibleBuffer, CpuBufferPool, DeviceLocalBuffer };

use egui_winit_vulkano::Gui;
use vulkano_util::context::{ VulkanoConfig, VulkanoContext };
use vulkano_util::window::{ VulkanoWindows, WindowDescriptor };
//use egui::{ ScrollArea, TextEdit, TextStyle, Label };

const VERSION: &str = env!("CARGO_PKG_VERSION");
const UI_SIZE: f32 = 0.7;

fn create_vulkan_instance() -> Result<Arc<Instance>, Box<dyn Error>> {
    let library = VulkanLibrary::new()?;
    let supported_extensions = library.supported_extensions();
    let enabled_extensions = InstanceInitInfo::default().confirm_extensions(supported_extensions)?;

    let layers: Vec<_> = library.layer_properties().unwrap()
        .filter(|l| l.name().contains("VK_LAYER_LUNARG_monitor"))
        .collect();

    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions,
            enabled_layers: layers.iter().map(|l| l.name().to_owned()).collect(),
            ..InstanceCreateInfo::application_from_cargo_toml()
        },
    )?;
    Ok(instance)
}

fn get_right_devices(instance: Arc<Instance>) -> Result<Vec<Arc<PhysicalDevice>>, VulkanError> {
    let physical_devices: Vec<Arc<PhysicalDevice>> = instance
        .enumerate_physical_devices()?.collect();

    let priority_devices = sort_physical_devices_by_device_type(physical_devices);
    let correct_devices = find_correct_physical_devices(&priority_devices);
    if correct_devices.len() == 0 { Err(VulkanError::DeviceLost) }
    else { Ok(correct_devices) }
}

fn sort_physical_devices_by_device_type(physical_devices: Vec<Arc<PhysicalDevice>>) 
-> Vec<Arc<PhysicalDevice>> {
    let mut priority_devices: Vec<Arc<PhysicalDevice>> = vec![];
    for device in physical_devices {
        match device.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => priority_devices.insert(0, device.clone()),
            PhysicalDeviceType::IntegratedGpu => priority_devices.push(device.clone()),
            _ => priority_devices.push(device.clone()),
        }
    }
    priority_devices
}

fn find_correct_physical_devices(physical_devices: &Vec<Arc<PhysicalDevice>>) 
-> Vec<Arc<PhysicalDevice>> {
    let mut correct_devices: Vec<Arc<PhysicalDevice>> = vec![];
    for device in physical_devices {
        let supported_extensions = device.supported_extensions();
        if let Err(_) = DeviceInitInfo::default().confirm_extensions(supported_extensions) {
            continue;
        }

        let supported_features = device.supported_features();
        if let Err(_) = DeviceInitInfo::default().confirm_features(supported_features) {
            continue;
        }
        correct_devices.push(device.clone());
    }
    correct_devices
}

fn get_physical_device_local_memory(physical_device: Arc<PhysicalDevice>) -> u64 {
    let memory_prop = physical_device.memory_properties();

    let mut local_memory = 0u64;
    for heap in memory_prop.memory_heaps.iter() {
        if heap.flags.device_local && heap.size > local_memory {
            local_memory = heap.size;
        }
    }
    local_memory
}

fn create_device_connection(physical_device: Arc<PhysicalDevice>) 
-> Result<Arc<Device>, Box<dyn Error>> {
    let supported_extensions = physical_device.supported_extensions();
    let enabled_extensions = DeviceInitInfo::default().confirm_extensions(supported_extensions)?;

    let supported_features = physical_device.supported_features();
    let enabled_features = DeviceInitInfo::default().confirm_features(supported_features)?;

    let queue_create_info = QueueCreateInfo {
        queue_family_index: 0,
        ..Default::default()
    };

    let (device, _) = Device::new(
        physical_device,
        DeviceCreateInfo {
            enabled_extensions,
            enabled_features,
            queue_create_infos: vec![queue_create_info],
            ..Default::default()
        }
    )?;
    Ok(device)
}

fn main() {
    let instance = match create_vulkan_instance() {
        Ok(inst) => inst,
        Err(err) => { println!("Vulkan instance error: {}", err); return; }
    };

    let physical_devices = match get_right_devices(instance.clone()) {
        Ok(pd) => pd,
        Err(err) => { println!("Physical devices error: {}", err); return; }
    };

    let device = match create_device_connection(physical_devices[0].clone()) {
        Ok(device) => device,
        Err(err) => { println!("Device create error: {}", err); return; }
    };

    // Стандартный распределитель, подходит для большинства выделений
    let free_list_memory_allocator = GenericMemoryAllocator::<Arc<FreeListAllocator>>::new_default(device.clone());

    // Лучше подходит для выделений на очень короткий период с полным сбросом
    let bump_memory_allocator = GenericMemoryAllocator::<Arc<BumpAllocator>>::new_default(device.clone());

    // // Лучше подходит для одинаковых выделений
    // let pool_memory_allocator = GenericMemoryAllocator::<Arc<PoolAllocator<{ 64 * 1024 }>>>::new(
    //     device.clone(),
    //     GenericMemoryAllocatorCreateInfo {
    //         block_sizes: &[(0, 64 * 1024 * 1024)],
    //         allocation_type: AllocationType::Linear,
    //         ..Default::default()
    //     },
    // ).unwrap();

    // // Может подойти для выделения большого количества изображений разных размеров?
    // let buddy_memory_allocator = GenericMemoryAllocator::<Arc<BuddyAllocator>>::new(
    //     device.clone(),
    //     GenericMemoryAllocatorCreateInfo {
    //         block_sizes: &[(0, 64 * 1024 * 1024)],
    //         ..Default::default()
    //     },
    // )
    // .unwrap();

    let data: Vec<i32> = (0..99_999).collect();

    let free_list_local_buffer = DeviceLocalBuffer::<i32>::new(
        &free_list_memory_allocator,
        BufferUsage {
            transfer_dst: true,
            storage_buffer: true,
            ..Default::default()
        },
        device.active_queue_family_indices().iter().copied()
    ).unwrap();
    println!("{}", ByteSize(free_list_local_buffer.size()));

    let bump_buffer = CpuBufferPool::new(
        Arc::new(bump_memory_allocator),
        BufferUsage {
            storage_buffer: true,
            ..Default::default()
        },
        MemoryUsage::Upload,
    );
    bump_buffer.from_iter(data.clone()).unwrap();

    // let pool_buffer = CpuAccessibleBuffer::from_iter(
    //     &pool_memory_allocator,
    //     BufferUsage {
    //         storage_buffer: true,
    //         ..Default::default()
    //     },
    //     false,
    //     data.clone()
    // ).unwrap();
    // println!("{}", ByteSize(pool_buffer.size()));

    // let buddy_buffer = CpuAccessibleBuffer::from_iter(
    //     &buddy_memory_allocator,
    //     BufferUsage {
    //         storage_buffer: true,
    //         ..Default::default()
    //     },
    //     false,
    //     data.clone()
    // ).unwrap();
    // println!("{}", ByteSize(buddy_buffer.size()));

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
                        ui.visuals_mut().collapsing_header_frame = true;

                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                show_lib_info(ui, instance.library().clone());
                                show_instance_info(ui, instance.clone());
                            });
                            ui.vertical(|ui| {
                                show_physical_devices_info(ui, &physical_devices);
                            });
                            ui.vertical(|ui| {
                                show_device_info(ui, device.clone())
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

fn sized_text(ui: &mut egui::Ui, text: impl Into<String>, size: f32) {
    ui.label(egui::RichText::new(text).size(size).color(egui::Color32::BLACK));
}

fn show_lib_info(ui: &mut egui::Ui, library: Arc<VulkanLibrary>) {
    egui::Frame::none()
    .fill(egui::Color32::from_rgb(180, 180, 180))
    .outer_margin(egui::style::Margin::same(5.0 * UI_SIZE))
    .inner_margin(egui::style::Margin::same(10.0 * UI_SIZE))
    .show(ui, |ui| {
        ui.set_max_size(egui::vec2(550.0 * UI_SIZE, 370.0 * UI_SIZE));
        ui.vertical_centered(|ui| {
            sized_text(
                ui, 
                format!("Vulkan library version: {}", 
                    library.api_version()), 30.0 * UI_SIZE
            );
        });

        sized_text(ui, format!("Extensions:"), 18.0 * UI_SIZE);
        egui::Frame::none()
        .fill(egui::Color32::from_rgb(160, 160, 160))
        .inner_margin(egui::style::Margin::same(10.0 * UI_SIZE))
        .show(ui, |ui| {
            egui::ScrollArea::vertical().id_source("lib_ext_scroll")
            .max_height(120.0 * UI_SIZE)
            .show(ui, |ui| {
                ui.set_min_width(530.0 * UI_SIZE);
                for i in library.extension_properties() {
                    sized_text(
                        ui,
                        format!("{} v{}",
                            i.extension_name, i.spec_version), 20.0 * UI_SIZE
                    );
                }
            });
        });

        sized_text(ui, format!("Layers:"), 18.0 * UI_SIZE);
        egui::Frame::none()
        .fill(egui::Color32::from_rgb(160, 160, 160))
        .inner_margin(egui::style::Margin::same(10.0 * UI_SIZE))
        .show(ui, |ui| {
            egui::ScrollArea::vertical().id_source("lib_layers_scroll")
            .max_height(120.0 * UI_SIZE)
            .show(ui, |ui| {
                ui.set_min_width(530.0 * UI_SIZE);
                for i in library.layer_properties().unwrap() {
                    sized_text(
                        ui, 
                        format!("{} v{}",
                            i.name(), i.implementation_version()), 20.0 * UI_SIZE
                    );
                }
            });
        });
    });
}

fn show_instance_info(ui: &mut egui::Ui, instance: Arc<Instance>) {
    egui::Frame::none()
    .fill(egui::Color32::from_rgb(180, 180, 180))
    .outer_margin(egui::style::Margin::same(5.0 * UI_SIZE))
    .inner_margin(egui::style::Margin::same(10.0 * UI_SIZE))
    .show(ui, |ui| {
        ui.set_max_size(egui::vec2(550.0 * UI_SIZE, 370.0 * UI_SIZE));
        ui.vertical_centered(|ui| {
            sized_text(ui, 
                format!("Instance api version: {}", 
                    instance.api_version()), 30.0 * UI_SIZE
            );
        });

        sized_text(ui, format!("Enabled extensions:"), 18.0 * UI_SIZE);
        egui::Frame::none()
        .fill(egui::Color32::from_rgb(160, 160, 160))
        .inner_margin(egui::style::Margin::same(10.0 * UI_SIZE))
        .show(ui, |ui| {
            egui::ScrollArea::vertical().id_source("instance_ext_scroll")
            .max_height(120.0 * UI_SIZE)
            .show(ui, |ui| {
                ui.set_min_width(530.0 * UI_SIZE);
                sized_text(ui, format!("{:?}", instance.enabled_extensions()), 20.0 * UI_SIZE);
            });
        });

        sized_text(ui, format!("Enabled layers:"), 18.0 * UI_SIZE);
        egui::Frame::none()
        .fill(egui::Color32::from_rgb(160, 160, 160))
        .inner_margin(egui::style::Margin::same(10.0 * UI_SIZE))
        .show(ui, |ui| {
            egui::ScrollArea::vertical().id_source("instance_layers_scroll")
            .max_height(120.0 * UI_SIZE)
            .show(ui, |ui| {
                ui.set_min_width(530.0 * UI_SIZE);
                for i in instance.enabled_layers() {
                    sized_text(ui, i, 20.0 * UI_SIZE);
                }
            });
        });
    });
}

fn show_physical_device_info(ui: &mut egui::Ui, physical_device: Arc<PhysicalDevice>) {
    let physical_device_name = physical_device.properties().device_name.clone();
    
    egui::Frame::none()
    .fill(egui::Color32::from_rgb(180, 180, 180))
    .outer_margin(egui::style::Margin::same(5.0 * UI_SIZE))
    .inner_margin(egui::style::Margin::same(10.0 * UI_SIZE))
    .show(ui, |ui| {
        ui.set_max_size(egui::vec2(550.0 * UI_SIZE, 550.0 * UI_SIZE));
        ui.vertical_centered(|ui| {
            sized_text(ui, 
                format!("{}", &physical_device_name), 26.0 * UI_SIZE
            );
        });

        sized_text(ui, format!("Properties:"), 18.0 * UI_SIZE);
        egui::Frame::none()
        .fill(egui::Color32::from_rgb(160, 160, 160))
        .inner_margin(egui::style::Margin::same(10.0 * UI_SIZE))
        .show(ui, |ui| {
            let mut id_source = physical_device_name.clone();
            id_source.push_str("_layers_scroll");
            egui::ScrollArea::vertical().id_source(id_source)
            .max_height(120.0 * UI_SIZE)
            .show(ui, |ui| {
                ui.set_min_width(530.0 * UI_SIZE);
                let prop = physical_device.properties();
                let local_memory = get_physical_device_local_memory(physical_device.clone());
                sized_text(ui, format!("ID: {}", prop.device_id.to_string()), 20.0 * UI_SIZE);
                sized_text(ui, format!("API version: {}", prop.api_version.to_string()), 20.0 * UI_SIZE);
                sized_text(ui, format!("Device type: {:?}", prop.device_type), 20.0 * UI_SIZE);
                sized_text(ui, format!("Local memory: {:?}", ByteSize(local_memory)), 20.0 * UI_SIZE);
            });
        });

        sized_text(ui, format!("Extensions:"), 18.0 * UI_SIZE);
        egui::Frame::none()
        .fill(egui::Color32::from_rgb(160, 160, 160))
        .inner_margin(egui::style::Margin::same(10.0 * UI_SIZE))
        .show(ui, |ui| {
            let mut id_source = physical_device_name.clone();
            id_source.push_str("_ext_scroll");
            egui::ScrollArea::vertical().id_source(id_source)
            .max_height(120.0 * UI_SIZE)
            .show(ui, |ui| {
                ui.set_min_width(530.0 * UI_SIZE);
                for i in physical_device.extension_properties() {
                    sized_text(
                        ui, 
                        format!("{} v{}", i.extension_name, i.spec_version), 
                        20.0 * UI_SIZE
                    );
                }
            });
        });

        sized_text(ui, format!("Features:"), 18.0 * UI_SIZE);
        egui::Frame::none()
        .fill(egui::Color32::from_rgb(160, 160, 160))
        .inner_margin(egui::style::Margin::same(10.0 * UI_SIZE))
        .show(ui, |ui| {
            let mut id_source = physical_device_name.clone();
            id_source.push_str("_fts_scroll");
            egui::ScrollArea::vertical().id_source(id_source)
            .max_height(120.0 * UI_SIZE)
            .show(ui, |ui| {
                ui.set_min_width(530.0 * UI_SIZE);
                sized_text(
                    ui, 
                    format!("{:?}", physical_device.supported_features()), 
                    20.0 * UI_SIZE
                );
            });
        });
    });
}

fn show_physical_devices_info(ui: &mut egui::Ui, physical_devices: &Vec<Arc<PhysicalDevice>>) {
    egui::Frame::none()
    .fill(egui::Color32::from_rgb(180, 180, 180))
    .outer_margin(egui::style::Margin::same(5.0 * UI_SIZE))
    .inner_margin(egui::style::Margin::same(10.0 * UI_SIZE))
    .show(ui, |ui| {
        ui.set_max_size(egui::vec2(650.0 * UI_SIZE, 370.0 * UI_SIZE));
        ui.vertical_centered(|ui| {
            sized_text(ui, "Physical devices", 30.0 * UI_SIZE);
        });

        egui::Frame::none()
        .fill(egui::Color32::from_rgb(160, 160, 160))
        .inner_margin(egui::style::Margin::same(10.0 * UI_SIZE))
        .show(ui, |ui| {
            ui.set_min_width(630.0 * UI_SIZE);
            for device in physical_devices.iter() {
                egui::CollapsingHeader::new(&device.properties().device_name)
                .show(ui, |ui| {
                    show_physical_device_info(ui, device.clone());
                });
            }
        });
    });
}

fn show_device_info(ui: &mut egui::Ui, device: Arc<Device>) {
    egui::Frame::none()
    .fill(egui::Color32::from_rgb(180, 180, 180))
    .outer_margin(egui::style::Margin::same(5.0 * UI_SIZE))
    .inner_margin(egui::style::Margin::same(10.0 * UI_SIZE))
    .show(ui, |ui| {
        ui.set_max_size(egui::vec2(550.0 * UI_SIZE, 370.0 * UI_SIZE));
        ui.vertical_centered(|ui| {
            sized_text(
                ui, 
                format!("Device ({})", device.physical_device().properties().device_name), 
                26.0 * UI_SIZE);
        });

        sized_text(ui, format!("Properties:"), 18.0 * UI_SIZE);
        egui::Frame::none()
        .fill(egui::Color32::from_rgb(160, 160, 160))
        .inner_margin(egui::style::Margin::same(10.0 * UI_SIZE))
        .show(ui, |ui| {
            egui::ScrollArea::vertical().id_source("device_layers_scroll")
            .max_height(120.0 * UI_SIZE)
            .show(ui, |ui| {
                ui.set_min_width(530.0 * UI_SIZE);
                sized_text(ui, format!("API version: {}", device.api_version().to_string()), 20.0 * UI_SIZE);
            });
        });

        sized_text(ui, format!("Enabled extensions:"), 18.0 * UI_SIZE);
        egui::Frame::none()
        .fill(egui::Color32::from_rgb(160, 160, 160))
        .inner_margin(egui::style::Margin::same(10.0 * UI_SIZE))
        .show(ui, |ui| {
            egui::ScrollArea::vertical().id_source("device_ext_scroll")
            .max_height(120.0 * UI_SIZE)
            .show(ui, |ui| {
                ui.set_min_width(530.0 * UI_SIZE);
                sized_text(ui, format!("{:?}", device.enabled_extensions()), 20.0 * UI_SIZE);
            });
        });

        sized_text(ui, format!("Enabled features:"), 18.0 * UI_SIZE);
        egui::Frame::none()
        .fill(egui::Color32::from_rgb(160, 160, 160))
        .inner_margin(egui::style::Margin::same(10.0 * UI_SIZE))
        .show(ui, |ui| {
            egui::ScrollArea::vertical().id_source("device_fts_scroll")
            .max_height(120.0 * UI_SIZE)
            .show(ui, |ui| {
                ui.set_min_width(530.0 * UI_SIZE);
                sized_text(ui, format!("{:?}", device.enabled_features()), 20.0 * UI_SIZE);
            });
        });
    });
}
