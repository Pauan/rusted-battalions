use futures_signals::signal::{Signal, SignalExt};
use crate::scene::builder::{Node, make_builder, base_methods, location_methods, simple_method};
use crate::scene::{
    NodeHandle, MinSize, Location, Origin, Size, Offset, Padding, Length,
    RealLocation, NodeLayout, SceneLayoutInfo, SceneRenderInfo, ScreenSize,
    RealSize, RealPosition,
};


pub struct BorderSize {
    pub up: Length,
    pub down: Length,
    pub left: Length,
    pub right: Length,
}

impl BorderSize {
    #[inline]
    pub fn all(length: Length) -> Self {
        Self {
            up: length,
            down: length,
            left: length,
            right: length,
        }
    }

    /// Calculates the size of the BorderSize
    // TODO code duplication with Padding::children_size
    fn min_size(&self, parent: &MinSize, screen_size: &ScreenSize) -> RealSize {
        let screen_width = screen_size.to_real_width();
        let screen_height = screen_size.to_real_height();

        let mut width = 0.0;
        let mut height = 0.0;

        let mut width_ratio = 1.0;
        let mut height_ratio = 1.0;

        let mut cross_width_ratio = 0.0;
        let mut cross_height_ratio = 0.0;

        match self.left.min_length(parent, &screen_width, screen.width) {
            MinLength::Screen(x) => {
                width += x;
            },
            MinLength::ChildrenWidth(x) => {
                width_ratio += x;
            },
            MinLength::ChildrenHeight(x) => {
                cross_width_ratio += x;
            },
        }

        match self.right.min_length(parent, &screen_width, screen.width) {
            MinLength::Screen(x) => {
                width += x;
            },
            MinLength::ChildrenWidth(x) => {
                width_ratio += x;
            },
            MinLength::ChildrenHeight(x) => {
                cross_width_ratio += x;
            },
        }

        match self.up.min_length(parent, &screen_height, screen.height) {
            MinLength::Screen(x) => {
                height += x;
            },
            MinLength::ChildrenWidth(x) => {
                cross_height_ratio += x;
            },
            MinLength::ChildrenHeight(x) => {
                height_ratio += x;
            },
        }

        match self.down.min_length(parent, &screen_height, screen.height) {
            MinLength::Screen(x) => {
                height += x;
            },
            MinLength::ChildrenWidth(x) => {
                cross_height_ratio += x;
            },
            MinLength::ChildrenHeight(x) => {
                height_ratio += x;
            },
        }

        let parent = parent.to_real_size();

        if cross_width_ratio != 0.0 {
            if cross_height_ratio != 0.0 {
                panic!("BorderSize has conflicting recursive ChildrenWidth and ChildrenHeight");

            } else {
                width += (parent.height - height).max(0.0) * cross_width_ratio;
            }

        } else if cross_height_ratio != 0.0 {
            height += (parent.width - width).max(0.0) * cross_height_ratio;
        }

        width += (parent.width - width).max(0.0) * (1.0 - (1.0 / width_ratio));
        height += (parent.height - height).max(0.0) * (1.0 - (1.0 / height_ratio));

        debug_assert!(width >= 0.0);
        debug_assert!(height >= 0.0);

        RealSize { width, height }
    }
}


pub struct Quadrants {
    pub up_left: Node,
    pub up: Node,
    pub up_right: Node,

    pub left: Node,
    pub center: Node,
    pub right: Node,

    pub down_left: Node,
    pub down: Node,
    pub down_right: Node,
}

impl Quadrants {
    fn iter_mut(&mut self) -> impl Iterator<Item = &mut Node> {
        [
            &mut self.up_left,
            &mut self.up,
            &mut self.up_right,

            &mut self.left,
            &mut self.center,
            &mut self.right,

            &mut self.down_left,
            &mut self.down,
            &mut self.down_right,
        ].into_iter()
    }
}


/// Displays children in a 3x3 grid where the center quadrant stretches.
pub struct BorderGrid {
    visible: bool,
    stretch: bool,
    location: Location,
    quadrants: Option<Quadrants>,
    border_size: Option<BorderSize>,
    min_size: Option<RealSize>,
}

impl BorderGrid {
    #[inline]
    fn new() -> Self {
        Self {
            visible: true,
            stretch: false,
            location: Location::default(),
            quadrants: None,
            border_size: None,
            min_size: None,
        }
    }

    fn update_child<'a>(child: &Node, info: &mut SceneLayoutInfo<'a>, location: &RealLocation) {
        let mut lock = child.handle.lock();

        if lock.is_visible() {
            lock.update_layout(&child.handle, location, info);
        }
    }
}

make_builder!(BorderGrid, BorderGridBuilder);
base_methods!(BorderGrid, BorderGridBuilder);
location_methods!(BorderGrid, BorderGridBuilder, true);

impl BorderGridBuilder {
    /// Sets the [`Quadrants`] for the border grid.
    pub fn quadrants(mut self, mut quadrants: Quadrants) -> Self {
        // TODO handle this better
        for quadrant in quadrants.iter_mut() {
            self.callbacks.transfer(&mut quadrant.callbacks);
        }

        self.state.lock().quadrants = Some(quadrants);
        self
    }

    simple_method!(
        /// Sets the [`BorderSize`] for the border grid.
        border_size,
        border_size_signal,
        true,
        true,
        |state, border_size: BorderSize| {
            state.border_size = Some(border_size);
        },
    );
}

impl NodeLayout for BorderGrid {
    #[inline]
    fn is_visible(&mut self) -> bool {
        self.visible
    }

    #[inline]
    fn is_stretch(&mut self) -> bool {
        self.stretch
    }

    fn min_size<'a>(&mut self, parent: &MinSize, info: &mut SceneLayoutInfo<'a>) -> RealSize {
        if let Some(min_size) = self.min_size {
            min_size

        } else {
            let quadrants = self.quadrants.as_ref().expect("BorderGrid is missing quadrants");
            let border_size = self.border_size.as_ref().expect("BorderGrid is missing border_size");

            let min_size = self.location.min_size(parent, &info.screen_size);

            let min_size = if min_size.width.is_err() || min_size.height.is_err() {
                let child_parent = self.location.padding.children_size(parent, &min_size, &info.screen_size);

                let border_size = border_size.min_size(&child_parent, &info.screen_size);

                let center_parent = RealSize {
                    width: (child_parent.width - border_size.width).max(0.0),
                    height: (child_parent.height - border_size.height).max(0.0),
                };

                let center_size = quadrants.center.handle.lock().min_size(&center_parent, info);

                let children_size = border_size + center_size;

                let padding = self.location.padding.to_screen_space(parent, &children_size, &info.screen_size);

                RealSize {
                    width: min_size.width.unwrap_or_else(|| {
                        children_size.width + padding.width
                    }),
                    height: min_size.height.unwrap_or_else(|| {
                        children_size.height + padding.height
                    }),
                }

            } else {
                min_size.to_real_size()
            };

            self.min_size = Some(min_size);
            min_size
        }
    }

    fn update_layout<'a>(&mut self, _handle: &NodeHandle, parent: &RealLocation, info: &mut SceneLayoutInfo<'a>) {
        let quadrants = self.quadrants.as_ref().expect("BorderGrid is missing quadrants");
        let border_size = self.border_size.as_ref().expect("BorderGrid is missing border_size");

        let children_min_size = self.min_size(&parent.size, info);

        let this_location = parent.modify(&self.location, &children_min_size, &info.screen_size);

        let screen_width = info.screen_size.to_real_width();
        let screen_height = info.screen_size.to_real_height();

        let size_left = border_size.left.to_screen_space(this_location.size, screen_width, info.screen_size.width);
        let size_right = border_size.right.to_screen_space(this_location.size, screen_width, info.screen_size.width);
        let size_up = border_size.up.to_screen_space(this_location.size, screen_height, info.screen_size.height);
        let size_down = border_size.down.to_screen_space(this_location.size, screen_height, info.screen_size.height);

        let position_left = this_location.position.x;
        let position_right = position_left + (this_location.size.width - size_right).max(0.0);

        let position_up = this_location.position.y;
        let position_down = position_up + (this_location.size.height - size_down).max(0.0);

        let center_width = (this_location.size.width - size_left - size_right).max(0.0);
        let center_height = (this_location.size.height - size_up - size_down).max(0.0);

        let center_left = position_left + size_left;
        let center_up = position_up + size_up;


        Self::update_child(&quadrants.up_left, info, &RealLocation {
            position: RealPosition {
                x: position_left,
                y: position_up,
            },
            size: RealSize {
                width: size_left,
                height: size_up,
            },
            z_index: info.renderer.get_max_z_index(),
        });

        Self::update_child(&quadrants.up, info, &RealLocation {
            position: RealPosition {
                x: center_left,
                y: position_up,
            },
            size: RealSize {
                width: center_width,
                height: size_up,
            },
            z_index: info.renderer.get_max_z_index(),
        });

        Self::update_child(&quadrants.up_right, info, &RealLocation {
            position: RealPosition {
                x: position_right,
                y: position_up,
            },
            size: RealSize {
                width: size_right,
                height: size_up,
            },
            z_index: info.renderer.get_max_z_index(),
        });


        Self::update_child(&quadrants.left, info, &RealLocation {
            position: RealPosition {
                x: position_left,
                y: center_up,
            },
            size: RealSize {
                width: size_left,
                height: center_height,
            },
            z_index: info.renderer.get_max_z_index(),
        });

        Self::update_child(&quadrants.center, info, &RealLocation {
            position: RealPosition {
                x: center_left,
                y: center_up,
            },
            size: RealSize {
                width: center_width,
                height: center_height,
            },
            z_index: info.renderer.get_max_z_index(),
        });

        Self::update_child(&quadrants.right, info, &RealLocation {
            position: RealPosition {
                x: position_right,
                y: center_up,
            },
            size: RealSize {
                width: size_right,
                height: center_height,
            },
            z_index: info.renderer.get_max_z_index(),
        });


        Self::update_child(&quadrants.down_left, info, &RealLocation {
            position: RealPosition {
                x: position_left,
                y: position_down,
            },
            size: RealSize {
                width: size_left,
                height: size_down,
            },
            z_index: info.renderer.get_max_z_index(),
        });

        Self::update_child(&quadrants.down, info, &RealLocation {
            position: RealPosition {
                x: center_left,
                y: position_down,
            },
            size: RealSize {
                width: center_width,
                height: size_down,
            },
            z_index: info.renderer.get_max_z_index(),
        });

        Self::update_child(&quadrants.down_right, info, &RealLocation {
            position: RealPosition {
                x: position_right,
                y: position_down,
            },
            size: RealSize {
                width: size_right,
                height: size_down,
            },
            z_index: info.renderer.get_max_z_index(),
        });


        self.min_size = None;
    }

    fn render<'a>(&mut self, _info: &mut SceneRenderInfo<'a>) {}
}
