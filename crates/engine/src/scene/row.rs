use futures_signals::signal::{Signal, SignalExt};
use futures_signals::signal_vec::{SignalVec, SignalVecExt};
use crate::scene::builder::{Node, make_builder, base_methods, location_methods, children_methods};
use crate::scene::{
    NodeHandle, Location, Origin, Size, Offset, Percentage, Padding, SmallestSize,
    SmallestLength, RealLocation, NodeLayout, SceneLayoutInfo, SceneRenderInfo, RealSize,
    internal_panic,
};


struct Child {
    size: SmallestSize,
    handle: NodeHandle,
}

/// Displays children in a row from left-to-right.
///
/// # Layout
///
/// Children are shrunk horizontally as much as possible.
///
/// If a child has a width of [`Length::ParentWidth`] then it is expanded
/// to fill the available empty space of the row.
///
/// If there are multiple children with a width of [`Length::ParentWidth`]
/// then the empty space is distributed to each child.
///
/// The empty space is distributed as a ratio. For example, if one child has
/// `Length::ParentWidth(2.0)` and another child has `Length::ParentWidth(1.0)`
/// then the first child will be twice as wide as the second child.
///
/// # Sizing
///
/// * [`Length::SmallestWidth`]: the sum of all the children's smallest width.
///
/// * [`Length::SmallestHeight`]: the maximum of all the children's smallest height.
pub struct Row {
    visible: bool,
    location: Location,
    children: Vec<NodeHandle>,

    // Internal state
    computed_children: Vec<Child>,
    ratio_sum: Percentage,
    min_width: Percentage,
    smallest_size: Option<SmallestSize>,
}

impl Row {
    #[inline]
    fn new() -> Self {
        Self {
            visible: true,
            location: Location::default(),
            children: vec![],

            computed_children: vec![],
            ratio_sum: 0.0,
            min_width: 0.0,
            smallest_size: None,
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

                match child_size.width {
                    SmallestLength::Screen(x) => {
                        smallest_size.width += x;
                    },
                    SmallestLength::ParentWidth(x) => {
                        self.ratio_sum += x;
                    },
                    SmallestLength::ParentHeight(_) => {
                        unimplemented!();
                    },
                    _ => internal_panic(),
                }

                match child_size.height {
                    SmallestLength::Screen(x) => {
                        smallest_size.height = smallest_size.height.max(x);
                    },
                    SmallestLength::ParentWidth(_) => {
                        unimplemented!();
                    },
                    // ParentHeight is treated as 0.0
                    SmallestLength::ParentHeight(_) => {},
                    _ => internal_panic(),
                }

                self.computed_children.push(Child {
                    size: child_size,
                    handle: child.clone(),
                });
            }
        }

        self.min_width = smallest_size.width;

        smallest_size
    }
}

make_builder!(Row, RowBuilder);
base_methods!(Row, RowBuilder);
location_methods!(Row, RowBuilder, true);
children_methods!(Row, RowBuilder);

impl NodeLayout for Row {
    #[inline]
    fn is_visible(&mut self) -> bool {
        self.visible
    }

    fn smallest_size<'a>(&mut self, parent: &SmallestSize, info: &mut SceneLayoutInfo<'a>) -> SmallestSize {
        if let Some(smallest_size) = self.smallest_size {
            smallest_size

        } else {
            let smallest_size = self.location.size.smallest_size(&info.screen_size);

            let padding = self.location.padding.to_screen(parent, &smallest_size, &info.screen_size);

            let smallest_size = smallest_size.with_padding(parent, padding, |parent| {
                // Shrinks the children horizontally as much as possible.
                parent.width = SmallestLength::SmallestWidth(1.0);

                // This needs to always run even if the Row has a fixed size, because we need
                // to calculate the min_width and ratio_sum.
                self.children_size(&parent, info)
            });

            self.smallest_size = Some(smallest_size);
            smallest_size
        }
    }

    fn update_layout<'a>(&mut self, _handle: &NodeHandle, parent: &RealLocation, smallest_size: &SmallestSize, info: &mut SceneLayoutInfo<'a>) {
        let mut this_location = self.location.children_location(parent, &smallest_size.real_size(), &info.screen_size);

        let empty_space = (this_location.size.width - self.min_width).max(0.0);

        let stretch_percentage = empty_space * (1.0 / self.ratio_sum);

        for child in self.computed_children.iter() {
            let max_z_index = info.renderer.get_max_z_index();

            assert!(max_z_index >= this_location.z_index);

            let child_size = match child.size.width {
                SmallestLength::Screen(width) => {
                    RealSize {
                        width: width,
                        height: this_location.size.height,
                    }
                },
                SmallestLength::ParentWidth(width) => {
                    RealSize {
                        width: stretch_percentage * width,
                        height: this_location.size.height,
                    }
                },
                SmallestLength::ParentHeight(_) => {
                    unimplemented!();
                },
            };

            let child_location = RealLocation {
                position: this_location.position,
                size: child_size,
                z_index: max_z_index,
            };

            child.handle.lock().update_layout(&child.handle, &child_location, &child.size, info);

            this_location.move_right(child_location.size.width);
        }

        self.computed_children.clear();
        self.ratio_sum = 0.0;
        self.min_width = 0.0;
        self.smallest_size = None;
    }

    fn render<'a>(&mut self, _info: &mut SceneRenderInfo<'a>) {}
}
