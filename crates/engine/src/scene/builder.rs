use futures_signals::signal_vec::VecDiff;
use futures::future::{AbortHandle, Abortable};
use std::future::Future;

use crate::util::{Arc, Lock};
use crate::scene::{SceneChanged, NodeHandle, MinSize, NodeLayout, ScreenSpace, SceneLayoutInfo, SceneRenderInfo};


pub(crate) struct Callbacks {
    after_inserted: Vec<Box<dyn FnOnce(Arc<SceneChanged>)>>,
    after_removed: Vec<Box<dyn FnOnce()>>,
}

impl Callbacks {
    pub(crate) fn new() -> Self {
        Self {
            after_inserted: vec![],
            after_removed: vec![],
        }
    }

    pub(crate) fn transfer(&mut self, other: &mut Self) {
        self.after_inserted.append(&mut other.after_inserted);
        self.after_removed.append(&mut other.after_removed);
    }

    fn after_inserted<F>(&mut self, f: F) where F: FnOnce(Arc<SceneChanged>) + 'static {
        self.after_inserted.push(Box::new(f));
    }

    fn after_removed<F>(&mut self, f: F) where F: FnOnce() + 'static {
        self.after_removed.push(Box::new(f));
    }

    pub(crate) fn trigger_after_inserted(&mut self, root: &Arc<SceneChanged>) {
        for f in self.after_inserted.drain(..) {
            f(root.clone());
        }
    }

    pub(crate) fn spawn_local<F, C>(&mut self, callback: C)
        where C: FnOnce(&Arc<SceneChanged>) -> F + 'static,
              F: Future<Output = ()> + 'static {

        let (handle, registration) = AbortHandle::new_pair();

        self.after_inserted({
            let handle = handle.clone();

            move |root| {
                if !handle.is_aborted() {
                    let future = callback(&root);

                    root.spawn_local(Box::pin(async move {
                        let _ = Abortable::new(future, registration).await;
                    }));
                }
            }
        });

        self.after_removed(move || {
            handle.abort();
        });
    }
}

impl Drop for Callbacks {
    fn drop(&mut self) {
        for f in self.after_removed.drain(..) {
            f();
        }
    }
}


pub(crate) struct ChildrenState {
    root: Arc<SceneChanged>,
    callbacks: Vec<Callbacks>,
}

impl ChildrenState {
    pub(crate) fn new(root: Arc<SceneChanged>) -> Self {
        Self {
            root,
            callbacks: vec![],
        }
    }

    pub(crate) fn update(&mut self, children: &mut Vec<NodeHandle>, change: VecDiff<Node>) {
        match change {
            VecDiff::Replace { values } => {
                let len = values.len();

                let mut handles = Vec::with_capacity(len);
                let mut callbacks = Vec::with_capacity(len);

                for mut child in values.into_iter() {
                    child.callbacks.trigger_after_inserted(&self.root);

                    handles.push(child.handle);
                    callbacks.push(child.callbacks);
                }

                *children = handles;
                self.callbacks = callbacks;
            },
            VecDiff::InsertAt { index, mut value } => {
                value.callbacks.trigger_after_inserted(&self.root);

                children.insert(index, value.handle);
                self.callbacks.insert(index, value.callbacks);
            },
            VecDiff::UpdateAt { index, mut value } => {
                value.callbacks.trigger_after_inserted(&self.root);

                children[index] = value.handle;
                self.callbacks[index] = value.callbacks;
            },
            VecDiff::RemoveAt { index } => {
                children.remove(index);
                self.callbacks.remove(index);
            },
            VecDiff::Move { old_index, new_index } => {
                let old_child = children.remove(old_index);
                let old_callback = self.callbacks.remove(old_index);

                children.insert(new_index, old_child);
                self.callbacks.insert(new_index, old_callback);
            },
            VecDiff::Push { mut value } => {
                value.callbacks.trigger_after_inserted(&self.root);

                children.push(value.handle);
                self.callbacks.push(value.callbacks);
            },
            VecDiff::Pop {} => {
                children.pop().unwrap();
                self.callbacks.pop().unwrap();
            },
            VecDiff::Clear {} => {
                children.clear();
                self.callbacks.clear();
            },
        }

        self.root.trigger_layout_change();
    }
}


pub(crate) struct OptionNode {
    pub(crate) child: Option<Node>,
}

impl OptionNode {
    #[inline]
    pub(crate) fn new() -> Lock<Self> {
        Lock::new(Self {
            child: None,
        })
    }

    pub(crate) fn update(&mut self, mut new_child: Option<Node>, root: &Arc<SceneChanged>) {
        if let Some(new_child) = &mut new_child {
            new_child.callbacks.trigger_after_inserted(&root);
        }

        if self.child.is_some() || new_child.is_some() {
            self.child = new_child;
            root.trigger_layout_change();
        }
    }
}

impl NodeLayout for OptionNode {
    fn is_visible(&mut self) -> bool {
        if let Some(child) = &self.child {
            child.handle.lock().is_visible()

        } else {
            false
        }
    }

    fn is_stretch(&mut self) -> bool {
        if let Some(child) = &self.child {
            child.handle.lock().is_stretch()

        } else {
            false
        }
    }

    fn min_size<'a>(&mut self, info: &mut SceneLayoutInfo<'a>) -> MinSize {
        if let Some(child) = &self.child {
            child.handle.lock().min_size(info)

        } else {
            MinSize { width: 0.0, height: 0.0 }
        }
    }

    fn update_layout<'a>(&mut self, _handle: &NodeHandle, parent: &ScreenSpace, info: &mut SceneLayoutInfo<'a>) {
        if let Some(child) = &self.child {
            let mut lock = child.handle.lock();
            lock.update_layout(&child.handle, parent, info);
        }
    }

    fn render<'a>(&mut self, _info: &mut SceneRenderInfo<'a>) {}
}


/// Node in the scene graph.
///
/// Each Node type ([`Row`], [`Column`], [`Sprite`], etc.) has their
/// own builder which creates a `Node`.
pub struct Node {
    pub(crate) handle: NodeHandle,
    pub(crate) callbacks: Callbacks,
}


macro_rules! make_builder {
    ($name:ident, $builder_name:ident) => {
        #[doc = ::std::concat!(
            "Builder for [`", ::std::stringify!($name), "`] which is used to create a [`Node`]\n.",
            "\n",
            "# Usage\n",
            "```rust\n",
            ::std::stringify!($name), "::builder()\n",
            "    .foo()\n",
            "    .bar()\n",
            "    .build()\n",
            "```",
        )]
        pub struct $builder_name {
            state: $crate::util::Lock<$name>,
            callbacks: $crate::scene::builder::Callbacks,

            #[allow(unused)]
            has_children: bool,
        }

        impl $builder_name {
            /// Finalizes the builder and returns a [`Node`].
            #[inline]
            pub fn build(self) -> Node {
                Node {
                    handle: self.state.into_handle(),
                    callbacks: self.callbacks,
                }
            }
        }

        impl $name {
            #[doc = ::std::concat!(
                "Builder which creates a [`", ::std::stringify!($name), "`], then sets properties on the [`", ::std::stringify!($name), "`], and lastly creates a [`Node`]."
            )]
            #[inline]
            pub fn builder() -> $builder_name {
                $builder_name {
                    state: $crate::util::Lock::new($name::new()),
                    callbacks: $crate::scene::builder::Callbacks::new(),
                    has_children: false,
                }
            }
        }
    };
}

pub(crate) use make_builder;


macro_rules! simple_method {
    (
        $(#[$attr:meta])*
        $name:ident,
        $signal_name:ident,
        $trigger_relayout:literal,
        $trigger_render:literal,
        |$state:ident, $value:ident: $type:ty| $set:block,
    ) => {
        $(#[$attr])*
        #[inline]
        pub fn $name(self, value: $type) -> Self {
            {
                let mut state = self.state.lock();

                let $state = &mut *state;
                let $value = value;
                $set
            }

            self
        }

        $(#[$attr])*
        pub fn $signal_name<S>(mut self, signal: S) -> Self where S: Signal<Item = $type> + 'static {
            let state = self.state.clone();

            self.callbacks.spawn_local(move |root| {
                let root = root.clone();

                signal.for_each(move |value| {
                    let mut state = state.lock();

                    {
                        let $state = &mut *state;
                        let $value = value;
                        $set
                    }

                    if state.visible {
                        if $trigger_relayout {
                            root.trigger_layout_change()

                        } else if $trigger_render {
                            root.trigger_render_change()
                        }
                    }

                    async {}
                })
            });

            self
        }
    };
}


pub(crate) use simple_method;


macro_rules! base_methods {
    ($name:ident, $builder_name:ident) => {
        impl $builder_name {
            /// Can be used to conditionally call methods on this builder.
            #[inline]
            pub fn apply<F>(self, f: F) -> Self where F: FnOnce(Self) -> Self {
                f(self)
            }

            $crate::scene::builder::simple_method!(
                /// If it isn't visible then it's treated as if it doesn't exist.
                ///
                /// The default is `true`, which means it is visible.
                stretch,
                stretch_signal,
                true,
                true,
                |state, value: bool| { state.stretch = value; },
            );

            $crate::scene::builder::simple_method!(
                /// If the parent is a [`Row`] or [`Column`] then it will stretch to fill the available space.
                ///
                /// The default is `false`, which means it does not stretch.
                visible,
                visible_signal,
                true,
                true,
                |state, value: bool| { state.visible = value; },
            );
        }
    };
}

pub(crate) use base_methods;


macro_rules! location_methods {
    ($name:ident, $builder_name:ident, $trigger_relayout:literal) => {
        $crate::scene::builder::location_methods!($name, $builder_name, $trigger_relayout, |_state| {});
    };
    ($name:ident, $builder_name:ident, $trigger_relayout:literal, |$var:ident| $body:block) => {
        impl $builder_name {
            $crate::scene::builder::simple_method!(
                /// Offset x / y (relative to the parent's width / height) which is added to the parent's x / y.
                ///
                /// The default is `{ x: Length::Parent(0.0), y: Length::Parent(0.0) }` which means no offset.
                offset,
                offset_signal,
                $trigger_relayout,
                true,
                |state, value: Offset| {
                    state.location.offset = value;

                    let $var = state;
                    $body
                },
            );

            $crate::scene::builder::simple_method!(
                /// Width / height relative to the parent's width / height.
                ///
                /// The default is `{ width: Length::Parent(1.0), height: Length::Parent(1.0) }`
                /// which means it's the same size as its parent.
                size,
                size_signal,
                $trigger_relayout,
                true,
                |state, value: Size| {
                    state.location.size = value;

                    let $var = state;
                    $body
                },
            );

            $crate::scene::builder::simple_method!(
                /// Empty space around the node.
                ///
                /// The padding does not increase the size, instead the
                /// empty space is created by subtracting from the size.
                ///
                /// The default is no padding.
                padding,
                padding_signal,
                $trigger_relayout,
                true,
                |state, value: Padding| {
                    state.location.padding = value;

                    let $var = state;
                    $body
                },
            );

            $crate::scene::builder::simple_method!(
                /// Position relative to the parent.
                ///
                /// By default, the origin is `{ x: 0.0, y: 0.0 }` which means that it will be
                /// positioned in the upper-left corner of the parent.
                ///
                /// But if you change it to `{ x: 1.0, y: 1.0 }` then it will now be positioned
                /// in the lower-right corner of the parent.
                ///
                /// And `{ x: 0.5, y: 0.5 }` will place it in the center of the parent.
                origin,
                origin_signal,
                $trigger_relayout,
                true,
                |state, value: Origin| {
                    state.location.origin = value;

                    let $var = state;
                    $body
                },
            );

            $crate::scene::builder::simple_method!(
                /// Z-index relative to the parent.
                ///
                /// Nodes with higher z-index will display on top of nodes with a lower z-index.
                ///
                /// The default is `0.0` which means the z-index doesn't change (it is the same as the parent).
                z_index,
                z_index_signal,
                $trigger_relayout,
                true,
                |state, value: f32| {
                    state.location.z_index = value;

                    let $var = state;
                    $body
                },
            );
        }
    };
}

pub(crate) use location_methods;


macro_rules! children_methods {
    ($name:ident, $builder_name:ident) => {
        impl $builder_name {
            /// Adds a child.
            ///
            /// The order is important: children which are added later are drawn on top.
            ///
            /// Children are always drawn on top of the parent.
            #[inline]
            pub fn child(mut self, mut child: Node) -> Self {
                self.has_children = true;

                self.callbacks.transfer(&mut child.callbacks);
                self.state.lock().children.push(child.handle);

                self
            }

            /// Adds multiple children.
            ///
            /// The order is important: children which are added later are drawn on top.
            ///
            /// Children are always drawn on top of the parent.
            #[inline]
            pub fn children<I>(mut self, children: I) -> Self where I: IntoIterator<Item = Node> {
                self.has_children = true;

                {
                    let mut lock = self.state.lock();

                    for mut child in children {
                        self.callbacks.transfer(&mut child.callbacks);
                        lock.children.push(child.handle);
                    }
                }

                self
            }

            /// Dynamically adds or removes a single child based on a Signal.
            ///
            /// If the Signal returns `None` then the existing child is removed.
            pub fn child_signal<S>(mut self, child: S) -> Self where S: Signal<Item = Option<Node>> + 'static {
                self.has_children = true;

                let option = $crate::scene::builder::OptionNode::new();

                self.state.lock().children.push(option.clone().into_handle());

                self.callbacks.spawn_local(move |root| {
                    let root = root.clone();

                    child.for_each(move |new_child| {
                        option.lock().update(new_child, &root);
                        async {}
                    })
                });

                self
            }

            /// Dynamically adds or removes multiple children based on a SignalVec.
            pub fn children_signal_vec<S>(mut self, children: S) -> Self where S: SignalVec<Item = Node> + 'static {
                if self.has_children {
                    panic!("Cannot use children_signal_vec with other child methods");
                }

                self.has_children = true;

                let state = self.state.clone();

                self.callbacks.spawn_local(move |root| {
                    let mut children_state = $crate::scene::builder::ChildrenState::new(root.clone());

                    children.for_each(move |change| {
                        let mut lock = state.lock();
                        children_state.update(&mut lock.children, change);
                        async {}
                    })
                });

                self
            }
        }
    };
}

pub(crate) use children_methods;
