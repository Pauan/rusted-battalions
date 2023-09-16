use futures_signals::signal::{Signal, SignalExt};
use futures_signals::signal_vec::{SignalVec, SignalVecExt};
use crate::scene::builder::{Node, make_builder, base_methods, location_methods, children_methods};
use crate::scene::{
    NodeHandle, MinSize, Location, Origin, Size, Offset, Percentage, Padding,
    RealLocation, NodeLayout, SceneLayoutInfo, SceneRenderInfo, RealSize,
};


/// Displays children in a row from left-to-right.
///
/// # Layout
///
/// * If a child has `stretch(false)` then the child keeps its normal size.
///
/// * If a child has `stretch(true)` then the child's height is normal, but
///    its width is expanded to fill the available empty space of the row.
///
///    If there are multiple children with `stretch(true)` then the empty space
///    is evenly distributed to each child.
///
/// ----
///
/// If the row has a fixed size ([`Length::Px`] or [`Length::Screen`])
/// then the row is displayed with that size.
///
/// However, if the row has a dynamic size ([`Length::Parent`]) then the
/// row's minimum size will be based on the children's size:
///
/// * The minimum width is based on the sum of all the non-stretch children's width.
///
/// * The minimum height is based on the maximum of all the children's height.
///
/// The final size of the row will depend on the row's parent:
///
/// * If the parent of the row is a [`Row`] or [`Column`] then the row's
///   size is the same as the row's minimum size.
///
/// * Otherwise the row's size is the same as the parent's size.
pub struct Row {
    visible: bool,
    stretch: bool,
    location: Location,
    children: Vec<NodeHandle>,
    stretch_children: usize,
    min_width: Percentage,
    min_size: Option<MinSize>,
}

impl Row {
    #[inline]
    fn new() -> Self {
        Self {
            visible: true,
            stretch: false,
            location: Location::default(),
            children: vec![],
            stretch_children: 0,
            min_width: 0.0,
            min_size: None,
        }
    }

    fn children_min_size<'a>(&mut self, info: &mut SceneLayoutInfo<'a>, fixed_width: bool, fixed_height: bool) -> MinSize {
        let mut min_size = MinSize {
            width: 0.0,
            height: 0.0,
        };

        for child in self.children.iter() {
            let mut child = child.lock();

            if child.is_visible() {
                let child_size = child.min_size(info);

                if child.is_stretch() {
                    self.stretch_children += 1;

                    if !fixed_height {
                        min_size = min_size.max_height(child_size);
                    }

                } else {
                    if !fixed_width {
                        min_size.width += child_size.width;
                    }

                    if !fixed_height {
                        min_size = min_size.max_height(child_size);
                    }
                }
            }
        }

        self.min_width = min_size.width;

        min_size
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

            let min_size = self.children_min_size(info, fixed_width, fixed_height);

            let padding = self.location.padding.min_size(&min_size, &info.screen_size);

            let this_size = self.location.min_size(&info.screen_size);

            let new_size = MinSize {
                width: if fixed_width {
                    this_size.width

                } else {
                    min_size.width + padding.width
                },

                height: if fixed_height {
                    this_size.height

                } else {
                    min_size.height + padding.height
                },
            };

            self.min_size = Some(new_size);
            new_size
        }
    }

    fn update_layout<'a>(&mut self, _handle: &NodeHandle, parent: &RealLocation, info: &mut SceneLayoutInfo<'a>) {
        // This is needed in order to calculate the min_width and stretch_children
        self.min_size(info);

        let mut this_location = parent.modify(&self.location, &info.screen_size);

        let empty_space = (this_location.size.width - self.min_width).max(0.0);

        let stretch_width = empty_space / (self.stretch_children as f32);

        for child in self.children.iter() {
            let mut lock = child.lock();

            if lock.is_visible() {
                let max_z_index = info.renderer.get_max_z_index();

                assert!(max_z_index >= this_location.z_index);

                let child_location = if lock.is_stretch() {
                    RealLocation {
                        position: this_location.position,
                        size: RealSize {
                            width: stretch_width,
                            height: this_location.size.height,
                        },
                        z_index: max_z_index,
                    }

                } else {
                    let child_size = lock.min_size(info);

                    RealLocation {
                        position: this_location.position,
                        size: RealSize {
                            width: child_size.width,
                            height: this_location.size.height,
                        },
                        z_index: max_z_index,
                    }
                };

                lock.update_layout(child, &child_location, info);

                this_location.move_right(child_location.size.width);
            }
        }

        self.stretch_children = 0;
        self.min_width = 0.0;
        self.min_size = None;
    }

    fn render<'a>(&mut self, _info: &mut SceneRenderInfo<'a>) {}
}
