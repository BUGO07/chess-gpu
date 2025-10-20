struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) position: vec3<f32>,
};

struct InstanceInput {
    @location(1) position: vec3<f32>,
    @location(2) idx: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_position: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

const font_size: vec2<f32> = vec2<f32>(6.0, 10.0);

const offsets: array<vec3<f32>, 4> = array(vec3(1.0, 0.0, 0.0),
    vec3(1.0, 1.0, 0.0),
    vec3(0.0, 1.0, 0.0),
    vec3(0.0, 0.0, 0.0));

fn get_uv(index: u32, position: vec2<f32>) -> vec2<f32> {
    let cr = vec2<f32>(f32(index % 13u), f32(index / 13u));

    return (font_size * (cr + position)) / vec2(78.0, 70.0);
}

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    let local_uv = offsets[vertex.vertex_index % 4u];
    out.clip_position = vec4<f32>(instance.position + vertex.position, 1.0);
    out.local_position = instance.position;
    out.uv = get_uv(instance.idx, vec2<f32>(local_uv.x, 1.0 - local_uv.y));

    return out;
}

@group(0) @binding(0)
var text_texture: texture_2d<f32>;
@group(0) @binding(1)
var text_sampler: sampler;

struct GameInfo {
    time: f32,
    state: u32,
    legal_moves_low: u32,
    legal_moves_high: u32,
};

@group(1) @binding(0)
var<uniform> game_info: GameInfo;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(text_texture, text_sampler, in.uv) * vec4<f32>(1.0, 0.0, 0.0, 1.0);
}