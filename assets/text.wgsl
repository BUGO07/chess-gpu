struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) position: vec3<f32>,
};

struct InstanceInput {
    @builtin(instance_index) instance_index: u32,
    @location(1) position: vec3<f32>,
    @location(2) idx: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) idx: u32,
};

const font_size: vec2<f32> = vec2<f32>(6.0, 10.0);

const offsets: array<vec3<f32>, 4> = array(
    vec3(1.0, 0.0, 0.0),
    vec3(1.0, 1.0, 0.0),
    vec3(0.0, 1.0, 0.0),
    vec3(0.0, 0.0, 0.0)
);

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
    out.idx = instance.instance_index;

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

fn hsl_to_rgb(hsl: vec3<f32>) -> vec3<f32> {
    let h = hsl.x;
    let s = hsl.y;
    let l = hsl.z;

    let c = (1.0 - abs(2.0 * l - 1.0)) * s;
    let x = c * (1.0 - abs((h * 6.0) % 2.0 - 1.0));
    let m = l - 0.5 * c;

    var rgb = vec3<f32>(0.0);
    if h < 1.0 / 6.0 { rgb = vec3<f32>(c, x, 0.0); } else if h < 2.0 / 6.0 { rgb = vec3<f32>(x, c, 0.0); } else if h < 3.0 / 6.0 { rgb = vec3<f32>(0.0, c, x); } else if h < 4.0 / 6.0 { rgb = vec3<f32>(0.0, x, c); } else if h < 5.0 / 6.0 { rgb = vec3<f32>(x, 0.0, c); } else { rgb = vec3<f32>(c, 0.0, x); }

    return rgb + vec3<f32>(m);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var rgb = hsl_to_rgb(vec3<f32>(sin(game_info.time + length(in.local_position.xy)) * 0.5 + 0.5, 1.0, 0.5));

    if white_to_play() == in.idx / 5u { // tacky solution
        rgb = vec3<f32>(0.0, 0.0, 0.0);
    }

    if game_over() {
        rgb = vec3<f32>(0.0, 1.0, 0.0);
    }

    return textureSample(text_texture, text_sampler, in.uv) * vec4<f32>(rgb, 1.0);
}

fn white_to_play() -> u32 {
    return (game_info.state & 1u);
}

fn game_over() -> bool {
    return ((game_info.state >> 15u) & 0x3u) != 0;
}