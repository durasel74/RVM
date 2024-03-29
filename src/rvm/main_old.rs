
use super::instance_init_info::InstanceInitInfo;
use super::device_init_info::DeviceInitInfo;
use super::shader_module;

use std::error::Error;
use std::sync::Arc;
use std::cmp;
use std::mem::{ size_of, size_of_val };
use std::time::{ self, Instant };

use bytemuck::{ Pod, Zeroable };

use winit::event::{ Event, WindowEvent, StartCause, KeyboardInput, ScanCode, 
    DeviceEvent, ElementState };
use winit::event_loop::{ ControlFlow, EventLoop, DeviceEventFilter };
use winit::window::WindowBuilder;
use winit::platform::windows::WindowExtWindows;
use winit::dpi::PhysicalSize;
use winit::window::Fullscreen;

use vulkano::{ VulkanLibrary, VulkanError };
use vulkano::instance::{ Instance, InstanceCreateInfo };
use vulkano::device::physical::{ PhysicalDevice, PhysicalDeviceType, 
    PhysicalDeviceError };
use vulkano::device::{ Device, DeviceCreateInfo, QueueCreateInfo, Queue };
use vulkano::memory::allocator::{ GenericMemoryAllocator, 
    GenericMemoryAllocatorCreateInfo, AllocationType, MemoryUsage };
use vulkano::memory::allocator::suballocator::{ FreeListAllocator, BumpAllocator, 
    PoolAllocator, BuddyAllocator };
use vulkano::buffer::{ BufferUsage, BufferAccess, CpuAccessibleBuffer, 
    CpuBufferPool, DeviceLocalBuffer };
use vulkano::buffer::cpu_pool::CpuBufferPoolSubbuffer;
use vulkano::image::{ StorageImage, SwapchainImage, ImageAccess, ImageUsage, 
    ImageLayout, SampleCount, ImageDimensions, ImageCreateFlags };
use vulkano::image::view::{ ImageViewAbstract, ImageView, ImageViewCreationError };
use vulkano::format::Format;
use vulkano::swapchain::{ self, Surface, SurfaceInfo, SurfaceCapabilities, Win32Monitor,
    ColorSpace, PresentMode, FullScreenExclusive, Swapchain, SwapchainCreateInfo, 
    SwapchainPresentInfo, SwapchainCreationError, AcquireError };
use vulkano::render_pass::{ RenderPass, RenderPassCreateInfo, RenderPassCreationError, 
    SubpassDescription, AttachmentDescription, AttachmentReference, LoadOp, StoreOp, 
    Framebuffer, FramebufferCreateInfo, FramebufferCreationError };
use vulkano::pipeline::{ Pipeline, ComputePipeline, PipelineBindPoint };
use vulkano::shader::spirv::SpirvError;
use vulkano::descriptor_set::{ PersistentDescriptorSet, WriteDescriptorSet, DescriptorSet, DescriptorBindingResources };
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::command_buffer::allocator::{ StandardCommandBufferAllocator, 
    StandardCommandBufferAllocatorCreateInfo };
use vulkano::command_buffer::{ AutoCommandBufferBuilder, CommandBufferUsage, 
    PrimaryAutoCommandBuffer, CopyImageInfo };
use vulkano::sync::{ self, GpuFuture, FlushError };


use egui_winit_vulkano::Gui;
use vulkano_win::VkSurfaceBuild;
use vulkano_shaders;
//use egui::{ ScrollArea, TextEdit, TextStyle, Label };


use super::super::ui;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Zeroable, Pod)]
struct ViewPosition {
    pub color: [f32; 3],
    pub quality: u32,
    pub fract_color: [f32; 3],
    pub zoom: f32,
    pub pos_x: f32,
    pub pos_y: f32,
}
impl ViewPosition {
    fn new() -> Self {
        ViewPosition {
            quality: 500,
            zoom: 1.0,
            pos_x: -500.0,
            pos_y: 0.0,
            color: [0.0, 1.0, 0.0],
            fract_color: [0.0, 0.0, 0.0],
        }
    }

    fn reset(self) -> Self {
        ViewPosition {
            color: self.color,
            fract_color: self.fract_color,
            ..ViewPosition::new()
        }
    }
}

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

fn get_device_queue_create_infos(physical_device: Arc<PhysicalDevice>)
-> Result<Vec<QueueCreateInfo>, Box<dyn Error>> {
    let queue_family_properties = physical_device.queue_family_properties();

    let mut queue_family_indices: Vec<u32> = vec![];
    for (i, q) in queue_family_properties.iter().enumerate() {
        if q.queue_flags.compute && q.queue_flags.transfer {
            queue_family_indices.push(i as u32);
        }
    }
    if queue_family_indices.len() < 1 {
        return Err(Box::new(VulkanError::InitializationFailed));
    }

    let queue_create_infos = queue_family_indices.into_iter()
    .map(|queue_family_index| {
        QueueCreateInfo {
            queue_family_index,
            ..Default::default()
        }
    }).collect();

    Ok(queue_create_infos)
}

fn create_device_connection(physical_device: Arc<PhysicalDevice>)
-> Result<(Arc<Device>, Vec<Arc<Queue>>), Box<dyn Error>> {
    let supported_extensions = physical_device.supported_extensions();
    let enabled_extensions = DeviceInitInfo::default().confirm_extensions(supported_extensions)?;

    let supported_features = physical_device.supported_features();
    let enabled_features = DeviceInitInfo::default().confirm_features(supported_features)?;

    let queue_create_infos = get_device_queue_create_infos(physical_device.clone())?;

    let (device, queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            enabled_extensions,
            enabled_features,
            queue_create_infos,
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
    //let full_screen_exclusive = surface_info.full_screen_exclusive;

    let surface_capabilities = device.physical_device().surface_capabilities(
        &surface,
        surface_info
    )?;

    let image_extent = surface_capabilities.current_extent.unwrap_or([0, 0]);
    let min_image_count = match surface_capabilities.max_image_count {
        None => cmp::max(3, surface_capabilities.min_image_count),
        Some(limit) => cmp::min(cmp::max(3, surface_capabilities.min_image_count), limit)
    };
    let image_usage = ImageUsage {
        transfer_dst: true,
        color_attachment: true,
        .. ImageUsage::empty()
    };

    let swapchain_and_images = Swapchain::new(
        device.clone(),
        surface,
        SwapchainCreateInfo {
            min_image_count,
            image_format: Some(Format::B8G8R8A8_SRGB),
            image_color_space: ColorSpace::SrgbNonLinear,
            image_extent,
            image_usage,
            present_mode: PresentMode::Fifo, // PresentMode::Immediate | Vsync
            clipped: true,
            // full_screen_exclusive,
            // win32_monitor: monitor,
            ..Default::default()
        }
    )?;
    Ok(swapchain_and_images)
}

fn recreate_swapchain(swapchain: Arc<Swapchain>, window: Arc<winit::window::Window>) 
-> Result<(Arc<Swapchain>, Vec<Arc<SwapchainImage>>), SwapchainCreationError> {
    let swapchain_images = swapchain.recreate(SwapchainCreateInfo {
        image_extent: [window.inner_size().width, window.inner_size().height],
        ..swapchain.create_info()
    })?;
    Ok(swapchain_images)
}

fn create_render_pass(device: Arc<Device>, image_format: Format) 
-> Result<Arc<RenderPass>, RenderPassCreationError> {
    let render_pass = RenderPass::new(
        device.clone(),
        RenderPassCreateInfo {
            attachments: vec![
                AttachmentDescription {
                    format: Some(image_format),
                    samples: SampleCount::Sample1,
                    load_op: LoadOp::DontCare,
                    store_op: StoreOp::Store,
                    initial_layout: ImageLayout::ColorAttachmentOptimal,
                    final_layout: ImageLayout::ColorAttachmentOptimal,
                    ..Default::default()
                }
            ],
            subpasses: vec![
                SubpassDescription {
                    color_attachments: vec![Some(AttachmentReference {
                        attachment: 0,
                        layout: ImageLayout::ColorAttachmentOptimal,
                        ..Default::default()
                    })],
                    ..Default::default()
                }
            ],
            ..Default::default()
        },
    )?;
    Ok(render_pass)
}

fn create_framebuffers(render_pass: Arc<RenderPass>, images: &Vec<Arc<SwapchainImage>>)
-> Result<Vec<Arc<Framebuffer>>, Box<dyn Error>> {
    let mut result = vec![];
    for image in images {
        let view = ImageView::new_default(image.clone())?;
        let framebuffer = Framebuffer::new(
            render_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![view],
                ..Default::default()
            },
        )?;
        result.push(framebuffer);
    }
    Ok(result)
}

fn create_pipeline(device: Arc<Device>) -> Result<Arc<ComputePipeline>, Box<dyn Error>> {
    let shader = shader_module::cs::load(device.clone())?;
    let entry_point = if let Some(ep) = shader.entry_point("main") { ep }
    else { return Err(Box::new(SpirvError::InvalidHeader)) };
    
    let pipeline = ComputePipeline::new(
        device.clone(),
        entry_point,
        &(),
        None,     // Добавить кеш!!!
        |_| {}
    )?;
    Ok(pipeline)
}

fn create_storage_images_views(
    allocator: &GenericMemoryAllocator::<Arc<BumpAllocator>>,
    swapchain: Arc<Swapchain>,
    queue_family_index: u32)
-> Result<Vec<Arc<ImageView<StorageImage>>>, Box<dyn Error>> {

    let mut images_views = vec![];
    for _ in 0..swapchain.image_count() {
        let image = StorageImage::with_usage(
            allocator,
            ImageDimensions::Dim2d { 
                width: swapchain.image_extent()[0],
                height: swapchain.image_extent()[1],
                array_layers: swapchain.image_array_layers()
            },
            Format::R8G8B8A8_UNORM,
            ImageUsage {
                transfer_src: true,
                storage: true,
                ..Default::default()
            },
            ImageCreateFlags::default(),
            [queue_family_index]
        )?;
        let view = ImageView::new_default(image)?;
        images_views.push(view);
    }
    Ok(images_views)
}

fn create_swapchain_images_views(images: &Vec<Arc<SwapchainImage>>) 
-> Result<Vec<Arc<ImageView<SwapchainImage>>>, ImageViewCreationError> {
    let mut images_views = vec![];
    for image in images {
        let view = ImageView::new_default(image.clone())?;
        images_views.push(view);
    }
    Ok(images_views)
}

fn create_descriptor_sets_for_swapchain(
    descriptor_allocator: &StandardDescriptorSetAllocator,
    pipeline: Arc<ComputePipeline>,
    images_views: &Vec<Arc<ImageView<StorageImage>>>,
    view_pos_buffer: Arc<CpuAccessibleBuffer<ViewPosition>>) 
-> Result<Vec<Arc<PersistentDescriptorSet>>, Box<dyn Error>> {
    let mut result = vec![];
    for image_view in images_views {
        let descriptor_set_layout = pipeline.layout().set_layouts().get(0)
            .expect("DescriptorSetLayout not found by index 0");
        let descriptor_set = PersistentDescriptorSet::new(
            descriptor_allocator,
            descriptor_set_layout.clone(),
            [
                WriteDescriptorSet::image_view(0, image_view.clone()),
                WriteDescriptorSet::buffer(1, view_pos_buffer.clone()),
            ]
        )?;
        result.push(descriptor_set);
    }
    Ok(result)
}

fn create_render_command_buffers(
    command_buffer_allocator: &StandardCommandBufferAllocator,
    pipeline: Arc<ComputePipeline>,
    extent: (u32, u32),
    descriptor_sets: &Vec<Arc<PersistentDescriptorSet>>,
    render_images_views: &Vec<Arc<ImageView<StorageImage>>>,
    present_images: &Vec<Arc<ImageView<SwapchainImage>>>,
    queue_family_index: u32)
-> Result<Vec<Arc<PrimaryAutoCommandBuffer>>, Box<dyn Error>> {

    let mut command_buffers = vec![];
    for i in 0..descriptor_sets.len() {
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            command_buffer_allocator,
            queue_family_index,
            CommandBufferUsage::SimultaneousUse
        )?;

        command_buffer_builder.bind_pipeline_compute(pipeline.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                pipeline.layout().clone(),
                0,
                vec![descriptor_sets[i].clone()]
            )
            .dispatch([extent.0 / 16, extent.1 / 16, 1])?
            .copy_image(CopyImageInfo::images(
                render_images_views[i].image().clone(), 
                present_images[i].image().clone()
            ))?;

        let command_buffer = command_buffer_builder.build()?;
        command_buffers.push(Arc::new(command_buffer));
    };
    Ok(command_buffers)
}

pub fn main_old() {
    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new()
        .with_title(format!("RVM {}", VERSION))
        .with_inner_size(PhysicalSize::new(1000, 800));
    let window = match window_builder.build(&event_loop) {
        Ok(win) => Arc::new(win),
        Err(err) => { println!("Window creating error: {:?}", err); return; }
    };

    let instance = match create_vulkan_instance() {
        Ok(inst) => inst,
        Err(err) => { println!("Vulkan instance creating error: {:?}", err); return; }
    };

    let surface = match vulkano_win::create_surface_from_winit(window.clone(), instance.clone()) {
        Ok(surface) => surface,
        Err(err) => { println!("Surface creating error: {:?}", err); return; }
    };

    let physical_devices = match get_right_devices(instance.clone()) {
        Ok(pd) => pd,
        Err(err) => { println!("Physical devices error: {:?}", err); return; }
    };

    let (device, queues) = match create_device_connection(physical_devices[0].clone()) {
        Ok(device) => device,
        Err(err) => { println!("Device creating error: {:?}", err); return; }
    };
    let main_queue = queues[0].clone();

    let win32_monitor = get_app_monitor(window.clone());
    let (mut swapchain, mut images) = match create_swapchain(surface.clone(), device.clone(), win32_monitor) {
        Ok(swapchain_images) => swapchain_images,
        Err(err) => { println!("Swapchain creating error: {:?}", err); return; }
    };

    // let render_pass = match create_render_pass(device.clone(), swapchain.image_format()) {
    //     Ok(render_pass) => render_pass,
    //     Err(err) => { println!("RenderPass creating error: {}", err); return; }
    // };

    // let mut framebuffers = match create_framebuffers(render_pass.clone(), &images) {
    //     Ok(framebuffers) => framebuffers,
    //     Err(err) => { println!("Framebuffers creating error: {}", err); return; }
    // };

    let mut pipeline = match create_pipeline(device.clone()) {
        Ok(pipeline) => pipeline,
        Err(err) => { println!("Pipeline creating error: {:?}", err); return; }
    };


    let descriptor_allocator = StandardDescriptorSetAllocator::new(device.clone());
    let command_buffer_allocator = StandardCommandBufferAllocator::new(
        device.clone(), 
        StandardCommandBufferAllocatorCreateInfo {
            primary_buffer_count: 50,
            secondary_buffer_count: 50,
            ..Default::default()
        }
    );
    let mut storage_images_allocator = GenericMemoryAllocator::<Arc<BumpAllocator>>::new_default(device.clone());
    let view_position_allocator = GenericMemoryAllocator::<Arc<FreeListAllocator>>
        ::new_default(device.clone());

    

    let mut storage_images_views = match create_storage_images_views(
        &storage_images_allocator,
        swapchain.clone(),
        main_queue.queue_family_index()
    ) {
        Ok(views) => views,
        Err(err) => { println!("Storage images views creating error: {:?}", err); return; }
    };

    let mut swapchain_images_views = match create_swapchain_images_views(&images) {
        Ok(views) => views,
        Err(err) => { println!("Swapchain images views creating error: {:?}", err); return; }
    };

    let mut view_position = ViewPosition::new();
    let view_pos_buffer = CpuAccessibleBuffer::from_data(
        &view_position_allocator,
        BufferUsage {
            storage_buffer: true,
            ..Default::default()
        },
        false,
        view_position
    ).expect("Failed to create buffer");


    let mut descriptor_sets = match create_descriptor_sets_for_swapchain(
        &descriptor_allocator, 
        pipeline.clone(), 
        &storage_images_views,
        view_pos_buffer.clone()
    ) {
        Ok(sets) => sets,
        Err(err) => { println!("Descriptor sets creating error: {:?}", err); return; }
    };

    let mut command_buffers = match create_render_command_buffers(
        &command_buffer_allocator,
        pipeline.clone(), 
        (window.inner_size().width, window.inner_size().height),
        &descriptor_sets,
        &storage_images_views,
        &swapchain_images_views,
        main_queue.queue_family_index()
    ) {
        Ok(command_buffers) => command_buffers,
        Err(err) => { println!("Command buffers creating error: {:?}", err); return; }
    };


    let mut gui = Gui::new(
        &event_loop,
        surface.clone(),
        Some(swapchain.image_format()),
        main_queue.clone(),
        true
    );
    let mut is_show_infos = false;
    let mut is_full_screen = false;
    let mut is_mouse_move_active = false;
    let mut is_mouse_zoom_active = false;

    // let now = Instant::now();
    // let mut old_since_time = now.elapsed().as_millis();
    event_loop.run(move |event, _, control_flow| {
        // let since_time = now.elapsed().as_millis();
        // let delta_time = (since_time - old_since_time) as f64;

        match event {
            Event::NewEvents(start_cause) => {},
            Event::WindowEvent { event, window_id } if window_id == window.id() => {
                let _pass_events_to_game = !gui.update(&event);
                match event {
                    WindowEvent::Resized(_) => {
                        match recreate_swapchain(swapchain.clone(), window.clone()) {
                            Ok(si) => { swapchain = si.0; images = si.1; },
                            Err(err) => println!("Swapchain recreating error: {:?}", err)
                        };
                        // match create_framebuffers(render_pass.clone(), &images) {
                        //     Ok(fb) => framebuffers = fb,
                        //     Err(err) => println!("Framebuffers recreating error: {}", err)
                        // };
                        match create_pipeline(device.clone()) {
                            Ok(pe) => pipeline = pe,
                            Err(err) => { println!("Pipeline recreating error: {:?}", err); return; }
                        };
                        storage_images_allocator = GenericMemoryAllocator::<Arc<BumpAllocator>>::new_default(device.clone());
                        match create_storage_images_views(
                            &storage_images_allocator,
                            swapchain.clone(),
                            main_queue.queue_family_index()
                        ) {
                            Ok(views) => storage_images_views = views,
                            Err(err) => { println!("Storage images views recreating error: {:?}", err); return; }
                        };
                        match create_swapchain_images_views(&images) {
                            Ok(views) => swapchain_images_views = views,
                            Err(err) => { println!("Swapchain images views recreating error: {:?}", err); return; }
                        };
                        match create_descriptor_sets_for_swapchain(
                            &descriptor_allocator, 
                            pipeline.clone(), 
                            &storage_images_views,
                            view_pos_buffer.clone()
                        ) {
                            Ok(ds) => descriptor_sets = ds,
                            Err(err) => { println!("Descriptor sets recreating error: {:?}", err); return; }
                        };
                        match create_render_command_buffers(
                            &command_buffer_allocator,
                            pipeline.clone(), 
                            (window.inner_size().width, window.inner_size().height),
                            &descriptor_sets,
                            &storage_images_views,
                            &swapchain_images_views,
                            main_queue.queue_family_index()
                        ) {
                            Ok(cb) => command_buffers = cb,
                            Err(err) => { println!("Command buffers recreating error: {:?}", err); return; }
                        };
                    }
                    WindowEvent::ScaleFactorChanged { .. } => {
                        //renderer.resize();
                        // Пересоздать конвейер
                    }
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => ()
                }
            }
            Event::DeviceEvent { device_id, event } => {
                match event {
                    DeviceEvent::Key(keyboard_input) => match keyboard_input {
                        KeyboardInput { scancode: 1, state: ElementState::Released, ..} 
                            => is_show_infos = !is_show_infos,
                        KeyboardInput { scancode: 28, state: ElementState::Released, ..} 
                            => {
                                if !is_full_screen {
                                    let current_monitor = match window.current_monitor() {
                                        Some(monitor) => monitor,
                                        None => return,
                                    };
                                    let video_mode = match current_monitor.video_modes().next() {
                                        Some(video_mode) => video_mode,
                                        None => return,
                                    };

                                    window.set_fullscreen(Some(Fullscreen::Exclusive(video_mode)));
                                    is_full_screen = true;
                                }
                                else {
                                    window.set_fullscreen(None);
                                    is_full_screen = false;
                                }
                            },
                        _ => ()
                    },
                    DeviceEvent::Button { button, state } => {
                        if state == ElementState::Pressed && button == 3 {
                            is_mouse_move_active = true;
                        }
                        else { is_mouse_move_active = false; }

                        if state == ElementState::Pressed && button == 2 {
                            is_mouse_zoom_active = true;
                        }
                        else { is_mouse_zoom_active = false; }
                    }
                    DeviceEvent::MouseMotion { delta } => {
                        if is_mouse_move_active {
                            view_position.pos_x -= (delta.0 as f32) / (view_position.zoom * 0.1).exp();
                            view_position.pos_y -= (delta.1 as f32) / (view_position.zoom * 0.1).exp();
                            //println!("{} {}", delta_x, delta_y);
                        }
                        if is_mouse_zoom_active {
                            view_position.zoom -= (delta.1 as f32) / 20.0;
                        }
                    },
                    DeviceEvent::MouseWheel { delta } => match delta {
                        winit::event::MouseScrollDelta::LineDelta(_, y) => {
                            if view_position.zoom + y > 0.0 {
                                view_position.zoom += y
                            }
                        },
                        _ => (),
                    },
                    _ => ()
                }
                // println!("{:?}", event)
            }
            //Event::RedrawRequested(window_id) if window_id == window_id => { }

            Event::MainEventsCleared => {
                // if delta_time > (1000.0 / 60.0)
                let (image_index, suboptimal, acquire_future) =
                match swapchain::acquire_next_image(swapchain.clone(), None) {
                    Ok(r) => r,
                    Err(AcquireError::OutOfDate) => { return; }
                    Err(e) => panic!("Failed to acquire next image: {:?}", e),
                };


                
                gui.immediate_ui(|gui| {
                    let ctx = gui.context();
                    if !is_show_infos {
                        let frame = egui::Frame::none();
                        egui::CentralPanel::default().frame(frame).show(&ctx, |ui| {
                            egui::Area::new("MiniPanel Area")
                            .show(&ctx, |ui| {
                                egui::Frame::none()
                                .fill(egui::Color32::from_rgb(200, 200, 200))
                                .outer_margin(egui::style::Margin::same(5.0))
                                .inner_margin(egui::style::Margin::same(10.0))
                                .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 0, 0)))
                                .show(ui, |ui| {
                                    ui.style_mut().spacing.slider_width = 300.0;
                                    ui.add(egui::Slider::new(&mut view_position.quality, 1..=1000).text("Quality"));
                                    ui.add(egui::Slider::new(&mut view_position.zoom, 1.0..=150.0).text("Zoom"));
                                    ui.add(egui::Slider::new(&mut view_position.pos_x, -1000.0..=1000.0).text("Pos X"));
                                    ui.add(egui::Slider::new(&mut view_position.pos_y, -1000.0..=1000.0).text("Pox Y"));
                                    ui.horizontal(|ui| {
                                        if ui.button("Reset").clicked() {
                                            view_position = view_position.reset();
                                        }
                                        ui.color_edit_button_rgb(&mut view_position.color);
                                        ui.color_edit_button_rgb(&mut view_position.fract_color);
                                    });
                                });
                            });
                        });
                        return;
                    }

                    let frame = egui::Frame::none()
                        .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 150));
                    egui::CentralPanel::default().frame(frame).show(&ctx, |ui| {
                        ui.visuals_mut().collapsing_header_frame = true;

                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui::ui_old::show_lib_info(ui, instance.library().clone());
                                ui::ui_old::show_instance_info(ui, instance.clone());
                            });
                            ui.vertical(|ui| {
                                ui::ui_old::show_physical_devices_info(ui, &physical_devices);
                            });
                            ui.vertical(|ui| {
                                ui::ui_old::show_device_info(ui, device.clone());
                                ui::ui_old::show_queues_info(ui, &queues);
                                ui::ui_old::show_swapchain_info(ui, swapchain.clone());
                            });
                        });
                    });
                });




                let exec_future = match sync::now(device.clone())
                    .join(acquire_future)
                    .then_execute(
                        queues[0].clone(),
                        command_buffers[image_index as usize].clone()
                    ) {
                        Ok(cbf) => cbf,
                        Err(err) => return
                    };

                let ui_future = gui.draw_on_image(
                    exec_future,
                    swapchain_images_views[image_index as usize].clone()
                );

                let fence_future = ui_future
                    .then_swapchain_present(
                        queues[0].clone(),
                        SwapchainPresentInfo::swapchain_image_index(
                            swapchain.clone(), 
                            image_index
                        )
                    )
                    .then_signal_fence_and_flush();
                
                match fence_future {
                    Ok(future) => { future.wait(None).unwrap(); }
                    Err(FlushError::OutOfDate) => { return; }
                    Err(e) => { println!("Failed to flush future: {:?}", e); }
                }

                let mut content = view_pos_buffer.write().unwrap();
                content.quality = view_position.quality;
                content.zoom = view_position.zoom;
                content.pos_x = view_position.pos_x;
                content.pos_y = view_position.pos_y;
                content.color = view_position.color;
                content.fract_color = view_position.fract_color;
            }
            Event::RedrawEventsCleared => {},
            Event::LoopDestroyed => {},
            _ => (),
        }
    });
}