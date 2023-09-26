use futures_signals::signal::{Signal, SignalExt};
use futures_signals::signal_vec::{SignalVec, SignalVecExt};
use crate::scene::builder::{Node, make_builder, base_methods, location_methods, children_methods};
use crate::scene::{
    NodeHandle, Location, Origin, Size, Offset, Padding,
    RealLocation, NodeLayout, SceneLayoutInfo, SceneRenderInfo,
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

    // Internal state
    rows: Vec<Row>,
    min_size: Option<RealSize>,
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

    fn min_size<'a>(&mut self, parent: &RealSize, info: &mut SceneLayoutInfo<'a>) -> RealSize {
        if let Some(min_size) = self.min_size {
            min_size

        } else {
            let min_size = self.location.children_size(parent, info, |child_parent, info| {
                let max_width = child_parent.width;

                let mut width = 0.0;
                let mut row = Row::new();

                for child in self.children.iter() {
                    let mut lock = child.lock();

                    if lock.is_visible() {
                        let size = lock.min_size(child_parent, info);

                        width += size.width;

                        if width > size.width && width > max_width {
                            self.rows.push(row);

                            width = size.width;
                            row = Row::new();
                        }

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
            });

            self.min_size = Some(min_size);
            min_size
        }
    }

    fn update_layout<'a>(&mut self, _handle: &NodeHandle, parent: &RealLocation, info: &mut SceneLayoutInfo<'a>) {
        // This is needed in order to calculate the rows
        let children_min_size = self.min_size(&parent.size, info);

        let this_location = parent.modify(&self.location, &children_min_size, &info.screen_size);

        {
            let mut child_location = this_location;

            for row in self.rows.iter() {
                child_location.size.height = row.height;

                for child in row.children.iter() {
                    child_location.size.width = child.width;

                    let max_z_index = info.renderer.get_max_z_index();

                    assert!(max_z_index >= this_location.z_index);

                    child_location.z_index = max_z_index;

                    child.handle.lock().update_layout(&child.handle, &child_location, info);

                    child_location.move_right(child.width);
                }

                child_location.position.x = this_location.position.x;
                child_location.move_down(row.height);
            }
        }

        self.rows.clear();
        self.min_size = None;
    }

    fn render<'a>(&mut self, _info: &mut SceneRenderInfo<'a>) {}
}
