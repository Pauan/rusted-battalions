use futures_signals::signal::{Signal, SignalExt};
use futures_signals::signal_vec::{SignalVec, SignalVecExt};
use crate::scene::builder::{Node, make_builder, base_methods, location_methods, children_methods};
use crate::scene::{
    NodeHandle, MinSize, Location, Origin, Size, Offset, Padding,
    RealLocation, NodeLayout, SceneLayoutInfo, SceneRenderInfo,
};


/// Displays children on top of each other.
///
/// # Layout
///
/// The children are all displayed on the same position as the stack.
///
/// The children are ordered so that later children display on top of earlier children.
pub struct Stack {
    visible: bool,
    stretch: bool,
    location: Location,
    children: Vec<NodeHandle>,
    min_size: Option<MinSize>,
}

impl Stack {
    #[inline]
    fn new() -> Self {
        Self {
            visible: true,
            stretch: false,
            location: Location::default(),
            children: vec![],
            min_size: None,
        }
    }

    fn children_min_size<'a>(&mut self, info: &mut SceneLayoutInfo<'a>) -> MinSize {
        let mut min_size = MinSize {
            width: 0.0,
            height: 0.0,
        };

        for child in self.children.iter() {
            let mut child = child.lock();

            if child.is_visible() {
                let child_size = child.min_size(info);
                min_size = min_size.max(child_size);
            }
        }

        min_size
    }
}

make_builder!(Stack, StackBuilder);
base_methods!(Stack, StackBuilder);
location_methods!(Stack, StackBuilder, true);
children_methods!(Stack, StackBuilder);

impl NodeLayout for Stack {
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
                let child_size = self.children_min_size(info);

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

    fn update_layout<'a>(&mut self, _handle: &NodeHandle, parent: &RealLocation, info: &mut SceneLayoutInfo<'a>) {
        let mut this_location = parent.modify(&self.location, &info.screen_size);

        for child in self.children.iter() {
            let mut lock = child.lock();

            if lock.is_visible() {
                let max_z_index = info.renderer.get_max_z_index();

                assert!(max_z_index >= this_location.z_index);

                this_location.z_index = max_z_index;

                lock.update_layout(child, &this_location, info);
            }
        }

        self.min_size = None;
    }

    fn render<'a>(&mut self, _info: &mut SceneRenderInfo<'a>) {}
}
