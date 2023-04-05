#version 450

layout(local_size_x = 16, local_size_y = 16, local_size_z = 1) in;

layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;
// layout(set = 0, binding = 1) buffer Time {
//     float data[];
// } time;

void main() {
    vec2 norm_coordinates = (gl_GlobalInvocationID.xy + vec2(0.5)) / vec2(imageSize(img));
    vec2 c = (norm_coordinates - vec2(0.5)) * 2.0 - vec2(1.0, 0.0);

    vec2 z = vec2(0.0, 0.0);
    float i;
    for (i = 0.0; i < 1.0; i += 0.005) {
        // z = vec2(
        //     z.x * z.x - z.y * z.y + c.x * (1.0 / exp(time.data[0] / 3.0) * 5.0) + 0.005,
        //     z.y * z.x + z.x * z.y + c.y * (1.0 / exp(time.data[0] / 3.0) * 5.0) + 0.65
        // );

        z = vec2(
            z.x * z.x - z.y * z.y + c.x,
            z.y * z.x + z.x * z.y + c.y
        );

        if (length(z) > 4.0) {
            break;
        }
    }

    vec4 to_write = vec4(vec3(i), 1.0);
    imageStore(img, ivec2(gl_GlobalInvocationID.xy), to_write);
}