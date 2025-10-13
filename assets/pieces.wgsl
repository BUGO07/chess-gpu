struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct InstanceInput {
    @location(1) position: u32,
    @location(2) piece: u32,
    @location(3) white: u32,
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

    let instance_position = vec3<f32>(
        (f32(i32(instance.position) % 8 - 4) + 0.1) * 0.125,
        (f32(i32(instance.position) / 8 - 4) + 0.1) * 0.125,
        0.0,
    );
    out.clip_position = vec4<f32>(vertex.position + instance_position, 1.0);
    out.local_position = instance_position;
    out.uv = get_uv(instance.piece, vertex.position.xy * 10.0);
    let x = i32(instance_position.x * 8.0 - 0.1);
    let y = i32(instance_position.y * 8.0 - 0.1);
    out.idx = u32((y + 4) * 8 + (x + 5));
    out.white = instance.white;

    return out;
}


@group(0) @binding(0)
var pieces_texture: texture_2d<f32>;
@group(0) @binding(1)
var pieces_sampler: sampler;

struct U32Aligned {
    @align(16)
    value: u32,
}

struct GameInfo {
    hovered: u32,
    selected: u32,
    time: f32,
    white_to_play: u32,
    legal_moves: array<U32Aligned, 64>,
};

@group(1) @binding(0)
var<uniform> game_info: GameInfo;

fn hsl_to_rgb(hsl: vec3<f32>) -> vec3<f32> {
    let h = hsl.x; // normalize hue to 0-1
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
    if game_info.selected == in.idx && game_info.hovered == in.idx {
        return vec4<f32>(mix(texture.rgb, vec3<f32>(0.0, 0.4, 1.0), 0.5), texture.a);
    } else if game_info.selected == in.idx {
        return vec4<f32>(mix(texture.rgb, vec3<f32>(0.0, 0.0, 1.0), 0.5), texture.a);
    } else if game_info.hovered == in.idx {
        return vec4<f32>(mix(texture.rgb, vec3<f32>(1.0, 1.0, 0.0), 0.5), texture.a);
    }
    let rgb = hsl_to_rgb(vec3<f32>(sin(game_info.time + length(in.local_position.xy)) * 0.5 + 0.5, 1.0, 0.5));
    if in.white == game_info.white_to_play {
        return vec4<f32>(mix(texture.rgb, rgb, 0.5), texture.a);
    } else {
        return texture;
    }
}