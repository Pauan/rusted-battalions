use super::{TerrainRule, TerrainFlag};

const EMPTY: TerrainFlag = TerrainFlag::EMPTY;
const GROUND: TerrainFlag = TerrainFlag::GROUND;
const RIVER: TerrainFlag = TerrainFlag::RIVER;
const ANY: TerrainFlag = TerrainFlag::ANY;
const WATER: TerrainFlag = TerrainFlag::WATER;

pub(crate) fn rules() -> impl Iterator<Item = TerrainRule> {
    // Starting coordinate for the river tiles
    let x: u32 = 4;
    let y: u32 = 4;

    [
        // None
        TerrainRule {
            tile_x: x + 3,
            tile_y: y + 3,

            up: GROUND | EMPTY,
            down: GROUND | EMPTY,
            left: GROUND | EMPTY,
            right: GROUND | EMPTY,

            up_left: ANY,
            up_right: ANY,
            down_left: ANY,
            down_right: ANY,
        },


        // I shapes
        TerrainRule {
            tile_x: x + 3,
            tile_y: y + 0,

            up: WATER,
            down: RIVER,
            left: WATER,
            right: WATER,

            up_left: WATER,
            up_right: WATER,
            down_left: GROUND,
            down_right: GROUND,
        },

        TerrainRule {
            tile_x: x + 3,
            tile_y: y + 1,

            up: RIVER,
            down: RIVER,
            left: GROUND,
            right: GROUND,

            up_left: ANY,
            up_right: ANY,
            down_left: ANY,
            down_right: ANY,
        },

        TerrainRule {
            tile_x: x + 3,
            tile_y: y + 2,

            up: RIVER,
            down: WATER,
            left: WATER,
            right: WATER,

            up_left: GROUND,
            up_right: GROUND,
            down_left: WATER,
            down_right: WATER,
        },

        TerrainRule {
            tile_x: x + 0,
            tile_y: y + 3,

            up: WATER,
            down: WATER,
            left: WATER,
            right: RIVER,

            up_left: WATER,
            up_right: GROUND,
            down_left: WATER,
            down_right: GROUND,
        },

        TerrainRule {
            tile_x: x + 1,
            tile_y: y + 3,

            up: GROUND,
            down: GROUND,
            left: RIVER,
            right: RIVER,

            up_left: ANY,
            up_right: ANY,
            down_left: ANY,
            down_right: ANY,
        },

        TerrainRule {
            tile_x: x + 2,
            tile_y: y + 3,

            up: WATER,
            down: WATER,
            left: RIVER,
            right: WATER,

            up_left: GROUND,
            up_right: WATER,
            down_left: GROUND,
            down_right: WATER,
        },


        // L shapes
        TerrainRule {
            tile_x: x + 0,
            tile_y: y + 0,

            up: GROUND,
            down: RIVER,
            left: GROUND,
            right: RIVER,

            up_left: ANY,
            up_right: ANY,
            down_left: ANY,
            down_right: GROUND,
        },

        TerrainRule {
            tile_x: x + 2,
            tile_y: y + 0,

            up: GROUND,
            down: RIVER,
            left: RIVER,
            right: GROUND,

            up_left: ANY,
            up_right: ANY,
            down_left: GROUND,
            down_right: ANY,
        },

        TerrainRule {
            tile_x: x + 0,
            tile_y: y + 2,

            up: RIVER,
            down: GROUND,
            left: GROUND,
            right: RIVER,

            up_left: ANY,
            up_right: GROUND,
            down_left: ANY,
            down_right: ANY,
        },

        TerrainRule {
            tile_x: x + 2,
            tile_y: y + 2,

            up: RIVER,
            down: GROUND,
            left: RIVER,
            right: GROUND,

            up_left: GROUND,
            up_right: ANY,
            down_left: ANY,
            down_right: ANY,
        },


        // All
        TerrainRule {
            tile_x: x + 1,
            tile_y: y + 1,

            up: RIVER,
            down: RIVER,
            left: RIVER,
            right: RIVER,

            up_left: GROUND,
            up_right: GROUND,
            down_left: GROUND,
            down_right: GROUND,
        },
    ].into_iter()
}
