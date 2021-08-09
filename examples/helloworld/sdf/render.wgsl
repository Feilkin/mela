let EPSILON: f32 = 0.5;
let light_pos: vec3<f32> = vec3<f32>(-70., 100., 13.);

[[group(0), binding(0)]]
var world_data: texture_3d<f32>;
[[group(0), binding(1)]]
var data_sampler: sampler;

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec4<f32>,
) -> [[builtin(position)]] vec4<f32> {
    return position;
}

fn radians(degree: f32) -> f32 {
    return degree * 3.14159265359 / 180.;
}

// stolen from https://www.shadertoy.com/view/4tcGDr
// ray direction in view space (Y up)
fn ray_direction(fieldOfView: f32, size: vec2<f32>, fragCoord: vec2<f32>) -> vec3<f32> {
    let xy = fragCoord - size / 2.0;
    let z = size.y / tan(radians(fieldOfView) / 2.0);
    return normalize(vec3<f32>(xy.x, -xy.y, -z));
}

fn viewMatrix(eye: vec3<f32>, center: vec3<f32>, up: vec3<f32>) -> mat3x3<f32> {
    // Based on gluLookAt man page
    let f = normalize(center - eye);
    let s = normalize(cross(f, up));
    let u = cross(s, f);
    return mat3x3<f32>(s, u, -f);
}

fn world_to_uv(pos: vec3<f32>) -> vec3<f32> {
    return
        clamp(
        vec3<f32>(0., 0., 0.),
        pos.xzy,
        vec3<f32>(255., 255., 63.));
}

fn sceneSDF(p: vec3<f32>) -> f32 {
    return textureSample(world_data, data_sampler, world_to_uv(p) / vec3<f32>(256., 256., 64.)).r;
}

fn estimateNormal(p: vec3<f32>) -> vec3<f32> {
    return normalize(vec3<f32>(
        sceneSDF(vec3<f32>(p.x + EPSILON, p.y, p.z)) - sceneSDF(vec3<f32>(p.x - EPSILON, p.y, p.z)),
        sceneSDF(vec3<f32>(p.x, p.y + EPSILON, p.z)) - sceneSDF(vec3<f32>(p.x, p.y - EPSILON, p.z)),
        sceneSDF(vec3<f32>(p.x, p.y, p.z  + EPSILON)) - sceneSDF(vec3<f32>(p.x, p.y, p.z - EPSILON))
    ));
}

[[stage(fragment)]]
fn fs_main([[builtin(position)]] in: vec4<f32>) -> [[location(0)]] vec4<f32> {
    let eye_pos = vec3<f32>(-30., 43., -128.);
    let ray_dir_in_view = ray_direction(45., vec2<f32>(1920., 1080.), in.xy);
    let view_to_world = viewMatrix(eye_pos, vec3<f32>(128., 32., 128.), vec3<f32>(0., 1., 0.));
    let ray_dir = view_to_world * ray_dir_in_view;
    //let ray_dir = ray_dir_in_view;

    var ray: vec3<f32> = eye_pos;

    var depth: i32 = 0;
    let max_depth = 32;

    var total_distance: f32 = 0.;
    var dist: f32 = 0.;

    loop {
        if (depth >= max_depth) { break; }
        //dist = textureLoad(world_data, vec3<i32>(world_to_uv(ray)), 0).r;
        dist = sceneSDF(ray);


        ray = ray + dist * ray_dir;

        if (dist <= EPSILON) {
            break;
        }
        total_distance = total_distance + dist;

        depth = depth + 1;
    }

    var fragColor: vec4<f32> = vec4<f32>(0.1, 0.5, 0.1, 1.0);
    if (depth != max_depth)  {
        let normal = estimateNormal(ray);

        // Output to screen
        fragColor = vec4<f32>(ray.xy / 256., 1. - total_distance / 512., 1.0) * clamp(0.2, dot(light_pos, normal) * 0.6 + 0.4, 1.0);
    }

    return fragColor;
}
