use std::borrow::Cow;
use wgpu_helpers::VertexLayout;
use bytemuck::{Pod, Zeroable};
use futures_signals::signal::{Signal, SignalExt};

use crate::{DEBUG, Engine, Handle};
use crate::util::unicode;
use crate::util::macros::wgsl;
use crate::util::buffer::{Uniform, InstanceVec, InstanceVecOptions, GrayscaleImage, TextureBuffer};
use crate::util::builders;
use crate::scene::builder::{Node, BuilderChanged, make_builder, base_methods, location_methods, simple_method};
use crate::scene::sprite::{GPUSprite, Tile, SpritesheetPipeline, SCENE_SHADER, SPRITE_SHADER};
use crate::scene::{
    NodeHandle, Location, Origin, Size, Offset, Padding, SmallestLength,
    RealLocation, NodeLayout, SceneLayoutInfo, SceneRenderInfo, Order,
    Length, Percentage, Handles, Prerender, Texture, SceneUniform,
    ScenePrerender, RealSize, ScreenSize, SmallestSize, RealPosition,
};


/// RGB color.
///
/// Each color channel is from 0.0 to 1.0
#[derive(Debug, Clone, Copy, Default)]
pub struct ColorRgb {
    pub r: Percentage,
    pub g: Percentage,
    pub b: Percentage,
}


/// Size of each character.
///
/// This is the size of a half-width Unicode character.
///
/// A full-width Unicode character will be double the width of the [`CharSize`].
///
/// # Sizing
///
/// * [`Length::ParentWidth`]: the width is relative to the node's width minus padding.
///
/// * [`Length::ParentHeight`]: the height is relative to the node's height minus padding.
///
/// * [`Length::SmallestWidth`]: it is an error to use `SmallestWidth`.
///
/// * [`Length::SmallestHeight`]: it is an error to use `SmallestHeight`.
#[derive(Debug, Default)]
pub struct CharSize {
    pub width: Length,
    pub height: Length,
}

impl CharSize {
    fn to_screen(&self, parent: &SmallestSize, screen: &ScreenSize) -> RealSize {
        let width = self.width.smallest_length(&screen.width).parent_to_screen(parent).unwrap();
        let height = self.height.smallest_length(&screen.height).parent_to_screen(parent).unwrap();
        RealSize { width, height }
    }
}


#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, VertexLayout, Default, PartialEq)]
#[layout(step_mode = Instance)]
#[layout(location = 6)]
pub(crate) struct GPUChar {
    pub(crate) color: [f32; 3],
}


struct Glyph {
    character: char,

    position: RealPosition,
    size: RealSize,

    gpu_sprite: GPUSprite,
    gpu_char: GPUChar,
}


/// Displays text which is stored in a spritesheet.
///
/// # Layout
///
/// Automatically wraps the text to the next line when
/// there isn't enough horizontal space.
///
/// # Sizing
///
/// * [`Length::SmallestWidth`]: the width of the text laid out on one line.
///
/// * [`Length::SmallestHeight`]: the height of the text (with wrapping).
pub struct BitmapText {
    // Standard fields
    visible: bool,
    location: Location,

    // Required fields
    font: Option<BitmapFont>,
    char_size: Option<CharSize>,

    // Optional fields
    text: Cow<'static, str>,
    text_color: ColorRgb,
    line_spacing: Length,

    // Internal state
    glyphs: Vec<Glyph>,
}

impl BitmapText {
    #[inline]
    fn new() -> Self {
        Self {
            visible: true,
            location: Location::default(),

            font: None,
            char_size: None,

            text: "".into(),
            text_color: ColorRgb::default(),
            line_spacing: Length::Zero,

            glyphs: vec![],
        }
    }

    fn layout_glyphs<'a>(&mut self, parent: &SmallestSize, max_width: Option<Percentage>, screen_size: &ScreenSize) -> RealSize {
        let char_size = self.char_size.as_ref().expect("BitmapText is missing char_size");
        let char_size = char_size.to_screen(parent, screen_size);

        let line_spacing = self.line_spacing.smallest_length(&screen_size.height).parent_to_screen(parent).unwrap();

        let line_height = char_size.height + line_spacing;

        let glyph_size = RealSize {
            width: 2.0 * char_size.width,
            height: char_size.height,
        };


        let mut position = RealPosition::zero();
        let mut size = RealSize::zero();

        if self.text == "" {
            debug_assert_eq!(self.glyphs.len(), 0);

        } else {
            for text_line in self.text.lines() {
                let mut width = 0.0;

                for grapheme in unicode::graphemes(text_line) {
                    // TODO figure out a way to avoid iterating over the characters twice
                    let unicode_width = grapheme.chars()
                        .map(unicode::char_width)
                        .max();

                    if let Some(unicode_width) = unicode_width {
                        let unicode_display_width = if unicode_width == 0 {
                            2

                        } else {
                            unicode_width
                        };

                        let max_char_width = (unicode_display_width as f32) * char_size.width;

                        width += max_char_width;

                        if width > max_char_width && max_width.map(|max_width| width > max_width).unwrap_or(false) {
                            width = max_char_width;
                            position.x = 0.0;
                            position.y += line_height;
                        }

                        let mut has_char = false;

                        for c in grapheme.chars() {
                            has_char = true;

                            let mut position = position;

                            position.x += unicode::char_offset(c, unicode_width) * char_size.width;

                            let mut gpu_sprite = GPUSprite::default();
                            let gpu_char = GPUChar::default();

                            gpu_sprite.uv = [1.0, 1.0];

                            self.glyphs.push(Glyph {
                                character: c,
                                position,
                                size: glyph_size,
                                gpu_sprite,
                                gpu_char,
                            });
                        }

                        position.x = width;

                        if has_char {
                            size.width = size.width.max(width);
                            size.height = size.height.max(position.y + char_size.height);
                        }
                    }
                }

                position.x = 0.0;
                position.y += line_height;
            }
        }

        size
    }

    fn calculate_glyphs<'a>(&mut self, parent: &SmallestSize, width: Percentage, screen_size: &ScreenSize) {
        if self.glyphs.is_empty() {
            let _ = self.layout_glyphs(parent, Some(width), screen_size);
        }
    }

    fn children_size<'a>(&mut self, parent: &SmallestSize, screen_size: &ScreenSize) -> RealSize {
        match parent.width {
            SmallestLength::Screen(width) => self.layout_glyphs(parent, Some(width), screen_size),
            SmallestLength::SmallestWidth(_) => self.layout_glyphs(parent, None, screen_size),
            SmallestLength::SmallestHeight(_) => panic!("BitmapText smallest height is unknown"),
            SmallestLength::ParentWidth(_) => panic!("BitmapText width is unknown"),
            SmallestLength::ParentHeight(_) => panic!("BitmapText height is unknown"),
        }
    }
}

make_builder!(BitmapText, BitmapTextBuilder);
base_methods!(BitmapText, BitmapTextBuilder);
location_methods!(BitmapText, BitmapTextBuilder);

impl BitmapTextBuilder {
    simple_method!(
        /// Sets the [`BitmapFont`] which will be used for this text.
        font,
        font_signal,
        |state, value: BitmapFont| {
            state.font = Some(value);
            BuilderChanged::Layout
        },
    );

    simple_method!(
        /// Sets the [`CharSize`] which specifies the width / height of each character.
        char_size,
        char_size_signal,
        |state, value: CharSize| {
            state.char_size = Some(value);
            BuilderChanged::Layout
        },
    );

    simple_method!(
        /// Sets the text which will be displayed.
        ///
        /// Defaults to "".
        text,
        text_signal,
        |state, value: Cow<'static, str>| {
            state.text = value;
            BuilderChanged::Layout
        },
    );

    simple_method!(
        /// Sets the [`ColorRgb`] which specifies the text's color.
        ///
        /// Defaults to `{ r: 0.0, g: 0.0, b: 0.0 }` (black).
        text_color,
        text_color_signal,
        |state, value: ColorRgb| {
            state.text_color = value;
            // TODO it should update the text color without needing to relayout
            BuilderChanged::Layout
        },
    );

    simple_method!(
        /// Sets the spacing between each line of text.
        ///
        /// Defaults to [`Length::Zero`] (no spacing).
        ///
        /// # Sizing
        ///
        /// * [`Length::ParentWidth`]: the width is relative to the node's width minus padding.
        ///
        /// * [`Length::ParentHeight`]: the height is relative to the node's height minus padding.
        ///
        /// * [`Length::SmallestWidth`]: it is an error to use `SmallestWidth`.
        ///
        /// * [`Length::SmallestHeight`]: it is an error to use `SmallestHeight`.
        line_spacing,
        line_spacing_signal,
        |state, value: Length| {
            state.line_spacing = value;
            BuilderChanged::Layout
        },
    );
}

impl NodeLayout for BitmapText {
    #[inline]
    fn is_visible(&mut self) -> bool {
        self.visible
    }

    fn smallest_size<'a>(&mut self, parent: &SmallestSize, info: &mut SceneLayoutInfo<'a>) -> SmallestSize {
        assert_eq!(self.glyphs.len(), 0);

        let smallest_size = self.location.size.smallest_size(&info.screen_size).parent_to_smallest(parent);

        if smallest_size.is_smallest() {
            let padding = self.location.padding.to_screen(parent, &smallest_size, &info.screen_size);

            smallest_size.with_padding(parent, padding, |parent| {
                self.children_size(&parent, &info.screen_size)
            })

        } else {
            smallest_size
        }
    }

    fn update_layout<'a>(&mut self, handle: &NodeHandle, parent: &RealLocation, smallest_size: &SmallestSize, info: &mut SceneLayoutInfo<'a>) {
        let max_order = info.renderer.get_max_order();

        let font = self.font.as_ref().expect("BitmapText is missing font");

        if let Some(font) = info.renderer.bitmap_text.fonts.get_mut(&font.handle) {
            let this_location = self.location.children_location_explicit(parent, &smallest_size.real_size(), &info.screen_size, max_order);

            // If it has a fixed size then we need to calculate the glyphs.
            self.calculate_glyphs(&this_location.size.smallest_size(), this_location.size.width, &info.screen_size);

            if !self.glyphs.is_empty() {
                for glyph in self.glyphs.iter_mut() {
                    let character = font.supported.replace(glyph.character);

                    // Always display the full width tile
                    let tile = font.tile(character, 2);

                    let char_location = RealLocation {
                        position: this_location.position + glyph.position,
                        size: glyph.size,
                        order: this_location.order,
                    };

                    glyph.gpu_sprite.update(&char_location);
                    glyph.gpu_sprite.tile = [tile.start_x, tile.start_y, tile.end_x, tile.end_y];

                    glyph.gpu_char.color = [self.text_color.r, self.text_color.g, self.text_color.b];

                    font.sprites.push(glyph.gpu_sprite);
                    font.chars.push(glyph.gpu_char);
                }

                info.rendered_nodes.push(handle.clone());
                info.renderer.set_max_order(this_location.order);
            }
        }

        self.glyphs.clear();
    }

    #[inline]
    fn render<'a>(&mut self, _info: &mut SceneRenderInfo<'a>) {}
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
        prerender.opaques.reserve(self.fonts.len());

        for (_, font) in self.fonts.iter_mut() {
            let instances = font.sprites.len() as u32;

            if DEBUG {
                log::warn!("BitmapText {}", instances);
            }

            let bind_groups = vec![
                scene_uniform,
                &font.bind_group,
            ];

            let pipeline = &self.pipeline.opaque;

            let slices = vec![
                font.sprites.update_buffer(engine, &InstanceVecOptions {
                    label: Some("BitmapText sprites"),
                }),

                font.chars.update_buffer(engine, &InstanceVecOptions {
                    label: Some("BitmapText chars"),
                }),
            ];

            prerender.opaques.push(Prerender {
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

    pub fn load<'a>(&self, engine: &mut Engine, settings: BitmapFontSettings<'a>) {
        let texture = engine.scene.textures.get(&settings.texture.handle)
            .expect("BitmapFontSettings texture is not loaded");

        assert_eq!(texture.texture.format(), GrayscaleImage::FORMAT, "BitmapFontSettings texture must be a GrayscaleImage");

        engine.scene.renderer.bitmap_text.new_font(&engine.state, &self.handle, texture, settings);

        // TODO test this
        engine.scene.changed.trigger_layout_change();
    }

    pub fn unload(&self, engine: &mut Engine) {
        engine.scene.renderer.bitmap_text.remove_font(&self.handle);

        // TODO test this
        engine.scene.changed.trigger_layout_change();
    }
}
