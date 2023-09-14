use wgpu_helpers::VertexLayout;


#[test]
fn test_types() {
    #[repr(C)]
    #[derive(VertexLayout)]
    struct Vertex {
        u8_1: [u8; 2],
        u8_2: [u8; 4],
        #[layout(norm)]
        u8_3: [u8; 2],
        #[layout(norm)]
        u8_4: [u8; 4],

        i8_1: [i8; 2],
        i8_2: [i8; 4],
        #[layout(norm)]
        i8_3: [i8; 2],
        #[layout(norm)]
        i8_4: [i8; 4],

        u16_1: [u16; 2],
        u16_2: [u16; 4],
        #[layout(norm)]
        u16_3: [u16; 2],
        #[layout(norm)]
        u16_4: [u16; 4],

        i16_1: [i16; 2],
        i16_2: [i16; 4],
        #[layout(norm)]
        i16_3: [i16; 2],
        #[layout(norm)]
        i16_4: [i16; 4],

        u32_1: u32,
        u32_2: [u32; 1],
        u32_3: [u32; 2],
        u32_4: [u32; 3],
        u32_5: [u32; 4],

        i32_1: i32,
        i32_2: [i32; 1],
        i32_3: [i32; 2],
        i32_4: [i32; 3],
        i32_5: [i32; 4],

        f32_1: f32,
        f32_2: [f32; 1],
        f32_3: [f32; 2],
        f32_4: [f32; 3],
        f32_5: [f32; 4],

        f64_1: f64,
        f64_2: [f64; 1],
        f64_3: [f64; 2],
        f64_4: [f64; 3],
        f64_5: [f64; 4],
    }

    assert_eq!(
        Vertex::LAYOUT,

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![
                0 => Uint8x2,
                1 => Uint8x4,
                2 => Unorm8x2,
                3 => Unorm8x4,

                4 => Sint8x2,
                5 => Sint8x4,
                6 => Snorm8x2,
                7 => Snorm8x4,

                8 => Uint16x2,
                9 => Uint16x4,
                10 => Unorm16x2,
                11 => Unorm16x4,

                12 => Sint16x2,
                13 => Sint16x4,
                14 => Snorm16x2,
                15 => Snorm16x4,

                16 => Uint32,
                17 => Uint32,
                18 => Uint32x2,
                19 => Uint32x3,
                20 => Uint32x4,

                21 => Sint32,
                22 => Sint32,
                23 => Sint32x2,
                24 => Sint32x3,
                25 => Sint32x4,

                26 => Float32,
                27 => Float32,
                28 => Float32x2,
                29 => Float32x3,
                30 => Float32x4,

                31 => Float64,
                32 => Float64,
                33 => Float64x2,
                34 => Float64x3,
                35 => Float64x4,
            ],
        },
    )
}


#[test]
fn test_step_mode() {
    #[repr(C)]
    #[derive(VertexLayout)]
    #[layout(step_mode = Instance)]
    struct Vertex {
        u32_1: u32,
        u32_2: [u32; 1],
        u32_3: [u32; 2],
        u32_4: [u32; 3],
        u32_5: [u32; 4],
    }

    assert_eq!(
        Vertex::LAYOUT,

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![
                0 => Uint32,
                1 => Uint32,
                2 => Uint32x2,
                3 => Uint32x3,
                4 => Uint32x4,
            ],
        },
    )
}


#[test]
fn test_struct_location() {
    #[repr(C)]
    #[derive(VertexLayout)]
    #[layout(location = 4)]
    struct Vertex {
        u32_1: u32,
        u32_2: [u32; 1],
        u32_3: [u32; 2],
        u32_4: [u32; 3],
        u32_5: [u32; 4],
    }

    assert_eq!(
        Vertex::LAYOUT,

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![
                4 => Uint32,
                5 => Uint32,
                6 => Uint32x2,
                7 => Uint32x3,
                8 => Uint32x4,
            ],
        },
    )
}


#[test]
fn test_field_location() {
    #[repr(C)]
    #[derive(VertexLayout)]
    struct Vertex {
        #[layout(location = 4)]
        u32_1: u32,
        u32_2: [u32; 1],
        u32_3: [u32; 2],
        #[layout(location = 10)]
        u32_4: [u32; 3],
        u32_5: [u32; 4],

        f32_1: f32,
        f32_2: [f32; 1],
        f32_3: [f32; 2],
        f32_4: [f32; 3],
        f32_5: [f32; 4],
    }

    assert_eq!(
        Vertex::LAYOUT,

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![
                4 => Uint32,
                5 => Uint32,
                6 => Uint32x2,
                10 => Uint32x3,
                11 => Uint32x4,
                12 => Float32,
                13 => Float32,
                14 => Float32x2,
                15 => Float32x3,
                16 => Float32x4,
            ],
        },
    )
}


#[test]
fn test_format() {
    #[repr(C)]
    #[derive(VertexLayout)]
    struct Vertex {
        u32_1: u32,
        u32_2: [u32; 1],
        #[layout(format = Uint8x4)]
        u32_3: [u32; 2],
        u32_4: [u32; 3],
        u32_5: [u32; 4],
    }

    assert_eq!(
        Vertex::LAYOUT,

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![
                0 => Uint32,
                1 => Uint32,
                2 => Uint8x4,
                3 => Uint32x3,
                4 => Uint32x4,
            ],
        },
    )
}


#[test]
fn test_multiple() {
    #[repr(C)]
    #[derive(VertexLayout)]
    #[layout(step_mode = Instance, location = 3)]
    struct Vertex {
        #[layout(location = 20)]
        a: u32,
        #[layout(location = 10, norm)]
        b: [u8; 2],
        c: [u32; 2],
        #[layout(location = 10, norm, norm, norm, location = 15)]
        d: [u8; 2],
        e: [u32; 4],
    }

    assert_eq!(
        Vertex::LAYOUT,

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![
                20 => Uint32,
                10 => Unorm8x2,
                11 => Uint32x2,
                15 => Unorm8x2,
                16 => Uint32x4,
            ],
        },
    )
}
