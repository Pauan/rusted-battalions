@group(1) @binding(0) var spritesheet: texture_2d<f32>;


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) @interpolate(flat) alpha: f32,
    @location(1) uv: vec2<f32>,
    @location(2) tile: vec4<u32>,
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
    out.alpha = sprite.alpha;
    out.uv = make_uv(sprite.uv, vert_x, vert_y);
    out.tile = sprite.tile;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = tile_uv(normalize_uv(in.uv), in.tile);

    let color = textureLoad(spritesheet, uv, 0);

    if color.a == 0.0 {
        discard;

    } else {
        return vec4(color.rgb, in.alpha);
    }
}
