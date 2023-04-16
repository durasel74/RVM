#version 450

layout(local_size_x = 16, local_size_y = 16, local_size_z = 1) in;

layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;
layout(set = 0, binding = 1) buffer ViewPosition {
    vec3 color;
    uint quality;
    vec3 fract_color;
    float zoom;
    float pos_x;
    float pos_y;
} view_position;

void main() {
    float aspect_ratio = float(imageSize(img).x) / float(imageSize(img).y);
    vec2 norm_coordinates = gl_GlobalInvocationID.xy;
    vec2 c = (2.0 * norm_coordinates - vec2(imageSize(img))) / float(imageSize(img).y);
    vec2 z = vec2(0.0, 0.0);

    float actual_zoom = exp(view_position.zoom / 10.0);
    vec2 actual_pos = vec2(
        (view_position.pos_x * 0.001) * actual_zoom,
        (view_position.pos_y * 0.001) * actual_zoom
    );
    c.x = (c.x + actual_pos.x) / actual_zoom;
    c.y = (c.y + actual_pos.y) / actual_zoom;

    int iterations = 0;
    while (iterations < view_position.quality)
    {
        z = vec2(
            (z.x * z.x - z.y * z.y) + c.x,
            (2.0 * z.x * z.y) + c.y
        );

        if (length(z) > 4.0) break;
        iterations += 1;
    }

    if (iterations == view_position.quality)
    {
        vec4 to_write = vec4(view_position.fract_color.bgr, 1.0);
        imageStore(img, ivec2(gl_GlobalInvocationID.xy), to_write);
    }
    else
    {
        float iters = float(iterations) / view_position.quality;
        vec4 to_write = vec4(view_position.color.bgr * iters, 1.0);
        imageStore(img, ivec2(gl_GlobalInvocationID.xy), to_write);
    }
}
