use std::sync::Arc;
use futures_signals::map_ref;
use futures_signals::signal::{Mutable, Signal, SignalExt};
use dominator::clone;
use rusted_battalions_engine as engine;
use rusted_battalions_engine::{Node, Size, Offset, Tile, ParentWidth, ParentHeight, Order};

use crate::Game;
use crate::grid::{UNIT_ANIMATION_TIME, Grid, Coord, Nation};
use crate::grid::explosion::{ExplosionAnimation};


#[derive(Debug, Clone, Copy)]
pub enum UnitClass {
    Infantry,
    Mech,
    Recon,
    APC,
    Artillery,
    Tank,
    AntiAir,
    Missile,
    Rocket,
    MediumTank,
    Piperunner,
    Neotank,
    MegaTank,
    BCopter,
    TCopter,
    Fighter,
    Bomber,
    Stealth,
    Battleship,
    Cruiser,
    Submarine,
    Lander,
    Carrier,
    BlackBoat,
    BlackBomb,
    Oozium,
}

impl UnitClass {
    pub const ALL: &[Self] = &[
        Self::Infantry,
        Self::Mech,
        Self::Recon,
        Self::APC,
        Self::Artillery,
        Self::Tank,
        Self::AntiAir,
        Self::Missile,
        Self::Rocket,
        Self::MediumTank,
        Self::Piperunner,
        Self::Neotank,
        Self::MegaTank,
        Self::BCopter,
        Self::TCopter,
        Self::Fighter,
        Self::Bomber,
        Self::Stealth,
        Self::Battleship,
        Self::Cruiser,
        Self::Submarine,
        Self::Lander,
        Self::Carrier,
        Self::BlackBoat,
        Self::BlackBomb,
        Self::Oozium,
    ];

    fn tile_y(&self, nation: &Nation) -> u32 {
        match self {
            Self::Infantry => match nation {
                Nation::OrangeStar => 0,
                Nation::BlueMoon => 1,
                Nation::GreenEarth => 2,
                Nation::YellowComet => 3,
                Nation::BlackHole => 4,
            },
            Self::Mech => match nation {
                Nation::OrangeStar => 5,
                Nation::BlueMoon => 6,
                Nation::GreenEarth => 7,
                Nation::YellowComet => 8,
                Nation::BlackHole => 9,
            },
            Self::Recon => 10,
            Self::Tank => 11,
            Self::MediumTank => 12,
            Self::Neotank => 13,
            Self::MegaTank => 14,
            Self::APC => 15,
            Self::AntiAir => 16,
            Self::Artillery => 17,
            Self::Rocket => 18,
            Self::Missile => 19,
            Self::Piperunner => 20,
            Self::Oozium => 21,
            Self::Fighter => 22,
            Self::Bomber => 23,
            Self::BlackBomb => 24,
            Self::Stealth => 25,
            Self::BCopter => 26,
            Self::TCopter => 27,
            Self::Battleship => 28,
            Self::Cruiser => 29,
            Self::Submarine => 30,
            Self::Lander => 31,
            Self::BlackBoat => 32,
            Self::Carrier => 33,
        }
    }

    pub fn explosion_animation(&self) -> ExplosionAnimation {
        match self {
            UnitClass::Infantry |
            UnitClass::Mech |
            UnitClass::Recon |
            UnitClass::APC |
            UnitClass::Artillery |
            UnitClass::Tank |
            UnitClass::AntiAir |
            UnitClass::Missile |
            UnitClass::Rocket |
            UnitClass::MediumTank |
            UnitClass::Neotank => ExplosionAnimation::Land,

            UnitClass::BCopter |
            UnitClass::TCopter |
            UnitClass::Fighter |
            UnitClass::Bomber |
            UnitClass::Stealth |
            UnitClass::BlackBomb => ExplosionAnimation::Air,

            UnitClass::Battleship |
            UnitClass::Cruiser |
            UnitClass::Submarine |
            UnitClass::Lander |
            UnitClass::Carrier |
            UnitClass::BlackBoat => ExplosionAnimation::Sea,

            UnitClass::MegaTank |
            UnitClass::Piperunner => ExplosionAnimation::Mega,

            // TODO
            UnitClass::Oozium => ExplosionAnimation::Mega,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
enum UnitDirection {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnitAnimation {
    Idle,
    Left,
    Right,
    Up,
    Down,
}

impl UnitAnimation {
    fn is_idle(&self) -> bool {
        if let Self::Idle = self {
            true

        } else {
            false
        }
    }

    fn tile_x(&self) -> u32 {
        match self {
            Self::Idle => 0,
            Self::Left => 3,
            Self::Right => 3,
            Self::Down => 6,
            Self::Up => 9,
        }
    }

    fn direction(&self, nation: &Nation) -> UnitDirection {
        match self {
            UnitAnimation::Idle => match nation {
                Nation::OrangeStar | Nation::GreenEarth => UnitDirection::Right,
                Nation::BlueMoon | Nation::YellowComet | Nation::BlackHole => UnitDirection::Left,
            },
            UnitAnimation::Right => UnitDirection::Right,
            _ => UnitDirection::Left,
        }
    }
}


pub struct Unit {
    pub coord: Mutable<Coord>,
    pub animation: Mutable<UnitAnimation>,
    pub waited: Mutable<bool>,
    pub nation: Nation,
    pub class: UnitClass,
}

impl Unit {
    pub fn new(coord: Coord, class: UnitClass, nation: Nation) -> Arc<Self> {
        Arc::new(Self {
            coord: Mutable::new(coord),
            animation: Mutable::new(UnitAnimation::Idle),
            waited: Mutable::new(false),
            nation,
            class,
        })
    }

    fn tile_x(&self) -> impl Signal<Item = u32> {
        self.animation.signal_ref(move |animation| animation.tile_x()).dedupe()
    }

    fn direction(&self) -> impl Signal<Item = UnitDirection> {
        let nation = self.nation;

        self.animation.signal_ref(move |animation| animation.direction(&nation)).dedupe()
    }

    pub fn render(game: &Arc<Game>, grid: &Arc<Grid>, this: &Arc<Self>) -> Node {
        let nation = this.nation;

        let tile_y = this.class.tile_y(&nation);

        engine::Sprite::builder()
            .spritesheet_signal(game.unit_spritesheet())

            .offset_signal(this.coord.signal_ref(clone!(grid => move |coord| {
                let (x, y) = grid.tile_offset(coord);

                Offset {
                    x: ParentWidth(x - (grid.width * 0.5)),
                    y: ParentHeight(y - grid.height),
                }
            })))

            .size(Size {
                width: ParentWidth(grid.width * 2.0),
                height: ParentHeight(grid.height * 2.0),
            })

            .order_signal(this.coord.signal_ref(clone!(grid => move |coord| {
                Order::Parent(grid.order(coord) + 0.50)
            })).dedupe())

            .tile_signal(map_ref! {
                let tile_x = this.tile_x(),
                let direction = this.direction(),
                let tile_size = game.unit_tile_size(),
                let frame = grid.animation_pendulum(UNIT_ANIMATION_TIME, 3) => {
                    let tile_x = (tile_x + frame) * tile_size;
                    let tile_y = tile_y * tile_size;

                    let tile = Tile {
                        start_x: tile_x,
                        start_y: tile_y,
                        end_x: tile_x + tile_size,
                        end_y: tile_y + tile_size,
                    };

                    match direction {
                        UnitDirection::Left => tile,
                        UnitDirection::Right => tile.mirror_x(),
                    }
                }
            })

            .palette_signal(this.waited.signal_ref(move |waited| {
                let palette = match nation {
                    Nation::OrangeStar => 0,
                    Nation::BlueMoon => 2,
                    Nation::GreenEarth => 4,
                    Nation::YellowComet => 6,
                    Nation::BlackHole => 8,
                };

                if *waited {
                    palette + 1

                } else {
                    palette
                }
            }))

            .build()
    }
}
