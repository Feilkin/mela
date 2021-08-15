struct BrushInstance {
    translation: vec3<f32>;
    shape: u32;
    shape_data: vec4<f32>;
};

[[block]]
struct BrushInstances {
    count: u32;
    instances: array<BrushInstance, 500>;
};

// SDF functions
fn ball(point: vec3<f32>, offset: vec3<f32>, radius: f32) -> f32 {
    return length(point - offset) - radius;
}

fn box(p: vec3<f32>, b: vec3<f32>) -> f32
{
  let q = abs(p) - b;
  return length(max(q,vec3<f32>(0.0, 0.0, 0.0))) + min(max(q.x,max(q.y,q.z)),0.0);
}

fn smin(a: f32, b: f32, k: f32) -> f32 {
    let h = max(k - abs(a - b), 0.) / k;
    return min(a, b) - h*h*h*k*1./6.0;
}

[[group(0), binding(0)]]
var<uniform> brush_instances: BrushInstances;
[[group(0), binding(1)]]
var world_data: [[access(write)]] texture_storage_3d<r32float>;

[[stage(compute), workgroup_size(8, 8, 16)]]
fn bake([[builtin(global_invocation_id)]] global_id: vec3<u32>) {
    let grid_in_world = vec3<f32>(vec3<u32>(global_id.x, global_id.y, global_id.z)) - vec3<f32>(128.5, 128.5, 0.5);
    let instance_count = brush_instances.count;

    var sdf: f32 = 1000.;
    var i: u32 = 0u;
    loop {
        var brush_distance: f32 = 0.0;
        switch (i32(brush_instances.instances[i].shape)) {
            case 1: {
                brush_distance = ball(grid_in_world,
                                      brush_instances.instances[i].translation,
                                      brush_instances.instances[i].shape_data[0]);
            }
            case 2: {
                brush_distance = box(grid_in_world - brush_instances.instances[i].translation,
                                     vec3<f32>(brush_instances.instances[i].shape_data[0],
                                               brush_instances.instances[i].shape_data[1],
                                               brush_instances.instances[i].shape_data[2]
                                     ));
            }
            default: {}
        }

        if (brush_instances.instances[i].shape_data[3] > 0.) {
            sdf = smin(sdf, brush_distance, brush_instances.instances[i].shape_data[3]);
        } else {
            sdf = min(sdf, brush_distance);
        }

        i = i + 1u;
        if (i >= instance_count) { break; }
    }

    if (global_id.z == 127u) { sdf = 4.; }
    if (global_id.z ==   0u) { sdf = 4.; }
    if (global_id.y == 255u) { sdf = 4.; }
    if (global_id.y ==   0u) { sdf = 4.; }
    if (global_id.z == 255u) { sdf = 4.; }
    if (global_id.z ==   0u) { sdf = 4.; }

    textureStore(world_data, vec3<i32>(global_id), vec4<f32>(sdf, 0., 0., 1.));
}