use futures_signals::signal::{Signal, SignalExt};
use futures_signals::signal_vec::{SignalVec, SignalVecExt};
use crate::scene::builder::{Node, make_builder, base_methods, location_methods, children_methods};
use crate::scene::{
    NodeHandle, Location, Origin, Size, Offset, Padding,
    RealLocation, NodeLayout, SceneLayoutInfo, SceneRenderInfo,
};


/// Displays children on top of each other.
///
/// # Layout
///
/// The children are all displayed on the same position as the stack.
///
/// # Sizing
///
/// * [`Length::ChildrenWidth`]: the maximum of all the children's width.
///
/// * [`Length::ChildrenHeight`]: the maximum of all the children's height.
pub struct Stack {
    visible: bool,
    stretch: bool,
    location: Location,
    children: Vec<NodeHandle>,
    min_size: Option<RealSize>,
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

    fn children_size<'a>(&mut self, parent: &RealSize, info: &mut SceneLayoutInfo<'a>) -> RealSize {
        let mut min_size = RealSize {
            width: 0.0,
            height: 0.0,
        };

        for child in self.children.iter() {
            let mut child = child.lock();

            if child.is_visible() {
                let child_size = child.min_size(parent, info);

                min_size.width = min_size.width.max(child_size.width);
                min_size.height = min_size.height.max(child_size.height);
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

    fn min_size<'a>(&mut self, parent: &RealSize, info: &mut SceneLayoutInfo<'a>) -> RealSize {
        if let Some(min_size) = self.min_size {
            min_size

        } else {
            let min_size = self.location.min_size(parent, &info.screen_size);

            let min_size = if min_size.width.is_err() || min_size.height.is_err() {
                let child_parent = self.location.padding.children_size(parent, &min_size, &info.screen_size);

                let children_size = self.children_size(&child_parent, info);

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
        let children_min_size = self.min_size(&parent.size, info);

        let mut this_location = parent.modify(&self.location, &children_min_size, &info.screen_size);

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
