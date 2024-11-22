use super::{TerrainRule, TerrainFlag};

const EMPTY: TerrainFlag = TerrainFlag::EMPTY;
const GROUND: TerrainFlag = TerrainFlag::GROUND;
const SEA: TerrainFlag = TerrainFlag::SEA;
const SHOAL: TerrainFlag = TerrainFlag::SHOAL;
const ANY: TerrainFlag = TerrainFlag::ANY;
const WATER: TerrainFlag = TerrainFlag::WATER;

pub(crate) fn rules() -> impl Iterator<Item = TerrainRule> {
    // Starting coordinate for the shoal tiles
    let x: u32 = 8;
    let y: u32 = 4;

    [
        // None
        TerrainRule {
            tile_x: x + 3,
            tile_y: y + 3,

            up: GROUND,
            down: GROUND,
            left: GROUND,
            right: GROUND,

            up_left: ANY,
            up_right: ANY,
            down_left: ANY,
            down_right: ANY,
        },


        // I shapes
        TerrainRule {
            tile_x: x + 3,
            tile_y: y + 0,

            up: GROUND,
            down: SHOAL | EMPTY,
            left: GROUND,
            right: GROUND,

            up_left: ANY,
            up_right: ANY,
            down_left: ANY,
            down_right: ANY,
        },

        TerrainRule {
            tile_x: x + 3,
            tile_y: y + 1,

            up: SHOAL | EMPTY,
            down: SHOAL | EMPTY,
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

            up: SHOAL | EMPTY,
            down: GROUND,
            left: GROUND,
            right: GROUND,

            up_left: ANY,
            up_right: ANY,
            down_left: ANY,
            down_right: ANY,
        },

        TerrainRule {
            tile_x: x + 0,
            tile_y: y + 3,

            up: GROUND,
            down: GROUND,
            left: GROUND,
            right: SHOAL | EMPTY,

            up_left: ANY,
            up_right: ANY,
            down_left: ANY,
            down_right: ANY,
        },

        TerrainRule {
            tile_x: x + 1,
            tile_y: y + 3,

            up: GROUND,
            down: GROUND,
            left: SHOAL | EMPTY,
            right: SHOAL | EMPTY,

            up_left: ANY,
            up_right: ANY,
            down_left: ANY,
            down_right: ANY,
        },

        TerrainRule {
            tile_x: x + 2,
            tile_y: y + 3,

            up: GROUND,
            down: GROUND,
            left: SHOAL | EMPTY,
            right: GROUND,

            up_left: ANY,
            up_right: ANY,
            down_left: ANY,
            down_right: ANY,
        },


        // L shapes (shoal)
        TerrainRule {
            tile_x: x + 0,
            tile_y: y + 0,

            up: GROUND,
            down: SHOAL,
            left: GROUND,
            right: SHOAL,

            up_left: ANY,
            up_right: ANY,
            down_left: ANY,
            down_right: WATER,
        },

        TerrainRule {
            tile_x: x + 2,
            tile_y: y + 0,

            up: GROUND,
            down: SHOAL,
            left: SHOAL,
            right: GROUND,

            up_left: ANY,
            up_right: ANY,
            down_left: WATER,
            down_right: ANY,
        },

        TerrainRule {
            tile_x: x + 2,
            tile_y: y + 2,

            up: SHOAL,
            down: GROUND,
            left: SHOAL,
            right: GROUND,

            up_left: WATER,
            up_right: ANY,
            down_left: ANY,
            down_right: ANY,
        },

        TerrainRule {
            tile_x: x + 0,
            tile_y: y + 2,

            up: SHOAL,
            down: GROUND,
            left: GROUND,
            right: SHOAL,

            up_left: ANY,
            up_right: WATER,
            down_left: ANY,
            down_right: ANY,
        },


        // L shapes (1 ocean)
        TerrainRule {
            tile_x: x + 7,
            tile_y: y + 0,

            up: GROUND,
            down: SHOAL,
            left: GROUND,
            right: SEA,

            up_left: ANY,
            up_right: ANY,
            down_left: ANY,
            down_right: WATER,
        },

        TerrainRule {
            tile_x: x + 8,
            tile_y: y + 0,

            up: GROUND,
            down: SEA,
            left: SHOAL,
            right: GROUND,

            up_left: ANY,
            up_right: ANY,
            down_left: WATER,
            down_right: ANY,
        },

        TerrainRule {
            tile_x: x + 8,
            tile_y: y + 1,

            up: SHOAL,
            down: GROUND,
            left: SEA,
            right: GROUND,

            up_left: WATER,
            up_right: ANY,
            down_left: ANY,
            down_right: ANY,
        },

        TerrainRule {
            tile_x: x + 7,
            tile_y: y + 1,

            up: SHOAL,
            down: GROUND,
            left: GROUND,
            right: SEA,

            up_left: ANY,
            up_right: WATER,
            down_left: ANY,
            down_right: ANY,
        },

        TerrainRule {
            tile_x: x + 9,
            tile_y: y + 0,

            up: GROUND,
            down: SEA,
            left: GROUND,
            right: SHOAL,

            up_left: ANY,
            up_right: ANY,
            down_left: ANY,
            down_right: WATER,
        },

        TerrainRule {
            tile_x: x + 10,
            tile_y: y + 0,

            up: GROUND,
            down: SHOAL,
            left: SEA,
            right: GROUND,

            up_left: ANY,
            up_right: ANY,
            down_left: WATER,
            down_right: ANY,
        },

        TerrainRule {
            tile_x: x + 10,
            tile_y: y + 1,

            up: SEA,
            down: GROUND,
            left: SHOAL,
            right: GROUND,

            up_left: WATER,
            up_right: ANY,
            down_left: ANY,
            down_right: ANY,
        },

        TerrainRule {
            tile_x: x + 9,
            tile_y: y + 1,

            up: SEA,
            down: GROUND,
            left: GROUND,
            right: SHOAL,

            up_left: ANY,
            up_right: WATER,
            down_left: ANY,
            down_right: ANY,
        },


        // L shapes (2 ocean)
        TerrainRule {
            tile_x: x + 4,
            tile_y: y + 0,

            up: GROUND,
            down: SEA,
            left: GROUND,
            right: SEA,

            up_left: ANY,
            up_right: ANY,
            down_left: ANY,
            down_right: WATER,
        },

        TerrainRule {
            tile_x: x + 6,
            tile_y: y + 0,

            up: GROUND,
            down: SEA,
            left: SEA,
            right: GROUND,

            up_left: ANY,
            up_right: ANY,
            down_left: WATER,
            down_right: ANY,
        },

        TerrainRule {
            tile_x: x + 6,
            tile_y: y + 2,

            up: SEA,
            down: GROUND,
            left: SEA,
            right: GROUND,

            up_left: WATER,
            up_right: ANY,
            down_left: ANY,
            down_right: ANY,
        },

        TerrainRule {
            tile_x: x + 4,
            tile_y: y + 2,

            up: SEA,
            down: GROUND,
            left: GROUND,
            right: SEA,

            up_left: ANY,
            up_right: WATER,
            down_left: ANY,
            down_right: ANY,
        },


        // T shapes (shoal)
        TerrainRule {
            tile_x: x + 1,
            tile_y: y + 0,

            up: GROUND,
            down: WATER,
            left: SHOAL,
            right: SHOAL,

            up_left: ANY,
            up_right: ANY,
            down_left: WATER,
            down_right: WATER,
        },

        TerrainRule {
            tile_x: x + 2,
            tile_y: y + 1,

            up: SHOAL,
            down: SHOAL,
            left: WATER,
            right: GROUND,

            up_left: WATER,
            up_right: ANY,
            down_left: WATER,
            down_right: ANY,
        },

        TerrainRule {
            tile_x: x + 0,
            tile_y: y + 1,

            up: SHOAL,
            down: SHOAL,
            left: GROUND,
            right: WATER,

            up_left: ANY,
            up_right: WATER,
            down_left: ANY,
            down_right: WATER,
        },

        TerrainRule {
            tile_x: x + 1,
            tile_y: y + 2,

            up: WATER,
            down: GROUND,
            left: SHOAL,
            right: SHOAL,

            up_left: WATER,
            up_right: WATER,
            down_left: ANY,
            down_right: ANY,
        },


        // T shapes (1 ocean)
        TerrainRule {
            tile_x: x + 11,
            tile_y: y + 0,

            up: GROUND,
            down: WATER,
            left: SHOAL,
            right: SEA,

            up_left: ANY,
            up_right: ANY,
            down_left: WATER,
            down_right: WATER,
        },

        TerrainRule {
            tile_x: x + 12,
            tile_y: y + 0,

            up: GROUND,
            down: WATER,
            left: SEA,
            right: SHOAL,

            up_left: ANY,
            up_right: ANY,
            down_left: WATER,
            down_right: WATER,
        },

        TerrainRule {
            tile_x: x + 14,
            tile_y: y + 0,

            up: SEA,
            down: SHOAL,
            left: WATER,
            right: GROUND,

            up_left: WATER,
            up_right: ANY,
            down_left: WATER,
            down_right: ANY,
        },

        TerrainRule {
            tile_x: x + 14,
            tile_y: y + 1,

            up: SHOAL,
            down: SEA,
            left: WATER,
            right: GROUND,

            up_left: WATER,
            up_right: ANY,
            down_left: WATER,
            down_right: ANY,
        },

        TerrainRule {
            tile_x: x + 13,
            tile_y: y + 0,

            up: SEA,
            down: SHOAL,
            left: GROUND,
            right: WATER,

            up_left: ANY,
            up_right: WATER,
            down_left: ANY,
            down_right: WATER,
        },

        TerrainRule {
            tile_x: x + 13,
            tile_y: y + 1,

            up: SHOAL,
            down: SEA,
            left: GROUND,
            right: WATER,

            up_left: ANY,
            up_right: WATER,
            down_left: ANY,
            down_right: WATER,
        },

        TerrainRule {
            tile_x: x + 11,
            tile_y: y + 1,

            up: WATER,
            down: GROUND,
            left: SHOAL,
            right: SEA,

            up_left: WATER,
            up_right: WATER,
            down_left: ANY,
            down_right: ANY,
        },

        TerrainRule {
            tile_x: x + 12,
            tile_y: y + 1,

            up: WATER,
            down: GROUND,
            left: SEA,
            right: SHOAL,

            up_left: WATER,
            up_right: WATER,
            down_left: ANY,
            down_right: ANY,
        },


        // T shapes (2 ocean)
        TerrainRule {
            tile_x: x + 5,
            tile_y: y + 0,

            up: GROUND,
            down: WATER,
            left: SEA,
            right: SEA,

            up_left: ANY,
            up_right: ANY,
            down_left: WATER,
            down_right: WATER,
        },

        TerrainRule {
            tile_x: x + 6,
            tile_y: y + 1,

            up: SEA,
            down: SEA,
            left: WATER,
            right: GROUND,

            up_left: WATER,
            up_right: ANY,
            down_left: WATER,
            down_right: ANY,
        },

        TerrainRule {
            tile_x: x + 4,
            tile_y: y + 1,

            up: SEA,
            down: SEA,
            left: GROUND,
            right: WATER,

            up_left: ANY,
            up_right: WATER,
            down_left: ANY,
            down_right: WATER,
        },

        TerrainRule {
            tile_x: x + 5,
            tile_y: y + 2,

            up: WATER,
            down: GROUND,
            left: SEA,
            right: SEA,

            up_left: WATER,
            up_right: WATER,
            down_left: ANY,
            down_right: ANY,
        },
    ].into_iter()
}
