@compute @workgroup_size(8, 8, 1)
fn julia(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {}
