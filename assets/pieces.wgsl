struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct InstanceInput {
    @location(1) position: u32,
    @location(2) piece: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) idx: u32,
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

    return out;
}


@group(0) @binding(0)
var pieces_texture: texture_2d<f32>;
@group(0) @binding(1)
var pieces_sampler: sampler;

struct GameInfo {
    hovered: u32,
    selected: u32,
    _pad: vec2<u32>,
};

@group(1) @binding(0)
var<uniform> game_info: GameInfo;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture = textureSample(pieces_texture, pieces_sampler, in.uv);
    var color = in.local_position;
    if game_info.hovered == in.idx {
        color = vec3<f32>(1.0, 1.0, 0.0);
    }
    return vec4<f32>(mix(texture.rgb, color, 0.5), texture.a);
}