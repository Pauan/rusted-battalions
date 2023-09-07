use rusted_battalions_game::{Game, Grid, UnitAppearance};

use dominator::{Dom, DomBuilder, clone, html, dom_builder, with_node, apply_methods, events};
use dominator::animation::{timestamps};
use futures_signals::signal::{Mutable, SignalExt};

use std::sync::Arc;
use std::future::Future;

use crate::window::Window;


// TODO this is a general utility helper, move it someplace else
fn wait_for_inserted<A, F>(f: F) -> impl FnOnce(DomBuilder<A>) -> DomBuilder<A>
    where A: Clone + 'static,
          F: Future<Output = ()> + 'static {
    move |dom| {
        let inserted = Mutable::new(false);

        let signal = inserted.signal();

        apply_methods!(dom, {
            .after_inserted(move |_| {
                inserted.set(true);
            })

            .future(async move {
                signal.wait_for(true).await;
                f.await;
            })
        })
    }
}


pub struct Renderer {
    game: Arc<Game>,
}

impl Renderer {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            game: Game::new(UnitAppearance::default(), Grid::test()),
        })
    }

    /*fn clear(&self, ctx: &web_sys::CanvasRenderingContext2d) {
        let size = self.size.lock_ref();
        ctx.clear_rect(0.0, 0.0, size.width, size.height);
    }

    fn draw(&self, ctx: &web_sys::CanvasRenderingContext2d, delta: f64) {
        let size = self.size.lock_ref();

        log::info!("HELLO {}", delta);
        ctx.set_fill_style(&JsValue::from("red"));
        ctx.fill_rect(20.0, 20.0, size.width - 40.0, size.height - 40.0);
    }*/

    pub fn render(this: &Arc<Self>) -> Dom {
        let window = Window::new();

        html!("div", {
            .child(dom_builder!(window.canvas(), {
                .attr_signal("width", this.game.screen_size().map(|size| format!("{}", size.width)))
                .attr_signal("height", this.game.screen_size().map(|size| format!("{}", size.height)))

                .apply(wait_for_inserted(clone!(this => async move {
                    let mut game = this.game.start_engine(window).await;

                    timestamps().for_each(move |time| {
                        if let Some(time) = time {
                            game.render(time);
                        }

                        async {}
                    }).await;
                })))
            }))

            .child(html!("div", {
                .style("margin", "20px")
                .style("margin-left", "30px")

                .child(html!("label", {
                    .child(html!("input" => web_sys::HtmlInputElement, {
                        .attr("type", "checkbox")

                        .attr_signal("checked", this.game.unit_appearance.signal_ref(|appearance| {
                            if *appearance == UnitAppearance::DualStrikeBig {
                                Some("")

                            } else {
                                None
                            }
                        }))

                        .with_node!(element => {
                            .event(clone!(this => move |_: events::Change| {
                                this.game.unit_appearance.set_neq(if element.checked() {
                                    UnitAppearance::DualStrikeBig

                                } else {
                                    UnitAppearance::DualStrikeSmall
                                });
                            }))
                        })
                    }))

                    .text("HD Graphics")
                }))
            }))
        })
    }
}
