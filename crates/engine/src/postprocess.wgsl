@group(0) @binding(0) var texture_sampler: sampler;
@group(0) @binding(1) var depth_sampler: sampler;
@group(0) @binding(2) var color: texture_2d<f32>;
@group(0) @binding(3) var depth: texture_depth_2d;
//@group(0) @binding(4) var stencil: texture_depth_2d;
@group(0) @binding(4) var stencil: texture_2d<u32>;


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    let vert_x = i32(in_vertex_index) < 2;
    let vert_y = i32(in_vertex_index) % 2;

    let uv = vec2(
        select(0.0, 1.0, vert_x),
        select(0.0, 1.0, vert_y == 0),
    );

    let x = f32(vert_x) * 2.0 - 1.0;
    let y = f32(vert_y) * 2.0 - 1.0;

    var out: VertexOutput;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = uv;
    return out;
}

fn debug_depth(in: VertexOutput) -> vec4<f32> {
    return vec4<f32>(0.0, textureSample(depth, texture_sampler, in.uv), 0.0, 1.0);
}

/*
    let size = textureDimensions(stencil);
    let coord = vec2<u32>(in.uv * vec2<f32>(size));
    /*let coord = vec2(
        u32(in.uv.x * f32(size.x)),
        u32(in.uv.y * f32(size.y)),
    );*/
    let s: vec4<u32> = textureLoad(stencil, coord, 0);
    return vec4(f32(s.r), 0.0, 0.0, 1.0);
*/

/*fn debug_stencil(in: VertexOutput) -> vec4<f32> {
    return vec4<f32>(textureSample(depth, texture_sampler, in.uv), 0.0, 0.0, 1.0);
}*/

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, textureSample(depth, texture_sampler, in.uv), 0.0, 1.0);
    //return debug_depth(in);
    //return textureSample(color, texture_sampler, in.uv);
    //return vec4(in.uv.x, in.uv.y, 0.0, 1.0);
}
