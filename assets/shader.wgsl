struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) vertex_position: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

const tile_size: vec2<f32> = vec2<f32>(6.0, 2.0);

fn get_uv(index: u32, position: vec2<f32>) -> vec2<f32> {
    let x = f32(index % u32(tile_size.x));
    let y = f32(index / u32(tile_size.x));

    return (vec2<f32>(position.x + x, -position.y + y + 1.0)) / tile_size;
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.vertex_position = model.position;
    out.clip_position = vec4<f32>(model.position, 1.0);

    out.uv = get_uv(0u, model.position.xy * 0.5 + 0.5);

    return out;
}


@group(0) @binding(0)
var pieces_texture: texture_2d<f32>;
@group(0) @binding(1)
var pieces_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(pieces_texture, pieces_sampler, in.uv);
}