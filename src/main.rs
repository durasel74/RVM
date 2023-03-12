//#![allow(non_snake_case)]

mod instance_init_info;
mod device_init_info;
use instance_init_info::InstanceInitInfo;
use device_init_info::DeviceInitInfo;
use winit::platform::windows::WindowExtWindows;

use std::error::Error;
use std::sync::Arc;
use std::cmp;
use std::mem::{ size_of, size_of_val };

use winit::event::{ Event, WindowEvent, StartCause };
use winit::event_loop::{ ControlFlow, EventLoop, DeviceEventFilter };
use winit::window::WindowBuilder;

use vulkano::{ VulkanLibrary, VulkanError };
use vulkano::instance::{ Instance, InstanceCreateInfo };
use vulkano::device::physical::{ PhysicalDevice, PhysicalDeviceType, PhysicalDeviceError };
use vulkano::device::{ Device, DeviceCreateInfo, QueueCreateInfo, Queue };
use vulkano::memory::allocator::{ GenericMemoryAllocator, GenericMemoryAllocatorCreateInfo, AllocationType, MemoryUsage };
use vulkano::memory::allocator::suballocator::{ FreeListAllocator, BumpAllocator, PoolAllocator, BuddyAllocator };
use vulkano::buffer::{ BufferUsage, BufferAccess, CpuAccessibleBuffer, CpuBufferPool, DeviceLocalBuffer };
use vulkano::image::{ AttachmentImage, ImageAccess, ImageUsage, SwapchainImage };
use vulkano::image::view::ImageView;
use vulkano::format::Format;
use vulkano::swapchain::{ Surface, SurfaceInfo, SurfaceCapabilities, Win32Monitor,
    ColorSpace, PresentMode, FullScreenExclusive, Swapchain, SwapchainCreateInfo, 
    SwapchainCreationError };

use egui_winit_vulkano::Gui;
use vulkano_win::VkSurfaceBuild;
use vulkano_util::context::{ VulkanoConfig, VulkanoContext };
use vulkano_util::window::{ VulkanoWindows, WindowDescriptor };
//use egui::{ ScrollArea, TextEdit, TextStyle, Label };

const VERSION: &str = env!("CARGO_PKG_VERSION");
const UI_SIZE: f32 = 0.7;

fn byte_size(byte: u64) -> String {
    let size_sign = ["B", "Kb", "Mb", "Gb"];

    let mut result = byte as f64;
    let mut sign_index = 0;
    for i in 0..size_sign.len() {
        sign_index = i;
        let new_size = result / 1024.0;
        if new_size.floor() == 0.0 { break }
        if i == size_sign.len() - 1 { break }
        result = new_size;
    }
    return format!("{:.2} {}", result, size_sign[sign_index]);
}

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
        if heap.flags.device_local { local_memory += heap.size }
    }
    local_memory
}

fn create_device_connection(physical_device: Arc<PhysicalDevice>)
-> Result<(Arc<Device>, Vec<Arc<Queue>>), Box<dyn Error>> {
    let supported_extensions = physical_device.supported_extensions();
    let enabled_extensions = DeviceInitInfo::default().confirm_extensions(supported_extensions)?;

    let supported_features = physical_device.supported_features();
    let enabled_features = DeviceInitInfo::default().confirm_features(supported_features)?;

    let queue_create_info = QueueCreateInfo {
        queue_family_index: 0,
        ..Default::default()
    };

    // let queue_family_index = physical_device
    //     .queue_family_properties()
    //     .iter()
    //     .enumerate()
    //     .position(|(_, q)| q.queue_flags.graphics)
    //     .expect("couldn't find a graphical queue family") as u32;

    let (device, queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            enabled_extensions,
            enabled_features,
            queue_create_infos: vec![queue_create_info],
            ..Default::default()
        }
    )?;
    Ok((device, queues.collect()))
}

fn get_app_monitor(window: Arc<winit::window::Window>) -> Option<Win32Monitor> {
    if let Some(monitor) = window.current_monitor() {
        Some(vulkano_win::create_win32_monitor_from_winit(&monitor))
    }
    else if let Some(monitor) = window.primary_monitor() {
        Some(vulkano_win::create_win32_monitor_from_winit(&monitor))
    }
    else { None }
}

fn create_swapchain(surface: Arc<Surface>, device: Arc<Device>, monitor: Option<Win32Monitor>)
-> Result<(Arc<Swapchain>, Vec<Arc<SwapchainImage>>), Box<dyn Error>> {
    let surface_info = match monitor {
        Some(monitor) => SurfaceInfo { 
            full_screen_exclusive: FullScreenExclusive::ApplicationControlled, 
            win32_monitor: Some(monitor), 
            ..Default::default()
        },
        None => SurfaceInfo::default()
    };

    let surface_capabilities = device.physical_device().surface_capabilities(
        &surface,
        surface_info
    )?;
    let image_formats: Vec<Format> = device.physical_device().surface_formats(
        &surface,
        Default::default()
    )?.iter().map(|c| c.0).collect();

    let image_extent = surface_capabilities.current_extent.unwrap_or([0, 0]);
    let min_image_count = match surface_capabilities.max_image_count {
        None => cmp::max(3, surface_capabilities.min_image_count),
        Some(limit) => cmp::min(cmp::max(3, surface_capabilities.min_image_count), limit)
    };
    let image_usage = ImageUsage {
        //storage: true,
        color_attachment: true,
        .. ImageUsage::empty()
    };
    let image_format = *image_formats.iter()
        .find(|f| **f == Format::B8G8R8A8_SRGB)
        .unwrap_or(&image_formats[0]);

    let swapchain_images = Swapchain::new(
        device.clone(),
        surface,
        SwapchainCreateInfo {
            min_image_count,
            image_format: Some(image_format),
            image_color_space: ColorSpace::SrgbNonLinear,
            image_extent,
            image_usage,
            present_mode: PresentMode::Fifo, // PresentMode::Immediate | Vsync
            clipped: true,
            // full_screen_exclusive,
            // win32_monitor,
            ..Default::default()
        }
    )?;
    Ok(swapchain_images)
}

fn recreate_swapchain(swapchain: Arc<Swapchain>, window: Arc<winit::window::Window>) 
-> Result<(Arc<Swapchain>, Vec<Arc<SwapchainImage>>), SwapchainCreationError> {
    let swapchain_images = swapchain.recreate(SwapchainCreateInfo {
        image_extent: [window.inner_size().width, window.inner_size().height],
        ..swapchain.create_info()
    })?;
    Ok(swapchain_images)
}

fn main() {
    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new().with_title("RVM");
    let window = match window_builder.build(&event_loop) {
        Ok(win) => Arc::new(win),
        Err(err) => { println!("Window create error: {}", err); return; }
    };

    let instance = match create_vulkan_instance() {
        Ok(inst) => inst,
        Err(err) => { println!("Vulkan instance create error: {}", err); return; }
    };

    let surface = match vulkano_win::create_surface_from_winit(window.clone(), instance.clone()) {
        Ok(surface) => surface,
        Err(err) => { println!("Surface create error: {}", err); return; }
    };

    let physical_devices = match get_right_devices(instance.clone()) {
        Ok(pd) => pd,
        Err(err) => { println!("Physical devices error: {}", err); return; }
    };

    let (device, _) = match create_device_connection(physical_devices[0].clone()) {
        Ok(device) => device,
        Err(err) => { println!("Device create error: {}", err); return; }
    };

    let win32_monitor = get_app_monitor(window.clone());
    let (mut swapchain, mut images) = match create_swapchain(surface, device.clone(), win32_monitor) {
        Ok(swapchain_images) => swapchain_images,
        Err(err) => { println!("Swapchain create error: {}", err); return; }
    };



    

    // Стандартный распределитель, подходит для большинства выделений
    let free_list_memory_allocator = GenericMemoryAllocator::<Arc<FreeListAllocator>>::new_default(device.clone());

    // Лучше подходит для выделений на очень короткий период с полным сбросом
    let bump_memory_allocator = Arc::new(GenericMemoryAllocator::<Arc<BumpAllocator>>::new_default(device.clone()));

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
    //println!("{}", byte_size(free_list_local_buffer.size() / 8));

    let bump_buffer = CpuBufferPool::new(
        Arc::new(bump_memory_allocator.clone()),
        BufferUsage {
            storage_buffer: true,
            ..Default::default()
        },
        MemoryUsage::Upload,
    );
    bump_buffer.from_iter(data.clone()).unwrap();

    let bump_attachment_image = AttachmentImage::new(
        &bump_memory_allocator,
        [800, 600],
        Format::R8G8B8A8_SRGB
    ).unwrap();
    let bump_image_view = ImageView::new_default(bump_attachment_image.clone());
    println!("{}", byte_size((bump_attachment_image.dimensions().num_texels() * 4) as u64));

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
            Event::WindowEvent { event, window_id }  => {
                // Update Egui integration so the UI works!
                let _pass_events_to_game = !gui.update(&event);
                match event {
                    WindowEvent::Resized(_) => {
                        renderer.resize();
                        match recreate_swapchain(swapchain.clone(), window.clone()) {
                            Ok(si) => { swapchain = si.0; images = si.1; },
                            Err(err) => { println!("Swapchain recreate error: {}", err) }
                        };
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
                                show_device_info(ui, device.clone());
                                show_swapchain_info(ui, swapchain.clone());
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
                sized_text(ui, format!("Local memory: {}", byte_size(local_memory)), 20.0 * UI_SIZE);
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

fn show_swapchain_info(ui: &mut egui::Ui, swapchain: Arc<Swapchain>) {
    egui::Frame::none()
    .fill(egui::Color32::from_rgb(180, 180, 180))
    .outer_margin(egui::style::Margin::same(5.0 * UI_SIZE))
    .inner_margin(egui::style::Margin::same(10.0 * UI_SIZE))
    .show(ui, |ui| {
        ui.set_max_size(egui::vec2(550.0 * UI_SIZE, 370.0 * UI_SIZE));
        ui.vertical_centered(|ui| {
            sized_text(ui, format!("Swapchain"), 26.0 * UI_SIZE);
        });

        sized_text(ui, format!("Properties:"), 18.0 * UI_SIZE);
        egui::Frame::none()
        .fill(egui::Color32::from_rgb(160, 160, 160))
        .inner_margin(egui::style::Margin::same(10.0 * UI_SIZE))
        .show(ui, |ui| {
            ui.set_min_width(530.0 * UI_SIZE);
            let images_size = swapchain.image_extent();
            sized_text(ui, format!("Images count: {}", swapchain.image_count()), 20.0 * UI_SIZE);
            sized_text(ui, format!("Images format: {:?}", swapchain.image_format()), 20.0 * UI_SIZE);
            sized_text(ui, format!("Images color space: {:?}", swapchain.image_color_space()), 20.0 * UI_SIZE);
            sized_text(ui, format!("Images size: {}x{}", images_size[0], images_size[1]), 20.0 * UI_SIZE);
            sized_text(ui, format!("Clipped: {}", swapchain.clipped()), 20.0 * UI_SIZE);
            sized_text(ui, format!("Vsync: {}", if swapchain.present_mode() == 
                PresentMode::Immediate { false } else { true }), 20.0 * UI_SIZE);
        });
    });
}
