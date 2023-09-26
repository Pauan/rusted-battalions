use futures_signals::signal::{Signal, SignalExt};
use futures_signals::signal_vec::{SignalVec, SignalVecExt};
use crate::scene::builder::{Node, make_builder, base_methods, location_methods, simple_method, children_methods};
use crate::scene::{
    NodeHandle, MinSize, Location, Origin, Size, Offset, Padding, Length,
    RealLocation, NodeLayout, SceneLayoutInfo, SceneRenderInfo, ScreenSize,
    RealSize,
};


/// Size of each child in the grid.
///
/// # Sizing
///
/// * [`Length::ParentWidth`]: the width is relative to the grid's width minus padding.
///
/// * [`Length::ParentHeight`]: the height is relative to the grid's height minus padding.
///
/// * [`Length::ChildrenWidth`]: the maximum width of each child.
///
/// * [`Length::ChildrenHeight`]: the maximum height of each child.
pub struct GridSize {
    pub width: Length,
    pub height: Length,
}

impl GridSize {
    #[inline]
    fn zero() -> Self {
        Self {
            width: Length::Zero,
            height: Length::Zero,
        }
    }

    fn min_size(&self, parent: &RealSize, screen_size: &ScreenSize) -> MinSize {
        let screen_width = screen_size.to_real_width();
        let screen_height = screen_size.to_real_height();

        MinSize {
            width: self.width.min_length(parent, &screen_width, screen_size.width),
            height: self.height.min_length(parent, &screen_height, screen_size.height),
        }
    }
}


/// Displays children in a grid where every child has the same size.
///
/// When the children overflow horizontally, it moves them to the next vertical row.
///
/// # Sizing
///
/// * [`Length::ChildrenWidth`]: the sum of the width of all the children (laid out on one row).
///
/// * [`Length::ChildrenHeight`]: the sum of the height of all the children (laid out on multiple rows).
pub struct Grid {
    visible: bool,
    stretch: bool,
    location: Location,
    children: Vec<NodeHandle>,

    grid_size: Option<GridSize>,

    // Internal state
    computed_grid_size: RealSize,
    min_size: Option<RealSize>,
}

impl Grid {
    #[inline]
    fn new() -> Self {
        Self {
            visible: true,
            stretch: false,
            location: Location::default(),
            children: vec![],

            grid_size: None,

            computed_grid_size: RealSize {
                width: 0.0,
                height: 0.0,
            },
            min_size: None,
        }
    }
}

make_builder!(Grid, GridBuilder);
base_methods!(Grid, GridBuilder);
location_methods!(Grid, GridBuilder, true);
children_methods!(Grid, GridBuilder);

impl GridBuilder {
    simple_method!(
        /// Sets the [`GridSize`] for the grid.
        grid_size,
        grid_size_signal,
        true,
        true,
        |state, grid_size: GridSize| {
            state.grid_size = Some(grid_size);
        },
    );
}

impl NodeLayout for Grid {
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
            let grid_size = self.grid_size.as_ref().expect("Grid is missing grid_size");

            let min_size = self.location.min_size(parent, &info.screen_size);

            let child_parent = self.location.padding.children_size(parent, &min_size, &info.screen_size);

            let grid_size = grid_size.min_size(&child_parent, &info.screen_size);

            let mut visible_children = 0.0;

            let mut min_grid_size = RealSize {
                width: 0.0,
                height: 0.0,
            };

            for child in self.children.iter() {
                let mut lock = child.lock();

                if lock.is_visible() {
                    visible_children += 1.0;

                    let child_size = lock.min_size(&child_parent, info);

                    min_grid_size.width = min_grid_size.width.max(child_size.width);
                    min_grid_size.height = min_grid_size.height.max(child_size.height);
                }
            }

            let grid_size = RealSize {
                width: grid_size.width.unwrap_or(min_grid_size.width),
                height: grid_size.height.unwrap_or(min_grid_size.height),
            };

            let columns;
            let rows;

            // Displays all children in a single row
            if min_size.width.is_err() {
                columns = visible_children;
                rows = 1.0;

            // Displays children in a grid, overflowing to the next row.
            } else {
                if visible_children == 0.0 {
                    columns = 0.0;
                    rows = 0.0;

                } else {
                    columns = (child_parent.width / grid_size.width).trunc().min(visible_children);
                    rows = (visible_children / columns).ceil();
                }
            };

            let min_size = RealSize {
                width: min_size.width.unwrap_or_else(|| {
                    columns * grid_size.width
                }),
                height: min_size.height.unwrap_or_else(|| {
                    rows * grid_size.height
                }),
            };

            self.computed_grid_size = grid_size;
            self.min_size = Some(min_size);
            min_size
        }
    }

    fn update_layout<'a>(&mut self, _handle: &NodeHandle, parent: &RealLocation, info: &mut SceneLayoutInfo<'a>) {
        // This is needed in order to calculate the computed_grid_size
        let children_min_size = self.min_size(&parent.size, info);

        let this_location = parent.modify(&self.location, &children_min_size, &info.screen_size);

        let max_width = this_location.size.width;

        let mut width = 0.0;

        let mut child_location = this_location;

        child_location.size = self.computed_grid_size;

        for child in self.children.iter() {
            let mut lock = child.lock();

            if lock.is_visible() {
                width += child_location.size.width;

                if width > child_location.size.width && width > max_width {
                    width = child_location.size.width;
                    child_location.position.x = this_location.position.x;
                    child_location.move_down(child_location.size.height);
                }

                let max_z_index = info.renderer.get_max_z_index();

                assert!(max_z_index >= this_location.z_index);

                child_location.z_index = max_z_index;

                lock.update_layout(child, &child_location, info);

                child_location.position.x = this_location.position.x + width;
            }
        }

        self.min_size = None;
    }

    fn render<'a>(&mut self, _info: &mut SceneRenderInfo<'a>) {}
}
