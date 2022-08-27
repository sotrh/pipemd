struct VSIn {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

struct VSOut {
    @location(0) uv: vec2<f32>,
    @builtin(position) clip_pos: vec4<f32>,
}

@group(0)
@binding(0)
var tex: texture_2d<f32>;
@group(0)
@binding(1)
var samp: sampler;

@vertex
fn vs_textured(in: VSIn) -> VSOut {
    let clip_pos = vec4(in.position, 0.0, 1.0);
    return VSOut(in.uv, clip_pos);
}

@fragment
fn fs_textured(in: VSIn) -> @location(0) vec4<f32> {
    return textureSample(tex, samp, in.uv);
}