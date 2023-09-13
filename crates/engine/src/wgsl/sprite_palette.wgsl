struct Scene {
    max_z_index: f32,

    // TODO figure out how to get rid of this padding
    _padding1: f32,
    _padding2: f32,
    _padding3: f32,
};
@group(0) @binding(0) var<uniform> scene: Scene;

@group(1) @binding(0) var spritesheet: texture_2d<u32>;
@group(1) @binding(1) var palette: texture_2d<f32>;


struct Sprite {
    @location(0) position: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) z_index: f32,
    @location(3) tile: vec4<u32>,
};

struct Palette {
    @location(4) palette: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) palette: u32,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    sprite: Sprite,
    palette: Palette,
) -> VertexOutput {
    let vert_x = i32(in_vertex_index) < 2;
    let vert_y = i32(in_vertex_index) % 2;

    let uv_x = select(sprite.tile[0], sprite.tile[2], vert_x);
    let uv_y = select(sprite.tile[1], sprite.tile[3], vert_y == 0);

    let uv = vec2(f32(uv_x), f32(uv_y));

    let x = f32(vert_x) * sprite.size.x + sprite.position.x;
    let y = f32(vert_y) * sprite.size.y + sprite.position.y;

    let z_index = sprite.z_index;
    let max_z_index = scene.max_z_index;

    var out: VertexOutput;
    out.clip_position = vec4<f32>(x * max_z_index, y * max_z_index, z_index, max_z_index);
    out.uv = uv;
    out.palette = palette.palette;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let index: vec4<u32> = textureLoad(spritesheet, vec2(u32(in.uv.x), u32(in.uv.y)), 0);

    if index.g == 0u {
        discard;

    } else {
        let color = textureLoad(palette, vec2(index.r, in.palette), 0);
        return vec4(color.r, color.g, color.b, 1.0);
    }
}
