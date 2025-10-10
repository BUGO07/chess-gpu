struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) vertex_position: vec3<f32>,
};

fn get_uv(index: u32) -> vec2<f32> {
    let tiles_x: f32 = 6.0;
    let tiles_y: f32 = 2.0;

    let tile_size = vec2<f32>(1.0 / tiles_x, 1.0 / tiles_y);

    let x = f32(index % u32(tiles_x));
    let y = f32(index / u32(tiles_x));

    let uv = (vec2<f32>(x, y) + 0.5) * tile_size;
    return uv;
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.vertex_position = model.position;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.vertex_position.xy * 0.5 + 0.5;

    let board_size = vec2<f32>(0.5, 0.5);
    let board_start = vec2<f32>(0.5, 0.5) - board_size * 0.5;
    let board_end = board_start + board_size;

    if all(uv >= board_start) && all(uv <= board_end) {
        let local = (uv - board_start) / board_size;

        let squares = 8.0;
        let x = i32(floor(local.x * squares));
        let y = i32(floor(local.y * squares));
        let checker = x + y;

        if checker % 2 == 0 {
            return vec4<f32>(1.0, 1.0, 1.0, 1.0);
        }

        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    return vec4<f32>(0.5, 0.5, 0.5, 1.0);
}
