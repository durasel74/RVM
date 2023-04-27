use std::sync::Arc;

use vulkano::{ VulkanLibrary };
use vulkano::instance::{ Instance };
use vulkano::device::physical::PhysicalDevice;
use vulkano::device::{ Device, Queue };
use vulkano::swapchain::{ PresentMode, Swapchain };

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

fn get_physical_device_local_memory(physical_device: Arc<PhysicalDevice>) -> u64 {
    let memory_prop = physical_device.memory_properties();

    let mut local_memory = 0u64;
    for heap in memory_prop.memory_heaps.iter() {
        if heap.flags.device_local { local_memory += heap.size }
    }
    local_memory
}



pub fn sized_text(ui: &mut egui::Ui, text: impl Into<String>, size: f32) {
    ui.label(egui::RichText::new(text).size(size).color(egui::Color32::BLACK));
}

pub fn show_lib_info(ui: &mut egui::Ui, library: Arc<VulkanLibrary>) {
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

pub fn show_instance_info(ui: &mut egui::Ui, instance: Arc<Instance>) {
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

pub fn show_physical_device_info(ui: &mut egui::Ui, physical_device: Arc<PhysicalDevice>) {
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

pub fn show_physical_devices_info(ui: &mut egui::Ui, physical_devices: &Vec<Arc<PhysicalDevice>>) {
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

pub fn show_device_info(ui: &mut egui::Ui, device: Arc<Device>) {
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

pub fn show_queues_info(ui: &mut egui::Ui, queues: &Vec<Arc<Queue>>) {
    egui::Frame::none()
    .fill(egui::Color32::from_rgb(180, 180, 180))
    .outer_margin(egui::style::Margin::same(5.0 * UI_SIZE))
    .inner_margin(egui::style::Margin::same(10.0 * UI_SIZE))
    .show(ui, |ui| {
        ui.set_max_size(egui::vec2(550.0 * UI_SIZE, 370.0 * UI_SIZE));
        ui.vertical_centered(|ui| {
            sized_text(ui, format!("Queues"), 26.0 * UI_SIZE);
        });

        sized_text(ui, format!("Info:"), 18.0 * UI_SIZE);
        egui::Frame::none()
        .fill(egui::Color32::from_rgb(160, 160, 160))
        .inner_margin(egui::style::Margin::same(10.0 * UI_SIZE))
        .show(ui, |ui| {
            ui.set_min_width(530.0 * UI_SIZE);
            for queue in queues {
                sized_text(ui, format!("Queue family_index: {} Queue index: {}", 
                    queue.queue_family_index(), queue.id_within_family()), 20.0 * UI_SIZE);
            }
        });
    });
}

pub fn show_swapchain_info(ui: &mut egui::Ui, swapchain: Arc<Swapchain>) {
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
