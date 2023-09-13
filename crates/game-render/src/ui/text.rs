use rusted_battalions_engine as engine;
use rusted_battalions_engine::{Spritesheet, SpriteBuilder, Tile};


pub struct BitmapText {
    pub spritesheet: Spritesheet,
    pub tile_width: u32,
    pub tile_height: u32,
    pub columns: u32,
}

impl BitmapText {
    fn to_tile(&self, char: char) -> Tile {
        let index = char as u32;

        let row = index / self.columns;
        let column = index - (row * self.columns);

        let start_x = column * self.tile_width;
        let start_y = row * self.tile_height;

        Tile {
            start_x,
            start_y,
            end_x: start_x + self.tile_width,
            end_y: start_y + self.tile_height,
        }
    }

    pub fn sprite(&self, c: char) -> SpriteBuilder {
        engine::Sprite::builder()
            .spritesheet(self.spritesheet.clone())
            .tile(self.to_tile(c))
    }

    pub fn sprites<'a>(&'a self, text: &'a str) -> impl Iterator<Item = SpriteBuilder> + 'a {
        text.chars().map(|c| self.sprite(c))
    }
}
