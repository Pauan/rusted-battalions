use futures_signals::signal::{Signal, SignalExt};
use crate::scene::builder::{Node, make_builder, base_methods, location_methods, simple_method};
use crate::scene::{
    NodeHandle, Location, Origin, Size, Offset, Padding, Length, SmallestSize,
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

    fn real_size(&self, parent: &SmallestSize, screen: &ScreenSize) -> RealSize {
        let up = self.up.smallest_length(&screen.height).parent_to_screen(parent).unwrap();
        let down = self.down.smallest_length(&screen.height).parent_to_screen(parent).unwrap();
        let left = self.left.smallest_length(&screen.width).parent_to_screen(parent).unwrap();
        let right = self.right.smallest_length(&screen.width).parent_to_screen(parent).unwrap();

        RealSize {
            width: left + right,
            height: up + down,
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
    location: Location,

    quadrants: Option<Quadrants>,
    border_size: Option<BorderSize>,

    center_size: Option<SmallestSize>,
}

impl BorderGrid {
    #[inline]
    fn new() -> Self {
        Self {
            visible: true,
            location: Location::default(),

            quadrants: None,
            border_size: None,

            center_size: None,
        }
    }

    fn update_child<'a>(child: &Node, info: &mut SceneLayoutInfo<'a>, location: &RealLocation) {
        let mut lock = child.handle.lock();

        if lock.is_visible() {
            let smallest = lock.smallest_size(&location.size.smallest_size(), info);
            lock.update_layout(&child.handle, location, &smallest, info);
        }
    }

    fn children_size<'a>(&mut self, parent: &SmallestSize, info: &mut SceneLayoutInfo<'a>) -> RealSize {
        debug_assert!(self.center_size.is_none());

        let quadrants = self.quadrants.as_ref().expect("BorderGrid is missing quadrants");
        let border_size = self.border_size.as_ref().expect("BorderGrid is missing border_size");

        let border_size = border_size.real_size(&parent, &info.screen_size);

        let center_size = {
            let mut lock = quadrants.center.handle.lock();

            if lock.is_visible() {
                let center_parent = *parent - border_size;
                let smallest = lock.smallest_size(&center_parent, info);
                self.center_size = Some(smallest);
                smallest.real_size()

            } else {
                RealSize::zero()
            }
        };

        RealSize {
            width: center_size.width + border_size.width,
            height: center_size.height + border_size.height,
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

        {
            let mut state = self.state.lock();
            assert!(state.quadrants.is_none());
            state.quadrants = Some(quadrants);
        }

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

    fn smallest_size<'a>(&mut self, parent: &SmallestSize, info: &mut SceneLayoutInfo<'a>) -> SmallestSize {
        let smallest_size = self.location.size.smallest_size(&info.screen_size);

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
        let quadrants = self.quadrants.as_ref().expect("BorderGrid is missing quadrants");
        let border_size = self.border_size.as_ref().expect("BorderGrid is missing border_size");

        let smallest_size = smallest_size.real_size();

        let this_location = self.location.children_location(parent, &smallest_size, &info.screen_size);

        let size_up = border_size.up.real_length(&parent.size, &smallest_size, &info.screen_size.height);
        let size_down = border_size.down.real_length(&parent.size, &smallest_size, &info.screen_size.height);
        let size_left = border_size.left.real_length(&parent.size, &smallest_size, &info.screen_size.width);
        let size_right = border_size.right.real_length(&parent.size, &smallest_size, &info.screen_size.width);

        let center_width = (this_location.size.width - size_left - size_right).max(0.0);
        let center_height = (this_location.size.height - size_up - size_down).max(0.0);

        let position_up = this_location.position.y;
        let position_down = position_up + (this_location.size.height - size_down).max(0.0);

        let position_left = this_location.position.x;
        let position_right = position_left + (this_location.size.width - size_right).max(0.0);

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

        {
            let mut lock = quadrants.center.handle.lock();

            if lock.is_visible() {
                let location = RealLocation {
                    position: RealPosition {
                        x: center_left,
                        y: center_up,
                    },
                    size: RealSize {
                        width: center_width,
                        height: center_height,
                    },
                    z_index: info.renderer.get_max_z_index(),
                };

                let smallest = match self.center_size {
                    Some(center_size) => center_size,
                    None => lock.smallest_size(&location.size.smallest_size(), info),
                };

                lock.update_layout(&quadrants.center.handle, &location, &smallest, info);
            }
        }

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


        self.center_size = None;
    }

    fn render<'a>(&mut self, _info: &mut SceneRenderInfo<'a>) {}
}
