// Cascaded Shadow Map Shader
struct CascadeUniform {
    cascade_view_proj: array<mat4x4<f32>, 4>,
    cascade_splits: array<f32, 4>,
    cascade_count: u32,
};

@group(0) @binding(0) var<uniform> uniforms: CascadeUniform;

struct VSOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) depth: f32,
};

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) instance_id: u32,
) -> VSOutput {
    var out: VSOutput;
    let cascade_idx = min(instance_id, uniforms.cascade_count - 1u);
    out.position = uniforms.cascade_view_proj[cascade_idx] * vec4<f32>(position, 1.0);
    out.depth = out.position.z;
    return out;
}

@fragment
fn fs_main(input: VSOutput) -> @builtin(frag_depth) f32 {
    return input.depth;
}
