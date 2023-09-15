mod grid;
mod util;
mod ui;

use std::sync::{Arc};

use raw_window_handle::{HasRawWindowHandle, HasRawDisplayHandle};
use futures_signals::signal::{Mutable, Signal, SignalExt};
use dominator::clone;

use rusted_battalions_engine as engine;
use rusted_battalions_engine::{
    Engine, EngineSettings, Spritesheet, SpritesheetSettings, RgbaImage,
    GrayscaleImage, IndexedImage, Texture, Node, BitmapFont,
    CharSize, ColorRgb, BitmapText, BitmapFontSettings, BitmapFontSupported,
};

use crate::util::future::executor;
use grid::{ScreenSize};

pub use grid::{Grid};


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


struct Fonts {
    aw_big: BitmapFont,
    unison: BitmapFont,
    unifont: BitmapFont,
}

impl Fonts {
    fn new() -> Self {
        Self {
            aw_big: BitmapFont::new(),
            unison: BitmapFont::new(),
            unifont: BitmapFont::new(),
        }
    }
}


pub struct GameSettings {
    pub appearance: UnitAppearance,
    pub grid: Arc<Grid>,
}


pub struct Game {
    pub unit_appearance: Mutable<UnitAppearance>,

    spritesheets: Spritesheets,
    fonts: Fonts,

    grid: Mutable<Arc<Grid>>,
}

impl Game {
    pub fn new(settings: GameSettings) -> Arc<Self> {
        Arc::new(Self {
            unit_appearance: Mutable::new(settings.appearance),

            spritesheets: Spritesheets::new(),
            fonts: Fonts::new(),

            grid: Mutable::new(settings.grid),
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

            .child(engine::Stack::builder()
                .size(engine::Size {
                    width: engine::Length::Parent(1.0),
                    //width: engine::Length::Px(832),
                    height: engine::Length::Parent(1.0),
                })

                .child(BitmapText::builder()
                    .text(" '-.\nABCDEFGHIJKLMNOPQRSTUVWXYZ\nabcdefghijklmnopqrstuvwxyz\nÆÖÜß\nàáäæèéêíïñóùü\n\nHello there world.\nHow's it going.\nT\u{031A}e\u{0303}s\u{0309}t\u{0310}i\u{1AB4}n\u{20DD}g  o\u{0489}\n\u{0000}\u{0000}\u{0000}\u{0000}T\u{0000}e\u{0000}s\u{0000}t\u{0000}i\u{0000}n\u{0000}g\n\nH̶̢̜̣̰̮͔̜̞͕̖̤͈̒͋͊̇̆̓͗͘ę̶̛͉͎̲̙͈͛̆̇̐̍̓͝͝ͅļ̵̰͓̗̩͎̈̓̎͗̈̇̓̀̀̓͘l̶̡̧̧̛̝͈̻͎̱̰̘͚̪̝̰̫̠̼͔̥̝͚͉̻̙̰̟̫͍̫̳̟̟͕̪̝͚̀́̆̓̉̒̓̈̿͌̀̃͑̚͘ͅͅǫ̵̨̢̢̡̛̙̼̤͍̩̘̬̟̞̹͔͕͙̠͉̟̥̲̝̙̥̺͉͇͓̱̗͖͖͔͍̪̰̳̳̩̠̿̇̍̐̈́́͌̓̀̊́̑̈́̈̊̋̃͛̇̃̍̇͌̆́͜͜͜͜͜͝ ̶̛̫̭͈͎̆̍̌̎̄͌̂̋̉̈́́̀͌́̐̆̓͊̽̉̎́̌̆̾̽͌́̕͘͘͘͘͘͜ẗ̴̘̙̜̤̳̺́̍̃̿̆̌̊͒̀̾̍̋̄̍̇͆͂̀͋̏̈̓̓͘͘͝͝h̵̨̪͓̯̫̯̥͇̭̭̱͉̯̮̻͙̘̻̩̠͉̥̰̟̰̗̠͕̘͈̘͎͉̜̞̤̪͖̍͂͂̋̀̃́̍̍̊̾̊̆̃͂̃̆̊̈́̔̐̽̓͘͘̕̚͘͜͝͝e̴̠̘̹͍̝̐́̂̕͝͠r̴̨̢̨̨̡̤̰͔̬̘͉̩̺̭͓̦̠̞̺͇̲̭̉͆͆͗̅̉̉̾̐̐̈́́̉͛̾͌͗͑́͋̎͗́̑͘̚̕͠͠͝͝ͅȩ̸̧̛̛̳̤̞͇̄̀̀͒̾̾͗͋̓̄̽̃͂̓͑͛̈͋̾̈́̊̔̕̕͝͝ ̶̧̡̗̳̗̳͋̈́͋̅̆͛͗͌̆̆͂̿͌͐͒͑͆m̴̧̢̢̛͎͉̩̺̥̲̺͙͎̱̱̖̼̪͍̪̱̬̩̮̞̲͈̫̭͕̗͈͉̥̙̣̺̻̩̯̪̒̆̈́̂̈́̀͊̑̅͂̀͂͊͑̽̽̃́͛̽̿͗̀̈́̀̓̈́̕͘͘̕͜͜͠ͅy̷̧͍͉̲̟̙͉͍̍̂̍͋̾̈́̋̒͌́̿̏͒̒́̊̈́͆̒́̊̆̈̀̎͛̏̆̈́̓̓̒̆͘̕͠͝ ̵̛͓̲̠͖̠̞͂̓̈͆͆̈́̇̇̄͒͋͑̉̏̈́̓́͐̅͐̉̃̃̚̕͘f̴̧̨̩̱̖̜͔̜̣͎̜͖̰̦͈̞̳̥͙̺̜̺̻̳̦̗̜̣͔̘̲̻̩̙̫̱͆̃͊̓͌̈́͊̂̌̊͐͊̂̋̑̂͗͑͜ͅŗ̵̮̺̱͔͖͖̖̲̯͚̬̰͎̜̺̫̠̮̺̰̮͖̳̜̈́̓̇̈́̓͊͋̓̈̀͌͊̆̈̂͑́̊̕͝í̸̢̡̨̢̡̡͇̪̗̬̹̺̝̪͍͙̻̯̲̮͔̼̟̰̞̱̩̱͉̭̹̬͚̼̮͎͚̙̤̱̰̙̯̩̼̬̊̋̓̏̅̒̔͋͑̿̀͛͊͒͌̄̔̉͠ͅͅê̷̛̘̣̞̮͉͙̣̘̦̝̯̰̠͉͉̖̞̘̰͕̻̯̰͖͙̜͖̮͉̖̪̲̪̩͇̥̠͎̲̜͓͈̥̋̈́̄͛͗̈́̿̀͌͘͜͜͠ͅͅͅñ̷̨̡̧̗̣̣̠̥̺̫͓̹̲͓̮̜͕̯̦͚͓̝̩̲͕̳̹͓̻̝̺̼͇̟̜̙̬̤͚̭̠̪̼̫̣̬͈̎̆̒̅͋͛̃͐͌͒̏̃͊̕͜͜ͅd̵̢̧̡̛͚͕͍͖̯̝̦̠̬̬̺̩̯̜̠̱̥̤̼͖̪͙̪̩̼̠͚̘͍̎̏̃.̸̨̩̖̱̭̯̤͔͓͎̙̼̲̮͍͉̦͓͙̠̦̲͈̯͉̯̱̲͙̤̳͍̏̽̂̂͊̈̀̇̐̉́̀̑͑́̌̈́̾̇̏̈͒̊̉̾̀̓̀̋͆͗͌̌̊͐͋̀̈́̀͑̐͋̾͊̚͜͠ͅͅ".into())
                    .font(this.fonts.unifont.clone())
                    .char_size(CharSize {
                        width: engine::Length::Px(32),
                        height: engine::Length::Px(64),
                    })
                    .build())

                .build())

            .build()
    }

    pub async fn start_engine<Window>(self: &Arc<Self>, window: Window) -> GameEngine<Window>
        where Window: HasRawWindowHandle + HasRawDisplayHandle {

        let screen_size = self.grid.lock_ref().screen_size;

        let mut engine = Engine::new(EngineSettings {
            window,
            scene: Game::render(&self),
            spawner: Arc::new(executor::CustomSpawner),
            window_size: engine::WindowSize {
                width: screen_size.width,
                height: screen_size.height,
            },
        }).await;

        // TODO preprocess the images ?
        fn palettize_spritesheet(palette: &RgbaImage, label: &'static str, bytes: &[u8]) -> IndexedImage {
            let default_palette = palette.image.rows()
                .take(1)
                .flatten()
                .collect::<Vec<&image::Rgba<u8>>>();

            let spritesheet = RgbaImage::from_bytes(label, bytes);

            let (width, height) = spritesheet.image.dimensions();

            IndexedImage::from_fn(label, width, height, |x, y| {
                let pixel = spritesheet.image.get_pixel(x, y);

                let alpha = pixel[3];

                if alpha > 0 {
                    for (index, color) in default_palette.iter().enumerate() {
                        if pixel == *color {
                            return image::LumaA([index as u8, alpha]);
                        }
                    }

                    panic!("Color not found in palette: {:?}", pixel);

                } else {
                    image::LumaA([0, 0])
                }
            })
        }

        {
            let effect = RgbaImage::from_bytes("effect", include_bytes!("../../../dist/sprites/effect.png"));

            let texture = Texture::new();

            texture.load(&mut engine, &effect);

            self.spritesheets.effect.load(&mut engine, SpritesheetSettings {
                texture: &texture,
                palette: None,
            });
        }

        {
            let unit_palette = RgbaImage::from_bytes(
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

            let palette_texture = Texture::new();

            palette_texture.load(&mut engine, &unit_palette);

            let texture = Texture::new();

            texture.load(&mut engine, &unit_small);

            self.spritesheets.unit_small.load(&mut engine, SpritesheetSettings {
                texture: &texture,
                palette: Some(&palette_texture),
            });

            let texture = Texture::new();

            texture.load(&mut engine, &unit_big);

            self.spritesheets.unit_big.load(&mut engine, SpritesheetSettings {
                texture: &texture,
                palette: Some(&palette_texture),
            });
        }

        {
            let buildings_palette = RgbaImage::from_bytes(
                "buildings_palette",
                include_bytes!("../../../dist/sprites/buildings_palette.png"),
            );

            let buildings_small = palettize_spritesheet(
                &buildings_palette,
                "buildings_small",
                include_bytes!("../../../dist/sprites/buildings_small.png"),
            );

            let texture = Texture::new();
            let palette = Texture::new();

            texture.load(&mut engine, &buildings_small);
            palette.load(&mut engine, &buildings_palette);

            self.spritesheets.building.load(&mut engine, SpritesheetSettings {
                texture: &texture,
                palette: Some(&palette),
            });
        }

        {
            let terrain_palette = RgbaImage::from_bytes(
                "terrain_palette",
                include_bytes!("../../../dist/sprites/terrain_palette.png"),
            );

            let terrain_small = palettize_spritesheet(
                &terrain_palette,
                "terrain_small",
                include_bytes!("../../../dist/sprites/terrain_small.png"),
            );

            let texture = Texture::new();
            let palette = Texture::new();

            texture.load(&mut engine, &terrain_small);
            palette.load(&mut engine, &terrain_palette);

            self.spritesheets.terrain.load(&mut engine, SpritesheetSettings {
                texture: &texture,
                palette: Some(&palette),
            });
        }

        /*{
            let aw_font = RgbaImage::from_bytes(
                "aw_font",
                include_bytes!("../../../dist/sprites/text.png"),
            );

            let texture = Texture::new();

            texture.load(&mut engine, &aw_font);

            self.fonts.aw_big.load(&mut engine, BitmapFontSettings {
                texture: &texture,
                columns: 32,
                tile_width: 16,
                tile_height: 32,
            });
        }*/

        /*{
            let unison_font = GrayscaleImage::from_bytes(
                "unison_font",
                include_bytes!("../../../dist/fonts/unison.png"),
            );

            let texture = Texture::new();

            texture.load(&mut engine, &unison_font);

            self.fonts.unison.load(&mut engine, BitmapFontSettings {
                texture: &texture,
                columns: 64,
                tile_width: 4,
                tile_height: 16,
            });
        }*/

        #[cfg(feature = "unicode")]
        {
            let image = GrayscaleImage::from_bytes(
                "unifont_bmp",
                include_bytes!("../../../dist/fonts/unifont_bmp.png"),
            );

            let texture = Texture::new();

            texture.load(&mut engine, &image);

            self.fonts.unifont.load(&mut engine, BitmapFontSettings {
                texture: &texture,
                supported: BitmapFontSupported {
                    start: '\u{0000}',
                    end: '\u{FFFD}',
                    replace: '\u{FFFD}',
                },
                columns: 256,
                tile_width: 8,
                tile_height: 16,
            });
        }

        #[cfg(not(feature = "unicode"))]
        {
            let image = GrayscaleImage::from_bytes(
                "unifont_ascii",
                include_bytes!("../../../dist/fonts/unifont_ascii.png"),
            );

            let texture = Texture::new();

            texture.load(&mut engine, &image);

            self.fonts.unifont.load(&mut engine, BitmapFontSettings {
                texture: &texture,
                supported: BitmapFontSupported {
                    start: '\u{0000}',
                    end: '\u{007F}',
                    replace: '\u{001A}',
                },
                columns: 16,
                tile_width: 8,
                tile_height: 16,
            });
        }

        self.init();

        GameEngine {
            game: self.clone(),
            engine,
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

            grid.spawn_futures(grid.terrain.iter().map(|tile| {
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

            grid.spawn_futures(futures);
        }
    }
}


pub struct GameEngine<Window> {
    game: Arc<Game>,
    engine: Engine<Window>,
}

impl<Window> GameEngine<Window> where Window: HasRawWindowHandle + HasRawDisplayHandle {
    pub fn render(&mut self, time: f64) {
        {
            let grid = self.game.grid.lock_ref();

            grid.time.set(time);

            executor::run_futures();

            // This ensures that we only start updating the grid after the first frame has been displayed.
            // This is necessary to make sure that the engine is fully warmed up and initialized before
            // it starts processing things.
            grid.start_futures();
        }

        self.engine.render().unwrap();
    }
}
