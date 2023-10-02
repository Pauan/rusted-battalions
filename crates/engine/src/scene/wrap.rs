use futures_signals::signal::{Signal, SignalExt};
use futures_signals::signal_vec::{SignalVec, SignalVecExt};
use crate::scene::builder::{Node, make_builder, base_methods, location_methods, children_methods};
use crate::scene::{
    NodeHandle, Location, Origin, Size, Offset, Padding, SmallestSize, SmallestLength,
    RealLocation, NodeLayout, SceneLayoutInfo, SceneRenderInfo, RealSize, Order,
};


struct Child {
    width: f32,
    size: SmallestSize,
    handle: NodeHandle,
}

struct Row {
    height: f32,
    children: Vec<Child>,
}

impl Row {
    fn new() -> Self {
        Self {
            height: 0.0,
            children: vec![],
        }
    }
}


/// Displays children in a row, wrapping to the next row when running out of space.
///
/// # Layout
///
/// Children are shrunk horizontally and vertically as much as possible.
pub struct Wrap {
    visible: bool,
    location: Location,
    children: Vec<NodeHandle>,

    // Internal state
    rows: Vec<Row>,
}

impl Wrap {
    #[inline]
    fn new() -> Self {
        Self {
            visible: true,
            location: Location::default(),
            children: vec![],

            rows: vec![],
        }
    }

    fn children_size<'a>(&mut self, mut parent: SmallestSize, info: &mut SceneLayoutInfo<'a>) -> RealSize {
        let mut min_size = RealSize::zero();

        let max_width = parent.width.unwrap();

        let mut width = 0.0;
        let mut row = Row::new();

        // Shrinks each child to the smallest size
        parent.width = SmallestLength::SmallestWidth(1.0);
        parent.height = SmallestLength::SmallestHeight(1.0);

        for child in self.children.iter() {
            let mut lock = child.lock();

            if lock.is_visible() {
                let size = lock.smallest_size(&parent, info);

                let real_size = size.real_size();

                width += real_size.width;

                if width > real_size.width && width > max_width {
                    self.rows.push(row);

                    width = real_size.width;
                    row = Row::new();
                }

                row.height = row.height.max(real_size.height);

                row.children.push(Child {
                    width: real_size.width,
                    size: size,
                    handle: child.clone(),
                });

                min_size.width = min_size.width.max(width);
            }
        }

        if !row.children.is_empty() {
            self.rows.push(row);
        }

        for row in self.rows.iter() {
            min_size.height += row.height;
        }

        min_size
    }
}

make_builder!(Wrap, WrapBuilder);
base_methods!(Wrap, WrapBuilder);
location_methods!(Wrap, WrapBuilder, true);
children_methods!(Wrap, WrapBuilder);

impl NodeLayout for Wrap {
    #[inline]
    fn is_visible(&mut self) -> bool {
        self.visible
    }

    fn smallest_size<'a>(&mut self, parent: &SmallestSize, info: &mut SceneLayoutInfo<'a>) -> SmallestSize {
        let smallest_size = self.location.size.smallest_size(&info.screen_size).parent_to_smallest(parent);

        let padding = self.location.padding.to_screen(parent, &smallest_size, &info.screen_size);

        smallest_size.with_padding(parent, padding, |parent| {
            self.children_size(parent, info)
        })
    }

    fn update_layout<'a>(&mut self, _handle: &NodeHandle, parent: &RealLocation, smallest_size: &SmallestSize, info: &mut SceneLayoutInfo<'a>) {
        let this_location = self.location.children_location(parent, &smallest_size.real_size(), &info);

        {
            let mut child_location = this_location;

            for row in self.rows.iter() {
                child_location.size.height = row.height;

                for child in row.children.iter() {
                    child_location.size.width = child.width;

                    child.handle.lock().update_layout(&child.handle, &child_location, &child.size, info);

                    child_location.move_right(child.width);
                }

                child_location.position.x = this_location.position.x;
                child_location.move_down(row.height);
            }
        }

        self.rows.clear();
    }

    fn render<'a>(&mut self, _info: &mut SceneRenderInfo<'a>) {}
}
