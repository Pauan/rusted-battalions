use std::borrow::Cow;
use wgpu_helpers::VertexLayout;
use bytemuck::{Pod, Zeroable};
use futures_signals::signal::{Signal, SignalExt};

use crate::{Engine, Handle};
use crate::util::unicode;
use crate::util::macros::wgsl;
use crate::util::buffer::{Uniform, InstanceVec, InstanceVecOptions, GrayscaleImage, TextureBuffer};
use crate::util::builders;
use crate::scene::builder::{Node, make_builder, base_methods, location_methods, simple_method};
use crate::scene::sprite::{GPUSprite, Tile, SpritesheetPipeline, SCENE_SHADER, SPRITE_SHADER};
use crate::scene::{
    NodeHandle, MinSize, Location, Origin, Size, Offset, Padding,
    ScreenSpace, NodeLayout, SceneLayoutInfo, SceneRenderInfo,
    Length, Percentage, Handles, Prerender, Texture, SceneUniform,
    ScenePrerender,
};


/// Each color channel is from 0.0 to 1.0
#[derive(Debug, Clone, Copy, Default)]
pub struct ColorRgb {
    pub r: Percentage,
    pub g: Percentage,
    pub b: Percentage,
}


pub struct CharSize {
    pub width: Length,
    pub height: Length,
}


#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, VertexLayout, Default, PartialEq)]
#[layout(step_mode = Instance)]
#[layout(location = 4)]
pub(crate) struct GPUChar {
    pub(crate) color: [f32; 3],
}


/// Displays text which is stored in a spritesheet.
pub struct BitmapText {
    // Standard fields
    visible: bool,
    stretch: bool,
    location: Location,

    // Required fields
    font: Option<BitmapFont>,
    char_size: Option<CharSize>,

    // Optional fields
    text: Cow<'static, str>,
    text_color: ColorRgb,
    line_spacing: Length,

    // Internal state
    z_index: f32,
    min_size: Option<MinSize>,
}

impl BitmapText {
    #[inline]
    fn new() -> Self {
        Self {
            visible: true,
            stretch: false,
            location: Location::default(),

            font: None,
            char_size: None,

            text: "".into(),
            text_color: ColorRgb::default(),
            line_spacing: Length::Parent(0.0),

            z_index: 0.0,
            min_size: None,
        }
    }
}

make_builder!(BitmapText, BitmapTextBuilder);
base_methods!(BitmapText, BitmapTextBuilder);
location_methods!(BitmapText, BitmapTextBuilder, true);

impl BitmapTextBuilder {
    simple_method!(
        /// Sets the [`BitmapFont`] which will be used for this text.
        font,
        font_signal,
        true,
        true,
        |state, value: BitmapFont| {
            state.font = Some(value);
        },
    );

    simple_method!(
        /// Sets the [`CharSize`] which specifies the width / height of each character.
        char_size,
        char_size_signal,
        true,
        true,
        |state, value: CharSize| {
            state.char_size = Some(value);
        },
    );

    simple_method!(
        /// Sets the text which will be displayed.
        ///
        /// Defaults to "".
        text,
        text_signal,
        true,
        true,
        |state, value: Cow<'static, str>| {
            state.text = value;
        },
    );

    simple_method!(
        /// Sets the [`ColorRgb`] which specifies the text's color.
        ///
        /// Defaults to `{ r: 0.0, g: 0.0, b: 0.0 }`.
        text_color,
        text_color_signal,
        true,
        true,
        |state, value: ColorRgb| {
            state.text_color = value;
        },
    );

    simple_method!(
        /// Sets the spacing between each line of text.
        ///
        /// Defaults to `Length::Parent(0.0)` (no spacing).
        line_spacing,
        line_spacing_signal,
        true,
        true,
        |state, value: Length| {
            state.line_spacing = value;
        },
    );
}

impl NodeLayout for BitmapText {
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
        let font = self.font.as_ref().expect("BitmapText is missing font");
        let char_size = self.char_size.as_ref().expect("BitmapText is missing char_size");

        if let Some(font) = info.renderer.bitmap_text.fonts.get_mut(&font.handle) {
            let this_space = parent.modify(&self.location, &info.screen_size);

            self.z_index = this_space.z_index;

            let char_width = char_size.width.to_screen_space(this_space.size[0], info.screen_size.width);
            let char_height = char_size.height.to_screen_space(this_space.size[1], info.screen_size.height);
            let line_spacing = self.line_spacing.to_screen_space(this_space.size[1], info.screen_size.height);

            let line_height = char_height + line_spacing;

            let max_width = this_space.size[0];

            let mut char_space = this_space;

            for line in self.text.lines() {
                let mut width = 0.0;

                for grapheme in unicode::graphemes(line) {
                    // TODO figure out a way to avoid iterating over the characters twice
                    let unicode_width = grapheme.chars()
                        .map(|c| font.supported.replace(c))
                        .map(unicode::char_width)
                        .max();

                    if let Some(unicode_width) = unicode_width {
                        let unicode_display_width = if unicode_width == 0 {
                            2

                        } else {
                            unicode_width
                        };

                        let max_char_width = (unicode_display_width as f32) * char_width;

                        width += max_char_width;

                        if width > max_char_width && width > max_width {
                            width = max_char_width;
                            char_space.position[0] = this_space.position[0];
                            char_space.move_down(line_height);
                        }

                        char_space.size = [2.0 * char_width, char_height];

                        for c in grapheme.chars() {
                            let c = font.supported.replace(c);

                            let mut char_space = char_space;

                            char_space.move_right(unicode::char_offset(c, unicode_width) * char_width);

                            let mut gpu_sprite = GPUSprite::default();
                            let mut gpu_char = GPUChar::default();

                            gpu_sprite.update(&char_space);

                            // Always display the full width tile
                            let tile = font.tile(c, 2);
                            gpu_sprite.tile = [tile.start_x, tile.start_y, tile.end_x, tile.end_y];

                            gpu_char.color = [self.text_color.r, self.text_color.g, self.text_color.b];

                            font.sprites.push(gpu_sprite);
                            font.chars.push(gpu_char);
                        }

                        char_space.position[0] = this_space.position[0] + width;
                    }
                }

                char_space.position[0] = this_space.position[0];
                char_space.move_down(line_height);
            }

            info.renderer.set_max_z_index(self.z_index);
            info.rendered_nodes.push(handle.clone());
        }

        self.min_size = None;
    }

    #[inline]
    fn render<'a>(&mut self, info: &mut SceneRenderInfo<'a>) {
        info.renderer.set_max_z_index(self.z_index);
    }
}


struct BitmapFontState {
    columns: u32,
    tile_width: u32,
    tile_height: u32,
    supported: BitmapFontSupported,
    sprites: InstanceVec<GPUSprite>,
    chars: InstanceVec<GPUChar>,
    bind_group: wgpu::BindGroup,
}

impl BitmapFontState {
    fn tile(&self, c: char, width: u32) -> Tile {
        let index = c as u32;

        let row = index / self.columns;
        let column = index - (row * self.columns);

        let start_x = column * (self.tile_width * 2);
        let start_y = row * self.tile_height;

        Tile {
            start_x,
            start_y,
            end_x: start_x + (self.tile_width * width),
            end_y: start_y + self.tile_height,
        }
    }
}


pub(crate) struct BitmapTextRenderer {
    pipeline: SpritesheetPipeline,

    fonts: Handles<BitmapFontState>,
}

impl BitmapTextRenderer {
    #[inline]
    pub(crate) fn new(engine: &crate::EngineState, scene_uniform: &mut Uniform<SceneUniform>) -> Self {
        let scene_uniform_layout = Uniform::bind_group_layout(scene_uniform, engine);

        let pipeline = SpritesheetPipeline::new(
            engine,
            scene_uniform_layout,

            // TODO lazy load this ?
            wgsl![
                "spritesheet/text.wgsl",
                SCENE_SHADER,
                SPRITE_SHADER,
                include_str!("../wgsl/spritesheet/text.wgsl"),
            ],

            &[GPUSprite::LAYOUT, GPUChar::LAYOUT],

            builders::BindGroupLayout::builder()
                .label("BitmapText")
                .texture(wgpu::ShaderStages::FRAGMENT, wgpu::TextureSampleType::Uint)
                .build(engine),
        );

        Self {
            pipeline,
            fonts: Handles::new(),
        }
    }

    fn new_font<'a>(
        &mut self,
        engine: &crate::EngineState,
        handle: &Handle,
        texture: &TextureBuffer,
        settings: BitmapFontSettings<'a>,
    ) {
        self.fonts.insert(handle, BitmapFontState {
            columns: settings.columns,
            tile_width: settings.tile_width,
            tile_height: settings.tile_height,
            supported: settings.supported,
            sprites: InstanceVec::new(),
            chars: InstanceVec::new(),
            bind_group: builders::BindGroup::builder()
                .label("BitmapText")
                .layout(&self.pipeline.bind_group_layout)
                .texture_view(&texture.view)
                .build(engine),
        });
    }

    fn remove_font(&mut self, handle: &Handle) {
        self.fonts.remove(handle);
    }

    #[inline]
    pub(crate) fn before_layout(&mut self) {
        for (_, font) in self.fonts.iter_mut() {
            font.sprites.clear();
            font.chars.clear();
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
        prerender.prerenders.reserve(self.fonts.len());

        for (_, font) in self.fonts.iter_mut() {
            let instances = font.sprites.len() as u32;

            let bind_groups = vec![
                scene_uniform,
                &font.bind_group,
            ];

            let pipeline = &self.pipeline.pipeline;

            let slices = vec![
                font.sprites.update_buffer(engine, &InstanceVecOptions {
                    label: Some("BitmapText sprites"),
                }),

                font.chars.update_buffer(engine, &InstanceVecOptions {
                    label: Some("BitmapText chars"),
                }),
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


pub struct BitmapFontSupported {
    pub start: char,
    pub end: char,
    pub replace: char,
}

impl BitmapFontSupported {
    fn replace(&self, c: char) -> char {
        if c < self.start || c > self.end {
            self.replace

        } else {
            c
        }
    }
}


pub struct BitmapFontSettings<'a> {
    pub texture: &'a Texture,
    pub supported: BitmapFontSupported,
    pub columns: u32,
    pub tile_width: u32,
    pub tile_height: u32,
}

#[derive(Clone)]
pub struct BitmapFont {
    pub(crate) handle: Handle,
}

impl BitmapFont {
    #[inline]
    pub fn new() -> Self {
        Self { handle: Handle::new() }
    }

    pub fn load<'a, Window>(&self, engine: &mut Engine<Window>, settings: BitmapFontSettings<'a>) {
        let texture = engine.scene.textures.get(&settings.texture.handle)
            .expect("BitmapFontSettings texture is not loaded");

        assert_eq!(texture.texture.format(), GrayscaleImage::FORMAT, "BitmapFontSettings texture must be a GrayscaleImage");

        engine.scene.renderer.bitmap_text.new_font(&engine.state, &self.handle, texture, settings);

        // TODO test this
        engine.scene.changed.trigger_layout_change();
    }

    pub fn unload<Window>(&self, engine: &mut Engine<Window>) {
        engine.scene.renderer.bitmap_text.remove_font(&self.handle);

        // TODO test this
        engine.scene.changed.trigger_layout_change();
    }
}
