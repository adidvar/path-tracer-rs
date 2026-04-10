@group(0) @binding(0) var hdr_texture: texture_2d<f32>;

struct PostProcessParams {
    enable_tonemapping: u32,
    enable_gamma: u32,
}

@group(0) @binding(1) var<uniform> params: PostProcessParams;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(vertex_index) / 2) * 4.0 - 1.0;
    let y = f32(i32(vertex_index) % 2) * 4.0 - 1.0;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>(x * 0.5 + 0.5, 1.0 - (y * 0.5 + 0.5));
    return out;
}

fn ACESFilm(x: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let pixel_coords = vec2<i32>(in.clip_position.xy);
    let raw_color = textureLoad(hdr_texture, pixel_coords, 0).rgb;

    var final_color = raw_color;

    if (params.enable_tonemapping == 1u) {
        final_color = ACESFilm(final_color);
    }

    if (params.enable_gamma == 1u) {
        final_color = pow(final_color, vec3<f32>(1.0 / 2.2));
    }

    return vec4<f32>(final_color, 1.0);
}
