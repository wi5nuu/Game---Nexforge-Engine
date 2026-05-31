// Nexforge PBR Shader — Deferred geometry pass
struct Uniforms {
    model: mat4x4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    light_count: u32,
    _pad: u32,
};

struct Light {
    position: vec4<f32>,
    color: vec4<f32>,
    intensity: f32,
    radius: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<uniform> lights: array<Light, 64>;

struct VSOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
) -> VSOutput {
    var out: VSOutput;
    out.position = uniforms.proj * uniforms.view * uniforms.model * vec4<f32>(position, 1.0);
    out.world_pos = (uniforms.model * vec4<f32>(position, 1.0)).xyz;
    out.normal = normalize((uniforms.model * vec4<f32>(normal, 0.0)).xyz);
    out.uv = uv;
    return out;
}

struct GBuffer {
    albedo: @location(0) vec4<f32>,
    normal: @location(1) vec4<f32>,
    pbr: @location(2) vec4<f32>,
};

@fragment
fn fs_main(input: VSOutput) -> GBuffer {
    var gb: GBuffer;
    gb.albedo = vec4<f32>(0.8, 0.2, 0.2, 1.0); // Placeholder albedo
    gb.normal = vec4<f32>(normalize(input.normal), 1.0);
    gb.pbr = vec4<f32>(0.0, 0.5, 1.0, 0.0); // metallic=0, roughness=0.5, ao=1.0
    return gb;
}
