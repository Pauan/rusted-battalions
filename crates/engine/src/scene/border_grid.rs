use futures_signals::signal::{Signal, SignalExt};
use crate::scene::builder::{Node, make_builder, base_methods, location_methods, simple_method};
use crate::scene::{
    NodeHandle, MinSize, Location, Origin, Size, Offset, Padding, Length,
    ScreenSpace, NodeLayout, SceneLayoutInfo, SceneRenderInfo, ScreenSize,
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

    fn min_size(&self, screen_size: &ScreenSize) -> MinSize {
        MinSize {
            width: self.left.min_length(screen_size.width) + self.right.min_length(screen_size.width),
            height: self.up.min_length(screen_size.height) + self.down.min_length(screen_size.height),
        }
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
    min_size: Option<MinSize>,
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

    fn update_child<'a>(child: &Node, info: &mut SceneLayoutInfo<'a>, space: &ScreenSpace) {
        let mut lock = child.handle.lock();

        if lock.is_visible() {
            lock.update_layout(&child.handle, space, info);
        }
    }

    /*fn children_min_size<'a>(&mut self, info: &mut SceneLayoutInfo<'a>) -> MinSize {
        let quadrants = self.quadrants.expect("Missing quadrants");

        let up_left    = quadrants.up_left.min_size(info);
        let up         = quadrants.up.min_size(info);
        let up_right   = quadrants.up_right.min_size(info);

        let left       = quadrants.left.min_size(info);
        let center     = quadrants.center.min_size(info);
        let right      = quadrants.right.min_size(info);

        let down_left  = quadrants.down_left.min_size(info);
        let down       = quadrants.down.min_size(info);
        let down_right = quadrants.down_right.min_size(info);


        min_size = min_size.max_width(up_left.width + up_right.width);
        min_size = min_size.max_width(left.width + right.width);
        min_size = min_size.max_width(down_left.width + down_right.width);

        min_size = min_size.max_height(up_left.height + down_left.height);
        min_size = min_size.max_height(up.height + down.height);
        min_size = min_size.max_height(up_right.height + down_right.height);

        min_size
    }*/
}

make_builder!(BorderGrid, BorderGridBuilder);
base_methods!(BorderGrid, BorderGridBuilder);
location_methods!(BorderGrid, BorderGridBuilder, true);

impl BorderGridBuilder {
    /// Sets the [`Quadrants`] for the border grid.
    pub fn quadrants(mut self, mut quadrants: Quadrants) -> Self {
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

    fn min_size<'a>(&mut self, info: &mut SceneLayoutInfo<'a>) -> MinSize {
        if let Some(min_size) = self.min_size {
            min_size

        } else {
            let fixed_width = self.location.size.width.is_fixed();
            let fixed_height = self.location.size.height.is_fixed();

            let min_size = self.location.min_size(&info.screen_size);

            let new_size = if fixed_width && fixed_height {
                min_size

            } else {
                let border_size = self.border_size.as_ref().expect("Missing border_size");

                let child_size = border_size.min_size(&info.screen_size);

                let padding = self.location.padding.min_size(&child_size, &info.screen_size);

                MinSize {
                    width: if fixed_width {
                        min_size.width

                    } else {
                        child_size.width + padding.width
                    },

                    height: if fixed_height {
                        min_size.height

                    } else {
                        child_size.height + padding.height
                    },
                }
            };

            self.min_size = Some(new_size);
            new_size
        }
    }

    fn update_layout<'a>(&mut self, _handle: &NodeHandle, parent: &ScreenSpace, info: &mut SceneLayoutInfo<'a>) {
        let quadrants = self.quadrants.as_ref().expect("Missing quadrants");
        let border_size = self.border_size.as_ref().expect("Missing border_size");

        let this_space = parent.modify(&self.location, &info.screen_size);

        let size_left = border_size.left.to_screen_space(this_space.size[0], info.screen_size.width);
        let size_right = border_size.right.to_screen_space(this_space.size[0], info.screen_size.width);
        let size_up = border_size.up.to_screen_space(this_space.size[1], info.screen_size.height);
        let size_down = border_size.down.to_screen_space(this_space.size[1], info.screen_size.height);

        let position_left = this_space.position[0];
        let position_right = position_left + (this_space.size[0] - size_right).max(0.0);

        let position_up = this_space.position[1];
        let position_down = position_up + (this_space.size[1] - size_down).max(0.0);

        let center_width = (this_space.size[0] - size_left - size_right).max(0.0);
        let center_height = (this_space.size[1] - size_up - size_down).max(0.0);

        let center_left = position_left + size_left;
        let center_up = position_up + size_up;


        Self::update_child(&quadrants.up_left, info, &ScreenSpace {
            position: [position_left, position_up],
            size: [size_left, size_up],
            z_index: info.renderer.get_max_z_index(),
        });

        Self::update_child(&quadrants.up, info, &ScreenSpace {
            position: [center_left, position_up],
            size: [center_width, size_up],
            z_index: info.renderer.get_max_z_index(),
        });

        Self::update_child(&quadrants.up_right, info, &ScreenSpace {
            position: [position_right, position_up],
            size: [size_right, size_up],
            z_index: info.renderer.get_max_z_index(),
        });


        Self::update_child(&quadrants.left, info, &ScreenSpace {
            position: [position_left, center_up],
            size: [size_left, center_height],
            z_index: info.renderer.get_max_z_index(),
        });

        Self::update_child(&quadrants.center, info, &ScreenSpace {
            position: [center_left, center_up],
            size: [center_width, center_height],
            z_index: info.renderer.get_max_z_index(),
        });

        Self::update_child(&quadrants.right, info, &ScreenSpace {
            position: [position_right, center_up],
            size: [size_right, center_height],
            z_index: info.renderer.get_max_z_index(),
        });


        Self::update_child(&quadrants.down_left, info, &ScreenSpace {
            position: [position_left, position_down],
            size: [size_left, size_down],
            z_index: info.renderer.get_max_z_index(),
        });

        Self::update_child(&quadrants.down, info, &ScreenSpace {
            position: [center_left, position_down],
            size: [center_width, size_down],
            z_index: info.renderer.get_max_z_index(),
        });

        Self::update_child(&quadrants.down_right, info, &ScreenSpace {
            position: [position_right, position_down],
            size: [size_right, size_down],
            z_index: info.renderer.get_max_z_index(),
        });


        self.min_size = None;
    }

    fn render<'a>(&mut self, _info: &mut SceneRenderInfo<'a>) {}
}
