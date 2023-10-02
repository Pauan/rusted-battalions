use futures_signals::signal::{Signal, SignalExt};
use futures_signals::signal_vec::{SignalVec, SignalVecExt};
use crate::scene::builder::{Node, make_builder, base_methods, location_methods, simple_method, children_methods};
use crate::scene::{
    NodeHandle, Location, Origin, Size, Offset, Padding, Length, SmallestSize, SmallestLength,
    RealLocation, NodeLayout, SceneLayoutInfo, SceneRenderInfo, ScreenSize, RealSize,
};


/// Size of each child in the grid.
///
/// # Sizing
///
/// * [`Length::ParentWidth`]: the width is relative to the grid's width minus padding.
///
/// * [`Length::ParentHeight`]: the height is relative to the grid's height minus padding.
///
/// * [`Length::SmallestWidth`]: it is an error to use `SmallestWidth`.
///
/// * [`Length::SmallestHeight`]: it is an error to use `SmallestHeight`.
pub struct GridSize {
    pub width: Length,
    pub height: Length,
}

impl GridSize {
    fn real_size(&self, parent: &SmallestSize, screen_size: &ScreenSize) -> RealSize {
        let width = self.width.smallest_length(&screen_size.width).parent_to_screen(parent).unwrap();
        let height = self.height.smallest_length(&screen_size.height).parent_to_screen(parent).unwrap();
        RealSize { width, height }
    }
}


/// Displays children in a grid where every child has the same size.
///
/// When the children overflow horizontally, it moves them to the next vertical row.
///
/// # Sizing
///
/// * [`Length::SmallestWidth`]: the sum of the width of all the children (laid out on one row).
///
/// * [`Length::SmallestHeight`]: the sum of the height of all the children (laid out on multiple rows).
pub struct Grid {
    visible: bool,
    location: Location,
    children: Vec<NodeHandle>,

    grid_size: Option<GridSize>,

    // Internal state
    computed_grid_size: RealSize,
}

impl Grid {
    #[inline]
    fn new() -> Self {
        Self {
            visible: true,
            location: Location::default(),
            children: vec![],

            grid_size: None,

            computed_grid_size: RealSize::zero(),
        }
    }

    fn children_size<'a>(&mut self, parent: &SmallestSize, info: &mut SceneLayoutInfo<'a>) -> RealSize {
        let grid_size = self.grid_size.as_ref().expect("Grid is missing grid_size");
        let grid_size = grid_size.real_size(parent, &info.screen_size);


        let mut visible_children = 0;

        for child in self.children.iter() {
            let mut lock = child.lock();

            if lock.is_visible() {
                visible_children += 1;
            }
        }


        let visible_children = visible_children as f32;
        let mut columns = 0.0;
        let mut rows = 0.0;

        match parent.width {
            // Displays children in a grid, overflowing to the next row.
            SmallestLength::Screen(max_width) => {
                if visible_children == 0.0 {
                    columns = 0.0;
                    rows = 0.0;

                } else {
                    columns = (max_width / grid_size.width).trunc().min(visible_children);
                    rows = (visible_children / columns).ceil();
                }
            },

            // Displays all children in a single row
            SmallestLength::SmallestWidth(_) => {
                columns = visible_children;
                rows = 1.0;
            },

            _ => {
                // TODO better error handling ?
                parent.width.unwrap();
            },
        }

        self.computed_grid_size = grid_size;
        
        RealSize {
            width: columns * grid_size.width,
            height: rows * grid_size.height,
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

    fn smallest_size<'a>(&mut self, parent: &SmallestSize, info: &mut SceneLayoutInfo<'a>) -> SmallestSize {
        let smallest_size = self.location.size.smallest_size(&info.screen_size).parent_to_smallest(parent);

        if smallest_size.is_smallest() {
            let padding = self.location.padding.to_screen(parent, &smallest_size, &info.screen_size);

            smallest_size.with_padding(parent, padding, |parent| {
                self.children_size(&parent, info)
            })

        } else {
            smallest_size
        }
    }

    fn update_layout<'a>(&mut self, _handle: &NodeHandle, parent: &RealLocation, smallest_size: &SmallestSize, info: &mut SceneLayoutInfo<'a>) {
        let this_location = self.location.children_location(parent, &smallest_size.real_size(), &info.screen_size);

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

                let smallest = lock.smallest_size(&child_location.size.smallest_size(), info);
                lock.update_layout(child, &child_location, &smallest, info);

                child_location.position.x = this_location.position.x + width;
            }
        }
    }

    fn render<'a>(&mut self, _info: &mut SceneRenderInfo<'a>) {}
}
