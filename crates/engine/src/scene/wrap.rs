use futures_signals::signal::{Signal, SignalExt};
use futures_signals::signal_vec::{SignalVec, SignalVecExt};
use crate::scene::builder::{Node, make_builder, base_methods, location_methods, children_methods};
use crate::scene::{
    NodeHandle, MinSize, Location, Origin, Size, Offset, Padding,
    ScreenSpace, NodeLayout, SceneLayoutInfo, SceneRenderInfo,
};


struct Child {
    width: f32,
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
pub struct Wrap {
    visible: bool,
    stretch: bool,
    location: Location,
    children: Vec<NodeHandle>,
    rows: Vec<Row>,
    min_size: Option<MinSize>,
}

impl Wrap {
    #[inline]
    fn new() -> Self {
        Self {
            visible: true,
            stretch: false,
            location: Location::default(),
            children: vec![],
            rows: vec![],
            min_size: None,
        }
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

    #[inline]
    fn is_stretch(&mut self) -> bool {
        self.stretch
    }

    fn min_size<'a>(&mut self, info: &mut SceneLayoutInfo<'a>) -> MinSize {
        *self.min_size.get_or_insert_with(|| {
            self.location.min_size(&info.screen_size)
        })
    }

    fn update_layout<'a>(&mut self, _handle: &NodeHandle, parent: &ScreenSpace, info: &mut SceneLayoutInfo<'a>) {
        let this_space = parent.modify(&self.location, &info.screen_size);

        {
            let max_width = this_space.size[0];

            let mut width = 0.0;
            let mut row = Row::new();

            for child in self.children.iter() {
                let mut lock = child.lock();

                if lock.is_visible() {
                    let size = lock.min_size(info);

                    if (width + size.width) > max_width {
                        self.rows.push(row);

                        width = 0.0;
                        row = Row::new();
                    }

                    width += size.width;

                    row.height = row.height.max(size.height);

                    row.children.push(Child {
                        width: size.width,
                        handle: child.clone(),
                    });
                }
            }

            if !row.children.is_empty() {
                self.rows.push(row);
            }
        }

        {
            let mut child_space = this_space;

            for row in self.rows.iter() {
                child_space.size[1] = row.height;

                for child in row.children.iter() {
                    child_space.size[0] = child.width;

                    child.handle.lock().update_layout(&child.handle, &child_space, info);

                    child_space.position[0] += child.width;
                }

                child_space.position[0] = this_space.position[0];
                child_space.position[1] += row.height;
            }
        }

        self.rows.clear();
        self.min_size = None;
    }

    fn render<'a>(&mut self, _info: &mut SceneRenderInfo<'a>) {}
}