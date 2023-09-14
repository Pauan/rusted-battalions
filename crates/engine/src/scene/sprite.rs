use wgpu_helpers::VertexLayout;
use bytemuck::{Pod, Zeroable};
use futures_signals::signal::{Signal, SignalExt};

use crate::util::macros::wgsl;
use crate::util::builders;
use crate::util::buffer::{
    Uniform, TextureBuffer, InstanceVec, InstanceVecOptions,
    RgbaImage, GrayscaleImage, IndexedImage,
};
use crate::scene::builder::{Node, make_builder, base_methods, location_methods, simple_method};
use crate::scene::{
    Handle, Handles, Texture, MinSize, Location, Padding, Origin, Offset, Size, ScreenSize,
    SceneLayoutInfo, SceneRenderInfo, ScreenSpace, NodeLayout,  NodeHandle, SceneUniform,
};


/// Specifies which tile should be displayed (in pixel coordinates).
#[derive(Debug, Clone, Copy)]
pub struct Tile {
    pub start_x: u32,
    pub start_y: u32,
    pub end_x: u32,
    pub end_y: u32,
}

impl Tile {
    #[inline]
    pub fn mirror_x(self) -> Self {
        Self {
            start_x: self.end_x,
            start_y: self.start_y,
            end_x: self.start_x,
            end_y: self.end_y,
        }
    }

    #[inline]
    pub fn mirror_y(self) -> Self {
        Self {
            start_x: self.start_x,
            start_y: self.end_y,
            end_x: self.end_x,
            end_y: self.start_y,
        }
    }
}


#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, VertexLayout, Default, PartialEq)]
#[layout(step_mode = Instance)]
pub(crate) struct GPUSprite {
    pub(crate) position: [f32; 2],
    pub(crate) size: [f32; 2],
    pub(crate) z_index: f32,
    pub(crate) tile: [u32; 4],
}

impl GPUSprite {
    pub(crate) fn update(&mut self, space: &ScreenSpace) {
        self.position = [
            space.position[0],

            // The origin point of our sprites is in the upper-left corner,
            // but with wgpu the origin point is in the lower-left corner.
            // So we shift the y position into the lower-left corner of the sprite.
            space.position[1] - space.size[1],
        ];

        self.size = space.size;
        self.z_index = space.z_index;
    }
}


#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, VertexLayout, Default, PartialEq)]
#[layout(step_mode = Instance)]
#[layout(location = 4)]
pub(crate) struct GPUPalette {
    pub(crate) palette: u32,
}


/*pub struct ColorRgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}*/


#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, VertexLayout, Default, PartialEq)]
#[layout(step_mode = Instance)]
#[layout(location = 4)]
pub(crate) struct GPUText {
    pub(crate) color: [f32; 3],
}


pub struct Sprite {
    /// Whether any of the properties changed which require a re-render.
    render_changed: bool,

    /// Whether it needs to recalculate the location.
    location_changed: bool,

    visible: bool,
    stretch: bool,
    location: Location,
    spritesheet: Option<Spritesheet>,
    parent_space: Option<ScreenSpace>,
    min_size: Option<MinSize>,

    gpu: GPUSprite,
    gpu_index: usize,

    palette: GPUPalette,
}

impl Sprite {
    #[inline]
    fn new() -> Self {
        Self {
            render_changed: false,
            location_changed: false,
            visible: true,
            stretch: false,
            location: Location::default(),
            spritesheet: None,
            parent_space: None,
            min_size: None,

            gpu: GPUSprite::default(),
            gpu_index: 0,

            palette: GPUPalette {
                palette: 0,
            },
        }
    }

    fn update_gpu(&mut self, screen_size: &ScreenSize) {
        let parent = self.parent_space.as_ref().unwrap();

        let space = parent.modify(&self.location, &screen_size).convert_to_wgpu_coordinates();

        self.gpu.update(&space);
    }
}

make_builder!(Sprite, SpriteBuilder);
base_methods!(Sprite, SpriteBuilder);

location_methods!(Sprite, SpriteBuilder, false, |state| {
    state.render_changed = true;
    state.location_changed = true;
});

impl SpriteBuilder {
    simple_method!(
        /// Sets the [`Spritesheet`] which will be used for this sprite.
        spritesheet,
        spritesheet_signal,
        true,
        true,
        |state, value: Spritesheet| {
            state.spritesheet = Some(value);
        },
    );

    simple_method!(
        /// Sets the [`Tile`] which specifies which tile to display (in pixel coordinates).
        tile,
        tile_signal,
        false,
        true,
        |state, value: Tile| {
            state.gpu.tile = [value.start_x, value.start_y, value.end_x, value.end_y];
            state.render_changed = true;
        },
    );

    simple_method!(
        /// Sets the palette for this sprite.
        palette,
        palette_signal,
        false,
        true,
        |state, value: u32| {
            state.palette.palette = value;
            state.render_changed = true;
        },
    );
}

impl NodeLayout for Sprite {
    #[inline]
    fn is_visible(&mut self) -> bool {
        self.visible
    }

    #[inline]
    fn is_stretch(&mut self) -> bool {
        self.stretch
    }

    fn min_size<'a>(&mut self, info: &mut SceneLayoutInfo<'a>) -> MinSize {
        *self.min_size.get_or_insert_with(|| {
            self.location.min_size(&info.screen_size)
        })
    }

    fn update_layout<'a>(&mut self, handle: &NodeHandle, parent: &ScreenSpace, info: &mut SceneLayoutInfo<'a>) {
        self.render_changed = false;
        self.location_changed = false;
        self.parent_space = Some(*parent);

        self.update_gpu(&info.screen_size);

        info.renderer.set_max_z_index(self.gpu.z_index);

        let spritesheet = self.spritesheet.as_ref().expect("Sprite is missing spritesheet");

        if let Some(spritesheet) = info.renderer.sprite.spritesheets.get_mut(&spritesheet.handle) {
            self.gpu_index = spritesheet.locations.len();
            spritesheet.locations.push(self.gpu);

            match spritesheet.extra {
                SpritesheetExtra::Normal => {},
                SpritesheetExtra::Palette { ref mut palettes } => {
                    palettes.push(self.palette);
                },
                SpritesheetExtra::Text { .. } => {

                },
            }
        }

        info.rendered_nodes.push(handle.clone());

        self.min_size = None;
    }

    fn render<'a>(&mut self, info: &mut SceneRenderInfo<'a>) {
        if self.render_changed {
            self.render_changed = false;

            if self.location_changed {
                self.location_changed = false;

                self.update_gpu(&info.screen_size);
            }

            let spritesheet = self.spritesheet.as_ref().expect("Sprite is missing spritesheet");

            if let Some(spritesheet) = info.renderer.sprite.spritesheets.get_mut(&spritesheet.handle) {
                spritesheet.locations[self.gpu_index] = self.gpu;

                match spritesheet.extra {
                    SpritesheetExtra::Normal => {},
                    SpritesheetExtra::Palette { ref mut palettes } => {
                        palettes[self.gpu_index] = self.palette;
                    },
                    SpritesheetExtra::Text { .. } => {

                    },
                }
            }
        }

        info.renderer.set_max_z_index(self.gpu.z_index);
    }
}


struct SpritesheetPrerender<'a> {
    pipeline: &'a wgpu::RenderPipeline,
    // TODO figure out a way to avoid the Vec
    bind_groups: Vec<&'a wgpu::BindGroup>,
    slices: Vec<Option<wgpu::BufferSlice<'a>>>,
    len: u32,
}

impl<'a> SpritesheetPrerender<'a> {
    fn render<'b>(&'a mut self, render_pass: &mut wgpu::RenderPass<'b>) where 'a: 'b {
        if self.len > 0 {
            render_pass.set_pipeline(&self.pipeline);

            for (index, bind_group) in self.bind_groups.iter().enumerate() {
                render_pass.set_bind_group(index as u32, bind_group, &[]);
            }

            {
                let mut index = 0;

                for slice in self.slices.iter() {
                    if let Some(slice) = slice {
                        render_pass.set_vertex_buffer(index, *slice);
                        index += 1;
                    }
                }
            }

            render_pass.draw(0..4, 0..self.len);
        }
    }
}


pub(crate) struct SpritePrerender<'a> {
    spritesheets: Vec<SpritesheetPrerender<'a>>,
}

impl<'a> SpritePrerender<'a> {
    pub(crate) fn render<'b>(&'a mut self, render_pass: &mut wgpu::RenderPass<'b>) where 'a: 'b {
        for sheet in self.spritesheets.iter_mut() {
            sheet.render(render_pass);
        }
    }
}


struct SpritesheetPipeline {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
}

impl SpritesheetPipeline {
    fn new<'a>(
        engine: &crate::EngineState,
        scene_uniform_layout: &wgpu::BindGroupLayout,
        shader: wgpu::ShaderModuleDescriptor<'a>,
        vertex_buffers: &[wgpu::VertexBufferLayout],
        bind_group_layout: wgpu::BindGroupLayout
    ) -> Self {
        let stencil = wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::GreaterEqual,
            fail_op: wgpu::StencilOperation::Keep,
            depth_fail_op: wgpu::StencilOperation::Keep,
            pass_op: wgpu::StencilOperation::Replace,
        };

        let pipeline = builders::Pipeline::builder()
            .label("Sprite")
            // TODO lazy load this ?
            .shader(shader)
            .bind_groups(&[
                scene_uniform_layout,
                &bind_group_layout,
            ])
            .vertex_buffers(vertex_buffers)
            .topology(wgpu::PrimitiveTopology::TriangleStrip)
            .strip_index_format(wgpu::IndexFormat::Uint32)
            .depth_stencil(wgpu::StencilState {
                front: stencil,
                back: stencil,
                read_mask: 0xFF,
                write_mask: 0xFF,
            })
            .build(engine);

        Self { bind_group_layout, pipeline }
    }
}


enum SpritesheetExtra {
    Normal,
    Palette {
        palettes: InstanceVec<GPUPalette>,
    },
    Text {
        texts: InstanceVec<GPUText>,
    },
}

struct SpritesheetState {
    locations: InstanceVec<GPUSprite>,
    bind_group: wgpu::BindGroup,
    extra: SpritesheetExtra,
}

pub(crate) struct SpriteRenderer {
    normal: SpritesheetPipeline,
    palette: SpritesheetPipeline,
    text: SpritesheetPipeline,
    spritesheets: Handles<SpritesheetState>,
}

impl SpriteRenderer {
    #[inline]
    pub(crate) fn new(engine: &crate::EngineState, scene_uniform: &mut Uniform<SceneUniform>) -> Self {
        let scene_uniform_layout = Uniform::bind_group_layout(scene_uniform, engine);

        static SCENE: &'static str = include_str!("../wgsl/common/scene.wgsl");
        static SPRITE: &'static str = include_str!("../wgsl/common/sprite.wgsl");

        let normal = SpritesheetPipeline::new(
            engine,
            scene_uniform_layout,

            // TODO lazy load this ?
            wgsl![
                "spritesheet/normal.wgsl",
                SCENE,
                SPRITE,
                include_str!("../wgsl/spritesheet/normal.wgsl"),
            ],

            &[GPUSprite::LAYOUT],

            builders::BindGroupLayout::builder()
                .label("Sprite")
                .texture(wgpu::ShaderStages::FRAGMENT, wgpu::TextureSampleType::Float { filterable: false })
                .build(engine),
        );

        let palette = SpritesheetPipeline::new(
            engine,
            scene_uniform_layout,

            // TODO lazy load this ?
            wgsl![
                "spritesheet/palette.wgsl",
                SCENE,
                SPRITE,
                include_str!("../wgsl/spritesheet/palette.wgsl"),
            ],

            &[GPUSprite::LAYOUT, GPUPalette::LAYOUT],

            builders::BindGroupLayout::builder()
                .label("Sprite")
                .texture(wgpu::ShaderStages::FRAGMENT, wgpu::TextureSampleType::Uint)
                .texture(wgpu::ShaderStages::FRAGMENT, wgpu::TextureSampleType::Float { filterable: false })
                .build(engine),
        );

        let text = SpritesheetPipeline::new(
            engine,
            scene_uniform_layout,

            // TODO lazy load this ?
            wgsl![
                "spritesheet/text.wgsl",
                SCENE,
                SPRITE,
                include_str!("../wgsl/spritesheet/text.wgsl"),
            ],

            &[GPUSprite::LAYOUT, GPUText::LAYOUT],

            builders::BindGroupLayout::builder()
                .label("Sprite")
                .texture(wgpu::ShaderStages::FRAGMENT, wgpu::TextureSampleType::Uint)
                .build(engine),
        );

        Self {
            normal,
            palette,
            text,
            spritesheets: Handles::new(),
        }
    }

    fn spritesheet_state(&self, engine: &crate::EngineState, texture: &TextureBuffer, palette: Option<&TextureBuffer>) -> SpritesheetState {
        let locations = InstanceVec::new();

        if let Some(palette) = palette {
            assert_eq!(texture.texture.format(), IndexedImage::FORMAT, "texture must be an IndexedImage");
            assert_eq!(palette.texture.format(), RgbaImage::FORMAT, "palette must be an RgbaImage");

            SpritesheetState {
                locations,
                extra: SpritesheetExtra::Palette {
                    palettes: InstanceVec::new(),
                },
                bind_group: builders::BindGroup::builder()
                    .label("Spritesheet")
                    .layout(&self.palette.bind_group_layout)
                    .texture_view(&texture.view)
                    .texture_view(&palette.view)
                    .build(engine),
            }

        } else if texture.texture.format() == GrayscaleImage::FORMAT {
            SpritesheetState {
                locations,
                extra: SpritesheetExtra::Text {
                    texts: InstanceVec::new(),
                },
                bind_group: builders::BindGroup::builder()
                    .label("Spritesheet")
                    .layout(&self.text.bind_group_layout)
                    .texture_view(&texture.view)
                    .build(engine),
            }

        } else {
            assert_eq!(texture.texture.format(), RgbaImage::FORMAT, "texture must be an RgbaImage");

            SpritesheetState {
                locations,
                extra: SpritesheetExtra::Normal,
                bind_group: builders::BindGroup::builder()
                    .label("Spritesheet")
                    .layout(&self.normal.bind_group_layout)
                    .texture_view(&texture.view)
                    .build(engine),
            }
        }
    }

    #[inline]
    pub(crate) fn before_layout(&mut self) {
        for (_, sheet) in self.spritesheets.iter_mut() {
            sheet.locations.clear();

            match sheet.extra {
                SpritesheetExtra::Normal => {},
                SpritesheetExtra::Palette { ref mut palettes } => {
                    palettes.clear();
                },
                SpritesheetExtra::Text { ref mut texts } => {
                    texts.clear();
                },
            }
        }
    }

    #[inline]
    pub(crate) fn before_render(&mut self) {}

    #[inline]
    pub(crate) fn prerender<'a>(&'a mut self, engine: &crate::EngineState, scene_uniform: &'a wgpu::BindGroup) -> SpritePrerender<'a> {
        SpritePrerender {
            spritesheets: self.spritesheets.iter_mut()
                .map(|(_, sheet)| {
                    let len = sheet.locations.len() as u32;

                    let bind_groups = vec![
                        scene_uniform,
                        &sheet.bind_group,
                    ];

                    let pipeline = match sheet.extra {
                        SpritesheetExtra::Normal => &self.normal.pipeline,
                        SpritesheetExtra::Palette { .. } => &self.palette.pipeline,
                        SpritesheetExtra::Text { .. } => &self.text.pipeline,
                    };

                    let slices = vec![
                        sheet.locations.update_buffer(engine, &InstanceVecOptions {
                            label: Some("Sprite Instance Buffer"),
                        }),

                        match sheet.extra {
                            SpritesheetExtra::Normal => None,
                            SpritesheetExtra::Palette { ref mut palettes } => {
                                palettes.update_buffer(engine, &InstanceVecOptions {
                                    label: Some("Sprite Palettes Buffer"),
                                })
                            },
                            SpritesheetExtra::Text { ref mut texts } => {
                                texts.update_buffer(engine, &InstanceVecOptions {
                                    label: Some("Sprite Texts Buffer"),
                                })
                            },
                        }
                    ];

                    SpritesheetPrerender {
                        pipeline,
                        bind_groups,
                        slices,
                        len,
                    }
                })
                .collect(),
        }
    }
}


pub struct SpritesheetSettings<'a, 'b> {
    pub texture: &'a Texture,
    pub palette: Option<&'b Texture>,
}

#[derive(Clone)]
pub struct Spritesheet {
    pub(crate) handle: Handle,
}

impl Spritesheet {
    #[inline]
    pub fn new() -> Self {
        Self {
            handle: Handle::new(),
        }
    }

    #[inline]
    pub fn new_load<'a, 'b, Window>(engine: &mut crate::Engine<Window>, settings: SpritesheetSettings<'a, 'b>) -> Self {
        let x = Self::new();
        x.load(engine, settings);
        x
    }

    pub fn load<'a, 'b, Window>(&self, engine: &mut crate::Engine<Window>, settings: SpritesheetSettings<'a, 'b>) {
        let texture = engine.scene.textures.get(&settings.texture.handle).expect("SpritesheetSettings texture is not loaded");

        let palette = settings.palette.map(|palette| {
            engine.scene.textures.get(&palette.handle).expect("SpritesheetSettings palette is not loaded")
        });

        let renderer = &mut engine.scene.renderer.sprite;
        renderer.spritesheets.insert(&self.handle, renderer.spritesheet_state(&engine.state, texture, palette));

        // TODO test this
        engine.scene.changed.trigger_layout_change();
    }

    pub fn unload<Window>(&self, engine: &mut crate::Engine<Window>) {
        engine.scene.renderer.sprite.spritesheets.remove(&self.handle);

        // TODO test this
        engine.scene.changed.trigger_layout_change();
    }
}
