use futures_signals::signal::{Signal, SignalExt};
use futures_signals::signal_vec::{SignalVec, SignalVecExt};
use crate::scene::builder::{Node, make_builder, base_methods, location_methods, children_methods};
use crate::scene::{
    NodeHandle, Location, Origin, Size, Offset, Percentage, Padding,
    RealLocation, NodeLayout, SceneLayoutInfo, SceneRenderInfo, RealSize,
};


struct Child {
    stretch: bool,
    height: Percentage,
    handle: NodeHandle,
}

/// Displays children in a column from up-to-down.
///
/// # Layout
///
/// Children are shrunk vertically as much as possible.
///
/// If a child has `stretch(true)` then the child's height is expanded to fill
/// the available empty space of the column.
///
/// If there are multiple children with `stretch(true)` then the empty space
/// is evenly distributed to each child.
///
/// # Sizing
///
/// * [`Length::ChildrenWidth`]: the maximum of all the children's width.
///
/// * [`Length::ChildrenHeight`]: the sum of all the non-stretch children's height.
pub struct Column {
    visible: bool,
    stretch: bool,
    location: Location,
    children: Vec<NodeHandle>,

    // Internal state
    computed_children: Vec<Child>,
    stretch_children: usize,
    min_height: Percentage,
    min_size: Option<RealSize>,
}

impl Column {
    #[inline]
    fn new() -> Self {
        Self {
            visible: true,
            stretch: false,
            location: Location::default(),
            children: vec![],

            computed_children: vec![],
            stretch_children: 0,
            min_height: 0.0,
            min_size: None,
        }
    }

    fn children_size<'a>(&mut self, info: &mut SceneLayoutInfo<'a>) -> RealSize {
        let mut min_size = RealSize {
            width: 0.0,
            height: 0.0,
        };

        self.computed_children.reserve(self.children.len());

        for child in self.children.iter() {
            let mut lock = child.lock();

            if lock.is_visible() {
                let child_size = lock.min_size(info);

                let stretch = lock.is_stretch();

                if stretch {
                    self.stretch_children += 1;

                } else {
                    min_size.height += child_size.height;
                }

                min_size.width = min_size.width.max(child_size.width);

                self.computed_children.push(Child {
                    stretch,
                    height: child_size.height,
                    handle: child.clone(),
                });
            }
        }

        self.min_height = min_size.height;

        min_size
    }
}

make_builder!(Column, ColumnBuilder);
base_methods!(Column, ColumnBuilder);
location_methods!(Column, ColumnBuilder, true);
children_methods!(Column, ColumnBuilder);

impl NodeLayout for Column {
    #[inline]
    fn is_visible(&mut self) -> bool {
        self.visible
    }

    #[inline]
    fn is_stretch(&mut self) -> bool {
        self.stretch
    }

    fn min_size<'a>(&mut self, info: &mut SceneLayoutInfo<'a>) -> RealSize {
        if let Some(min_size) = self.min_size {
            min_size

        } else {
            let min_size = self.location.size.min_size(&info.screen_size);

            // This needs to always run even if the Column has a fixed size, because we need
            // to calculate the min_height and stretch_children.
            let children_size = self.children_size(info);

            let min_size = min_size.unwrap_or_else(|| {
                self.location.padding.parent_size(&min_size, &children_size, &info.screen_size)
            });

            self.min_size = Some(min_size);
            min_size
        }
    }

    fn update_layout<'a>(&mut self, _handle: &NodeHandle, parent: &RealLocation, info: &mut SceneLayoutInfo<'a>) {
        // This is needed in order to calculate the min_height and stretch_children
        let min_size = self.min_size(info);

        let mut this_location = self.location.children_location(parent, &info.screen_size, || min_size);

        let empty_space = (this_location.size.height - self.min_height).max(0.0);

        let stretch_height = empty_space / (self.stretch_children as f32);

        for child in self.computed_children.iter() {
            let max_z_index = info.renderer.get_max_z_index();

            assert!(max_z_index >= this_location.z_index);

            let child_location = if child.stretch {
                RealLocation {
                    position: this_location.position,
                    size: RealSize {
                        width: this_location.size.width,
                        height: stretch_height,
                    },
                    z_index: max_z_index,
                }

            } else {
                RealLocation {
                    position: this_location.position,
                    size: RealSize {
                        width: this_location.size.width,
                        height: child.height,
                    },
                    z_index: max_z_index,
                }
            };

            child.handle.lock().update_layout(&child.handle, &child_location, info);

            this_location.move_down(child_location.size.height);
        }

        self.computed_children.clear();
        self.stretch_children = 0;
        self.min_height = 0.0;
        self.min_size = None;
    }

    fn render<'a>(&mut self, _info: &mut SceneRenderInfo<'a>) {}
}
