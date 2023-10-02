use wgpu_helpers::VertexLayout;
use bytemuck::{Pod, Zeroable};
use futures_signals::signal::{Signal, SignalExt};

use crate::util::macros::wgsl;
use crate::util::builders;
use crate::util::buffer::{
    Uniform, TextureBuffer, InstanceVec, InstanceVecOptions,
    RgbaImage, IndexedImage,
};
use crate::scene::builder::{Node, make_builder, base_methods, location_methods, simple_method};
use crate::scene::{
    Handle, Handles, Texture, Location, Padding, Origin, Offset, Size, ScreenSize, SmallestSize,
    SceneLayoutInfo, SceneRenderInfo, RealLocation, NodeLayout,  NodeHandle, SceneUniform,
    ScenePrerender, Prerender, Length, RealSize, ScreenLength,
};


#[derive(Debug, Clone, Copy)]
pub enum Repeat {
    /// Don't repeat, stretches to fill the entire sprite.
    None,

    /// Repeats the tile every length.
    ///
    /// # Sizing
    ///
    /// * [`Length::SmallestWidth`]: the smallest width of the Sprite.
    ///
    /// * [`Length::SmallestHeight`]: the smallest height of the Sprite.
    Length(Length),

    /// Repeats the tile a certain number of times.
    Count(f32),
}

impl Repeat {
    fn to_uv(&self, parent: &RealSize, smallest: &RealSize, screen: &ScreenLength, distance: f32) -> f32 {
        match self {
            Self::None => 1.0,
            Self::Length(length) => {
                let length = length.real_length(parent, smallest, screen);

                distance / length
            },
            Self::Count(count) => *count,
        }
    }
}

impl Default for Repeat {
    /// Returns [`Repeat::None`].
    #[inline]
    fn default() -> Self {
        Self::None
    }
}


/// Specifies the repetition of the sprite tile.
#[derive(Debug, Clone, Copy, Default)]
pub struct RepeatTile {
    pub width: Repeat,
    pub height: Repeat,
}

impl RepeatTile {
    fn to_uv(&self, this: &RealSize, parent: &RealSize, smallest: &RealSize, screen: &ScreenSize) -> [f32; 2] {
        [
            self.width.to_uv(parent, smallest, &screen.width, this.width),
            self.height.to_uv(parent, smallest, &screen.height, this.height),
        ]
    }
}


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
    pub(crate) uv: [f32; 2],
    pub(crate) tile: [u32; 4],
}

impl GPUSprite {
    pub(crate) fn update(&mut self, location: &RealLocation) {
        let location = location.convert_to_wgpu_coordinates();

        self.position = [
            location.position.x,

            // The origin point of our sprites is in the upper-left corner,
            // but with wgpu the origin point is in the lower-left corner.
            // So we shift the y position into the lower-left corner of the sprite.
            location.position.y - location.size.height,
        ];

        self.size = [location.size.width, location.size.height];
        self.z_index = location.z_index;
    }
}


#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, VertexLayout, Default, PartialEq)]
#[layout(step_mode = Instance)]
#[layout(location = 5)]
pub(crate) struct GPUPalette {
    pub(crate) palette: u32,
}


/// Displays a sprite from a spritesheet.
///
/// # Sizing
///
/// * [`Length::SmallestWidth`]: it is an error to use `SmallestWidth`.
///
/// * [`Length::SmallestHeight`]: it is an error to use `SmallestHeight`.
pub struct Sprite {
    visible: bool,
    location: Location,
    spritesheet: Option<Spritesheet>,
    repeat_tile: RepeatTile,

    /// Whether any of the properties changed which require a re-render.
    render_changed: bool,

    /// Whether it needs to recalculate the location.
    location_changed: bool,

    parent_location: Option<RealLocation>,
    smallest_size: Option<RealSize>,

    gpu_index: usize,
    gpu_sprite: GPUSprite,
    gpu_palette: GPUPalette,
}

impl Sprite {
    #[inline]
    fn new() -> Self {
        Self {
            visible: true,
            location: Location::default(),
            spritesheet: None,
            repeat_tile: RepeatTile::default(),

            render_changed: false,
            location_changed: false,

            parent_location: None,
            smallest_size: None,

            gpu_index: 0,
            gpu_sprite: GPUSprite::default(),
            gpu_palette: GPUPalette::default(),
        }
    }

    fn location_changed(&mut self) {
        self.location_changed = true;
        self.render_changed();
    }

    fn render_changed(&mut self) {
        self.render_changed = true;
    }

    fn update_gpu(&mut self, screen: &ScreenSize) {
        let parent = self.parent_location.as_ref().unwrap();
        let smallest = self.smallest_size.as_ref().unwrap();

        let location = self.location.children_location(parent, smallest, screen);

        self.gpu_sprite.uv = self.repeat_tile.to_uv(&location.size, &parent.size, smallest, screen);

        self.gpu_sprite.update(&location);
    }
}

make_builder!(Sprite, SpriteBuilder);
base_methods!(Sprite, SpriteBuilder);

location_methods!(Sprite, SpriteBuilder, false, |state| {
    state.location_changed();
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
            state.gpu_sprite.tile = [
                value.start_x,
                value.start_y,
                value.end_x,
                value.end_y,
            ];

            state.render_changed();
        },
    );

    simple_method!(
        /// Sets the [`RepeatTile`] which specifies how to repeat the sprite tile.
        repeat_tile,
        repeat_tile_signal,
        false,
        true,
        |state, value: RepeatTile| {
            state.repeat_tile = value;
            state.location_changed();
        },
    );

    simple_method!(
        /// Sets the palette for this sprite.
        palette,
        palette_signal,
        false,
        true,
        |state, value: u32| {
            state.gpu_palette.palette = value;
            state.render_changed();
        },
    );
}

impl NodeLayout for Sprite {
    #[inline]
    fn is_visible(&mut self) -> bool {
        self.visible
    }

    fn smallest_size<'a>(&mut self, _parent: &SmallestSize, info: &mut SceneLayoutInfo<'a>) -> SmallestSize {
        self.location.size.smallest_size(&info.screen_size)
    }

    fn update_layout<'a>(&mut self, handle: &NodeHandle, parent: &RealLocation, smallest_size: &SmallestSize, info: &mut SceneLayoutInfo<'a>) {
        let smallest_size = smallest_size.real_size();

        self.render_changed = false;
        self.location_changed = false;
        self.parent_location = Some(*parent);
        self.smallest_size = Some(smallest_size);

        self.update_gpu(&info.screen_size);

        info.renderer.set_max_z_index(self.gpu_sprite.z_index);

        let spritesheet = self.spritesheet.as_ref().expect("Sprite is missing spritesheet");

        if let Some(spritesheet) = info.renderer.sprite.spritesheets.get_mut(&spritesheet.handle) {
            self.gpu_index = spritesheet.sprites.len();

            spritesheet.sprites.push(self.gpu_sprite);

            match spritesheet.extra {
                SpritesheetExtra::Normal => {},
                SpritesheetExtra::Palette { ref mut palettes } => {
                    palettes.push(self.gpu_palette);
                },
            }
        }

        info.rendered_nodes.push(handle.clone());
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
                spritesheet.sprites[self.gpu_index] = self.gpu_sprite;

                match spritesheet.extra {
                    SpritesheetExtra::Normal => {},
                    SpritesheetExtra::Palette { ref mut palettes } => {
                        palettes[self.gpu_index] = self.gpu_palette;
                    },
                }
            }
        }

        info.renderer.set_max_z_index(self.gpu_sprite.z_index);
    }
}


pub(crate) struct SpritesheetPipeline {
    pub(crate) bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) pipeline: wgpu::RenderPipeline,
}

impl SpritesheetPipeline {
    pub(crate) fn new<'a>(
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


pub(crate) static SCENE_SHADER: &'static str = include_str!("../wgsl/common/scene.wgsl");
pub(crate) static SPRITE_SHADER: &'static str = include_str!("../wgsl/common/sprite.wgsl");


enum SpritesheetExtra {
    Normal,
    Palette {
        palettes: InstanceVec<GPUPalette>,
    },
}

struct SpritesheetState {
    sprites: InstanceVec<GPUSprite>,
    bind_group: wgpu::BindGroup,
    extra: SpritesheetExtra,
}

pub(crate) struct SpriteRenderer {
    normal: SpritesheetPipeline,
    palette: SpritesheetPipeline,
    spritesheets: Handles<SpritesheetState>,
}

impl SpriteRenderer {
    #[inline]
    pub(crate) fn new(engine: &crate::EngineState, scene_uniform: &mut Uniform<SceneUniform>) -> Self {
        let scene_uniform_layout = Uniform::bind_group_layout(scene_uniform, engine);

        let normal = SpritesheetPipeline::new(
            engine,
            scene_uniform_layout,

            // TODO lazy load this ?
            wgsl![
                "spritesheet/normal.wgsl",
                SCENE_SHADER,
                SPRITE_SHADER,
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
                SCENE_SHADER,
                SPRITE_SHADER,
                include_str!("../wgsl/spritesheet/palette.wgsl"),
            ],

            &[GPUSprite::LAYOUT, GPUPalette::LAYOUT],

            builders::BindGroupLayout::builder()
                .label("Sprite")
                .texture(wgpu::ShaderStages::FRAGMENT, wgpu::TextureSampleType::Uint)
                .texture(wgpu::ShaderStages::FRAGMENT, wgpu::TextureSampleType::Float { filterable: false })
                .build(engine),
        );

        Self {
            normal,
            palette,
            spritesheets: Handles::new(),
        }
    }

    fn new_spritesheet(&mut self, engine: &crate::EngineState, handle: &Handle, texture: &TextureBuffer, palette: Option<&TextureBuffer>) {
        let sprites = InstanceVec::new();

        let state = if let Some(palette) = palette {
            assert_eq!(texture.texture.format(), IndexedImage::FORMAT, "texture must be an IndexedImage");
            assert_eq!(palette.texture.format(), RgbaImage::FORMAT, "palette must be an RgbaImage");

            SpritesheetState {
                sprites,
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

        } else {
            assert_eq!(texture.texture.format(), RgbaImage::FORMAT, "texture must be an RgbaImage");

            SpritesheetState {
                sprites,
                extra: SpritesheetExtra::Normal,
                bind_group: builders::BindGroup::builder()
                    .label("Spritesheet")
                    .layout(&self.normal.bind_group_layout)
                    .texture_view(&texture.view)
                    .build(engine),
            }
        };

        self.spritesheets.insert(handle, state);
    }

    fn remove_spritesheet(&mut self, handle: &Handle) {
        self.spritesheets.remove(handle);
    }

    #[inline]
    pub(crate) fn before_layout(&mut self) {
        for (_, sheet) in self.spritesheets.iter_mut() {
            sheet.sprites.clear();

            match sheet.extra {
                SpritesheetExtra::Normal => {},
                SpritesheetExtra::Palette { ref mut palettes } => {
                    palettes.clear();
                },
            }
        }
    }

    #[inline]
    pub(crate) fn before_render(&mut self) {}

    #[inline]
    pub(crate) fn prerender<'a>(
        &'a mut self,
        engine: &crate::EngineState,
        scene_uniform: &'a wgpu::BindGroup,
        prerender: &mut ScenePrerender<'a>,
    ) {
        prerender.prerenders.reserve(self.spritesheets.len());

        for (_, sheet) in self.spritesheets.iter_mut() {
            let instances = sheet.sprites.len() as u32;

            let bind_groups = vec![
                scene_uniform,
                &sheet.bind_group,
            ];

            let pipeline = match sheet.extra {
                SpritesheetExtra::Normal => &self.normal.pipeline,
                SpritesheetExtra::Palette { .. } => &self.palette.pipeline,
            };

            let slices = vec![
                sheet.sprites.update_buffer(engine, &InstanceVecOptions {
                    label: Some("Sprite Instance Buffer"),
                }),

                match sheet.extra {
                    SpritesheetExtra::Normal => None,
                    SpritesheetExtra::Palette { ref mut palettes } => {
                        palettes.update_buffer(engine, &InstanceVecOptions {
                            label: Some("Sprite Palettes Buffer"),
                        })
                    },
                }
            ];

            prerender.prerenders.push(Prerender {
                vertices: 4,
                instances,
                pipeline,
                bind_groups,
                slices,
            });
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
        Self { handle: Handle::new() }
    }

    pub fn load<'a, 'b, Window>(&self, engine: &mut crate::Engine<Window>, settings: SpritesheetSettings<'a, 'b>) {
        let texture = engine.scene.textures.get(&settings.texture.handle)
            .expect("SpritesheetSettings texture is not loaded");

        let palette = settings.palette.map(|palette| {
            engine.scene.textures.get(&palette.handle)
                .expect("SpritesheetSettings palette is not loaded")
        });

        engine.scene.renderer.sprite.new_spritesheet(&engine.state, &self.handle, texture, palette);

        // TODO test this
        engine.scene.changed.trigger_layout_change();
    }

    pub fn unload<Window>(&self, engine: &mut crate::Engine<Window>) {
        engine.scene.renderer.sprite.remove_spritesheet(&self.handle);

        // TODO test this
        engine.scene.changed.trigger_layout_change();
    }
}
