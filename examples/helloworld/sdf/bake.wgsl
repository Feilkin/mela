struct BrushInstance {
    transform: mat4x4<f32>;
    shape_data: vec4<f32>;
};

[[block]]
struct BrushInstances {
    count: u32;
    instances: array<BrushInstance, 64>;
};

// SDF functions
fn ball(point: vec4<f32>, offset: vec4<f32>, radius: f32) -> f32 {
    return length(point - offset) - radius;
}

fn smin(a: f32, b: f32, k: f32) -> f32 {
    let h = max(k - abs(a - b), 0.) / k;
    return min(a, b) - h*h*h*k*1./6.0;
}

[[group(0), binding(0)]]
var<uniform> brush_instances: BrushInstances;
[[group(0), binding(1)]]
var world_data: [[access(write)]] texture_storage_3d<r32float>;

[[stage(compute), workgroup_size(1)]]
fn bake([[builtin(global_invocation_id)]] global_id: vec3<u32>) {
    let grid_in_world = vec4<f32>(vec4<u32>(global_id.x, global_id.y, global_id.z, 1u)) + vec4<f32>(0.5, 0.5, 0.5, 1.0);

    var sdf: f32 = 1000.;

    for (var i: u32 = 0u; i < brush_instances.count; i = i + 1u) {
        // TODO: different shapes
        let brush = brush_instances.instances[i];
        let brush_distance = ball(grid_in_world,
                       brush.transform * vec4<f32>(0., 0., 0., 1.),
                       brush.shape_data[1]);

        sdf = smin(sdf, brush_distance, 25.0);

    }

    textureStore(world_data, vec3<i32>(global_id), vec4<f32>(sdf, 0., 0., 1.));
}