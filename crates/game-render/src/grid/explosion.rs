use std::sync::Arc;
use futures_signals::signal::{Signal, Mutable};
use rusted_battalions_engine as engine;
use rusted_battalions_engine::{Node, Size, Offset, Tile, ParentWidth, ParentHeight, Order};

use crate::Game;
use crate::grid::{Grid, Coord};


#[derive(Debug, Clone, Copy)]
struct ExplosionInfo {
    width: f32,
    height: f32,

    offset_x: f32,
    offset_y: f32,

    tile_x: u32,
    tile_y: u32,
    tile_width: u32,
    tile_height: u32,

    frames: u32,
}


#[derive(Debug, Clone, Copy)]
pub enum ExplosionAnimation {
    Land,
    Air,
    Sea,
    Mega,
}

impl ExplosionAnimation {
    fn info(&self) -> ExplosionInfo {
        match self {
            Self::Land => ExplosionInfo {
                width: 2.0,
                height: 2.0,

                offset_x: -0.5,
                offset_y: -1.0,

                tile_x: 0,
                tile_y: 0,
                tile_width: 32,
                tile_height: 32,

                frames: 9,
            },

            Self::Air => ExplosionInfo {
                width: 2.0,
                height: 2.0,

                offset_x: -(6.0 / 16.0), // shift 6 pixels left
                offset_y: -(6.0 / 16.0), // shift 5 pixels up

                tile_x: 0,
                tile_y: 32,
                tile_width: 32,
                tile_height: 32,

                frames: 9,
            },

            Self::Sea => ExplosionInfo {
                width: 2.0,
                height: 2.0,

                offset_x: -0.5,
                offset_y: -1.0,

                tile_x: 0,
                tile_y: 64,
                tile_width: 32,
                tile_height: 32,

                frames: 7,
            },

            Self::Mega => ExplosionInfo {
                width: 7.0,
                height: 3.0,

                offset_x: -3.0,
                offset_y: -2.0,

                tile_x: 0,
                tile_y: 96,
                tile_width: 112,
                tile_height: 48,

                frames: 12,
            },
        }
    }
}


pub struct Explosion {
    coord: Coord,
    animation: ExplosionAnimation,
    pub percent: Mutable<f32>,
}

impl Explosion {
    pub fn new(coord: Coord, animation: ExplosionAnimation) -> Arc<Self> {
        Arc::new(Self {
            coord,
            animation,
            percent: Mutable::new(0.0),
        })
    }

    fn tile(&self, info: ExplosionInfo) -> impl Signal<Item = Tile> {
        let frames = info.frames as f32;
        let last = info.frames - 1;

        let start_y = info.tile_y;
        let end_y = start_y + info.tile_height;

        self.percent.signal_ref(move |percent| {
            let frame = ((percent * frames) as u32).min(last);

            let start_x = info.tile_x + (info.tile_width * frame);

            Tile {
                start_x,
                start_y,
                end_x: start_x + info.tile_width,
                end_y,
            }
        })
    }

    pub fn render(game: &Arc<Game>, grid: &Arc<Grid>, this: &Arc<Self>) -> Node {
        let info = this.animation.info();

        engine::Sprite::builder()
            .spritesheet(game.spritesheets.effect.clone())

            .apply(|builder| {
                match this.animation {
                    // Air explosion is always displayed on top of everything else.
                    ExplosionAnimation::Air => builder.order(Order::Above(1.0)),

                    // Other explosions follow the usual order, so they can be obscured by mountains / forests.
                    _ => builder.order(Order::Parent(grid.order(&this.coord) + 0.75)),
                }
            })

            .offset({
                let (x, y) = grid.tile_offset(&this.coord);

                Offset {
                    x: ParentWidth(x + (info.offset_x * grid.width)),
                    y: ParentHeight(y + (info.offset_y * grid.height)),
                }
            })

            .size(Size {
                width: ParentWidth(grid.width * info.width),
                height: ParentHeight(grid.height * info.height),
            })

            .tile_signal(this.tile(info))

            .build()
    }
}
