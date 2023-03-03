@group(0) @binding(0)
var texture: texture_storage_2d<rgba8unorm, read_write>;

@compute @workgroup_size(8, 8, 1)
fn julia(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let resolution = vec2<i32>(i32(num_workgroups.x) * 8, i32(num_workgroups.y) * 8);
    
    let uv = vec2<f32>(location) / vec2<f32>(resolution);
    let color = vec4<f32>(uv, 1.0, 1.0);

    textureStore(texture, location, color);
}
