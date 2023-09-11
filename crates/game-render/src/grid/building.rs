use std::sync::Arc;
use futures_signals::map_ref;
use futures_signals::signal::{Mutable, Signal, SignalExt};
use rusted_battalions_engine as engine;
use rusted_battalions_engine::{Node, Length, Size, Offset, Tile};

use crate::Game;
use crate::grid::{BUILDING_ANIMATION_TIME, Grid, Coord, Nation};


#[derive(Debug, Clone, Copy)]
pub enum BuildingClass {
    HQ1, // Orange Star
    HQ2, // Blue Moon
    HQ3, // Green Earth
    HQ4, // Yellow Comet
    HQ5, // Black Hole
    City,
    Base,
    Airport,
    Port,
    ComTower,
    Lab,
    MissileSilo,
    MissileSiloEmpty,
    /*BlackCrystal,
    Laser,
    Minicannon { direction: , grass: bool },
    Volcano,
    BlackOnyx, // Flying Fortress
    Fortress,
    BlackArmageddon,
    BlackCannon { direction: },
    BlackObelisk,*/
}

impl BuildingClass {
    pub const ALL: &[Self] = &[
        Self::HQ1,
        Self::HQ2,
        Self::HQ3,
        Self::HQ4,
        Self::HQ5,
        Self::City,
        Self::Base,
        Self::Airport,
        Self::Port,
        Self::ComTower,
        Self::Lab,
        Self::MissileSilo,
        Self::MissileSiloEmpty,
    ];

    fn can_have_nation(&self) -> bool {
        match self {
            Self::MissileSilo | Self::MissileSiloEmpty => false,
            _ => true,
        }
    }
}


pub struct Building {
    pub coord: Coord,
    pub nation: Mutable<Option<Nation>>,
    pub class: BuildingClass,
    pub fog: Mutable<bool>,
}

impl Building {
    const TILE_WIDTH: u32 = 16;
    const TILE_HEIGHT: u32 = 32;

    pub fn new(coord: Coord, class: BuildingClass, nation: Option<Nation>) -> Arc<Self> {
        Arc::new(Self {
            coord,
            class,
            nation: Mutable::new(nation),
            fog: Mutable::new(false),
        })
    }

    fn has_nation(&self) -> impl Signal<Item = bool> {
        self.nation.signal_ref(|nation| nation.is_some()).dedupe()
    }

    fn tile_x(&self, grid: &Arc<Grid>) -> impl Signal<Item = u32> {
        let can_have_nation = self.class.can_have_nation();

        map_ref! {
            let fog = self.fog.signal(),
            let has_nation = self.has_nation(),
            let frame = grid.animation_loop(BUILDING_ANIMATION_TIME, 4) => move {
                if *fog {
                    Self::TILE_WIDTH

                } else if can_have_nation && *has_nation {
                    (2 + frame) * Self::TILE_WIDTH

                } else {
                    0
                }
            }
        }.dedupe()
    }

    pub fn render(game: &Arc<Game>, grid: &Arc<Grid>, this: &Arc<Self>) -> Node {
        let tile_y = match this.class {
            BuildingClass::HQ1 => 0 * Self::TILE_HEIGHT,
            BuildingClass::HQ2 => 1 * Self::TILE_HEIGHT,
            BuildingClass::HQ3 => 2 * Self::TILE_HEIGHT,
            BuildingClass::HQ4 => 3 * Self::TILE_HEIGHT,
            BuildingClass::HQ5 => 4 * Self::TILE_HEIGHT,
            BuildingClass::City => 5 * Self::TILE_HEIGHT,
            BuildingClass::Base => 6 * Self::TILE_HEIGHT,
            BuildingClass::Airport => 7 * Self::TILE_HEIGHT,
            BuildingClass::Port => 8 * Self::TILE_HEIGHT,
            BuildingClass::ComTower => 9 * Self::TILE_HEIGHT,
            BuildingClass::Lab => 10 * Self::TILE_HEIGHT,
            BuildingClass::MissileSilo => 11 * Self::TILE_HEIGHT,
            BuildingClass::MissileSiloEmpty => 12 * Self::TILE_HEIGHT,
        };

        let (x, y) = grid.tile_offset(&this.coord);

        engine::Sprite::builder()
            .spritesheet(game.spritesheets.building.clone())

            .tile_signal(this.tile_x(grid).map(move |tile_x| {
                Tile {
                    start_x: tile_x,
                    start_y: tile_y,
                    end_x: tile_x + Self::TILE_WIDTH,
                    end_y: tile_y + Self::TILE_HEIGHT,
                }
            }))

            .palette_signal(this.nation.signal_ref(|nation| {
                match nation {
                    None => 0,
                    Some(Nation::OrangeStar) => 0,
                    Some(Nation::BlueMoon) => 1,
                    Some(Nation::GreenEarth) => 2,
                    Some(Nation::YellowComet) => 3,
                    Some(Nation::BlackHole) => 4,
                }
            }))

            .z_index(grid.z_index(&this.coord) + 0.25)

            .offset(Offset {
                x: Length::Parent(x),
                y: Length::Parent(y - grid.height),
            })

            .size(Size {
                width: Length::Parent(grid.width),
                height: Length::Parent(grid.height * 2.0),
            })

            .build()
    }
}