use wgpu_helpers::VertexLayout;
use bytemuck::{Pod, Zeroable};
use futures_signals::signal::{Signal, SignalExt};

use crate::DEBUG;
use crate::util::macros::wgsl;
use crate::util::builders;
use crate::util::buffer::{
    Uniform, TextureBuffer, InstanceVec, InstanceVecOptions,
    RgbaImage, IndexedImage,
};
use crate::scene::builder::{Node, BuilderChanged, make_builder, base_methods, location_methods, simple_method};
use crate::scene::{
    Handle, Handles, Texture, Location, Padding, Origin, Offset, Size, ScreenSize, SmallestSize,
    SceneLayoutInfo, SceneRenderInfo, RealLocation, NodeLayout,  NodeHandle, SceneUniform,
    ScenePrerender, Prerender, Length, RealSize, ScreenLength, Order, Percentage,
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
#[derive(Copy, Clone, Debug, Pod, Zeroable, VertexLayout, PartialEq)]
#[layout(step_mode = Instance)]
pub(crate) struct GPUSprite {
    pub(crate) position: [f32; 2],
    pub(crate) size: [f32; 2],
    pub(crate) order: f32,
    pub(crate) alpha: f32,
    pub(crate) uv: [f32; 2],
    pub(crate) tile: [u32; 4],
}

impl Default for GPUSprite {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0],
            size: [0.0, 0.0],
            order: 1.0,
            alpha: 1.0,
            uv: [1.0, 1.0],
            tile: [0, 0, 0, 0],
        }
    }
}

impl GPUSprite {
    pub(crate) fn update(&mut self, location: &RealLocation) {
        if location.order < 1.0 {
            panic!("Order cannot be lower than 1.0");
        }

        let location = location.convert_to_wgpu_coordinates();

        self.position = [
            location.position.x,

            // The origin point of our sprites is in the upper-left corner,
            // but with wgpu the origin point is in the lower-left corner.
            // So we shift the y position into the lower-left corner of the sprite.
            location.position.y - location.size.height,
        ];

        self.size = [location.size.width, location.size.height];
        self.order = location.order;
    }
}


#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, VertexLayout, Default, PartialEq)]
#[layout(step_mode = Instance)]
#[layout(location = 6)]
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
    max_order: f32,

    gpu_index: usize,
    gpu_sprite: GPUSprite,
    gpu_palette: Option<GPUPalette>,
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
            max_order: 1.0,

            gpu_index: 0,
            gpu_sprite: GPUSprite::default(),
            gpu_palette: None,
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

        let location = self.location.children_location_explicit(parent, smallest, screen, self.max_order);

        self.gpu_sprite.uv = self.repeat_tile.to_uv(&location.size, &parent.size, smallest, screen);

        self.gpu_sprite.update(&location);
    }
}

make_builder!(Sprite, SpriteBuilder);
base_methods!(Sprite, SpriteBuilder);

location_methods!(Sprite, SpriteBuilder, |state| {
    state.location_changed();
    BuilderChanged::Render
});

impl SpriteBuilder {
    simple_method!(
        /// Sets the alpha for the sprite.
        ///
        /// 1.0 means fully opaque, 0.0 means fully transparent.
        alpha,
        alpha_signal,
        |state, value: Percentage| {
            if state.gpu_sprite.alpha != value {
                let old = state.gpu_sprite.alpha == 1.0 || state.gpu_sprite.alpha == 0.0;
                let new = value == 1.0 || value == 0.0;

                state.gpu_sprite.alpha = value;

                if old || new {
                    BuilderChanged::Layout

                } else {
                    state.render_changed();
                    BuilderChanged::Render
                }

            } else {
                BuilderChanged::None
            }
        },
    );

    simple_method!(
        /// Sets the [`Spritesheet`] which will be used for this sprite.
        spritesheet,
        spritesheet_signal,
        |state, value: Spritesheet| {
            state.spritesheet = Some(value);
            BuilderChanged::Layout
        },
    );

    simple_method!(
        /// Sets the [`Tile`] which specifies which tile to display (in pixel coordinates).
        tile,
        tile_signal,
        |state, value: Tile| {
            state.gpu_sprite.tile = [
                value.start_x,
                value.start_y,
                value.end_x,
                value.end_y,
            ];

            state.render_changed();
            BuilderChanged::Render
        },
    );

    simple_method!(
        /// Sets the [`RepeatTile`] which specifies how to repeat the sprite tile.
        repeat_tile,
        repeat_tile_signal,
        |state, value: RepeatTile| {
            state.repeat_tile = value;
            state.location_changed();
            BuilderChanged::Render
        },
    );

    simple_method!(
        /// Sets the palette for this sprite.
        palette,
        palette_signal,
        |state, value: u32| {
            state.gpu_palette = Some(GPUPalette {
                palette: value,
            });

            state.render_changed();
            BuilderChanged::Render
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
        self.render_changed = false;
        self.location_changed = false;

        if self.gpu_sprite.alpha != 0.0 {
            let smallest_size = smallest_size.real_size();

            self.parent_location = Some(*parent);
            self.smallest_size = Some(smallest_size);
            self.max_order = info.renderer.get_max_order();

            self.update_gpu(&info.screen_size);

            info.renderer.set_max_order(self.gpu_sprite.order);

            let spritesheet = self.spritesheet.as_ref().expect("Sprite is missing spritesheet");

            if let Some(spritesheet) = info.renderer.sprite.spritesheets.get_mut(&spritesheet.handle) {
                self.gpu_index = spritesheet.push(self.gpu_sprite, self.gpu_palette);
            }

            info.rendered_nodes.push(handle.clone());
        }
    }

    fn render<'a>(&mut self, info: &mut SceneRenderInfo<'a>) {
        assert_ne!(self.gpu_sprite.alpha, 0.0);

        if self.render_changed {
            self.render_changed = false;

            if self.location_changed {
                self.location_changed = false;

                self.update_gpu(&info.screen_size);
            }

            let spritesheet = self.spritesheet.as_ref().expect("Sprite is missing spritesheet");

            if let Some(spritesheet) = info.renderer.sprite.spritesheets.get_mut(&spritesheet.handle) {
                spritesheet.update(self.gpu_index, self.gpu_sprite, self.gpu_palette);
            }
        }
    }
}


pub(crate) struct SpritesheetPipeline {
    pub(crate) bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) opaque: wgpu::RenderPipeline,
    pub(crate) alpha: wgpu::RenderPipeline,
}

impl SpritesheetPipeline {
    pub(crate) fn new<'a>(
        engine: &crate::EngineState,
        scene_uniform_layout: &wgpu::BindGroupLayout,
        shader: wgpu::ShaderModuleDescriptor<'a>,
        vertex_buffers: &[wgpu::VertexBufferLayout],
        bind_group_layout: wgpu::BindGroupLayout
    ) -> Self {
        /*let stencil = wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::GreaterEqual,
            fail_op: wgpu::StencilOperation::Keep,
            depth_fail_op: wgpu::StencilOperation::Keep,
            pass_op: wgpu::StencilOperation::Replace,
        };*/

        // TODO lazy load this ?
        let shader = engine.device.create_shader_module(shader);

        let opaque = builders::Pipeline::builder()
            .label("Sprite")
            // TODO lazy load this ?
            .shader(&shader)
            .bind_groups(&[
                scene_uniform_layout,
                &bind_group_layout,
            ])
            .vertex_buffers(vertex_buffers)
            .topology(wgpu::PrimitiveTopology::TriangleStrip)
            .strip_index_format(wgpu::IndexFormat::Uint32)
            /*.stencil(wgpu::StencilState {
                front: stencil,
                back: stencil,
                read_mask: 0xFF,
                write_mask: 0xFF,
            })*/
            .build(engine);

        let alpha = builders::Pipeline::builder()
            .label("Sprite")
            .shader(&shader)
            .bind_groups(&[
                scene_uniform_layout,
                &bind_group_layout,
            ])
            .vertex_buffers(vertex_buffers)
            .topology(wgpu::PrimitiveTopology::TriangleStrip)
            .strip_index_format(wgpu::IndexFormat::Uint32)
            .depth_write(false)
            .blend_state(wgpu::BlendState::ALPHA_BLENDING)
            .build(engine);

        Self { bind_group_layout, opaque, alpha }
    }
}


pub(crate) static SCENE_SHADER: &'static str = include_str!("../wgsl/common/scene.wgsl");
pub(crate) static SPRITE_SHADER: &'static str = include_str!("../wgsl/common/sprite.wgsl");


struct SpritesheetInstances {
    sprites: InstanceVec<GPUSprite>,
    palettes: Option<InstanceVec<GPUPalette>>,
}

struct SpritesheetState {
    opaque: SpritesheetInstances,
    alpha: SpritesheetInstances,
    bind_group: wgpu::BindGroup,
}

impl SpritesheetState {
    fn instances(&mut self, sprite: &GPUSprite) -> &mut SpritesheetInstances {
        if sprite.alpha == 1.0 {
            &mut self.opaque

        } else {
            &mut self.alpha
        }
    }

    fn push(&mut self, sprite: GPUSprite, palette: Option<GPUPalette>) -> usize {
        let instances = self.instances(&sprite);

        let len = instances.sprites.len();

        instances.sprites.push(sprite);

        match &mut instances.palettes {
            Some(palettes) => {
                palettes.push(palette.expect("Sprite is missing palette"));
            },
            None => {
                assert!(palette.is_none(), "Spritesheet does not support palette")
            },
        }

        return len;
    }

    fn update(&mut self, index: usize, sprite: GPUSprite, palette: Option<GPUPalette>) {
        let instances = self.instances(&sprite);

        instances.sprites[index] = sprite;

        match &mut instances.palettes {
            Some(palettes) => {
                palettes[index] = palette.expect("Sprite is missing palette");
            },
            None => {
                assert!(palette.is_none(), "Spritesheet does not support palette")
            },
        }
    }

    pub(crate) fn prerender<'a>(
        &'a mut self,
        engine: &crate::EngineState,
        scene_uniform: &'a wgpu::BindGroup,
        normal: &'a SpritesheetPipeline,
        palette: &'a SpritesheetPipeline,
    ) -> (Prerender<'a>, Prerender<'a>) {
        let opaque = {
            let instances = self.opaque.sprites.len() as u32;

            if DEBUG {
                log::warn!("Spritesheet opaque {}", instances);
            }

            let bind_groups = vec![
                scene_uniform,
                &self.bind_group,
            ];

            let pipeline = if self.opaque.palettes.is_some() {
                &palette.opaque
            } else {
                &normal.opaque
            };

            let slices = vec![
                self.opaque.sprites.update_buffer(engine, &InstanceVecOptions {
                    label: Some("Sprite Instance Buffer"),
                }),

                self.opaque.palettes.as_mut().and_then(|palettes| {
                    palettes.update_buffer(engine, &InstanceVecOptions {
                        label: Some("Sprite Palettes Buffer"),
                    })
                }),
            ];

            Prerender {
                vertices: 4,
                instances,
                pipeline,
                bind_groups,
                slices,
            }
        };

        let alpha = {
            let instances = self.alpha.sprites.len() as u32;

            if DEBUG {
                log::warn!("Spritesheet alpha {}", instances);
            }

            let bind_groups = vec![
                scene_uniform,
                &self.bind_group,
            ];

            let pipeline = if self.alpha.palettes.is_some() {
                &palette.alpha
            } else {
                &normal.alpha
            };

            let slices = vec![
                self.alpha.sprites.update_buffer(engine, &InstanceVecOptions {
                    label: Some("Sprite Instance Buffer"),
                }),

                self.alpha.palettes.as_mut().and_then(|palettes| {
                    palettes.update_buffer(engine, &InstanceVecOptions {
                        label: Some("Sprite Palettes Buffer"),
                    })
                }),
            ];

            Prerender {
                vertices: 4,
                instances,
                pipeline,
                bind_groups,
                slices,
            }
        };

        (opaque, alpha)
    }
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
        let opaque = SpritesheetInstances {
            sprites: InstanceVec::new(),
            palettes: palette.map(|_| InstanceVec::new()),
        };

        let alpha = SpritesheetInstances {
            sprites: InstanceVec::new(),
            palettes: palette.map(|_| InstanceVec::new()),
        };

        let state = if let Some(palette) = palette {
            assert_eq!(texture.texture.format(), IndexedImage::FORMAT, "texture must be an IndexedImage");
            assert_eq!(palette.texture.format(), RgbaImage::FORMAT, "palette must be an RgbaImage");

            SpritesheetState {
                opaque,
                alpha,
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
                opaque,
                alpha,
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
            sheet.opaque.sprites.clear();

            if let Some(palettes) = &mut sheet.opaque.palettes {
                palettes.clear();
            }

            sheet.alpha.sprites.clear();

            if let Some(palettes) = &mut sheet.alpha.palettes {
                palettes.clear();
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
        prerender.opaques.reserve(self.spritesheets.len());
        prerender.alphas.reserve(self.spritesheets.len());

        for (_, sheet) in self.spritesheets.iter_mut() {
            let (opaque, alpha) = sheet.prerender(engine, scene_uniform, &self.normal, &self.palette);

            prerender.opaques.push(opaque);
            prerender.alphas.push(alpha);
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
