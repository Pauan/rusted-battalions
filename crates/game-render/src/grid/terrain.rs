use std::sync::Arc;
use futures_signals::signal::{SignalExt};
use rusted_battalions_engine as engine;
use rusted_battalions_engine::{SpriteBuilder, Size, Offset, Tile, Node, ParentWidth, ParentHeight, Order};

use crate::grid::{Game, Grid, Coord, TERRAIN_ANIMATION_TIME, FOG_ANIMATION_TIME};
use crate::util::random::{random};

mod sea;
mod river;
mod shoal;


const TILE_SIZE: u32 = 16;


/// By using bitwise operators like | and & and ! it
/// can determine the adjacent tiles very efficiently.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub(crate) struct TerrainFlag(u32);

impl TerrainFlag {
    const ANY: Self      = Self(0xffffffff);

    const EMPTY: Self    = Self(0b00000000000000000000000000000001);
    const PLAIN: Self    = Self(0b00000000000000000000000000000010);
    const ROAD: Self     = Self(0b00000000000000000000000000000100);
    const WOOD: Self     = Self(0b00000000000000000000000000001000);
    const MOUNTAIN: Self = Self(0b00000000000000000000000000010000);
    const PIPELINE: Self = Self(0b00000000000000000000000000100000);
    const PIPESEAM: Self = Self(0b00000000000000000000000001000000);
    const RIVER: Self    = Self(0b00000000000000000000000010000000);
    const SEA: Self      = Self(0b00000000000000000000000100000000);
    const SHOAL: Self    = Self(0b00000000000000000000001000000000);
    const REEF: Self     = Self(0b00000000000000000000010000000000);
    const BRIDGE: Self   = Self(0b00000000000000000000100000000000);
    const SILO: Self     = Self(0b00000000000000000001000000000000);

    const PIPES: Self    = Self::PIPELINE.or(Self::PIPESEAM);
    const WATER: Self    = Self::SEA.or(Self::RIVER).or(Self::SHOAL).or(Self::REEF).or(Self::BRIDGE).or(Self::EMPTY);
    const GROUND: Self   = Self::WATER.not();
}

impl std::ops::Not for TerrainFlag {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        self.not()
    }
}

impl std::ops::BitAnd for TerrainFlag {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        self.and(rhs)
    }
}

impl std::ops::BitOr for TerrainFlag {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        self.or(rhs)
    }
}

impl Default for TerrainFlag {
    #[inline]
    fn default() -> Self {
        Self::EMPTY
    }
}

impl TerrainFlag {
    #[inline]
    const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    #[inline]
    const fn not(self) -> Self {
        Self(!self.0)
    }

    #[inline]
    const fn and(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }

    #[inline]
    const fn or(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }

    pub fn from_tile(class: &TerrainClass) -> Self {
        match class {
            TerrainClass::Empty => Self::EMPTY,
            TerrainClass::Grass => Self::PLAIN,
            TerrainClass::Road { .. } => Self::ROAD,
            TerrainClass::Forest => Self::WOOD,
            TerrainClass::Mountain { .. } => Self::MOUNTAIN,
            TerrainClass::Pipeline => Self::PIPELINE,
            TerrainClass::Pipeseam { .. } => Self::PIPESEAM,
            TerrainClass::River => Self::RIVER,
            TerrainClass::Ocean => Self::SEA,
            TerrainClass::Shoal => Self::SHOAL,
            TerrainClass::Reef => Self::REEF,
            TerrainClass::Bridge { .. } => Self::BRIDGE,
            //TerrainClass::Silo { .. } => Self::SILO,
        }
    }
}


pub struct Terrain {
    pub width: u32,
    pub height: u32,
    tiles: Vec<TerrainTile>,
}

impl Terrain {
    /*pub fn from_map(map: &Map) -> Self {
        let tiles = map.tiles().map(|tile| {
            TerrainTile::new(tile.coord.x, tile.coord.y, tile.class)
        }).collect();

        let mut terrain = Terrain {
            width: map.width,
            height: map.height,
            tiles,
        };

        terrain.update_tiles();

        terrain
    }*/


    pub fn new(width: u32, height: u32) -> Self {
        let mut tiles = Vec::with_capacity(width as usize * height as usize);

        for y in 0..height {
            for x in 0..width {
                tiles.push(TerrainTile::empty(x, y));
            }
        }

        Self {
            width,
            height,
            tiles,
        }
    }

    fn get_index(&self, x: u32, y: u32) -> usize {
        if x > self.width || y > self.height {
            panic!("Coordinate out of range {},{}", x, y);
        }

        ((y * self.width) + x) as usize
    }

    pub fn get_checked(&self, x: u32, y: u32) -> Option<&TerrainTile> {
        let index = self.get_index(x, y);
        self.tiles.get(index)
    }

    pub fn get(&self, x: u32, y: u32) -> &TerrainTile {
        let index = self.get_index(x, y);
        &self.tiles[index]
    }

    pub fn get_mut(&mut self, x: u32, y: u32) -> &mut TerrainTile {
        let index = self.get_index(x, y);
        &mut self.tiles[index]
    }

    pub fn len(&self) -> usize {
        self.tiles.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &TerrainTile> {
        self.tiles.iter()
    }

    pub fn iter_rev(&self) -> impl Iterator<Item = &TerrainTile> {
        self.tiles.iter().rev()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut TerrainTile> {
        self.tiles.iter_mut()
    }

    /// This updates all of the tile's info based on adjacency information
    pub fn update_tiles(&mut self) {
        struct Change {
            index: usize,
            info: TileInfo,
        }

        let changes = self.tiles.iter().enumerate()
            .map(|(index, tile)| {
                let mut adjacent = Adjacent::default();

                if let Some(down) = self.get_checked(tile.x, tile.y + 1) {
                    adjacent.down = down.class.flag();
                }

                if let Some(right) = self.get_checked(tile.x + 1, tile.y) {
                    adjacent.right = right.class.flag();
                }

                if let Some(down_right) = self.get_checked(tile.x + 1, tile.y + 1) {
                    adjacent.down_right = down_right.class.flag();
                }

                if let Some(x) = tile.x.checked_sub(1) {
                    if let Some(left) = self.get_checked(x, tile.y) {
                        adjacent.left = left.class.flag();
                    }

                    if let Some(down_left) = self.get_checked(x, tile.y + 1) {
                        adjacent.down_left = down_left.class.flag();
                    }

                    if let Some(y) = tile.y.checked_sub(1) {
                        if let Some(up_left) = self.get_checked(x, y) {
                            adjacent.up_left = up_left.class.flag();
                        }
                    }
                }

                if let Some(y) = tile.y.checked_sub(1) {
                    if let Some(up) = self.get_checked(tile.x, y) {
                        adjacent.up = up.class.flag();
                    }

                    if let Some(up_right) = self.get_checked(tile.x + 1, y) {
                        adjacent.up_right = up_right.class.flag();
                    }
                }

                Change {
                    index,
                    info: TileInfo::new(tile, &adjacent),
                }
            })
            .collect::<Vec<Change>>();

        for change in changes {
            self.tiles[change.index].info = change.info;
        }
    }
}


/// This gives information about which tiles are adjacent to this tile
#[derive(Debug, Clone, Copy, Default)]
struct Adjacent {
    up: TerrainFlag,
    down: TerrainFlag,
    left: TerrainFlag,
    right: TerrainFlag,

    up_left: TerrainFlag,
    up_right: TerrainFlag,
    down_left: TerrainFlag,
    down_right: TerrainFlag,
}

pub(crate) struct TerrainRule {
    pub(crate) tile_x: u32,
    pub(crate) tile_y: u32,

    pub(crate) up: TerrainFlag,
    pub(crate) down: TerrainFlag,
    pub(crate) left: TerrainFlag,
    pub(crate) right: TerrainFlag,

    pub(crate) up_left: TerrainFlag,
    pub(crate) up_right: TerrainFlag,
    pub(crate) down_left: TerrainFlag,
    pub(crate) down_right: TerrainFlag,
}

impl TerrainRule {
    fn matches(&self, adjacent: &Adjacent) -> bool {
        self.up.contains(adjacent.up) &&
        self.down.contains(adjacent.down) &&
        self.left.contains(adjacent.left) &&
        self.right.contains(adjacent.right) &&
        self.up_left.contains(adjacent.up_left) &&
        self.up_right.contains(adjacent.up_right) &&
        self.down_left.contains(adjacent.down_left) &&
        self.down_right.contains(adjacent.down_right)
    }

    fn block_matches(flag: TerrainFlag, x: u32, y: u32) -> impl Iterator<Item = Self> {
        let any = Self {
            tile_x: 0,
            tile_y: 0,
            up: !flag,
            down: !flag,
            left: !flag,
            right: !flag,
            up_left: TerrainFlag::ANY,
            up_right: TerrainFlag::ANY,
            down_left: TerrainFlag::ANY,
            down_right: TerrainFlag::ANY,
        };

        [
            Self {
                tile_x: x + 0,
                tile_y: y + 0,
                down: flag,
                right: flag,
                ..any
            },
            Self {
                tile_x: x + 1,
                tile_y: y + 0,
                down: flag,
                left: flag,
                right: flag,
                ..any
            },
            Self {
                tile_x: x + 2,
                tile_y: y + 0,
                down: flag,
                left: flag,
                ..any
            },
            Self {
                tile_x: x + 3,
                tile_y: y + 0,
                down: flag,
                ..any
            },

            Self {
                tile_x: x + 0,
                tile_y: y + 1,
                up: flag,
                down: flag,
                right: flag,
                ..any
            },
            Self {
                tile_x: x + 1,
                tile_y: y + 1,
                up: flag,
                down: flag,
                left: flag,
                right: flag,
                ..any
            },
            Self {
                tile_x: x + 2,
                tile_y: y + 1,
                up: flag,
                down: flag,
                left: flag,
                ..any
            },
            Self {
                tile_x: x + 3,
                tile_y: y + 1,
                up: flag,
                down: flag,
                ..any
            },

            Self {
                tile_x: x + 0,
                tile_y: y + 2,
                up: flag,
                right: flag,
                ..any
            },
            Self {
                tile_x: x + 1,
                tile_y: y + 2,
                up: flag,
                left: flag,
                right: flag,
                ..any
            },
            Self {
                tile_x: x + 2,
                tile_y: y + 2,
                up: flag,
                left: flag,
                ..any
            },
            Self {
                tile_x: x + 3,
                tile_y: y + 2,
                up: flag,
                ..any
            },

            Self {
                tile_x: x + 0,
                tile_y: y + 3,
                right: flag,
                ..any
            },
            Self {
                tile_x: x + 1,
                tile_y: y + 3,
                left: flag,
                right: flag,
                ..any
            },
            Self {
                tile_x: x + 2,
                tile_y: y + 3,
                left: flag,
                ..any
            },
            Self {
                tile_x: x + 3,
                tile_y: y + 3,
                ..any
            },
        ].into_iter()
    }
}


#[derive(Debug, Clone, Copy)]
pub enum Orientation {
    Horizontal,
    Vertical,
}


#[derive(Debug, Clone, Copy)]
pub enum TerrainClass {
    Empty,
    Grass,
    Road {
        ruins: bool,
    },
    Bridge {
        orientation: Orientation,
    },
    Forest,
    Mountain {
        variant: u32,
    },
    Pipeline,
    Pipeseam {
        destroyed: bool,
    },
    Ocean,
    River,
    Shoal,
    Reef,
}

impl TerrainClass {
    pub const ALL: &[Self] = &[
        Self::Empty,
        Self::Grass,
        Self::Road { ruins: false },
        Self::Bridge { orientation: Orientation::Horizontal },
        Self::Forest,
        Self::Mountain { variant: 0 },
        Self::Pipeline,
        Self::Pipeseam { destroyed: false, },
        Self::Ocean,
        Self::River,
        Self::Shoal,
        Self::Reef,
    ];

    fn flag(&self) -> TerrainFlag {
        match self {
            Self::Empty => TerrainFlag::EMPTY,
            Self::Grass => TerrainFlag::PLAIN,
            Self::Road { .. } => TerrainFlag::ROAD,
            Self::Bridge { .. } => TerrainFlag::BRIDGE,
            Self::Forest => TerrainFlag::WOOD,
            Self::Mountain { .. } => TerrainFlag::MOUNTAIN,
            Self::Pipeline => TerrainFlag::PIPELINE,
            Self::Pipeseam { .. } => TerrainFlag::PIPESEAM,
            Self::Ocean => TerrainFlag::SEA,
            Self::River => TerrainFlag::RIVER,
            Self::Shoal => TerrainFlag::SHOAL,
            Self::Reef => TerrainFlag::REEF,
        }
    }

    pub fn random_mountain() -> Self {
        Self::Mountain {
            variant: (random() * 3.0) as u32,
        }
    }
}


#[derive(Debug, Clone, Copy)]
struct FrameInfo {
    offset_y: u32,
    frames: u32,
}

#[derive(Debug, Clone, Copy)]
struct TileInfo {
    tile_x: u32,
    tile_y: u32,
    tile_width: u32,
    tile_height: u32,
    frame_info: Option<FrameInfo>,
}

impl TileInfo {
    const ERROR: Self = Self {
        tile_x: 1 * TILE_SIZE,
        tile_y: 0 * TILE_SIZE,
        tile_width: TILE_SIZE,
        tile_height: TILE_SIZE,
        frame_info: None,
    };


    fn new_road(adjacent: &Adjacent, ruins: bool) -> Self {
        for rule in TerrainRule::block_matches(TerrainFlag::ROAD, if ruins { 16 } else { 12 }, 0) {
            if rule.matches(adjacent) {
                return Self {
                    tile_x: rule.tile_x * TILE_SIZE,
                    tile_y: rule.tile_y * TILE_SIZE,
                    tile_width: TILE_SIZE,
                    tile_height: TILE_SIZE,
                    frame_info: None,
                };
            }
        }

        Self::ERROR
    }


    fn new_pipe(adjacent: &Adjacent) -> Self {
        for rule in TerrainRule::block_matches(TerrainFlag::PIPES, 0, 4) {
            if rule.matches(adjacent) {
                return Self {
                    tile_x: rule.tile_x * TILE_SIZE,
                    tile_y: rule.tile_y * TILE_SIZE,
                    tile_width: TILE_SIZE,
                    tile_height: TILE_SIZE,
                    frame_info: Some(FrameInfo {
                        offset_y: 4 * TILE_SIZE,
                        frames: 2,
                    }),
                };
            }
        }

        Self::ERROR
    }


    fn new_sea(adjacent: &Adjacent) -> Self {
        for rule in sea::rules() {
            if rule.matches(adjacent) {
                return Self {
                    tile_x: rule.tile_x * TILE_SIZE,
                    tile_y: rule.tile_y * TILE_SIZE,
                    tile_width: TILE_SIZE,
                    tile_height: TILE_SIZE,
                    frame_info: Some(FrameInfo {
                        offset_y: 4 * TILE_SIZE,
                        frames: 4,
                    }),
                };
            }
        }

        Self::ERROR
    }


    fn new_river(adjacent: &Adjacent) -> Self {
        for rule in river::rules() {
            if rule.matches(adjacent) {
                return Self {
                    tile_x: rule.tile_x * TILE_SIZE,
                    tile_y: rule.tile_y * TILE_SIZE,
                    tile_width: TILE_SIZE,
                    tile_height: TILE_SIZE,
                    frame_info: Some(FrameInfo {
                        offset_y: 4 * TILE_SIZE,
                        frames: 4,
                    }),
                };
            }
        }

        Self::ERROR
    }


    fn new_shoal(adjacent: &Adjacent) -> Self {
        for rule in shoal::rules() {
            if rule.matches(adjacent) {
                return Self {
                    tile_x: rule.tile_x * TILE_SIZE,
                    tile_y: rule.tile_y * TILE_SIZE,
                    tile_width: TILE_SIZE,
                    tile_height: TILE_SIZE,
                    frame_info: Some(FrameInfo {
                        offset_y: 4 * TILE_SIZE,
                        frames: 4,
                    }),
                };
            }
        }

        Self::ERROR
    }


    fn new_pipeseam(adjacent: &Adjacent, destroyed: bool) -> Self {
        if TerrainFlag::PIPES.contains(adjacent.up) && TerrainFlag::PIPES.contains(adjacent.down) {
            if destroyed {
                Self {
                    tile_x: 8 * TILE_SIZE,
                    tile_y: 0 * TILE_SIZE,
                    tile_width: TILE_SIZE,
                    tile_height: TILE_SIZE,
                    frame_info: None,
                }

            } else {
                Self {
                    tile_x: 9 * TILE_SIZE,
                    tile_y: 0 * TILE_SIZE,
                    tile_width: TILE_SIZE,
                    tile_height: TILE_SIZE,
                    frame_info: None,
                }
            }

        } else {
            if destroyed {
                Self {
                    tile_x: 8 * TILE_SIZE,
                    tile_y: 1 * TILE_SIZE,
                    tile_width: TILE_SIZE,
                    tile_height: TILE_SIZE,
                    frame_info: None,
                }

            } else {
                Self {
                    tile_x: 9 * TILE_SIZE,
                    tile_y: 1 * TILE_SIZE,
                    tile_width: TILE_SIZE,
                    tile_height: TILE_SIZE,
                    frame_info: Some(FrameInfo {
                        offset_y: 1 * TILE_SIZE,
                        frames: 2,
                    }),
                }
            }
        }
    }


    fn new(tile: &TerrainTile, adjacent: &Adjacent) -> Self {
        match tile.class {
            TerrainClass::Empty => Self::ERROR,

            TerrainClass::Grass => if TerrainFlag::MOUNTAIN.contains(adjacent.left) {
                Self {
                    tile_x: 3 * TILE_SIZE,
                    tile_y: 0 * TILE_SIZE,
                    tile_width: TILE_SIZE,
                    tile_height: TILE_SIZE,
                    frame_info: None,
                }
            } else {
                Self {
                    tile_x: 2 * TILE_SIZE,
                    tile_y: 0 * TILE_SIZE,
                    tile_width: TILE_SIZE,
                    tile_height: TILE_SIZE,
                    frame_info: None,
                }
            },

            TerrainClass::Forest => if TerrainFlag::MOUNTAIN.contains(adjacent.left) {
                Self {
                    tile_x: 2 * TILE_SIZE,
                    tile_y: 1 * TILE_SIZE,
                    tile_width: TILE_SIZE,
                    tile_height: 2 * TILE_SIZE,
                    frame_info: None,
                }
            } else {
                Self {
                    tile_x: 1 * TILE_SIZE,
                    tile_y: 1 * TILE_SIZE,
                    tile_width: TILE_SIZE,
                    tile_height: 2 * TILE_SIZE,
                    frame_info: None,
                }
            },

            TerrainClass::Mountain { variant } => Self {
                tile_x: (4 + variant) * TILE_SIZE,
                tile_y: 1 * TILE_SIZE,
                tile_width: TILE_SIZE,
                tile_height: 2 * TILE_SIZE,
                frame_info: None,
            },

            TerrainClass::Reef => Self {
                tile_x: 24 * TILE_SIZE,
                tile_y: 0 * TILE_SIZE,
                tile_width: TILE_SIZE,
                tile_height: TILE_SIZE,
                frame_info: Some(FrameInfo {
                    offset_y: 1 * TILE_SIZE,
                    frames: 4,
                }),
            },

            TerrainClass::Bridge { orientation } => match orientation {
                Orientation::Horizontal => Self {
                    tile_x: 21 * TILE_SIZE,
                    tile_y: 3 * TILE_SIZE,
                    tile_width: TILE_SIZE,
                    tile_height: TILE_SIZE,
                    frame_info: None,
                },
                Orientation::Vertical => Self {
                    tile_x: 23 * TILE_SIZE,
                    tile_y: 1 * TILE_SIZE,
                    tile_width: TILE_SIZE,
                    tile_height: TILE_SIZE,
                    frame_info: None,
                },
            },

            TerrainClass::Road { ruins } => Self::new_road(adjacent, ruins),
            TerrainClass::Pipeline => Self::new_pipe(adjacent),
            TerrainClass::Pipeseam { destroyed } => Self::new_pipeseam(adjacent, destroyed),
            TerrainClass::Ocean => Self::new_sea(adjacent),
            TerrainClass::River => Self::new_river(adjacent),
            TerrainClass::Shoal => Self::new_shoal(adjacent),
            //TerrainClass::Silo { has_missile } => Self::new_silo(has_missile),
        }
    }
}


pub struct TerrainTile {
    pub x: u32,
    pub y: u32,
    pub class: TerrainClass,
    info: TileInfo,
}

impl TerrainTile {
    fn new(x: u32, y: u32, class: TerrainClass) -> Self {
        Self {
            x,
            y,
            class,
            info: TileInfo::ERROR,
        }
    }

    fn empty(x: u32, y: u32) -> Self {
        Self {
            x,
            y,
            class: TerrainClass::Empty,
            info: TileInfo::ERROR,
        }
    }

    pub fn render(game: &Arc<Game>, grid: &Arc<Grid>, this: &Self) -> Node {
        let info = this.info;

        let coord = Coord {
            x: this.x as f32,
            y: this.y as f32,
        };

        let ratio = info.tile_height as f32 / info.tile_width as f32;

        let (x, y) = grid.tile_offset(&coord);

        let offset = Offset {
            x: ParentWidth(x),
            y: ParentHeight(y - (grid.height * (ratio - 1.0))),
        };

        let size = Size {
            width: ParentWidth(grid.width),
            height: ParentHeight(grid.height * ratio),
        };

        fn tile_animation(grid: &Arc<Grid>, info: TileInfo) -> impl FnOnce(SpriteBuilder) -> SpriteBuilder + '_ {
            move |builder| {
                let TileInfo { tile_x, tile_y, tile_width, tile_height, frame_info } = info;

                let mut tile = Tile {
                    start_x: tile_x,
                    start_y: tile_y,
                    end_x: tile_x + tile_width,
                    end_y: tile_y + tile_height,
                };

                if let Some(frame_info) = frame_info {
                    builder.tile_signal(grid.animation_pendulum(TERRAIN_ANIMATION_TIME, frame_info.frames).map(move |frame| {
                        tile.start_y = tile_y + (frame * frame_info.offset_y);
                        tile.end_y = tile.start_y + tile_width;
                        tile
                    }))

                } else {
                    builder.tile(tile)
                }
            }
        }

        engine::Stack::builder()
            .order(Order::Parent(0.0))

            .child(engine::Sprite::builder()
                .spritesheet(game.spritesheets.terrain.clone())
                .apply(tile_animation(grid, this.info))
                .order(Order::Parent(grid.order(&coord)))
                .offset(offset)
                .size(size)
                .palette(0)
                .build())

            .child(engine::Sprite::builder()
                /*.alpha_signal(grid.animation(FOG_ANIMATION_TIME).map(move |time| {
                    let time = (time % 2.0) as f32;

                    if time > 1.0 {
                        2.0 - time

                    } else {
                        time
                    }
                }))*/

                .alpha(if coord.x > 16.0 {
                    1.0
                } else {
                    0.0
                })

                .spritesheet(game.spritesheets.terrain.clone())
                .apply(tile_animation(grid, this.info))
                .order(Order::Parent(grid.order(&coord) + (1.0 / 6.0)))
                .offset(offset)
                .size(size)
                .palette(1)
                .build())

            .build()
    }
}
