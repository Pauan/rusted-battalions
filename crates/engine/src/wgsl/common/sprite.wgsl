struct Sprite {
    @location(0) position: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) order: f32,
    @location(3) uv: vec2<f32>,
    @location(4) tile: vec4<u32>,
};


fn quad_x(in_vertex_index: u32) -> i32 {
    // TODO get rid of the second cast somehow
    return i32(i32(in_vertex_index) < 2);
}

fn quad_y(in_vertex_index: u32) -> i32 {
    return i32(in_vertex_index) % 2;
}


fn normalize_f32(f: f32) -> f32 {
    let n = fract(f);
    return select(n, 1.0, n == 0.0 && f != 0.0);
}

fn normalize_uv(uv: vec2<f32>) -> vec2<f32> {
    return vec2(normalize_f32(uv.x), normalize_f32(uv.y));
}


fn make_uv(uv: vec2<f32>, vert_x: i32, vert_y: i32) -> vec2<f32> {
    let uv_x = select(uv.x, 0.0, vert_x == 0);
    let uv_y = select(0.0, uv.y, vert_y == 0);
    return vec2(uv_x, uv_y);
}

fn tile_uv(uv: vec2<f32>, tile: vec4<u32>) -> vec2<u32> {
    let x = u32(mix(f32(tile[0]), f32(tile[2]), uv.x));
    let y = u32(mix(f32(tile[1]), f32(tile[3]), uv.y));
    return vec2(x, y);
}


fn sprite_clip_position(sprite: Sprite, vert_x: i32, vert_y: i32) -> vec4<f32> {
    let x = f32(vert_x) * sprite.size.x + sprite.position.x;
    let y = f32(vert_y) * sprite.size.y + sprite.position.y;

    let order = sprite.order;
    let max_order = scene.max_order;

    return vec4<f32>(x * max_order, y * max_order, order, max_order);
}
