use rusted_battalions_engine as engine;
use rusted_battalions_engine::{Length, Tile, Origin, Padding, Size, Node, Spritesheet};


pub struct BorderSize {
    pub up: Length,
    pub down: Length,
    pub left: Length,
    pub right: Length,
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

impl Quadrants {
    pub fn from_grid(start_x: u32, start_y: u32, tile_width: u32, tile_height: u32) -> Self {
        let x1 = start_x;
        let x2 = start_x + tile_width;
        let x3 = start_x + tile_width * 2;
        let x4 = start_x + tile_width * 3;

        let y1 = start_y;
        let y2 = start_y + tile_height;
        let y3 = start_y + tile_height * 2;
        let y4 = start_y + tile_height * 3;

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


pub struct SpriteBorderBuilder {
    spritesheet: Option<Spritesheet>,
    border_size: Option<BorderSize>,
    quadrants: Option<Quadrants>,
    center: Option<Node>,
    builder: engine::StackBuilder,
}

impl SpriteBorderBuilder {
    #[inline]
    pub fn apply<F>(self, f: F) -> Self
        where F: FnOnce(engine::StackBuilder) -> engine::StackBuilder {
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

    pub fn build(self) -> Node {
        let spritesheet = self.spritesheet.expect("Missing spritesheet");
        let border_size = self.border_size.expect("Missing border_size");
        let quadrants = self.quadrants.expect("Missing quadrants");
        let center = self.center.expect("Missing center");

        self.builder.children([
            engine::Sprite::builder()
                .spritesheet(spritesheet.clone())
                .tile(quadrants.up_left)
                .size(Size {
                    width: border_size.left,
                    height: border_size.up,
                })
                .build(),

            engine::Sprite::builder()
                .spritesheet(spritesheet.clone())
                .tile(quadrants.up)
                .size(Size {
                    width: Length::Parent(1.0),
                    height: border_size.up,
                })
                .origin(Origin {
                    x: 0.5,
                    y: 0.0,
                })
                .padding(Padding {
                    left: border_size.left,
                    right: border_size.right,
                    ..Padding::default()
                })
                .build(),

            engine::Sprite::builder()
                .spritesheet(spritesheet.clone())
                .tile(quadrants.up_right)
                .size(Size {
                    width: border_size.right,
                    height: border_size.up,
                })
                .origin(Origin {
                    x: 1.0,
                    y: 0.0,
                })
                .build(),

            engine::Sprite::builder()
                .spritesheet(spritesheet.clone())
                .tile(quadrants.left)
                .size(Size {
                    width: border_size.left,
                    height: Length::Parent(1.0),
                })
                .origin(Origin {
                    x: 0.0,
                    y: 0.5,
                })
                .padding(Padding {
                    up: border_size.up,
                    down: border_size.down,
                    ..Padding::default()
                })
                .build(),

            engine::Stack::builder()
                .origin(Origin {
                    x: 0.5,
                    y: 0.5,
                })
                .padding(Padding {
                    up: border_size.up,
                    down: border_size.down,
                    left: border_size.left,
                    right: border_size.right,
                })
                .child(engine::Sprite::builder()
                    .spritesheet(spritesheet.clone())
                    .tile(quadrants.center)
                    .build())
                .child(center)
                .build(),

            engine::Sprite::builder()
                .spritesheet(spritesheet.clone())
                .tile(quadrants.right)
                .size(Size {
                    width: border_size.right,
                    height: Length::Parent(1.0),
                })
                .origin(Origin {
                    x: 1.0,
                    y: 0.5,
                })
                .padding(Padding {
                    up: border_size.up,
                    down: border_size.down,
                    ..Padding::default()
                })
                .build(),

            engine::Sprite::builder()
                .spritesheet(spritesheet.clone())
                .tile(quadrants.down_left)
                .size(Size {
                    width: border_size.left,
                    height: border_size.down,
                })
                .origin(Origin {
                    x: 0.0,
                    y: 1.0,
                })
                .build(),

            engine::Sprite::builder()
                .spritesheet(spritesheet.clone())
                .tile(quadrants.down)
                .size(Size {
                    width: Length::Parent(1.0),
                    height: border_size.down,
                })
                .origin(Origin {
                    x: 0.5,
                    y: 1.0,
                })
                .padding(Padding {
                    left: border_size.left,
                    right: border_size.right,
                    ..Padding::default()
                })
                .build(),

            engine::Sprite::builder()
                .spritesheet(spritesheet.clone())
                .tile(quadrants.down_right)
                .size(Size {
                    width: border_size.right,
                    height: border_size.down,
                })
                .origin(Origin {
                    x: 1.0,
                    y: 1.0,
                })
                .build(),
        ])
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
            builder: engine::Stack::builder(),
        }
    }
}
