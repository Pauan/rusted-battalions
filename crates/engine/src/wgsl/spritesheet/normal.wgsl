@group(1) @binding(0) var spritesheet: texture_2d<f32>;


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    sprite: Sprite,
) -> VertexOutput {
    let vert_x = quad_x(in_vertex_index);
    let vert_y = quad_y(in_vertex_index);

    var out: VertexOutput;
    out.clip_position = sprite_clip_position(sprite, vert_x, vert_y);
    out.uv = sprite_uv(sprite, vert_x, vert_y);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureLoad(spritesheet, uv_u32(in.uv), 0);

    if color.a == 0.0 {
        discard;

    } else {
        return vec4(color.rgb, 1.0);
    }
}
