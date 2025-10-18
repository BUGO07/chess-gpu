struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct InstanceInput {
    @location(1) position: vec3<f32>,
    @location(2) data: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) idx: u32,
    @location(3) white: u32,
};

const tile_size: vec2<f32> = vec2<f32>(6.0, 2.0);

fn get_uv(index: u32, position: vec2<f32>) -> vec2<f32> {
    let x = f32(index % u32(tile_size.x));
    let y = f32(index / u32(tile_size.x));

    return (vec2<f32>(position.x + x, -position.y + y + 1.0)) / tile_size;
}

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = vec4<f32>(vertex.position + instance.position, 1.0);
    out.local_position = instance.position;
    out.uv = get_uv(instance_piece(instance.data), vertex.position.xy * 8.2);
    out.idx = instance_index(instance.data) + 1u;
    out.white = instance_white(instance.data);

    return out;
}


@group(0) @binding(0)
var pieces_texture: texture_2d<f32>;
@group(0) @binding(1)
var pieces_sampler: sampler;

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
    let texture = textureSample(pieces_texture, pieces_sampler, in.uv);
    if is_selected(in.idx) && is_hovered(in.idx) {
        if length(texture - textureSample(pieces_texture, pieces_sampler, in.uv - 0.002)) > 0.1 {
            return vec4<f32>(mix(texture.rgb, vec3<f32>(0.0, 0.0, 0.0), 0.5), texture.a);
        } else {
            return texture;
        }
    } else if is_selected(in.idx) {
        return vec4<f32>(mix(texture.rgb, vec3<f32>(0.0, 0.0, 1.0), 0.5), texture.a);
    } else if is_hovered(in.idx) {
        if length(texture.rgb - textureSample(pieces_texture, pieces_sampler, in.uv - 0.002).rgb) > 0.1 {
            if in.white == 1u {
                return vec4<f32>(1.0, 1.0, 1.0, 1.0);
            } else {
                return vec4<f32>(0.0, 0.0, 0.0, 1.0);
            }
        } else {
            return vec4<f32>((1.0 - texture.rgb) * 0.7, texture.a);
        }
    }
    let rgb = hsl_to_rgb(vec3<f32>(sin(game_info.time + length(in.local_position.xy)) * 0.5 + 0.5, 1.0, 0.5));
    if in.white == white_to_play() {
        return vec4<f32>(mix(texture.rgb, rgb, 0.5), texture.a);
    } else {
        return texture;
    }
}

fn is_selected(idx: u32) -> bool {
    return ((game_info.state >> 8u) & 0x7Fu) == idx;
}

fn is_hovered(idx: u32) -> bool {
    return ((game_info.state >> 1u) & 0x7Fu) == idx;
}

fn white_to_play() -> u32 {
    return game_info.state & 1u;
}

fn instance_white(data: u32) -> u32 {
    return data & 1u;
}

fn instance_piece(data: u32) -> u32 {
    return (data >> 1u) & 0xFu;
}

fn instance_index(data: u32) -> u32 {
    return (data >> 5u) & 0x3Fu;
}