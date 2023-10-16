@group(1) @binding(0) var spritesheet: texture_2d<u32>;
@group(1) @binding(1) var palette: texture_2d<f32>;


struct Palette {
    @location(6) palette: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) @interpolate(flat) alpha: f32,
    @location(1) uv: vec2<f32>,
    @location(2) tile: vec4<u32>,
    @location(3) palette: u32,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    sprite: Sprite,
    palette: Palette,
) -> VertexOutput {
    let vert_x = quad_x(in_vertex_index);
    let vert_y = quad_y(in_vertex_index);

    var out: VertexOutput;
    out.clip_position = sprite_clip_position(sprite, vert_x, vert_y);
    out.alpha = sprite.alpha;
    out.uv = make_uv(sprite.uv, vert_x, vert_y);
    out.tile = sprite.tile;
    out.palette = palette.palette;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = tile_uv(normalize_uv(in.uv), in.tile);

    let index: vec4<u32> = textureLoad(spritesheet, uv, 0);

    if index.g == 0u {
        discard;

    } else {
        let color = textureLoad(palette, vec2(index.r, in.palette), 0);
        return vec4(color.rgb, in.alpha);
    }
}
