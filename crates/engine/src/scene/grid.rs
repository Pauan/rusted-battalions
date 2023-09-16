use futures_signals::signal::{Signal, SignalExt};
use futures_signals::signal_vec::{SignalVec, SignalVecExt};
use crate::scene::builder::{Node, make_builder, base_methods, location_methods, simple_method, children_methods};
use crate::scene::{
    NodeHandle, MinSize, Location, Origin, Size, Offset, Padding, Length,
    ScreenSpace, NodeLayout, SceneLayoutInfo, SceneRenderInfo, ScreenSize,
    RealSize,
};


pub struct GridSize {
    pub width: Length,
    pub height: Length,
}

impl GridSize {
    fn to_screen_space(&self, parent: &ScreenSpace, screen_size: &ScreenSize) -> ScreenSpace {
        let screen_width = screen_size.to_real_width();
        let screen_height = screen_size.to_real_height();

        ScreenSpace {
            position: parent.position,
            size: RealSize {
                width: self.width.to_screen_space(parent.size, screen_width, screen_size.width),
                height: self.height.to_screen_space(parent.size, screen_height, screen_size.height),
            },
            z_index: parent.z_index,
        }
    }
}


/// Displays children in a grid where each child has the same fixed size.
///
/// When the children overflow horizontally, it moves them to the next vertical row.
pub struct Grid {
    visible: bool,
    stretch: bool,
    location: Location,
    children: Vec<NodeHandle>,
    grid_size: Option<GridSize>,
    min_size: Option<MinSize>,
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

    fn min_size<'a>(&mut self, info: &mut SceneLayoutInfo<'a>) -> MinSize {
        *self.min_size.get_or_insert_with(|| {
            // TODO better min_size
            self.location.min_size(&info.screen_size)
        })
    }

    fn update_layout<'a>(&mut self, _handle: &NodeHandle, parent: &ScreenSpace, info: &mut SceneLayoutInfo<'a>) {
        let grid_size = self.grid_size.as_ref().expect("Grid is missing grid_size");

        let this_space = parent.modify(&self.location, &info.screen_size);

        let max_width = this_space.size.width;

        let mut child_space = grid_size.to_screen_space(&this_space, &info.screen_size);

        let child_width = child_space.size.width;
        let child_height = child_space.size.height;

        let mut width = 0.0;

        for child in self.children.iter() {
            let mut lock = child.lock();

            if lock.is_visible() {
                width += child_width;

                if width > child_width && width > max_width {
                    width = child_width;
                    child_space.position.x = this_space.position.x;
                    child_space.move_down(child_height);
                }

                let max_z_index = info.renderer.get_max_z_index();

                assert!(max_z_index >= this_space.z_index);

                child_space.z_index = max_z_index;

                lock.update_layout(child, &child_space, info);

                child_space.position.x = this_space.position.x + width;
            }
        }

        self.min_size = None;
    }

    fn render<'a>(&mut self, _info: &mut SceneRenderInfo<'a>) {}
}
