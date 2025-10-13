struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_position: vec3<f32>,
};

@vertex
fn vs_main(
    vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.local_position = vertex.position;
    out.clip_position = vec4<f32>(vertex.position, 1.0);
    return out;
}

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

@group(0) @binding(0)
var<uniform> game_info: GameInfo;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.local_position.xy * 0.5 + 0.5;

    let board_size = vec2<f32>(0.5, 0.5);
    let board_start = vec2<f32>(0.5, 0.5) - board_size * 0.5;
    let board_end = board_start + board_size;

    if all(uv >= board_start) && all(uv <= board_end) {
        let local = (uv - board_start) / board_size;

        let squares = 8.0;
        let x = i32(floor(local.x * squares));
        let y = i32(floor(local.y * squares));
        let square = x + y;

        var color = vec4<f32>(0.40, 0.40, 0.40, 1.0);

        if square % 2 == 0 {
            color = vec4<f32>(0.93, 0.93, 0.93, 1.0);
        }

        if game_info.legal_moves[y * 8 + x].value == 1u {
            return mix(color, vec4<f32>(0.20, 0.80, 0.20, 1.0), 0.7);
        }

        return color;
    }

    return vec4<f32>(0.5, 0.5, 0.5, 1.0);
}