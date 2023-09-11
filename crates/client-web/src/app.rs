use std::sync::Arc;
use dominator::{Dom, html};
use crate::renderer::{Renderer};


pub struct App {
    renderer: Arc<Renderer>,
}

impl App {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            renderer: Renderer::new(),
        })
    }

    pub fn render(this: &Arc<Self>) -> Dom {
        html!("div", {
            .child(Renderer::render(&this.renderer))
        })
    }
}
