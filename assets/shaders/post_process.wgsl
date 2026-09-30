@group(0) @binding(0) var hdr_texture: texture_2d<f32>;

struct PostProcessParams {
    enable_unorm: u32,
    enable_srgb: u32,
    enable_hdr: u32,
    mac_peak_brightness: f32,
}

@group(0) @binding(1) var<uniform> params: PostProcessParams;

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) uv: vec2f,
}

const EXPOSURE: f32 = 1.0;
const LOOK_SLOPE: f32 = 1.0;
const LOOK_POWER: f32 = 1.35;
const LOOK_SAT: f32 = 1.4;

const HDR_START: f32 = 2.9;

const SRGB_TO_REC2020 = mat3x3<f32>(
    vec3f(0.6274, 0.0691, 0.0164),
    vec3f(0.3293, 0.9195, 0.0880),
    vec3f(0.0433, 0.0113, 0.8956)
);

const REC2020_TO_SRGB = mat3x3<f32>(
    vec3f(1.6605, -0.1246, -0.0182),
    vec3f(-0.5876, 1.1329, -0.1006),
    vec3f(-0.0728, -0.0083, 1.1187)
);

const AGX_INSET = mat3x3<f32>(
    vec3f( 0.856627153315983,   0.137318972929847,   0.11189821299995),
    vec3f( 0.0951212405381588,  0.761241990602591,   0.0767994186031903),
    vec3f( 0.0482516061458583,  0.101439036467562,   0.811302368396859)
);

const AGX_OUTSET = mat3x3<f32>(
    vec3f( 1.1271005818144368,  -0.1413297634984383,  -0.14132976349843826),
    vec3f(-0.11060664309660323,  1.157823702216272,   -0.11060664309660294),
    vec3f(-0.016493938717834573, -0.016493938717834257, 1.2519364065950405)
);

const AGX_MIN_EV: f32 = -12.47393;
const AGX_MAX_EV: f32 = 4.026069;

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    let p = vec2f(f32((vi << 1u) & 2u), f32(vi & 2u));
    var out: VertexOutput;
    out.clip_position = vec4f(p * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2f(p.x, 1.0 - p.y);
    return out;
}

fn agx_contrast_approx(x: vec3f) -> vec3f {
    let x2 = x * x;
    let x4 = x2 * x2;
    return 15.5 * x4 * x2
         - 40.14 * x4 * x
         + 31.96 * x4
         - 6.868 * x2 * x
         + 0.4298 * x2
         + 0.1191 * x
         - 0.00232;
}

fn agx_look_punchy(v: vec3f) -> vec3f {
    var c = pow(max(v * LOOK_SLOPE, vec3f(0.0)), vec3f(LOOK_POWER));
    let luma = dot(c, vec3f(0.2126, 0.7152, 0.0722));
    return vec3f(luma) + LOOK_SAT * (c - vec3f(luma));
}

fn agx_punchy(color: vec3f) -> vec3f {
    var v = SRGB_TO_REC2020 * color;
    v = AGX_INSET * v;
    v = max(v, vec3f(1e-10));
    v = (log2(v) - AGX_MIN_EV) / (AGX_MAX_EV - AGX_MIN_EV);
    v = clamp(v, vec3f(0.0), vec3f(1.0));

    v = agx_contrast_approx(v);
    v = agx_look_punchy(v);

    v = AGX_OUTSET * v;
    v = pow(max(v, vec3f(0.0)), vec3f(2.2));
    v = REC2020_TO_SRGB * v;
    return clamp(v, vec3f(0.0), vec3f(1.0));
}

fn agx_punchy_hdr(color: vec3f, peak: f32) -> vec3f {
    let base = agx_punchy(color);
    let headroom = max(peak - 1.0, 0.001);
    let over = max(color - vec3f(HDR_START), vec3f(0.0));
    let extra = headroom * (vec3f(1.0) - exp(-over / headroom));
    return base + extra;
}

fn srgb_oetf(c: vec3f) -> vec3f {
    let lo = c * 12.92;
    let hi = 1.055 * pow(c, vec3f(1.0 / 2.4)) - 0.055;
    return select(hi, lo, c <= vec3f(0.0031308));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let pixel_coords = vec2i(in.clip_position.xy);
    let raw = clamp(
        textureLoad(hdr_texture, pixel_coords, 0).rgb,
        vec3f(0.0),
        vec3f(65000.0)
    );
    let exposed = raw * EXPOSURE;

    var c: vec3f;

    if (params.enable_hdr == 1u) {
        c = agx_punchy_hdr(exposed, params.mac_peak_brightness);
    } else {
        c = agx_punchy(exposed);
        if (params.enable_unorm == 1u) {
            c = srgb_oetf(c);
        }
    }

    return vec4f(c, 1.0);
}
