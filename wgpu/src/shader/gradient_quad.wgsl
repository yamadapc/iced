[[block]]
struct Globals {
    transform: mat4x4<f32>;
    scale: f32;
};

[[group(0), binding(0)]] var<uniform> globals: Globals;

struct VertexInput {
    [[location(0)]] v_pos: vec2<f32>;
    [[location(1)]] pos: vec2<f32>;
    [[location(2)]] scale: vec2<f32>;
    [[location(3)]] top_left_color: vec4<f32>;
    [[location(4)]] top_right_color: vec4<f32>;
    [[location(5)]] bottom_left_color: vec4<f32>;
    [[location(6)]] bottom_right_color: vec4<f32>;
    [[location(7)]] border_radius: f32;
};

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] pos: vec2<f32>;
    [[location(1)]] scale: vec2<f32>;
    [[location(2)]] top_left_color: vec4<f32>;
    [[location(3)]] top_right_color: vec4<f32>;
    [[location(4)]] bottom_left_color: vec4<f32>;
    [[location(5)]] bottom_right_color: vec4<f32>;
    [[location(6)]] border_radius: f32;
};

[[stage(vertex)]]
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    var pos: vec2<f32> = input.pos * globals.scale;
    var scale: vec2<f32> = input.scale * globals.scale;

    var border_radius: f32 = min(
        input.border_radius,
        min(input.scale.x, input.scale.y) / 2.0
    );

    var transform: mat4x4<f32> = mat4x4<f32>(
        vec4<f32>(scale.x + 1.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, scale.y + 1.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(pos - vec2<f32>(0.5, 0.5), 0.0, 1.0)
    );

    out.top_left_color = input.top_left_color;
    out.top_right_color = input.top_right_color;
    out.bottom_left_color = input.bottom_left_color;
    out.bottom_right_color = input.bottom_right_color;
    out.pos = pos;
    out.scale = scale;
    out.border_radius = border_radius * globals.scale;
    out.position = globals.transform * transform * vec4<f32>(input.v_pos, 0.0, 1.0);

    return out;
}

fn distance_alg(
    frag_coord: vec2<f32>,
    position: vec2<f32>,
    size: vec2<f32>,
    radius: f32
) -> f32 {
    var inner_size: vec2<f32> = size - vec2<f32>(radius, radius) * 2.0;
    var top_left: vec2<f32> = position + vec2<f32>(radius, radius);
    var bottom_right: vec2<f32> = top_left + inner_size;

    var top_left_distance: vec2<f32> = top_left - frag_coord;
    var bottom_right_distance: vec2<f32> = frag_coord - bottom_right;

    var dist: vec2<f32> = vec2<f32>(
        max(max(top_left_distance.x, bottom_right_distance.x), 0.0),
        max(max(top_left_distance.y, bottom_right_distance.y), 0.0)
    );

    return sqrt(dist.x * dist.x + dist.y * dist.y);
}


[[stage(fragment)]]
fn fs_main(
    input: VertexOutput
) -> [[location(0)]] vec4<f32> {
    var pixel_position: vec2<f32> = vec2<f32>(input.position.x, input.position.y);
    var top_left_position: vec2<f32> = vec2<f32>(input.pos.x, input.pos.y);
    var width: f32 = input.scale.x;
    var height: f32 = input.scale.y;

    var x: f32 = pixel_position.x - top_left_position.x;
    var y: f32 = pixel_position.y - top_left_position.y;

    var perc_x: f32 = x / width;
    var perc_y: f32 = y / height;

    var mixed_color: vec4<f32> = mix(
        mix(
            input.top_left_color,
            input.top_right_color,
            vec4<f32>(perc_x, perc_x, perc_x, perc_x)
        ),
        mix(
            input.bottom_left_color,
            input.bottom_right_color,
            vec4<f32>(perc_x, perc_x, perc_x, perc_x)
        ),
        vec4<f32>(perc_y, perc_y, perc_y, perc_y)
    );

   var dist: f32 = distance_alg(
       pixel_position,
       input.pos,
       input.scale,
       input.border_radius
   );

   var radius_alpha: f32 = 1.0 - smoothStep(
       max(input.border_radius - 0.5, 0.0),
       input.border_radius + 0.5,
       dist
   );

    return vec4<f32>(mixed_color.x, mixed_color.y, mixed_color.z, mixed_color.w * radius_alpha);
}
