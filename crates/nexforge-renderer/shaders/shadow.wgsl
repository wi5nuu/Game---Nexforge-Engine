// Shadow Map Shader — Cascaded Shadow Maps
// Will be fully implemented in Phase 3

struct VSInput {
    @location(0) position: vec3<f32>,
};

struct VSOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) depth: f32,
};

@vertex
fn vs_main(input: VSInput) -> VSOutput {
    var output: VSOutput;
    output.position = vec4<f32>(input.position, 1.0);
    output.depth = input.position.z;
    return output;
}

@fragment
fn fs_main(input: VSOutput) -> @builtin(frag_depth) f32 {
    return input.depth;
}
