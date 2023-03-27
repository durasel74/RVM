pub mod cs {
    vulkano_shaders::shader!(
        ty: "compute", 
        path: "src/compute.glsl",
        vulkan_version: "1.2",
        spirv_version: "1.6",
    );
}