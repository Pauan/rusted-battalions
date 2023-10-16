use futures_signals::signal::{Signal, SignalExt};
use futures_signals::signal_vec::{SignalVec, SignalVecExt};
use crate::scene::builder::{Node, BuilderChanged, make_builder, base_methods, location_methods, children_methods};
use crate::scene::{
    NodeHandle, Location, Origin, Size, Offset, Padding, SmallestSize, Order,
    RealLocation, NodeLayout, SceneLayoutInfo, SceneRenderInfo, RealSize,
};


struct Child {
    size: SmallestSize,
    handle: NodeHandle,
}


/// Displays children on top of each other.
///
/// # Layout
///
/// The children are all displayed on the same position as the stack.
///
/// # Sizing
///
/// * [`Length::SmallestWidth`]: the maximum of all the children's smallest width.
///
/// * [`Length::SmallestHeight`]: the maximum of all the children's smallest height.
pub struct Stack {
    visible: bool,
    location: Location,
    children: Vec<NodeHandle>,

    computed_children: Vec<Child>,
}

impl Stack {
    #[inline]
    fn new() -> Self {
        Self {
            visible: true,
            location: Location::default(),
            children: vec![],

            computed_children: vec![],
        }
    }

    fn children_size<'a>(&mut self, parent: &SmallestSize, info: &mut SceneLayoutInfo<'a>) -> RealSize {
        let mut min_size = RealSize {
            width: 0.0,
            height: 0.0,
        };

        self.computed_children.reserve(self.children.len());

        for child in self.children.iter() {
            let mut lock = child.lock();

            if lock.is_visible() {
                let size = lock.smallest_size(parent, info);

                let real_size = size.real_size();

                min_size.width = min_size.width.max(real_size.width);
                min_size.height = min_size.height.max(real_size.height);

                self.computed_children.push(Child {
                    size,
                    handle: child.clone(),
                });
            }
        }

        min_size
    }
}

make_builder!(Stack, StackBuilder);
base_methods!(Stack, StackBuilder);
location_methods!(Stack, StackBuilder);
children_methods!(Stack, StackBuilder);

impl NodeLayout for Stack {
    #[inline]
    fn is_visible(&mut self) -> bool {
        self.visible
    }

    fn smallest_size<'a>(&mut self, parent: &SmallestSize, info: &mut SceneLayoutInfo<'a>) -> SmallestSize {
        let smallest_size = self.location.size.smallest_size(&info.screen_size).parent_to_smallest(parent);

        let padding = self.location.padding.to_screen(parent, &smallest_size, &info.screen_size);

        smallest_size.with_padding(parent, padding, |parent| {
            self.children_size(&parent, info)
        })
    }

    fn update_layout<'a>(&mut self, _handle: &NodeHandle, parent: &RealLocation, smallest_size: &SmallestSize, info: &mut SceneLayoutInfo<'a>) {
        let this_location = self.location.children_location(parent, &smallest_size.real_size(), &info);

        for child in self.computed_children.iter() {
            let mut lock = child.handle.lock();
            lock.update_layout(&child.handle, &this_location, &child.size, info);
        }

        self.computed_children.clear();
    }

    fn render<'a>(&mut self, _info: &mut SceneRenderInfo<'a>) {}
}
