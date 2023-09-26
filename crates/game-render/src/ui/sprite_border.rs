use rusted_battalions_engine as engine;
use rusted_battalions_engine::{Length, Tile, Origin, Padding, Size, Node, Spritesheet};

pub use rusted_battalions_engine::{BorderSize, RepeatTile, Repeat};


pub struct QuadrantGrid {
    pub start_x: u32,
    pub start_y: u32,

    pub up_height: u32,
    pub down_height: u32,

    pub left_width: u32,
    pub right_width: u32,

    pub center_width: u32,
    pub center_height: u32,
}

impl QuadrantGrid {
    pub fn equal_size(start_x: u32, start_y: u32, tile_width: u32, tile_height: u32) -> Self {
        Self {
            start_x,
            start_y,

            up_height: tile_height,
            down_height: tile_height,

            left_width: tile_width,
            right_width: tile_width,

            center_width: tile_width,
            center_height: tile_height,
        }
    }
}

impl From<QuadrantGrid> for Quadrants {
    fn from(value: QuadrantGrid) -> Self {
        let x1 = start_x;
        let x2 = x1 + value.left_width;
        let x3 = x2 + value.center_width;
        let x4 = x3 + value.right_width;

        let y1 = start_y;
        let y2 = y1 + value.up_height;
        let y3 = y2 + value.center_height;
        let y4 = y3 + value.down_height;

        Self {
            up_left: Tile {
                start_x: x1,
                start_y: y1,
                end_x: x2,
                end_y: y2,
            },
            up: Tile {
                start_x: x2,
                start_y: y1,
                end_x: x3,
                end_y: y2,
            },
            up_right: Tile {
                start_x: x3,
                start_y: y1,
                end_x: x4,
                end_y: y2,
            },

            left: Tile {
                start_x: x1,
                start_y: y2,
                end_x: x2,
                end_y: y3,
            },
            center: Tile {
                start_x: x2,
                start_y: y2,
                end_x: x3,
                end_y: y3,
            },
            right: Tile {
                start_x: x3,
                start_y: y2,
                end_x: x4,
                end_y: y3,
            },

            down_left: Tile {
                start_x: x1,
                start_y: y3,
                end_x: x2,
                end_y: y4,
            },
            down: Tile {
                start_x: x2,
                start_y: y3,
                end_x: x3,
                end_y: y4,
            },
            down_right: Tile {
                start_x: x3,
                start_y: y3,
                end_x: x4,
                end_y: y4,
            },
        }
    }
}


pub struct Quadrants {
    pub up: Tile,
    pub down: Tile,
    pub left: Tile,
    pub right: Tile,
    pub center: Tile,
    pub up_left: Tile,
    pub up_right: Tile,
    pub down_left: Tile,
    pub down_right: Tile,
}


pub struct SpriteBorderBuilder {
    spritesheet: Option<Spritesheet>,
    border_size: Option<BorderSize>,
    quadrants: Option<Quadrants>,
    center: Option<Node>,
    repeat_tile: RepeatTile,
    builder: engine::BorderGridBuilder,
}

impl SpriteBorderBuilder {
    #[inline]
    pub fn apply<F>(self, f: F) -> Self
        where F: FnOnce(engine::BorderGridBuilder) -> engine::BorderGridBuilder {
        Self {
            builder: f(self.builder),
            ..self
        }
    }

    #[inline]
    pub fn spritesheet(mut self, spritesheet: Spritesheet) -> Self {
        self.spritesheet = Some(spritesheet);
        self
    }

    #[inline]
    pub fn border_size(mut self, border_size: BorderSize) -> Self {
        self.border_size = Some(border_size);
        self
    }

    #[inline]
    pub fn quadrants(mut self, quadrants: Quadrants) -> Self {
        self.quadrants = Some(quadrants);
        self
    }

    #[inline]
    pub fn center(mut self, center: Node) -> Self {
        self.center = Some(center);
        self
    }

    #[inline]
    pub fn repeat_tile(mut self, repeat_tile: RepeatTile) -> Self {
        self.repeat_tile = repeat_tile;
        self
    }

    pub fn build(self) -> Node {
        let spritesheet = self.spritesheet.expect("Missing spritesheet");
        let border_size = self.border_size.expect("Missing border_size");
        let quadrants = self.quadrants.expect("Missing quadrants");
        let center = self.center.expect("Missing center");

        self.builder
            .border_size(border_size)
            .quadrants(engine::Quadrants {
                up_left: engine::Sprite::builder()
                    .spritesheet(spritesheet.clone())
                    .tile(quadrants.up_left)
                    .build(),

                up: engine::Sprite::builder()
                    .spritesheet(spritesheet.clone())
                    .tile(quadrants.up)
                    .repeat_tile(RepeatTile {
                        height: Repeat::None,
                        ..self.repeat_tile
                    })
                    .build(),

                up_right: engine::Sprite::builder()
                    .spritesheet(spritesheet.clone())
                    .tile(quadrants.up_right)
                    .build(),

                left: engine::Sprite::builder()
                    .spritesheet(spritesheet.clone())
                    .tile(quadrants.left)
                    .repeat_tile(RepeatTile {
                        width: Repeat::None,
                        ..self.repeat_tile
                    })
                    .build(),

                center: engine::Stack::builder()
                    .child(engine::Sprite::builder()
                        .spritesheet(spritesheet.clone())
                        .tile(quadrants.center)
                        .repeat_tile(self.repeat_tile)
                        .build())
                    .child(center)
                    .build(),

                right: engine::Sprite::builder()
                    .spritesheet(spritesheet.clone())
                    .tile(quadrants.right)
                    .repeat_tile(RepeatTile {
                        width: Repeat::None,
                        ..self.repeat_tile
                    })
                    .build(),

                down_left: engine::Sprite::builder()
                    .spritesheet(spritesheet.clone())
                    .tile(quadrants.down_left)
                    .build(),

                down: engine::Sprite::builder()
                    .spritesheet(spritesheet.clone())
                    .tile(quadrants.down)
                    .repeat_tile(RepeatTile {
                        height: Repeat::None,
                        ..self.repeat_tile
                    })
                    .build(),

                down_right: engine::Sprite::builder()
                    .spritesheet(spritesheet.clone())
                    .tile(quadrants.down_right)
                    .build(),
            })
            .build()
    }
}


pub struct SpriteBorder;

impl SpriteBorder {
    #[inline]
    pub fn builder() -> SpriteBorderBuilder {
        SpriteBorderBuilder {
            spritesheet: None,
            border_size: None,
            quadrants: None,
            center: None,
            repeat_tile: RepeatTile::default(),
            builder: engine::BorderGrid::builder(),
        }
    }
}
