use std::sync::Arc;
use std::future::Future;
use futures_signals::signal::{Mutable, Signal, SignalExt};
use futures_signals::signal_vec::{SignalVecExt};
use dominator::clone;
use rusted_battalions_engine as engine;
use rusted_battalions_engine::{Node, Order};

use crate::{Game};
use crate::util::future::{FutureSpawner};
use crate::util::signal::{SortedVec};

use terrain::{Terrain, TerrainClass, Orientation, TerrainTile};
use building::{Building, BuildingClass};
use unit::{Unit, UnitClass};
use explosion::{Explosion};

pub mod action;
pub mod terrain;
pub mod unit;
pub mod building;
pub mod explosion;


const UNIT_ANIMATION_TIME: f64 = 250.0;
const EXPLOSION_ANIMATION_TIME: f64 = 500.0;
const BUILDING_ANIMATION_TIME: f64 = 500.0;
const TERRAIN_ANIMATION_TIME: f64 = 500.0;
const FOG_ANIMATION_TIME: f64 = 1000.0;


fn lerp_f32(from: f32, to: f32, percent: f32) -> f32 {
    ((1.0 - percent) * from) + (percent * to)
}


#[derive(Debug, Clone, Copy)]
pub enum Nation {
    OrangeStar,
    BlueMoon,
    GreenEarth,
    YellowComet,
    BlackHole,
}

impl Nation {
    pub const ALL: &[Self] = &[
        Self::OrangeStar,
        Self::BlueMoon,
        Self::GreenEarth,
        Self::YellowComet,
        Self::BlackHole,
    ];
}


#[derive(Debug, Clone, Copy)]
pub struct Coord {
    pub x: f32,
    pub y: f32,
}

impl Coord {
    pub fn lerp(self, other: Self, percent: f32) -> Self {
        Self {
            x: lerp_f32(self.x, other.x, percent),
            y: lerp_f32(self.y, other.y, percent),
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenSize {
    pub width: u32,
    pub height: u32,
}


pub struct Grid {
    pub screen_size: ScreenSize,

    pub(crate) width: f32,
    pub(crate) height: f32,

    pub(crate) terrain: Terrain,

    pub(crate) buildings: Vec<Arc<Building>>,

    pub(crate) units: SortedVec<Unit>,

    pub(crate) explosions: SortedVec<Explosion>,

    pub(crate) time: Mutable<f64>,

    spawner: FutureSpawner,
}

impl Grid {
    pub fn new(terrain: Terrain, buildings: Vec<Arc<Building>>, units: Vec<Arc<Unit>>) -> Arc<Self> {
        Arc::new(Self {
            screen_size: ScreenSize {
                width: terrain.width * 32,
                height: terrain.height * 32,
            },

            width: 1.0 / (terrain.width as f32),
            height: 1.0 / (terrain.height as f32),

            units: SortedVec::with_values(units),
            explosions: SortedVec::new(),
            buildings,
            terrain,

            time: Mutable::new(0.0),

            spawner: FutureSpawner::new(),
        })
    }


    /// Returns a Signal that will last for `duration` number of milliseconds.
    ///
    /// The value of the Signal is the percentage of time from now until `duration`:
    ///
    ///   0.0 = now
    ///   1.0 = now + duration
    ///
    /// Once the Signal reaches 1.0 it will stop.
    pub(crate) fn timer(&self, duration: f64) -> impl Signal<Item = f64> + Send {
        struct TimerState {
            start: f64,
            end: f64,
        }

        let mut state = None;

        self.time.signal_ref(move |time| {
            let state = state.get_or_insert_with(|| {
                TimerState {
                    start: *time,
                    end: time + duration,
                }
            });

            if *time >= state.end {
                1.0

            } else {
                (time - state.start) / duration
            }
        }).stop_if(|value| *value == 1.0)
    }

    pub(crate) fn animation(&self, duration: f64) -> impl Signal<Item = f64> {
        self.time.signal_ref(move |time| (time / duration))
    }

    /// When it reaches the end of the frames, it starts again from the beginning.
    pub(crate) fn animation_loop(&self, duration: f64, frames: u32) -> impl Signal<Item = u32> {
        let modulo = frames as f64;

        self.time.signal_ref(move |time| {
            ((time / duration) % modulo) as u32
        }).dedupe()
    }

    /// Behaves like a pendulum, oscillating between the start and end.
    ///
    /// When it reaches the end of the frames, it then reverses direction.
    /// When it reaches the start of the frames, it then reverses direction again.
    pub(crate) fn animation_pendulum(&self, duration: f64, frames: u32) -> impl Signal<Item = u32> {
        let frames = frames - 1;
        let total_frames = frames * 2;

        let modulo = total_frames as f64;

        self.time.signal_ref(move |time| {
            let frame = ((time / duration) % modulo) as u32;

            if frame > frames {
                total_frames - frame

            } else {
                frame
            }
        }).dedupe()
    }


    #[inline]
    pub(crate) fn start_futures(&self) {
        self.spawner.start();
    }

    #[inline]
    pub(crate) fn spawn_future<F>(&self, f: F)
        where F: Future<Output = ()> + 'static {
        self.spawner.spawn(f);
    }

    #[inline]
    pub(crate) fn spawn_futures<I>(&self, iter: I)
        where I: IntoIterator,
              I::Item: Future<Output = ()> + 'static {
        self.spawner.spawn_iter(iter);
    }


    pub(crate) fn tile_offset(&self, coord: &Coord) -> (f32, f32) {
        (
            coord.x * self.width,
            coord.y * self.height,
        )
    }

    pub(crate) fn order(&self, coord: &Coord) -> f32 {
        coord.y.ceil()
    }


    pub(crate) fn render(game: &Arc<Game>, this: &Arc<Self>) -> Node {
        engine::Stack::builder()
            .children(this.terrain.iter().map(|tile| {
                TerrainTile::render(game, this, tile)
            }))

            .children(this.buildings.iter().map(|building| {
                Building::render(game, this, building)
            }))

            .child(engine::Stack::builder()
                .order(Order::Parent(0.0))
                .children_signal_vec(this.units.signal_vec().map(clone!(game, this => move |unit| {
                    Unit::render(&game, &this, &unit)
                })))
                .build())

            .child(engine::Stack::builder()
                .order(Order::Parent(0.0))
                .children_signal_vec(this.explosions.signal_vec().map(clone!(game, this => move |explosion| {
                    Explosion::render(&game, &this, &explosion)
                })))
                .build())

            .build()
    }


    pub fn test() -> Arc<Self> {
        /*self.engine.ui.boxes.update(|boxes| {
            boxes.push(UiBox {
                position: [0.0, 0.0],
                scale: [0.3, 0.3],
                origin: [0.0, 0.0],
                z_index: 2,
            });
        });*/

        /*let mut index = 0;

        for y in 0..self.grid.height {
            for x in 0..self.grid.width {
                self.grid.terrain.get_mut(x, y).class = TerrainClass::ALL[index % TerrainClass::ALL.len()];

                //self.grid.terrain.push(terrain);

                index += 1;
            }
        }

        self.grid.terrain.update_tiles();*/


        /*let mut index = 0;

        for x in 0..self.grid.width {
            for y in 0..self.grid.height {
                let building = Building {
                    coord: Coord { x, y },
                    nation: Some(Nation::ALL[index % Nation::ALL.len()]),
                    class: BuildingClass::ALL[index % BuildingClass::ALL.len()],
                    frame_offset: 0,
                    //frame_offset: (js_sys::Math::random() * 10.0).floor() as u32,
                    fog: false,
                };

                self.grid.buildings.push(building);

                index += 1;
            }
        }*/


        /*let mut index = 0;

        let i = 0;

        //for i in 0..500 {
            for x in 0..self.grid.width {
                for y in 0..self.grid.height {
                    let unit = Unit {
                        coord: Coord { x, y },
                        moving: UnitMoving::Idle,
                        nation: Nation::ALL[(index + i) % Nation::ALL.len()],
                        //class: UnitClass::Fighter,
                        class: UnitClass::ALL[(index + i) % UnitClass::ALL.len()],
                        waited: false,
                    };

                    self.grid.units.push(unit);

                    index += 1;
                }
            }
        //}*/

        /*let mut x = 0;
        let mut y = 0;

        for nation in Nation::ALL {
            for class in UnitClass::ALL {
                let unit = Unit {
                    coord: Coord { x, y },
                    moving: UnitMoving::Left,
                    nation: *nation,
                    class: *class,
                    waited: false,
                };

                self.grid.units.push(unit);

                x += 1;

                if x >= self.grid.width {
                    x = 0;
                    y += 1;
                }
            }
        }*/

        let mut terrain = Terrain::new(78, 30);
        let mut buildings = vec![];
        let mut units = vec![];

        for tile in terrain.iter_mut() {
            tile.class = TerrainClass::Grass;
        }

        let mut tiles = vec![
            (0, 0, TerrainClass::random_mountain()),
            (0, 1, TerrainClass::random_mountain()),
            (0, 2, TerrainClass::random_mountain()),
            (1, 1, TerrainClass::Forest),
            (1, 2, TerrainClass::random_mountain()),
            (0, 3, TerrainClass::random_mountain()),
            (1, 3, TerrainClass::random_mountain()),

            (0, 4, TerrainClass::Forest),
            (1, 4, TerrainClass::Forest),
            (0, 5, TerrainClass::Forest),
            (1, 5, TerrainClass::Forest),

            (0, 7, TerrainClass::Bridge { orientation: Orientation::Vertical }),
            (1, 7, TerrainClass::Bridge { orientation: Orientation::Horizontal }),

            (0, 9, TerrainClass::Pipe),
            (1, 9, TerrainClass::Pipeseam { destroyed: false }),
            (2, 9, TerrainClass::Pipe),

            (0, 11, TerrainClass::Pipe),
            (1, 11, TerrainClass::Pipeseam { destroyed: true }),
            (2, 11, TerrainClass::Pipe),

            (0, 13, TerrainClass::Pipe),
            (0, 14, TerrainClass::Pipeseam { destroyed: false }),
            (0, 15, TerrainClass::Pipe),

            (2, 13, TerrainClass::Pipe),
            (2, 14, TerrainClass::Pipeseam { destroyed: true }),
            (2, 15, TerrainClass::Pipe),
        ];

        fn test_all(tiles: &mut Vec<(u32, u32, TerrainClass)>, x: u32, y: u32, class: TerrainClass) {
            tiles.append(&mut vec![
                (x + 0, y + 0, class),

                (x + 0, y + 2, class),
                (x + 1, y + 2, class),

                (x + 0, y + 4, class),
                (x + 1, y + 4, class),
                (x + 2, y + 4, class),

                (x + 0, y + 6, class),
                (x + 1, y + 6, class),
                (x + 0, y + 7, class),
                (x + 1, y + 7, class),

                (x + 0, y + 9, class),
                (x + 1, y + 9, class),
                (x + 2, y + 9, class),
                (x + 0, y + 10, class),
                (x + 0, y + 11, class),
                (x + 1, y + 11, class),
                (x + 2, y + 10, class),
                (x + 2, y + 11, class),

                (x + 0, y + 13, class),
                (x + 1, y + 13, class),
                (x + 2, y + 13, class),
                (x + 0, y + 14, class),
                (x + 1, y + 14, class),
                (x + 0, y + 15, class),
                (x + 1, y + 15, class),
                (x + 2, y + 14, class),
                (x + 2, y + 15, class),

                (x + 1, y + 17, class),
                (x + 2, y + 17, class),
                (x + 0, y + 18, class),
                (x + 1, y + 18, class),
                (x + 0, y + 19, class),
                (x + 1, y + 19, class),
                (x + 2, y + 18, class),

                (x + 5, y + 0, class),
                (x + 4, y + 1, class),
                (x + 5, y + 1, class),
                (x + 5, y + 2, class),
                (x + 6, y + 1, class),

                (x + 4, y + 3, class),
                (x + 4, y + 4, class),

                (x + 6, y + 3, class),
                (x + 6, y + 4, class),
                (x + 6, y + 5, class),

                (x + 4, y + 6, class),
                (x + 4, y + 7, class),
                (x + 5, y + 7, class),
                (x + 4, y + 8, class),

                (x + 6, y + 8, class),
                (x + 6, y + 9, class),
                (x + 5, y + 9, class),
                (x + 6, y + 10, class),

                (x + 4, y + 12, class),
                (x + 5, y + 12, class),
                (x + 6, y + 12, class),
                (x + 5, y + 11, class),

                (x + 4, y + 14, class),
                (x + 5, y + 14, class),
                (x + 6, y + 14, class),
                (x + 5, y + 15, class),

                (x + 4, y + 17, class),
                (x + 5, y + 17, class),
                (x + 6, y + 17, class),
                (x + 4, y + 18, class),
                (x + 5, y + 18, class),
                (x + 6, y + 18, class),
                (x + 5, y + 19, class),

                (x + 4, y + 22, class),
                (x + 5, y + 22, class),
                (x + 6, y + 22, class),
                (x + 4, y + 23, class),
                (x + 5, y + 23, class),
                (x + 6, y + 23, class),
                (x + 5, y + 21, class),
            ]);
        }

        test_all(&mut tiles, 4, 1, TerrainClass::Pipe);
        test_all(&mut tiles, 12, 1, TerrainClass::Road { ruins: false });
        test_all(&mut tiles, 20, 1, TerrainClass::Road { ruins: true });
        test_all(&mut tiles, 28, 1, TerrainClass::Ocean);
        test_all(&mut tiles, 36, 0, TerrainClass::River);
        test_all(&mut tiles, 44, 1, TerrainClass::Shoal);

        for (x, y, class) in tiles {
            terrain.get_mut(x, y).class = class;
        }

        terrain.update_tiles();

        units.push(Unit::new(
            Coord { x: 13.0, y: 11.0 },
            UnitClass::Infantry,
            Nation::OrangeStar,
        ));

        units.push(Unit::new(
            Coord { x: 0.0, y: 17.0 },
            UnitClass::Infantry,
            Nation::OrangeStar,
        ));

        units.push(Unit::new(
            Coord { x: 1.0, y: 5.0 },
            UnitClass::Infantry,
            Nation::OrangeStar,
        ));

        units.push(Unit::new(
            Coord { x: 0.0, y: 4.0 },
            UnitClass::Infantry,
            Nation::OrangeStar,
        ));

        units.push(Unit::new(
            Coord { x: 0.0, y: 2.0 },
            UnitClass::Infantry,
            Nation::OrangeStar,
        ));

        buildings.push(Building::new(
            Coord { x: 0.0, y: 17.0 },
            BuildingClass::City,
            Some(Nation::OrangeStar),
        ));

        Self::new(terrain, buildings, units)
    }

    pub fn test_performance() -> Arc<Self> {
        let mut terrain = Terrain::new(40, 30);

        for tile in terrain.iter_mut() {
            tile.class = TerrainClass::Grass;
        }

        terrain.update_tiles();

        let mut units = vec![];

        let mut index = 0;

        // Raw implementation:
        //   3000 =  9.6 FPS
        //   2000 = 13.8 FPS
        //   1000 = 27.6 FPS
        //    700 = 30.0 FPS
        //    500 = 39.6 FPS
        //   ^320 = 49.0 FPS
        //
        // Signal implementation:
        //   1000 =  1.8 FPS
        //    700 =  7.2 FPS
        //    500 = 21.6 FPS
        //    200 = 33.6 FPS
        //   ^110 = 48.0 FPS
        for i in 0..1 {
            for y in 0..terrain.height {
                for x in 0..terrain.width {
                    units.push(Unit::new(
                        Coord { x: x as f32, y: y as f32 },
                        UnitClass::ALL[(index + i) % UnitClass::ALL.len()],
                        Nation::ALL[(index + i) % Nation::ALL.len()],
                    ));

                    index += 1;
                }
            }
        }

        Self::new(terrain, vec![], units)
    }
}
