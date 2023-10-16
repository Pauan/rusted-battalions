use futures_signals::signal::{Signal, SignalExt};
use futures_signals::signal_vec::{SignalVec, SignalVecExt};
use crate::scene::builder::{Node, BuilderChanged, make_builder, base_methods, location_methods, children_methods};
use crate::scene::{
    NodeHandle, Location, Origin, Size, Offset, Percentage, Padding, SmallestSize,
    SmallestLength, RealLocation, NodeLayout, SceneLayoutInfo, SceneRenderInfo, RealSize,
    Order, internal_panic,
};


struct Child {
    size: SmallestSize,
    handle: NodeHandle,
}

/// Displays children in a column from up-to-down.
///
/// # Layout
///
/// Children are shrunk vertically as much as possible.
///
/// If a child has a height of [`Length::ParentHeight`] then it is expanded
/// to fill the available empty space of the column.
///
/// If there are multiple children with a height of [`Length::ParentHeight`]
/// then the empty space is distributed to each child.
///
/// The empty space is distributed as a ratio. For example, if one child has
/// `Length::ParentHeight(2.0)` and another child has `Length::ParentHeight(1.0)`
/// then the first child will be twice as tall as the second child.
///
/// # Sizing
///
/// * [`Length::SmallestWidth`]: the maximum of all the children's smallest width.
///
/// * [`Length::SmallestHeight`]: the sum of all the children's smallest height.
pub struct Column {
    visible: bool,
    location: Location,
    children: Vec<NodeHandle>,

    // Internal state
    computed_children: Vec<Child>,
    ratio_sum: Percentage,
    min_height: Percentage,
}

impl Column {
    #[inline]
    fn new() -> Self {
        Self {
            visible: true,
            location: Location::default(),
            children: vec![],

            computed_children: vec![],
            ratio_sum: 0.0,
            min_height: 0.0,
        }
    }

    fn children_size<'a>(&mut self, parent: &SmallestSize, info: &mut SceneLayoutInfo<'a>) -> RealSize {
        debug_assert!(self.ratio_sum == 0.0);

        let mut smallest_size = RealSize::zero();

        self.computed_children.reserve(self.children.len());

        for child in self.children.iter() {
            let mut lock = child.lock();

            if lock.is_visible() {
                let child_size = lock.smallest_size(parent, info);

                match child_size.height {
                    SmallestLength::Screen(x) => {
                        smallest_size.height += x;
                    },
                    SmallestLength::ParentHeight(x) => {
                        self.ratio_sum += x;
                    },
                    SmallestLength::ParentWidth(_) => {
                        unimplemented!();
                    },
                    _ => internal_panic(),
                }

                match child_size.width {
                    SmallestLength::Screen(x) => {
                        smallest_size.width = smallest_size.width.max(x);
                    },
                    SmallestLength::ParentHeight(_) => {
                        unimplemented!();
                    },
                    // ParentWidth is treated as 0.0
                    SmallestLength::ParentWidth(_) => {},
                    _ => internal_panic(),
                }

                self.computed_children.push(Child {
                    size: child_size,
                    handle: child.clone(),
                });
            }
        }

        self.min_height = smallest_size.height;

        smallest_size
    }
}

make_builder!(Column, ColumnBuilder);
base_methods!(Column, ColumnBuilder);
location_methods!(Column, ColumnBuilder);
children_methods!(Column, ColumnBuilder);

impl NodeLayout for Column {
    #[inline]
    fn is_visible(&mut self) -> bool {
        self.visible
    }

    fn smallest_size<'a>(&mut self, parent: &SmallestSize, info: &mut SceneLayoutInfo<'a>) -> SmallestSize {
        let smallest_size = self.location.size.smallest_size(&info.screen_size).parent_to_smallest(parent);

        let padding = self.location.padding.to_screen(parent, &smallest_size, &info.screen_size);

        smallest_size.with_padding(parent, padding, |mut parent| {
            // Shrinks the children vertically as much as possible.
            parent.height = SmallestLength::SmallestHeight(1.0);

            // This needs to always run even if the Column has a fixed size, because we need
            // to calculate the min_height and ratio_sum.
            self.children_size(&parent, info)
        })
    }

    fn update_layout<'a>(&mut self, _handle: &NodeHandle, parent: &RealLocation, smallest_size: &SmallestSize, info: &mut SceneLayoutInfo<'a>) {
        let mut this_location = self.location.children_location(parent, &smallest_size.real_size(), &info);

        let empty_space = (this_location.size.height - self.min_height).max(0.0);

        let stretch_percentage = empty_space * (1.0 / self.ratio_sum);

        for child in self.computed_children.iter() {
            let child_size = match child.size.height {
                SmallestLength::Screen(height) => {
                    RealSize {
                        width: this_location.size.width,
                        height: height,
                    }
                },
                SmallestLength::ParentHeight(height) => {
                    RealSize {
                        width: this_location.size.width,
                        height: stretch_percentage * height,
                    }
                },
                SmallestLength::ParentWidth(_) => {
                    unimplemented!();
                },
                _ => internal_panic(),
            };

            let child_location = RealLocation {
                position: this_location.position,
                size: child_size,
                order: this_location.order,
            };

            child.handle.lock().update_layout(&child.handle, &child_location, &child.size, info);

            this_location.move_down(child_location.size.height);
        }

        self.computed_children.clear();
        self.ratio_sum = 0.0;
        self.min_height = 0.0;
    }

    fn render<'a>(&mut self, _info: &mut SceneRenderInfo<'a>) {}
}
