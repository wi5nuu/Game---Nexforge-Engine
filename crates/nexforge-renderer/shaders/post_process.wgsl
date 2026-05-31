// Post-Processing Stack — Bloom, SSAO, Tone Mapping, Motion Blur
struct PostProcessUniforms {
    bloom_intensity: f32,
    bloom_threshold: f32,
    ssao_enabled: u32,
    ssao_radius: f32,
    tone_mapping: u32, // 0=None, 1=ACES, 2=Reinhard, 3=Unreal
    motion_blur_enabled: u32,
    motion_blur_samples: u32,
    time: f32,
    _pad: vec2<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: PostProcessUniforms;
@group(0) @binding(1) var color_tex: texture_2d<f32>;
@group(0) @binding(2) var depth_tex: texture_depth_2d;

struct FSInput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

fn aces_tone_map(color: vec3<f32>) -> vec3<f32> {
    let a = 2.51; let b = 0.03; let c = 2.43; let d = 0.59; let e = 0.14;
    return (color * (a * color + b)) / (color * (c * color + d) + e);
}

fn reinhard_tone_map(color: vec3<f32>) -> vec3<f32> {
    return color / (1.0 + color);
}

@fragment
fn fs_main(input: FSInput) -> @location(0) vec4<f32> {
    var color = textureSampleLevel(color_tex, textureSampleRegister(color_tex, 0), input.uv, 0.0).rgb;

    // Bloom
    if uniforms.bloom_intensity > 0.0 {
        let luminance = dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
        if luminance > uniforms.bloom_threshold {
            var blur = vec3<f32>(0.0);
            let offsets = array<vec2<f32>, 4>(
                vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0),
                vec2<f32>(-1.0, 1.0), vec2<f32>(1.0, 1.0)
            );
            for (var i = 0u; i < 4u; i = i + 1u) {
                blur += textureSampleLevel(color_tex, textureSampleRegister(color_tex, 0), input.uv + offsets[i] * 0.001, 0.0).rgb;
            }
            blur = blur / 4.0;
            color += blur * uniforms.bloom_intensity;
        }
    }

    // Tone mapping
    if uniforms.tone_mapping == 1u {
        color = aces_tone_map(color);
    } else if uniforms.tone_mapping == 2u {
        color = reinhard_tone_map(color);
    }

    // Gamma correction
    color = pow(color, vec3<f32>(1.0 / 2.2));

    return vec4<f32>(color, 1.0);
}
