
@group(0) @binding(0) var output_tex: texture_storage_2d<rgba32float, write>;

struct GlobalParams {
    resolution: vec2<u32>,
    frame_count: u32,
    max_bounces: u32,
    global_light_intensity: f32,
    rays_per_pixel: u32,
    use_msaa: u32,
    _pad0: u32,
};
@group(1) @binding(0) var<uniform> globals: GlobalParams;

struct Camera {
    position: vec3<f32>,
    fov: f32,
    forward: vec3<f32>,
    aperture: f32,
    right: vec3<f32>,
    focus_dist: f32,
    up: vec3<f32>,
    _padding: f32,
};
@group(1) @binding(1) var<uniform> camera: Camera;

struct Material {
    diffuse: vec3<f32>,
    light_power: f32,
    glossiness: f32,
    specular: f32,
    _padding: vec2<f32>,
};
@group(1) @binding(2) var<storage, read> materials: array<Material>;

struct Sphere {
    position: vec3<f32>,
    radius: f32,
    material_index: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};
@group(1) @binding(3) var<storage, read> spheres: array<Sphere>;

struct Triangle {
    v0: vec3<f32>,
    n0_x: f32,
    v1: vec3<f32>,
    n0_y: f32,
    v2: vec3<f32>,
    n0_z: f32,
    n1: vec3<f32>,
    material_index: u32,
    n2: vec3<f32>,
    _padding: u32,
};
@group(1) @binding(4) var<storage, read> triangles: array<Triangle>;

@group(0) @binding(1) var<storage, read_write> accum_buf: array<vec4<f32>>;

fn pcg_hash(seed: ptr<function, u32>) -> u32 {
    let state = *seed * 747796405u + 2891336453u;
    let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    *seed = (word >> 22u) ^ word;
    return *seed;
}

fn rand_f32(seed: ptr<function, u32>) -> f32 {
    return f32(pcg_hash(seed)) / 4294967296.0;
}

fn fresnel_schlick(cos_theta: f32, F0: vec3<f32>) -> vec3<f32> {
    return F0 + (vec3<f32>(1.0) - F0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

fn ggx_smith_g1(N_dot_V: f32, k: f32) -> f32 {
    return N_dot_V / (N_dot_V * (1.0 - k) + k);
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
};

struct Hit {
    is_hit: bool,
    distance: f32,
    material_index: u32,
    normal: vec3<f32>,
    position: vec3<f32>,
};

fn intersect_sphere(ray: Ray, sphere: Sphere) -> Hit {
    var hit: Hit;
    hit.is_hit = false;

    let oc = ray.origin - sphere.position;
    let half_b = dot(oc, ray.direction);
    let c = dot(oc, oc) - sphere.radius * sphere.radius;
    let discriminant = half_b * half_b - c;

    if (discriminant >= 0.0) {
        let sqrtd = sqrt(discriminant);
        var root = -half_b - sqrtd;

        if (root < 0.001) {
            root = -half_b + sqrtd;
        }

        if (root > 0.001) {
            hit.is_hit = true;
            hit.distance = root;
            hit.material_index = sphere.material_index;
            hit.position = ray.origin + ray.direction * root;
            hit.normal = normalize(hit.position - sphere.position);
        }
    }
    return hit;
}

fn intersect_triangle(ray: Ray, tri: Triangle) -> Hit {
    var hit: Hit;
    hit.is_hit = false;

    let edge1 = tri.v1 - tri.v0;
    let edge2 = tri.v2 - tri.v0;
    let h = cross(ray.direction, edge2);
    let a = dot(edge1, h);

    if (a > -1e-6 && a < 1e-6) {
        return hit;
    }

    let f = 1.0 / a;
    let s = ray.origin - tri.v0;
    let u = f * dot(s, h);

    if (u < 0.0 || u > 1.0) {
        return hit;
    }

    let q = cross(s, edge1);
    let v = f * dot(ray.direction, q);

    if (v < 0.0 || u + v > 1.0) {
        return hit;
    }

    let t = f * dot(edge2, q);

    if (t > 0.001) {
        let w = 1.0 - u - v;
        let n0 = vec3<f32>(tri.n0_x, tri.n0_y, tri.n0_z);

        hit.is_hit = true;
        hit.distance = t;
        hit.material_index = tri.material_index;
        hit.position = ray.origin + ray.direction * t;

        let interpolated_normal = normalize(w * n0 + u * tri.n1 + v * tri.n2);

        if (dot(ray.direction, interpolated_normal) > 0.0) {
            hit.normal = -interpolated_normal;
        } else {
            hit.normal = interpolated_normal;
        }
    }

    return hit;
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let coords = vec2<i32>(global_id.xy);

    if (coords.x >= i32(globals.resolution.x) || coords.y >= i32(globals.resolution.y)) {
        return;
    }

    let pixel_idx = global_id.y * globals.resolution.x + global_id.x;

    if (globals.frame_count == 1u) {
        accum_buf[pixel_idx] = vec4<f32>(0.0);
    }

    var seed = global_id.y * globals.resolution.x + global_id.x + globals.frame_count * 719393u;
    var total_color = vec3<f32>(0.0);

    for (var ray_idx = 0u; ray_idx < globals.rays_per_pixel; ray_idx = ray_idx + 1u) {
        var jitter_x = 0.5;
        var jitter_y = 0.5;

        if (globals.use_msaa == 1u) {
            jitter_x = rand_f32(&seed);
            jitter_y = rand_f32(&seed);
        }

        let uv = (vec2<f32>(global_id.xy) + vec2<f32>(jitter_x, jitter_y)) / vec2<f32>(globals.resolution);
        let ndc = uv * 2.0 - 1.0;
        let aspect_ratio = f32(globals.resolution.x) / f32(globals.resolution.y);

        let tan_half_fov = tan(radians(camera.fov) * 0.5);
        let viewport_x = ndc.x * aspect_ratio * tan_half_fov;
        let viewport_y = -ndc.y * tan_half_fov;

        let pinhole_dir = normalize(camera.forward + camera.right * viewport_x + camera.up * viewport_y);
        let focal_point = camera.position + pinhole_dir * camera.focus_dist;

        let lens_radius = camera.aperture;
        let r_lens = sqrt(rand_f32(&seed)) * lens_radius;
        let theta_lens = rand_f32(&seed) * 6.28318530718;
        let lens_offset = camera.right * (r_lens * cos(theta_lens)) + camera.up * (r_lens * sin(theta_lens));

        var current_ray: Ray;
        current_ray.origin = camera.position + lens_offset;
        current_ray.direction = normalize(focal_point - current_ray.origin);

        var pixel_color = vec3<f32>(0.0);
        var throughput = vec3<f32>(1.0);

        for (var bounce = 0u; bounce < globals.max_bounces; bounce = bounce + 1u) {
            var closest_hit: Hit;
            closest_hit.is_hit = false;
            closest_hit.distance = 999999.0;

            let num_spheres = arrayLength(&spheres);
            for (var i = 0u; i < num_spheres; i = i + 1u) {
                let hit = intersect_sphere(current_ray, spheres[i]);
                if (hit.is_hit && hit.distance < closest_hit.distance) {
                    closest_hit = hit;
                }
            }

            let num_triangles = arrayLength(&triangles);
            for (var i = 0u; i < num_triangles; i = i + 1u) {
                let hit = intersect_triangle(current_ray, triangles[i]);
                if (hit.is_hit && hit.distance < closest_hit.distance) {
                    closest_hit = hit;
                }
            }

            if (!closest_hit.is_hit) {
                pixel_color = pixel_color + throughput * vec3<f32>(0.05);
                break;
            }

            let mtl = materials[closest_hit.material_index];

            if (mtl.light_power > 0.0) {
                pixel_color = pixel_color + throughput * mtl.diffuse * mtl.light_power;
                break;
            }

            let r1 = rand_f32(&seed);
            let r2 = rand_f32(&seed);

            let roughness = clamp(1.0 - mtl.glossiness, 0.01, 1.0);
            let alpha = roughness * roughness;
            let k = alpha / 2.0;

            let v = -current_ray.direction;
            let n_dot_v = max(dot(closest_hit.normal, v), 0.0);
            let F0 = mix(vec3<f32>(0.04), mtl.diffuse, mtl.specular);
            let F = fresnel_schlick(n_dot_v, F0);

            let spec_prob = clamp((F.x + F.y + F.z) / 3.0, 0.1, 0.9);

            var next_dir: vec3<f32>;

            if (rand_f32(&seed) < spec_prob) {
                let phi = 2.0 * 3.1415926535 * r1;
                let cos_theta = sqrt((1.0 - r2) / (1.0 + (alpha * alpha - 1.0) * r2));
                let sin_theta = sqrt(1.0 - cos_theta * cos_theta);

                var tangent = vec3<f32>(1.0, 0.0, 0.0);
                if (abs(closest_hit.normal.x) > 0.999) {
                    tangent = vec3<f32>(0.0, 1.0, 0.0);
                }
                tangent = normalize(cross(tangent, closest_hit.normal));
                let bitangent = cross(closest_hit.normal, tangent);

                let H = normalize(tangent * (sin_theta * cos(phi)) + bitangent * (sin_theta * sin(phi)) + closest_hit.normal * cos_theta);

                next_dir = reflect(current_ray.direction, H);
                let n_dot_l = max(dot(closest_hit.normal, next_dir), 0.0);

                if (n_dot_l > 0.0) {
                    let v_dot_h = max(dot(v, H), 0.0);
                    let n_dot_h = max(dot(closest_hit.normal, H), 0.0);
                    let G = ggx_smith_g1(n_dot_l, k) * ggx_smith_g1(n_dot_v, k);
                    let weight = (F * G * v_dot_h) / max(n_dot_h * n_dot_v, 0.001);
                    throughput = throughput * (weight / spec_prob);
                } else {
                    break;
                }
            } else {
                let z = sqrt(1.0 - r2);
                let phi = 2.0 * 3.1415926535 * r1;
                let x = cos(phi) * sqrt(r2);
                let y = sin(phi) * sqrt(r2);

                var tangent = vec3<f32>(1.0, 0.0, 0.0);
                if (abs(closest_hit.normal.x) > 0.999) {
                    tangent = vec3<f32>(0.0, 1.0, 0.0);
                }
                tangent = normalize(cross(tangent, closest_hit.normal));
                let bitangent = cross(closest_hit.normal, tangent);

                next_dir = normalize(tangent * x + bitangent * y + closest_hit.normal * z);

                let diffuse_color = mtl.diffuse * (1.0 - mtl.specular);
                throughput = throughput * (diffuse_color / (1.0 - spec_prob));
            }

            current_ray.origin = closest_hit.position + closest_hit.normal * 0.001;
            current_ray.direction = next_dir;
        }
        total_color = total_color + pixel_color;
    }

    let final_pixel_color = (total_color / f32(max(1u, globals.rays_per_pixel))) * globals.global_light_intensity;

    let prev = accum_buf[pixel_idx].rgb;
    let accumulated = prev + final_pixel_color;
    accum_buf[pixel_idx] = vec4<f32>(accumulated, 1.0);

    let averaged = accumulated / f32(globals.frame_count);
    textureStore(output_tex, coords, vec4<f32>(averaged, 1.0));
}
