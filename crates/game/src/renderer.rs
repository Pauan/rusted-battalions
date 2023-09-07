use wasm_bindgen::prelude::*;
use dominator::{Dom, DomBuilder, clone, html, with_node, apply_methods, events};
use dominator::animation::{timestamps};
use futures_signals::signal::{Mutable, SignalExt};
use futures_signals::map_ref;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::future::Future;
use crate::game::{GameData, Grid, UnitAppearance};


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
    id: u32,
    game: Arc<GameData>,
}

impl Renderer {
    pub fn new() -> Arc<Self> {
        static ID: AtomicU32 = AtomicU32::new(1);

        let id = ID.fetch_add(1, Ordering::SeqCst);

        Arc::new(Self {
            id,
            game: GameData::new(UnitAppearance::default(), Grid::test()),
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
        html!("div", {
            .child(html!("canvas" => web_sys::HtmlCanvasElement, {
                //.style_signal("width", this.size.signal_ref(|size| format!("{}px", size.width)))
                //.style_signal("height", this.size.signal_ref(|size| format!("{}px", size.height)))

                .attr("data-raw-handle", &this.id.to_string())

                .attr_signal("width", this.game.screen_size().map(|size| format!("{}", size.width)))
                .attr_signal("height", this.game.screen_size().map(|size| format!("{}", size.height)))

                .with_node!(element => {
                    .apply(wait_for_inserted(clone!(this => async move {
                        let mut game = this.game.start_engine(this.id).await;

                        timestamps().for_each(move |time| {
                            if let Some(time) = time {
                                game.render(time);
                            }

                            async {}
                        }).await;
                    })))
                })

                /*.with_node!(element => {
                    .apply(|dom| {
                        let inserted = Mutable::new(false);

                        let ctx: web_sys::CanvasRenderingContext2d = element.get_context("2d")
                            .unwrap()
                            .unwrap()
                            .unchecked_into();

                        apply_methods!(dom, {
                            .after_inserted(clone!(inserted => move |_| {
                                inserted.set(true);
                            }))

                            .future(clone!(this => async move {
                                inserted.signal().wait_for(true).await;

                                timestamps_difference().for_each(move |delta| {
                                    if let Some(delta) = delta {
                                        this.clear(&ctx);
                                        this.draw(&ctx, delta);
                                    }

                                    async {}
                                }).await;
                            }))
                        })
                    })
                })*/
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
