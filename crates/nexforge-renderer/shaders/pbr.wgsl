// PBR Shader — Deferred geometry pass
// Will be fully implemented in Phase 3

struct VSInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tangent: vec3<f32>,
};

struct VSOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

@vertex
fn vs_main(input: VSInput) -> VSOutput {
    var output: VSOutput;
    output.position = vec4<f32>(input.position, 1.0);
    output.world_pos = input.position;
    output.normal = input.normal;
    output.uv = input.uv;
    return output;
}

struct GBuffer {
    albedo: vec4<f32>,
    normal: vec4<f32>,
    pbr: vec4<f32>,  // x: metallic, y: roughness, z: ao, w: emissive
    depth: f32,
};

@fragment
fn fs_main(input: VSOutput) -> GBuffer {
    var gbuffer: GBuffer;
    gbuffer.albedo = vec4<f32>(0.5, 0.5, 0.5, 1.0);
    gbuffer.normal = vec4<f32>(normalize(input.normal), 0.0);
    gbuffer.pbr = vec4<f32>(0.0, 0.5, 1.0, 0.0);
    gbuffer.depth = input.position.z;
    return gbuffer;
}
