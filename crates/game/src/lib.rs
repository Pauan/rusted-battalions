mod grid;
mod util;

use std::sync::{Arc};

use raw_window_handle::{HasRawWindowHandle, HasRawDisplayHandle};
use futures_signals::signal::{Mutable, Signal, SignalExt};
use dominator::clone;

use rusted_battalions_engine as engine;
use rusted_battalions_engine::{
    Engine, EngineSettings, Spritesheet, SpritesheetSettings, RgbaImage,
    Texture, Node, TextureFormat,
};

use grid::{ScreenSize};

pub use grid::{Grid};
pub use engine::Spawner;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnitAppearance {
    DualStrikeSmall,
    DualStrikeBig,
}

impl UnitAppearance {
    fn unit_tile_size(&self) -> u32 {
        match self {
            UnitAppearance::DualStrikeSmall => 32,
            UnitAppearance::DualStrikeBig => 64,
        }
    }
}

impl Default for UnitAppearance {
    fn default() -> Self {
        Self::DualStrikeBig
    }
}


struct Spritesheets {
    terrain: Spritesheet,
    building: Spritesheet,
    unit_small: Spritesheet,
    unit_big: Spritesheet,
    effect: Spritesheet,
}

impl Spritesheets {
    fn new() -> Self {
        Self {
            terrain: Spritesheet::new(),
            building: Spritesheet::new(),
            unit_small: Spritesheet::new(),
            unit_big: Spritesheet::new(),
            effect: Spritesheet::new(),
        }
    }
}


pub struct GameSettings {
    pub appearance: UnitAppearance,
    pub grid: Arc<Grid>,
    pub spawner: Arc<dyn Spawner>,
}


pub struct Game {
    pub unit_appearance: Mutable<UnitAppearance>,

    spritesheets: Spritesheets,

    grid: Mutable<Arc<Grid>>,

    spawner: Arc<dyn Spawner>,
}

impl Game {
    pub fn new(settings: GameSettings) -> Arc<Self> {
        Arc::new(Self {
            unit_appearance: Mutable::new(settings.appearance),

            spritesheets: Spritesheets::new(),

            grid: Mutable::new(settings.grid),

            spawner: settings.spawner,
        })
    }

    pub fn screen_size(&self) -> impl Signal<Item = ScreenSize> {
        self.grid.signal_ref(|grid| grid.screen_size).dedupe()
    }

    pub(crate) fn unit_spritesheet(&self) -> impl Signal<Item = Spritesheet> {
        let unit_small = self.spritesheets.unit_small.clone();
        let unit_big = self.spritesheets.unit_big.clone();

        self.unit_appearance.signal_ref(move |appearance| {
            match appearance {
                UnitAppearance::DualStrikeSmall => unit_small.clone(),
                UnitAppearance::DualStrikeBig => unit_big.clone(),
            }
        })
    }

    pub(crate) fn unit_tile_size(&self) -> impl Signal<Item = u32> {
        self.unit_appearance.signal_ref(|appearance| appearance.unit_tile_size()).dedupe()
    }

    fn render(this: &Arc<Self>) -> Node {
        engine::Stack::builder()
            .child_signal(this.grid.signal_ref(clone!(this => move |grid| {
                Some(Grid::render(&this, grid))
            })))
            .build()
    }

    pub async fn start_engine<Window>(self: &Arc<Self>, window: Window) -> GameEngine<Window>
        where Window: HasRawWindowHandle + HasRawDisplayHandle {

        let screen_size = self.grid.lock_ref().screen_size;

        let mut engine = Engine::new(EngineSettings {
            window,
            scene: Game::render(&self),
            spawner: self.spawner.clone(),
            window_size: engine::WindowSize {
                width: screen_size.width,
                height: screen_size.height,
            },
        }).await;

        // TODO preprocess the images ?
        fn palettize_spritesheet(palette: &RgbaImage, label: &'static str, bytes: &[u8]) -> RgbaImage {
            let mut spritesheet = RgbaImage::new(label, bytes);

            let default_palette = palette.bytes.rows()
                .take(1)
                .flatten()
                .collect::<Vec<&image::Rgba<u8>>>();

            fn get_color(default_palette: &[&image::Rgba<u8>], pixel: &image::Rgba<u8>) -> image::Rgba<u8> {
                for (index, color) in default_palette.into_iter().enumerate() {
                    if pixel == *color {
                        return image::Rgba([index as u8, 0, 0, 255]);
                    }
                }

                panic!("Color not found in palette: {:?}", pixel);
            }

            for pixel in spritesheet.bytes.pixels_mut() {
                if pixel[3] > 0 {
                    *pixel = get_color(&default_palette, pixel);
                }
            }

            spritesheet
        }

        {
            let effect = RgbaImage::new("effect", include_bytes!("../../../dist/sprites/effect.png"));

            let texture = Texture::new_load(&mut engine, &effect, TextureFormat::Rgba8UnormSrgb);

            self.spritesheets.effect.load(&mut engine, SpritesheetSettings {
                texture: &texture,
                palette: None,
            });
        }

        {
            let unit_palette = RgbaImage::new(
                "units_palette",
                include_bytes!("../../../dist/sprites/units_palette.png"),
            );

            let unit_small = palettize_spritesheet(
                &unit_palette,
                "units_small",
                include_bytes!("../../../dist/sprites/units_small.png"),
            );

            let unit_big = palettize_spritesheet(
                &unit_palette,
                "units_big",
                include_bytes!("../../../dist/sprites/units_big.png"),
            );

            let palette_texture = Texture::new_load(&mut engine, &unit_palette, TextureFormat::Rgba8UnormSrgb);

            let texture = Texture::new_load(&mut engine, &unit_small, TextureFormat::Rgba8Uint);

            self.spritesheets.unit_small.load(&mut engine, SpritesheetSettings {
                texture: &texture,
                palette: Some(&palette_texture),
            });

            let texture = Texture::new_load(&mut engine, &unit_big, TextureFormat::Rgba8Uint);

            self.spritesheets.unit_big.load(&mut engine, SpritesheetSettings {
                texture: &texture,
                palette: Some(&palette_texture),
            });
        }

        {
            let buildings_palette = RgbaImage::new(
                "buildings_palette",
                include_bytes!("../../../dist/sprites/buildings_palette.png"),
            );

            let buildings_small = palettize_spritesheet(
                &buildings_palette,
                "buildings_small",
                include_bytes!("../../../dist/sprites/buildings_small.png"),
            );

            let texture = Texture::new_load(&mut engine, &buildings_small, TextureFormat::Rgba8Uint);
            let palette = Texture::new_load(&mut engine, &buildings_palette, TextureFormat::Rgba8UnormSrgb);

            self.spritesheets.building.load(&mut engine, SpritesheetSettings {
                texture: &texture,
                palette: Some(&palette),
            });
        }

        {
            let terrain_palette = RgbaImage::new(
                "terrain_palette",
                include_bytes!("../../../dist/sprites/terrain_palette.png"),
            );

            let terrain_small = palettize_spritesheet(
                &terrain_palette,
                "terrain_small",
                include_bytes!("../../../dist/sprites/terrain_small.png"),
            );

            let texture = Texture::new_load(&mut engine, &terrain_small, TextureFormat::Rgba8Uint);
            let palette = Texture::new_load(&mut engine, &terrain_palette, TextureFormat::Rgba8UnormSrgb);

            self.spritesheets.terrain.load(&mut engine, SpritesheetSettings {
                texture: &texture,
                palette: Some(&palette),
            });
        }

        self.init();

        GameEngine {
            game: self.clone(),
            engine,
            started: false,
        }
    }

    fn init(&self) {
        {
            let grid = self.grid.lock_ref();
            let units = grid.units.lock_ref();

            use grid::{Coord, Nation};
            use grid::unit::{Unit, UnitClass};
            use grid::action::MoveDirection;
            use grid::explosion::ExplosionAnimation;
            use util::random::random;

            grid.spawn_futures(&self.spawner, grid.terrain.iter().map(|tile| {
                let x = tile.x as f32;
                let y = tile.y as f32;

                clone!(grid => async move {
                    let amount = (random() * 4.0) as u32;

                    for _ in 0..amount {
                        grid.wait(random() * 2000.0).await;
                        grid.explosion(ExplosionAnimation::Mega, Coord { x, y }).await;
                    }
                })
            }));

            let futures = units.iter().map(clone!(grid => move |unit| {
                let unit = unit.clone();

                clone!(grid => async move {
                    grid.move_unit(&unit, MoveDirection::Right, 3.0).await;
                    grid.move_unit(&unit, MoveDirection::Down, 3.0).await;
                    grid.move_unit(&unit, MoveDirection::Right, 5.0).await;
                    grid.move_unit(&unit, MoveDirection::Up, 4.0).await;
                    grid.move_unit(&unit, MoveDirection::Left, 2.0).await;
                    grid.move_unit(&unit, MoveDirection::Down, 2.0).await;
                    grid.move_unit(&unit, MoveDirection::Right, 6.0).await;
                    grid.move_unit(&unit, MoveDirection::Down, 7.0).await;

                    grid.wait(2000.0).await;
                    grid.destroy_unit(&unit).await;
                    grid.wait(1000.0).await;


                    let coord = unit.coord.get();

                    let fighter = Unit::new(
                        coord,
                        UnitClass::Fighter,
                        Nation::BlackHole,
                    );

                    grid.units.insert(fighter.clone());

                    grid.wait(2000.0).await;
                    grid.destroy_unit(&fighter).await;
                    grid.wait(1000.0).await;


                    let battleship = Unit::new(
                        coord,
                        UnitClass::Battleship,
                        Nation::GreenEarth,
                    );

                    grid.units.insert(battleship.clone());

                    grid.wait(2000.0).await;
                    grid.destroy_unit(&battleship).await;
                    grid.wait(1000.0).await;

                    let megatank = Unit::new(
                        coord,
                        UnitClass::MegaTank,
                        Nation::BlueMoon,
                    );

                    grid.units.insert(megatank.clone());

                    grid.wait(2000.0).await;
                    grid.destroy_unit(&megatank).await;
                })
            })).collect::<Vec<_>>();

            grid.spawn_futures(&self.spawner, futures);
        }
    }
}


pub struct GameEngine<Window> {
    game: Arc<Game>,
    engine: Engine<Window>,
    started: bool,
}

impl<Window> GameEngine<Window> where Window: HasRawWindowHandle + HasRawDisplayHandle {
    pub fn render(&mut self, time: f64) {
        {
            let grid = self.game.grid.lock_ref();

            grid.cleanup_futures();

            grid.time.set(time);

            // This ensures that we only start updating the grid after the first frame has been displayed.
            // This is necessary to make sure that the engine is fully warmed up and initialized before
            // it starts processing things.
            if self.started {
                grid.start_futures();

            } else {
                self.started = true;
            }
        }

        self.engine.render().unwrap();
    }
}
