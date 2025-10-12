struct VertexInput {
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
    out.uv = get_uv(instance.idx, vertex.position.xy * 10.0);

    return out;
}


@group(0) @binding(0)
var pieces_texture: texture_2d<f32>;
@group(0) @binding(1)
var pieces_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture = textureSample(pieces_texture, pieces_sampler, in.uv);
    return vec4<f32>(mix(texture.rgb, in.local_position, 0.5), texture.a);
}