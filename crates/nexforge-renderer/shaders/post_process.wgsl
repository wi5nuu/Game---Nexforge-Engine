// Post-Processing Stack — Bloom, SSAO, Tone Mapping, Motion Blur
// Will be fully implemented in Phase 3

struct FSInput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@fragment
fn fs_main(input: FSInput) -> @location(0) vec4<f32> {
    // Placeholder: pass-through
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
