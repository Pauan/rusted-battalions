struct Scene {
    max_order: f32,

    // TODO figure out how to get rid of this padding
    _padding1: f32,
    _padding2: f32,
    _padding3: f32,
};
@group(0) @binding(0) var<uniform> scene: Scene;
