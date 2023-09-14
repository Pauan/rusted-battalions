struct Sprite {
    @location(0) position: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) z_index: f32,
    @location(3) tile: vec4<u32>,
};


fn quad_x(in_vertex_index: u32) -> i32 {
    // TODO get rid of the second cast somehow
    return i32(i32(in_vertex_index) < 2);
}


fn quad_y(in_vertex_index: u32) -> i32 {
    return i32(in_vertex_index) % 2;
}


fn sprite_uv(sprite: Sprite, vert_x: i32, vert_y: i32) -> vec2<f32> {
    let uv_x = select(sprite.tile[2], sprite.tile[0], vert_x == 0);
    let uv_y = select(sprite.tile[1], sprite.tile[3], vert_y == 0);

    return vec2(f32(uv_x), f32(uv_y));
}


fn sprite_clip_position(sprite: Sprite, vert_x: i32, vert_y: i32) -> vec4<f32> {
    let x = f32(vert_x) * sprite.size.x + sprite.position.x;
    let y = f32(vert_y) * sprite.size.y + sprite.position.y;

    let z_index = sprite.z_index;
    let max_z_index = scene.max_z_index;

    return vec4<f32>(x * max_z_index, y * max_z_index, z_index, max_z_index);
}


fn uv_u32(uv: vec2<f32>) -> vec2<u32> {
    return vec2(u32(uv.x), u32(uv.y));
}
