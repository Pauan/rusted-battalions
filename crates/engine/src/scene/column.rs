use futures_signals::signal::{Signal, SignalExt};
use futures_signals::signal_vec::{SignalVec, SignalVecExt};
use crate::scene::builder::{Node, make_builder, base_methods, location_methods, children_methods};
use crate::scene::{
    NodeHandle, MinSize, Location, Origin, Size, Offset, Percentage, Padding,
    ScreenSpace, NodeLayout, SceneLayoutInfo, SceneRenderInfo,
};


/// Displays children in a column from top-to-bottom.
///
/// # Layout
///
/// * If a child has `stretch(false)` then the child keeps its normal size.
///
/// * If a child has `stretch(true)` then the child's width is normal, but
///    its height is expanded to fill the available empty space of the column.
///
///    If there are multiple children with `stretch(true)` then the empty space
///    is evenly distributed to each child.
///
/// ----
///
/// If the column has a fixed size ([`Length::Px`] or [`Length::Screen`])
/// then the column is displayed with that size.
///
/// However, if the column has a dynamic size ([`Length::Parent`]) then the
/// column's minimum size will be based on the children's size:
///
/// * The minimum height is based on the sum of all the non-stretch children's height.
///
/// * The minimum width is based on the maximum of all the children's width.
///
/// The final size of the column will depend on the column's parent:
///
/// * If the parent of the column is a [`Row`] or [`Column`] then the column's
///   size is the same as the column's minimum size.
///
/// * Otherwise the column's size is the same as the parent's size.
pub struct Column {
    visible: bool,
    stretch: bool,
    location: Location,
    children: Vec<NodeHandle>,
    stretch_children: usize,
    min_height: Percentage,
    min_size: Option<MinSize>,
}

impl Column {
    #[inline]
    fn new() -> Self {
        Self {
            visible: true,
            stretch: false,
            location: Location::default(),
            children: vec![],
            stretch_children: 0,
            min_height: 0.0,
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

                    if !fixed_width {
                        min_size = min_size.max_width(child_size);
                    }

                } else {
                    if !fixed_height {
                        min_size.height += child_size.height;
                    }

                    if !fixed_width {
                        min_size = min_size.max_width(child_size);
                    }
                }
            }
        }

        self.min_height = min_size.height;

        min_size
    }
}

make_builder!(Column, ColumnBuilder);
base_methods!(Column, ColumnBuilder);
location_methods!(Column, ColumnBuilder, true);
children_methods!(Column, ColumnBuilder);

impl NodeLayout for Column {
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

    fn update_layout<'a>(&mut self, _handle: &NodeHandle, parent: &ScreenSpace, info: &mut SceneLayoutInfo<'a>) {
        // This is needed in order to calculate the min_height and stretch_children
        self.min_size(info);

        let mut this_space = parent.modify(&self.location, &info.screen_size);

        let empty_space = (this_space.size[1] - self.min_height).max(0.0);

        let stretch_height = empty_space / (self.stretch_children as f32);

        for child in self.children.iter() {
            let mut lock = child.lock();

            if lock.is_visible() {
                let child_space = if lock.is_stretch() {
                    ScreenSpace {
                        position: this_space.position,
                        size: [this_space.size[0], stretch_height],
                        z_index: this_space.z_index,
                    }

                } else {
                    let child_size = lock.min_size(info);

                    ScreenSpace {
                        position: this_space.position,
                        size: [this_space.size[0], child_size.height],
                        z_index: this_space.z_index,
                    }
                };

                lock.update_layout(child, &child_space, info);

                this_space.move_down(child_space.size[1]);
            }
        }

        self.stretch_children = 0;
        self.min_height = 0.0;
        self.min_size = None;
    }

    fn render<'a>(&mut self, _info: &mut SceneRenderInfo<'a>) {}
}
