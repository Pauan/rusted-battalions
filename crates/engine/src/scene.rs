use bytemuck::{Zeroable, Pod};
use std::future::Future;
use std::pin::Pin;

use crate::Spawner;
use crate::util::{Arc, Atomic, Lock};
use crate::util::buffer::{Uniform, TextureBuffer, IntoTexture};
use sprite::{SpriteRenderer};
use bitmap_text::{BitmapTextRenderer};

mod builder;
mod sprite;
mod row;
mod column;
mod stack;
mod wrap;
mod grid;
mod border_grid;
mod bitmap_text;

pub use builder::{Node};
pub use sprite::{Sprite, SpriteBuilder, Spritesheet, SpritesheetSettings, Tile};
pub use row::{Row, RowBuilder};
pub use column::{Column, ColumnBuilder};
pub use stack::{Stack, StackBuilder};
pub use wrap::{Wrap, WrapBuilder};
pub use grid::{Grid, GridBuilder, GridSize};
pub use border_grid::{BorderGrid, BorderGridBuilder, BorderSize, Quadrants};
pub use bitmap_text::{
    BitmapText, BitmapTextBuilder, BitmapFont, BitmapFontSettings,
    BitmapFontSupported, ColorRgb, CharSize,
};


/// f32 from 0.0 to 1.0
pub type Percentage = f32;


/// x / y in screen space, percentage of the screen size
#[derive(Debug, Clone, Copy)]
pub(crate) struct RealPosition {
    pub(crate) x: Percentage,
    pub(crate) y: Percentage,
}


/// width / height in screen space, percentage of the screen size
#[derive(Debug, Clone, Copy)]
pub(crate) struct RealSize {
    pub(crate) width: Percentage,
    pub(crate) height: Percentage,
}


/// The x / y / width / height / z-index in screen space.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RealLocation {
    pub(crate) position: RealPosition,
    pub(crate) size: RealSize,
    pub(crate) z_index: f32,
}

impl RealLocation {
    /// Returns a [`RealLocation`] that covers the entire screen.
    pub(crate) fn full() -> Self {
         Self {
            position: RealPosition {
                x: 0.0,
                y: 0.0,
            },
            size: RealSize {
                width: 1.0,
                height: 1.0,
            },
            z_index: 1.0,
        }
    }

    /// Calculates the new [`RealLocation`] based on the [`Location`].
    ///
    /// Assume that `self` is the parent, and `location` is the child.
    pub(crate) fn modify(&self, location: &Location, screen: &ScreenSize) -> Self {
        let screen_width = screen.to_real_width();
        let screen_height = screen.to_real_height();

        let pad_up = location.padding.up.to_screen_space(self.size, screen_height, screen.height);
        let pad_down = location.padding.down.to_screen_space(self.size, screen_height, screen.height);
        let pad_left = location.padding.left.to_screen_space(self.size, screen_width, screen.width);
        let pad_right = location.padding.right.to_screen_space(self.size, screen_width, screen.width);

        let width = location.size.width.to_screen_space(self.size, screen_width, screen.width);
        let height = location.size.height.to_screen_space(self.size, screen_height, screen.height);

        let x = location.offset.x.to_screen_space(self.size, screen_width, screen.width);
        let y = location.offset.y.to_screen_space(self.size, screen_height, screen.height);

        let origin_x = (self.size.width - width) * location.origin.x;
        let origin_y = (self.size.height - height) * location.origin.y;

        Self {
            position: RealPosition {
                x: self.position.x + origin_x + pad_left + x,
                y: self.position.y + origin_y + pad_up + y,
            },
            size: RealSize {
                width: (width - pad_left - pad_right).max(0.0),
                height: (height - pad_up - pad_down).max(0.0),
            },
            z_index: self.z_index + location.z_index,
        }
    }

    /// Shifts the position to the right.
    #[inline]
    pub(crate) fn move_right(&mut self, amount: f32) {
        self.position.x += amount;
    }

    /// Shifts the position down.
    #[inline]
    pub(crate) fn move_down(&mut self, amount: f32) {
        self.position.y += amount;
    }

    /// Converts from our coordinate system into wgpu's coordinate system.
    ///
    /// Our coordinate system looks like this:
    ///
    ///   [0 0       1 0]
    ///   |             |
    ///   |   0.5 0.5   |
    ///   |             |
    ///   [0 1       1 1]
    ///
    /// However, wgpu uses a coordinate system that looks like this:
    ///
    ///   [-1  1    1  1]
    ///   |             |
    ///   |     0  0    |
    ///   |             |
    ///   [-1 -1    1 -1]
    ///
    pub(crate) fn convert_to_wgpu_coordinates(&self) -> Self {
        let width  = self.size.width * 2.0;
        let height = self.size.height * 2.0;

        let x = (self.position.x *  2.0) - 1.0;
        let y = (self.position.y * -2.0) + 1.0;

        Self {
            position: RealPosition { x, y },
            size: RealSize { width, height },
            z_index: self.z_index,
        }
    }
}


/// The minimum width / height for the Node, in screen space.
///
/// The maximum size might be higher than this.
///
/// In the case of Row / Colunm, this might also include the
/// width / height for the Node's children.
#[derive(Debug, Clone, Copy)]
pub(crate) struct MinSize {
    pub(crate) width: Percentage,
    pub(crate) height: Percentage,
}

impl std::ops::Add for MinSize {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self {
            width: self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}

impl MinSize {
    #[inline]
    pub(crate) fn max(self, other: Self) -> Self {
        Self {
            width: self.width.max(other.width),
            height: self.height.max(other.height),
        }
    }

    #[inline]
    pub(crate) fn max_width(self, other: Self) -> Self {
        Self {
            width: self.width.max(other.width),
            height: self.height,
        }
    }

    #[inline]
    pub(crate) fn max_height(self, other: Self) -> Self {
        Self {
            width: self.width,
            height: self.height.max(other.height),
        }
    }
}


/// Empty space around the node.
///
/// The padding does not increase the size, instead the
/// empty space is created by subtracting from the size.
///
/// The default is no padding.
#[derive(Debug, Clone, Copy)]
pub struct Padding {
    pub up: Length,
    pub down: Length,
    pub left: Length,
    pub right: Length,
}

impl Padding {
    /// Returns the minimum size of the padding.
    ///
    /// It uses `parent_size` for the parent width / height.
    // TODO should this handle negative padding ?
    pub(crate) fn min_size(&self, parent_size: &MinSize, screen: &ScreenSize) -> MinSize {
        let screen_width = screen.to_real_width();
        let screen_height = screen.to_real_height();

        let parent = RealSize {
            width: parent_size.width,
            height: parent_size.height,
        };

        let width =
            self.left.to_screen_space(parent, screen_width, screen.width) +
            self.right.to_screen_space(parent, screen_width, screen.width);

        let height =
            self.up.to_screen_space(parent, screen_height, screen.height) +
            self.down.to_screen_space(parent, screen_height, screen.height);

        MinSize { width, height }
    }
}

impl Default for Padding {
    fn default() -> Self {
        Self {
            up: Length::Zero,
            down: Length::Zero,
            left: Length::Zero,
            right: Length::Zero,
        }
    }
}


pub use Length::{Zero, Px, ScreenWidth, ScreenHeight, ParentWidth, ParentHeight};

/// Used for [`Offset`] / [`Size`] / [`Padding`].
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum Length {
    /// Zero length. Useful for [`Offset`] and [`Padding`].
    ///
    /// This is fixed, it will always be the same length.
    Zero,

    /// Pixel length.
    ///
    /// This is fixed, it will always be the same length.
    Px(i32),

    /// Percentage of the screen's width.
    ///
    /// This is fixed, it will always be the same length.
    ScreenWidth(Percentage),

    /// Percentage of the screen's height.
    ///
    /// This is fixed, it will always be the same length.
    ScreenHeight(Percentage),

    /// Percentage of the parent's width.
    ///
    /// This will dynamically change if the parent's width changes.
    ParentWidth(Percentage),

    /// Percentage of the parent's height.
    ///
    /// This will dynamically change if the parent's height changes.
    ParentHeight(Percentage),
}

impl Length {
    fn is_fixed(&self) -> bool {
        match self {
            Self::Zero => true,
            Self::Px(_) => true,
            Self::ScreenWidth(_) | Self::ScreenHeight(_) => true,
            Self::ParentWidth(_) | Self::ParentHeight(_) => false,
        }
    }

    /// Minimum length in screen space.
    #[inline]
    fn min_length(&self, screen: RealSize, pixels: f32) -> Percentage {
        let parent = RealSize {
            width: 0.0,
            height: 0.0,
        };

        self.to_screen_space(parent, screen, pixels)
    }

    /// Converts from local space into screen space.
    fn to_screen_space(&self, parent: RealSize, screen: RealSize, pixels: f32) -> Percentage {
        match self {
            Self::Zero => 0.0,
            Self::ScreenWidth(x) => x * screen.width,
            Self::ScreenHeight(x) => x * screen.height,
            Self::ParentWidth(x) => x * parent.width,
            Self::ParentHeight(x) => x * parent.height,
            Self::Px(x) => *x as Percentage / pixels,
        }
    }
}

impl Default for Length {
    /// Returns [`Length::Zero`].
    #[inline]
    fn default() -> Self {
        Self::Zero
    }
}


/// Offset x / y (relative to the parent) which is added to the parent's x / y.
///
/// The default is `{ x: Zero, y: Zero }` which means no offset.
#[derive(Debug, Clone, Copy)]
pub struct Offset {
    pub x: Length,
    pub y: Length,
}

impl Default for Offset {
    #[inline]
    fn default() -> Self {
        Self {
            x: Length::Zero,
            y: Length::Zero,
        }
    }
}


/// Width / height relative to the parent.
///
/// The default is `{ width: ParentWidth(1.0), height: ParentHeight(1.0) }`
/// which means it's the same size as its parent.
#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: Length,
    pub height: Length,
}

impl Default for Size {
    #[inline]
    fn default() -> Self {
        Self {
            width: Length::ParentWidth(1.0),
            height: Length::ParentHeight(1.0),
        }
    }
}


/// Position relative to the parent.
///
/// By default, the origin is `{ x: 0.0, y: 0.0 }` which means that it will be
/// positioned in the upper-left corner of the parent.
///
/// But if you change it to `{ x: 1.0, y: 1.0 }` then it will now be positioned
/// in the lower-right corner of the parent.
///
/// And `{ x: 0.5, y: 0.5 }` will place it in the center of the parent.
#[derive(Debug, Clone, Copy)]
pub struct Origin {
    pub x: Percentage,
    pub y: Percentage,
}

impl Default for Origin {
    #[inline]
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}


/// Describes the position of the Node relative to its parent.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Location {
    /// Offset which is added to the Node's position.
    pub(crate) offset: Offset,

    /// Width / height relative to the parent.
    pub(crate) size: Size,

    /// Empty space in the cardinal directions.
    pub(crate) padding: Padding,

    /// Origin point for the Node relative to the parent.
    pub(crate) origin: Origin,

    /// Z-index relative to parent.
    pub(crate) z_index: f32,
}

impl Location {
    // TODO should this take into account the offset and origin as well ?
    // TODO should this take into account negative padding ?
    pub(crate) fn min_size(&self, screen: &ScreenSize) -> MinSize {
        let screen_width = screen.to_real_width();
        let screen_height = screen.to_real_height();

        let width = self.size.width.min_length(screen_width, screen.width);
        let height = self.size.height.min_length(screen_height, screen.height);

        MinSize { width, height }
    }
}


/// The width / height of the screen in pixels.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ScreenSize {
    pub(crate) width: f32,
    pub(crate) height: f32,
}

impl ScreenSize {
    /// When calculating the [`Length::ScreenHeight`] it
    /// needs to scale it based on the ratio of height / width.
    #[inline]
    pub(crate) fn to_real_width(&self) -> RealSize {
        RealSize {
            width: 1.0,
            height: self.height / self.width,
        }
    }

    /// When calculating the [`Length::ScreenWidth`] it
    /// needs to scale it based on the ratio of width / height.
    #[inline]
    pub(crate) fn to_real_height(&self) -> RealSize {
        RealSize {
            width: self.width / self.height,
            height: 1.0,
        }
    }
}


/// Temporary state used for rerendering
pub(crate) struct SceneRenderInfo<'a> {
    /// Screen size in pixels.
    pub(crate) screen_size: &'a ScreenSize,

    /// Renderer-specific state.
    pub(crate) renderer: &'a mut SceneRenderer,
}

/// Temporary state used for relayout
pub(crate) struct SceneLayoutInfo<'a> {
    /// Screen size in pixels.
    pub(crate) screen_size: &'a ScreenSize,

    /// Renderer-specific state.
    pub(crate) renderer: &'a mut SceneRenderer,

    /// Nodes which can be rendered without relayout.
    pub(crate) rendered_nodes: &'a mut Vec<NodeHandle>,
}


pub(crate) trait NodeLayout {
    /// Whether the Node is visible or not.
    fn is_visible(&mut self) -> bool;

    /// Whether the Node should stretch to fill the available space in the parent.
    ///
    /// If the Node is invisible then this method MUST NOT be called.
    fn is_stretch(&mut self) -> bool;

    /// Returns the minimum size in screen space.
    ///
    /// If the Node is invisible then this method MUST NOT be called.
    fn min_size<'a>(&mut self, info: &mut SceneLayoutInfo<'a>) -> MinSize;

    /// Does re-layout AND re-render on the Node.
    ///
    /// If the Node is invisible then this method MUST NOT be called.
    ///
    /// If the Node is visible then update_layout MUST be called.
    ///
    /// The handle must be the same as this NodeLayout.
    fn update_layout<'a>(&mut self, handle: &NodeHandle, parent: &RealLocation, info: &mut SceneLayoutInfo<'a>);

    /// Re-renders the Node.
    ///
    /// This must only be called if the layout has NOT changed.
    ///
    /// This must only be called if the Node is visible.
    fn render<'a>(&mut self, info: &mut SceneRenderInfo<'a>);
}


/// Type-erased handle to a NodeLayout.
///
/// It uses an Arc so it can be cheaply cloned and passed around.
///
/// You can call `handle.lock()` to get access to the NodeLayout.
#[derive(Clone)]
pub(crate) struct NodeHandle {
    pub(crate) layout: Lock<dyn NodeLayout>,
}

impl std::ops::Deref for NodeHandle {
    type Target = Lock<dyn NodeLayout>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.layout
    }
}


#[derive(Clone)]
#[repr(transparent)]
pub(crate) struct Handle {
    ptr: Arc<()>,
}

impl Handle {
    pub(crate) fn new() -> Self {
        Self {
            ptr: Arc::new(()),
        }
    }

    #[inline]
    pub(crate) fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.ptr, &other.ptr)
    }
}


/// Container for looking up a `T` value based on a [`Handle`].
#[repr(transparent)]
pub(crate) struct Handles<T> {
    values: Vec<(Handle, T)>,
}

impl<T> Handles<T> {
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            values: vec![],
        }
    }

    #[inline]
    fn index(&self, handle: &Handle) -> Option<usize> {
        self.values.iter().position(|(x, _)| x.eq(handle))
    }

    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.values.len()
    }

    pub(crate) fn get(&self, handle: &Handle) -> Option<&T> {
        self.values.iter().find_map(|(x, value)| {
            if x.eq(handle) {
                Some(value)

            } else {
                None
            }
        })
    }

    pub(crate) fn get_mut(&mut self, handle: &Handle) -> Option<&mut T> {
        self.values.iter_mut().find_map(|(x, value)| {
            if x.eq(handle) {
                Some(value)

            } else {
                None
            }
        })
    }

    #[inline]
    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut (Handle, T)> {
        self.values.iter_mut()
    }

    pub(crate) fn insert(&mut self, handle: &Handle, value: T) -> Option<T> {
        let index = self.index(&handle);

        if let Some(index) = index {
            let old_value = std::mem::replace(&mut self.values[index].1, value);
            Some(old_value)

        } else {
            self.values.push((handle.clone(), value));
            None
        }
    }

    pub(crate) fn remove(&mut self, handle: &Handle) -> Option<T> {
        let index = self.index(&handle);

        if let Some(index) = index {
            Some(self.values.swap_remove(index).1)

        } else {
            None
        }
    }
}



#[derive(Clone)]
pub struct Texture {
    pub(crate) handle: Handle,
}

impl Texture {
    #[inline]
    pub fn new() -> Self {
        Self { handle: Handle::new() }
    }

    pub fn load<Window, T>(&self, engine: &mut crate::Engine<Window>, image: &T) where T: IntoTexture {
        let buffer = TextureBuffer::new(&engine.state, image);

        engine.scene.textures.insert(&self.handle, buffer);

        // TODO maybe this should trigger a relayout ?
        // TODO somehow update the existing Spritesheets which refer to this texture
        engine.scene.changed.trigger_render_change();
    }

    pub fn unload<Window>(&self, engine: &mut crate::Engine<Window>) {
        engine.scene.textures.remove(&self.handle);

        // TODO maybe this should trigger a relayout ?
        // TODO somehow update the existing Spritesheets which refer to this texture
        engine.scene.changed.trigger_render_change();
    }
}


/// Keeps track of whether the layout / render needs updating.
pub(crate) struct SceneChanged {
    layout: Atomic<bool>,
    render: Atomic<bool>,
    spawner: std::sync::Arc<dyn Spawner>,
}

impl SceneChanged {
    #[inline]
    fn new(spawner: std::sync::Arc<dyn Spawner>) -> Arc<Self> {
        Arc::new(Self {
            layout: Atomic::new(true),
            render: Atomic::new(true),
            spawner,
        })
    }

    #[inline]
    pub(crate) fn spawn_local(&self, future: Pin<Box<dyn Future<Output = ()> + 'static>>) {
        self.spawner.spawn_local(future);
    }

    /// Notifies that the layout has changed.
    #[inline]
    pub(crate) fn trigger_layout_change(&self) {
        self.layout.set(true);
        self.trigger_render_change();
    }

    /// Notifies that the rendering has changed.
    #[inline]
    pub(crate) fn trigger_render_change(&self) {
        self.render.set(true);
    }

    #[inline]
    fn is_render_changed(&self) -> bool {
        self.render.get()
    }

    #[inline]
    fn replace_layout_changed(&self) -> bool {
        self.layout.replace(false)
    }

    #[inline]
    fn replace_render_changed(&self) -> bool {
        self.render.replace(false)
    }
}


pub(crate) struct Prerender<'a> {
    pub(crate) vertices: u32,
    pub(crate) instances: u32,
    pub(crate) pipeline: &'a wgpu::RenderPipeline,
    // TODO figure out a way to avoid the Vec
    pub(crate) bind_groups: Vec<&'a wgpu::BindGroup>,
    pub(crate) slices: Vec<Option<wgpu::BufferSlice<'a>>>,
}

impl<'a> Prerender<'a> {
    fn render<'b>(&'a mut self, render_pass: &mut wgpu::RenderPass<'b>) where 'a: 'b {
        if self.instances > 0 {
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

            render_pass.draw(0..self.vertices, 0..self.instances);
        }
    }
}

pub(crate) struct ScenePrerender<'a> {
    pub(crate) prerenders: Vec<Prerender<'a>>,
}

impl<'a> ScenePrerender<'a> {
    #[inline]
    fn new() -> Self {
        Self { prerenders: vec![] }
    }

    /// Does the actual rendering, using the prepared data.
    /// The lifetimes are necessary in order to make it work with wgpu::RenderPass.
    #[inline]
    pub(crate) fn render<'b>(&'a mut self, render_pass: &mut wgpu::RenderPass<'b>) where 'a: 'b {
        for prerender in self.prerenders.iter_mut() {
            prerender.render(render_pass);
        }
    }
}


#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, Default)]
pub(crate) struct SceneUniform {
    pub(crate) max_z_index: f32,
    _padding1: f32,
    _padding2: f32,
    _padding3: f32,
}

pub(crate) struct SceneRenderer {
    pub(crate) scene_uniform: Uniform<SceneUniform>,
    pub(crate) sprite: SpriteRenderer,
    pub(crate) bitmap_text: BitmapTextRenderer,
}

impl SceneRenderer {
    #[inline]
    fn new(engine: &crate::EngineState) -> Self {
        let mut scene_uniform = Uniform::new(wgpu::ShaderStages::VERTEX, SceneUniform {
            max_z_index: 1.0,
            _padding1: 0.0,
            _padding2: 0.0,
            _padding3: 0.0,
        });

        Self {
            sprite: SpriteRenderer::new(engine, &mut scene_uniform),
            bitmap_text: BitmapTextRenderer::new(engine, &mut scene_uniform),
            scene_uniform,
        }
    }

    #[inline]
    pub(crate) fn get_max_z_index(&self) -> f32 {
        self.scene_uniform.max_z_index
    }

    pub(crate) fn set_max_z_index(&mut self, z_index: f32) {
        self.scene_uniform.max_z_index = self.scene_uniform.max_z_index.max(z_index);
    }

    /// This is run before doing the layout of the children,
    /// it allows the renderer to prepare any state that it
    /// needs for the layout.
    #[inline]
    fn before_layout(&mut self) {
        self.scene_uniform.max_z_index = 1.0;
        self.sprite.before_layout();
        self.bitmap_text.before_layout();
    }

    /// This is run before doing the rendering of the children,
    /// it allows the renderer to prepare any state that it
    /// needs for the render.
    #[inline]
    fn before_render(&mut self) {
        self.scene_uniform.max_z_index = 1.0;
        self.sprite.before_render();
        self.bitmap_text.before_render();
    }

    #[inline]
    fn prerender<'a>(&'a mut self, engine: &crate::EngineState) -> ScenePrerender<'a> {
        let bind_group = Uniform::write(&mut self.scene_uniform, engine);

        let mut prerender = ScenePrerender::new();

        self.sprite.prerender(engine, bind_group, &mut prerender);
        self.bitmap_text.prerender(engine, bind_group, &mut prerender);

        prerender
    }
}

pub(crate) struct Scene {
    root: Node,
    pub(crate) changed: Arc<SceneChanged>,
    pub(crate) renderer: SceneRenderer,
    pub(crate) rendered_nodes: Vec<NodeHandle>,

    /// Assets
    pub(crate) textures: Handles<TextureBuffer>,
}

impl Scene {
    #[inline]
    pub(crate) fn new(engine: &crate::EngineState, mut root: Node, spawner: std::sync::Arc<dyn Spawner>) -> Self {
        let changed = SceneChanged::new(spawner);

        // This passes the SceneChanged into the Node, so that way the
        // Node signals can notify that the layout / render has changed.
        root.callbacks.trigger_after_inserted(&changed);

        Self {
            root,
            changed,
            renderer: SceneRenderer::new(engine),
            textures: Handles::new(),
            rendered_nodes: vec![],
        }
    }

    #[inline]
    pub(crate) fn should_render(&self) -> bool {
        self.changed.is_render_changed()
    }

    /// Before rendering, this runs any necessary processing and prepares data for the render.
    /// The lifetimes are necessary in order to make it work with wgpu::RenderPass.
    pub(crate) fn prerender<'a>(&'a mut self, engine: &crate::EngineState) -> ScenePrerender<'a> {
        let layout_changed = self.changed.replace_layout_changed();
        let render_changed = self.changed.replace_render_changed();

        if layout_changed {
            self.renderer.before_layout();

            self.rendered_nodes.clear();

            let child = &self.root.handle;

            let mut lock = child.lock();

            if lock.is_visible() {
                let screen_size = ScreenSize {
                    width: engine.window_size.width as f32,
                    height: engine.window_size.height as f32,
                };

                let mut info = SceneLayoutInfo {
                    screen_size: &screen_size,
                    renderer: &mut self.renderer,
                    rendered_nodes: &mut self.rendered_nodes,
                };

                let parent = RealLocation::full();

                lock.update_layout(child, &parent, &mut info);
            }

        } else if render_changed {
            self.renderer.before_render();

            let screen_size = ScreenSize {
                width: engine.window_size.width as f32,
                height: engine.window_size.height as f32,
            };

            let mut info = SceneRenderInfo {
                screen_size: &screen_size,
                renderer: &mut self.renderer,
            };

            for child in self.rendered_nodes.iter() {
                child.lock().render(&mut info);
            }
        }

        self.renderer.prerender(engine)
    }
}
