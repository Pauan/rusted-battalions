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
pub use sprite::{Sprite, SpriteBuilder, Spritesheet, SpritesheetSettings, Tile, RepeatTile, Repeat};
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


static INTERNAL_BUG_MESSAGE: &'static str = "UNEXPECTED INTERNAL BUG, PLEASE REPORT THIS";

#[track_caller]
pub(crate) fn internal_panic() -> ! {
    panic!("{}", INTERNAL_BUG_MESSAGE);
}


/// f32 from 0.0 to 1.0
pub type Percentage = f32;


/// x / y in screen space, percentage of the screen size
#[derive(Debug, Clone, Copy)]
pub(crate) struct RealPosition {
    pub(crate) x: Percentage,
    pub(crate) y: Percentage,
}

impl RealPosition {
    pub(crate) fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
        }
    }
}

impl std::ops::Add for RealPosition {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}


/// width / height in screen space, percentage of the screen size
#[derive(Debug, Clone, Copy)]
pub(crate) struct RealSize {
    pub(crate) width: Percentage,
    pub(crate) height: Percentage,
}

impl RealSize {
    pub(crate) fn zero() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
        }
    }

    pub(crate) fn smallest_size(&self) -> SmallestSize {
        SmallestSize {
            width: SmallestLength::Screen(self.width),
            height: SmallestLength::Screen(self.height),
        }
    }
}

impl std::ops::Add for RealSize {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            width: self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}

impl std::ops::Sub for RealSize {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            width: (self.width - rhs.width).max(0.0),
            height: (self.height - rhs.height).max(0.0),
        }
    }
}


/// Padding in screen space, percentage of the screen size
#[derive(Debug, Clone, Copy)]
pub(crate) struct RealPadding {
    pub(crate) up: Percentage,
    pub(crate) down: Percentage,
    pub(crate) left: Percentage,
    pub(crate) right: Percentage,
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


/// Empty space around the node.
///
/// The padding does not increase the size, instead the empty
/// space is created by subtracting from the node's size.
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
    /// Uses the same [`Length`] for every side of the padding.
    pub fn all(length: Length) -> Self {
        Self {
            up: length,
            down: length,
            left: length,
            right: length,
        }
    }

    /// Calculates the RealSize for the padding, if possible.
    ///
    /// Panics if it can't convert into a RealSize.
    pub(crate) fn to_screen(&self, parent: &SmallestSize, smallest: &SmallestSize, screen: &ScreenSize) -> RealSize {
        let up = self.up.smallest_length(&screen.height).to_screen(parent, smallest).unwrap();
        let down = self.down.smallest_length(&screen.height).to_screen(parent, smallest).unwrap();
        let left = self.left.smallest_length(&screen.width).to_screen(parent, smallest).unwrap();
        let right = self.right.smallest_length(&screen.width).to_screen(parent, smallest).unwrap();

        RealSize {
            width: left + right,
            height: up + down,
        }
    }

    /// Converts the padding into screen space.
    pub(crate) fn real_padding(&self, parent: &RealSize, smallest: &RealSize, screen: &ScreenSize) -> RealPadding {
        let up = self.up.real_length(parent, smallest, &screen.height);
        let down = self.down.real_length(parent, smallest, &screen.height);

        let left = self.left.real_length(parent, smallest, &screen.width);
        let right = self.right.real_length(parent, smallest, &screen.width);

        RealPadding { up, down, left, right }
    }
}

impl Default for Padding {
    #[inline]
    fn default() -> Self {
        Self::all(Length::Zero)
    }
}


#[derive(Debug, Clone, Copy)]
pub(crate) enum SmallestLength {
    /// Fixed length in screen space
    Screen(Percentage),

    /// Stretches to fill up the parent space
    ParentWidth(Percentage),
    ParentHeight(Percentage),

    /// Must calculate the smallest size
    SmallestWidth(Percentage),
    SmallestHeight(Percentage),
}

impl SmallestLength {
    fn is_smallest(&self) -> bool {
        match self {
            Self::SmallestWidth(_) | Self::SmallestHeight(_) => true,
            _ => false,
        }
    }

    /// Converts ParentWidth / ParentHeight / SmallestWidth / SmallestHeight into Screen, if possible.
    fn to_screen(&self, parent: &SmallestSize, smallest: &SmallestSize) -> Self {
        match self {
            Self::ParentWidth(x) => match parent.width {
                Self::Screen(width) => Self::Screen(x * width),
                _ => Self::ParentWidth(*x),
            },

            Self::ParentHeight(x) => match parent.height {
                Self::Screen(height) => Self::Screen(x * height),
                _ => Self::ParentHeight(*x),
            },

            Self::SmallestWidth(x) => match smallest.width {
                Self::Screen(width) => Self::Screen(x * width),

                Self::ParentWidth(_) |
                Self::ParentHeight(_) => Self::Screen(0.0),

                _ => Self::SmallestWidth(*x),
            },

            Self::SmallestHeight(x) => match smallest.height {
                Self::Screen(height) => Self::Screen(x * height),

                Self::ParentWidth(_) |
                Self::ParentHeight(_) => Self::Screen(0.0),

                _ => Self::SmallestHeight(*x),
            },

            x => *x,
        }
    }

    /// Converts ParentWidth / ParentHeight into Screen, if possible.
    fn parent_to_screen(&self, parent: &SmallestSize) -> Self {
        match self {
            Self::ParentWidth(x) => match parent.width {
                Self::Screen(width) => Self::Screen(x * width),
                _ => Self::ParentWidth(*x),
            },

            Self::ParentHeight(x) => match parent.height {
                Self::Screen(height) => Self::Screen(x * height),
                _ => Self::ParentHeight(*x),
            },

            x => *x,
        }
    }

    /// Converts the SmallestWidth / SmallestHeight into Screen.
    fn set_smallest(&self, smallest: &RealSize) -> Self {
        match self {
            Self::SmallestWidth(x) => Self::Screen(x * smallest.width),
            Self::SmallestHeight(x) => Self::Screen(x * smallest.height),
            x => *x,
        }
    }

    /// ParentWidth / ParentHeight are converted to 0.0
    fn real_length(&self) -> Percentage {
        match self {
            Self::Screen(x) => *x,

            Self::ParentWidth(_) |
            Self::ParentHeight(_) => 0.0,

            Self::SmallestWidth(_) |
            Self::SmallestHeight(_) => internal_panic(),
        }
    }

    /// Panics if it's not a [`SmallestLength::Screen`].
    fn unwrap(&self) -> Percentage {
        match self {
            Self::Screen(x) => *x,

            Self::ParentWidth(_) => {
                panic!("Cannot use ParentWidth because the parent's width is unknown.");
            },
            Self::ParentHeight(_) => {
                panic!("Cannot use ParentHeight because the parent's height is unknown.");
            },

            Self::SmallestWidth(_) => {
                panic!("Cannot use SmallestWidth because the node's smallest width hasn't been calculated yet.");
            },
            Self::SmallestHeight(_) => {
                panic!("Cannot use SmallestHeight because the node's smallest height hasn't been calculated yet.");
            },
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub(crate) struct SmallestSize {
    pub(crate) width: SmallestLength,
    pub(crate) height: SmallestLength,
}

impl SmallestSize {
    pub(crate) fn zero() -> Self {
        Self {
            width: SmallestLength::Screen(0.0),
            height: SmallestLength::Screen(0.0),
        }
    }

    pub(crate) fn is_smallest(&self) -> bool {
        self.width.is_smallest() || self.height.is_smallest()
    }

    /// Returns `0.0` for [`SmallestLength::ParentWidth`] / [`SmallestLength::ParentHeight`].
    ///
    /// Panics if it's a [`SmallestLength::ScreenWidth`] or [`SmallestLength::ScreenHeight`].
    pub(crate) fn real_size(&self) -> RealSize {
        let width = self.width.real_length();
        let height = self.height.real_length();
        RealSize { width, height }
    }

    /// Converts ParentWidth / ParentHeight into Screen, if possible.
    pub(crate) fn parent_to_screen(&self, parent: &SmallestSize) -> Self {
        let width = self.width.parent_to_screen(parent);
        let height = self.height.parent_to_screen(parent);
        Self { width, height }
    }

    /// Used inside of [`NodeLayout::smallest_size`] to set the SmallestWidth / SmallestHeight.
    pub(crate) fn set_smallest(&self, smallest: &RealSize) -> Self {
        let width = self.width.set_smallest(smallest);
        let height = self.height.set_smallest(smallest);
        Self { width, height }
    }

    /// Used inside of [`NodeLayout::smallest_size`] to calculate the [`RealSize`] for the children.
    ///
    /// 1. Converts from self space into child space (by subtracting the padding).
    /// 2. Calls the function with the child space.
    /// 3. Converts from the child space back into self space (by adding the padding).
    pub(crate) fn with_padding<F>(&self, old_parent: &Self, padding: RealSize, f: F) -> Self
        where F: FnOnce(Self) -> RealSize {

        let new_parent = self.parent_to_screen(old_parent) - padding;

        let children_size = f(new_parent) + padding;

        self.set_smallest(&children_size)
    }
}

impl std::ops::Sub<RealSize> for SmallestSize {
    type Output = Self;

    fn sub(self, rhs: RealSize) -> Self {
        Self {
            width: match self.width {
                SmallestLength::Screen(x) => SmallestLength::Screen((x - rhs.width).max(0.0)),
                x => x,
            },
            height: match self.height {
                SmallestLength::Screen(x) => SmallestLength::Screen((x - rhs.height).max(0.0)),
                x => x,
            },
        }
    }
}


pub use Length::{
    Zero,
    Px,
    ScreenWidth, ScreenHeight,
    ParentWidth, ParentHeight,
    SmallestWidth, SmallestHeight,
};

/// Used for [`Offset`] / [`Size`] / [`Padding`].
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum Length {
    /// Zero length. Useful for [`Offset`] and [`Padding`].
    Zero,

    /// Pixel length.
    Px(i32),

    /// Percentage of the screen's width.
    ScreenWidth(Percentage),

    /// Percentage of the screen's height.
    ScreenHeight(Percentage),

    /// Percentage of the parent space's width.
    ParentWidth(Percentage),

    /// Percentage of the parent space's height.
    ParentHeight(Percentage),

    /// Percentage of the smallest possible width for this node.
    ///
    /// Each node type has its own algorithm for determining its smallest width.
    SmallestWidth(Percentage),

    /// Percentage of the smallest possible height for this node.
    ///
    /// Each node type has its own algorithm for determining its smallest height.
    SmallestHeight(Percentage),
}

impl Length {
    fn smallest_length(&self, screen: &ScreenLength) -> SmallestLength {
        match self {
            Self::Zero => SmallestLength::Screen(0.0),
            Self::Px(x) => SmallestLength::Screen(*x as Percentage / screen.pixels),

            Self::ScreenWidth(x) => SmallestLength::Screen(x * screen.ratio.width),
            Self::ScreenHeight(x) => SmallestLength::Screen(x * screen.ratio.height),

            Self::ParentWidth(x) => SmallestLength::ParentWidth(*x),
            Self::ParentHeight(x) => SmallestLength::ParentHeight(*x),

            Self::SmallestWidth(x) => SmallestLength::SmallestWidth(*x),
            Self::SmallestHeight(x) => SmallestLength::SmallestHeight(*x),
        }
    }

    /// Converts from local space into screen space.
    fn real_length(&self, parent: &RealSize, smallest: &RealSize, screen: &ScreenLength) -> Percentage {
        match self {
            Self::Zero => 0.0,
            Self::Px(x) => *x as Percentage / screen.pixels,

            Self::ScreenWidth(x) => x * screen.ratio.width,
            Self::ScreenHeight(x) => x * screen.ratio.height,

            Self::ParentWidth(x) => x * parent.width,
            Self::ParentHeight(x) => x * parent.height,

            Self::SmallestWidth(x) => x * smallest.width,
            Self::SmallestHeight(x) => x * smallest.height,
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

impl Offset {
    pub(crate) fn real_position(&self, parent: &RealSize, smallest: &RealSize, screen: &ScreenSize) -> RealPosition {
        let x = self.x.real_length(parent, smallest, &screen.width);
        let y = self.y.real_length(parent, smallest, &screen.height);
        RealPosition { x, y }
    }
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


/// Width / height relative to the parent space.
///
/// The default is `{ width: ParentWidth(1.0), height: ParentHeight(1.0) }`
/// which means it's the same size as its parent.
#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: Length,
    pub height: Length,
}

impl Size {
    pub(crate) fn smallest_size(&self, screen: &ScreenSize) -> SmallestSize {
        let width = self.width.smallest_length(&screen.width);
        let height = self.height.smallest_length(&screen.height);
        SmallestSize { width, height }
    }

    pub(crate) fn real_size(&self, parent: &RealSize, smallest: &RealSize, screen: &ScreenSize) -> RealSize {
        let width = self.width.real_length(parent, smallest, &screen.width);
        let height = self.height.real_length(parent, smallest, &screen.height);
        RealSize { width, height }
    }
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
    pub(crate) fn children_location(&self, parent: &RealLocation, smallest: &RealSize, screen: &ScreenSize) -> RealLocation {
        let size = self.size.real_size(&parent.size, smallest, screen);
        let offset = self.offset.real_position(&parent.size, smallest, screen);
        let padding = self.padding.real_padding(&parent.size, smallest, screen);

        let origin = RealPosition {
            x: (parent.size.width - size.width) * self.origin.x,
            y: (parent.size.height - size.height) * self.origin.y,
        };

        RealLocation {
            position: RealPosition {
                x: parent.position.x + origin.x + padding.left + offset.x,
                y: parent.position.y + origin.y + padding.up + offset.y,
            },
            size: RealSize {
                width: (size.width - padding.left - padding.right).max(0.0),
                height: (size.height - padding.up - padding.down).max(0.0),
            },
            z_index: parent.z_index + self.z_index,
        }
    }
}


#[derive(Debug, Clone)]
pub(crate) struct ScreenLength {
    /// The width / height of the screen in pixels.
    pub(crate) pixels: f32,

    /// Used for scaling the ratio when using ScreenWidth / ScreenHeight
    pub(crate) ratio: RealSize,
}

#[derive(Debug, Clone)]
pub(crate) struct ScreenSize {
    pub(crate) width: ScreenLength,
    pub(crate) height: ScreenLength,
}

impl ScreenSize {
    pub(crate) fn new(pixel_width: f32, pixel_height: f32) -> Self {
        let width = ScreenLength {
            pixels: pixel_width,
            ratio: RealSize {
                width: 1.0,
                height: pixel_height / pixel_width,
            },
        };

        let height = ScreenLength {
            pixels: pixel_height,
            ratio: RealSize {
                width: pixel_width / pixel_height,
                height: 1.0,
            },
        };

        Self { width, height }
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

    /// Returns the smallest size in screen space.
    ///
    /// If the Node is invisible then this method MUST NOT be called.
    ///
    /// This method must be called only once per layout.
    ///
    /// If the `parent` is [`SmallestLength::Screen`] then it MUST be the same as the `parent` in `update_layout`.
    ///
    /// This method MUST NOT return [`SmallestLength::SmallestWidth`] or [`SmallestLength::SmallestHeight`].
    fn smallest_size<'a>(&mut self, parent: &SmallestSize, info: &mut SceneLayoutInfo<'a>) -> SmallestSize;

    /// Does re-layout AND re-render on the Node.
    ///
    /// If the Node is invisible then this method MUST NOT be called.
    ///
    /// If the Node is visible then update_layout MUST be called.
    ///
    /// The `handle` must be the same as this `NodeLayout`.
    ///
    /// The `smallest_size` must be the same as the result of calling `NodeLayout::smallest_size` on this `NodeLayout`.
    fn update_layout<'a>(&mut self, handle: &NodeHandle, parent: &RealLocation, smallest_size: &SmallestSize, info: &mut SceneLayoutInfo<'a>);

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
                let screen_size = ScreenSize::new(
                    engine.window_size.width as f32,
                    engine.window_size.height as f32,
                );

                let mut info = SceneLayoutInfo {
                    screen_size: &screen_size,
                    renderer: &mut self.renderer,
                    rendered_nodes: &mut self.rendered_nodes,
                };

                let parent = RealLocation::full();

                let smallest_size = lock.smallest_size(&parent.size.smallest_size(), &mut info);

                lock.update_layout(child, &parent, &smallest_size, &mut info);
            }

        } else if render_changed {
            self.renderer.before_render();

            let screen_size = ScreenSize::new(
                engine.window_size.width as f32,
                engine.window_size.height as f32,
            );

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
