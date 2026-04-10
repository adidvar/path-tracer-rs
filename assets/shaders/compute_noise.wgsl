@group(0) @binding(0) var tex_out: texture_storage_2d<rgba32float, write>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let dim = textureDimensions(tex_out);

    if (global_id.x >= dim.x || global_id.y >= dim.y) {
        return;
    }

    var uv = vec2<f32>(global_id.xy) / vec2<f32>(dim);

    uv = uv * 2.0 - 1.0;

    uv.x *= f32(dim.x) / f32(dim.y);

    let uv0 = uv;
    var final_color = vec3<f32>(0.0);

    for (var i = 0.0; i < 4.0; i += 1.0) {
        uv = fract(uv * 1.5) - 0.5;

        var d = length(uv) * exp(-length(uv0));

        let col = vec3<f32>(
            sin(d * 10.0 + i * 1.0 + 0.0) * 0.5 + 0.5,
            sin(d * 10.0 + i * 1.2 + 2.0) * 0.5 + 0.5,
            sin(d * 10.0 + i * 1.5 + 4.0) * 0.5 + 0.5
        );

        d = sin(d * 8.0) / 8.0;
        d = abs(d);

        d = pow(0.01 / d, 1.2);

        final_color += col * d;
    }

    textureStore(tex_out, global_id.xy, vec4<f32>(final_color, 1.0));
}
