use super::{TerrainRule, TerrainFlag};

const EMPTY: TerrainFlag = TerrainFlag::EMPTY;
const BRIDGE: TerrainFlag = TerrainFlag::BRIDGE;
const GROUND: TerrainFlag = TerrainFlag::GROUND;
const SEA: TerrainFlag = TerrainFlag::SEA;
const SHOAL: TerrainFlag = TerrainFlag::SHOAL;
const ANY: TerrainFlag = TerrainFlag::ANY;
const WATER: TerrainFlag = TerrainFlag::WATER;

const BORDER: TerrainFlag = SEA.or(SHOAL).or(BRIDGE);

pub(crate) fn rules() -> impl Iterator<Item = TerrainRule> {
    // Starting coordinate for the sea tiles
    let x: u32 = 23;
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
            down: SEA | EMPTY,
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

            up: SEA | EMPTY,
            down: SEA | EMPTY,
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

            up: SEA | EMPTY,
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
            right: SEA | EMPTY,

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
            left: SEA | EMPTY,
            right: SEA | EMPTY,

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
            left: SEA | EMPTY,
            right: GROUND,

            up_left: ANY,
            up_right: ANY,
            down_left: ANY,
            down_right: ANY,
        },


        // L shapes
        TerrainRule {
            tile_x: x + 0,
            tile_y: y + 0,

            up: GROUND,
            down: BORDER,
            left: GROUND,
            right: BORDER,

            up_left: ANY,
            up_right: ANY,
            down_left: ANY,
            down_right: WATER,
        },

        TerrainRule {
            tile_x: x + 2,
            tile_y: y + 0,

            up: GROUND,
            down: BORDER,
            left: BORDER,
            right: GROUND,

            up_left: ANY,
            up_right: ANY,
            down_left: WATER,
            down_right: ANY,
        },

        TerrainRule {
            tile_x: x + 0,
            tile_y: y + 2,

            up: BORDER,
            down: GROUND,
            left: GROUND,
            right: BORDER,

            up_left: ANY,
            up_right: WATER,
            down_left: ANY,
            down_right: ANY,
        },

        TerrainRule {
            tile_x: x + 2,
            tile_y: y + 2,

            up: BORDER,
            down: GROUND,
            left: BORDER,
            right: GROUND,

            up_left: WATER,
            up_right: ANY,
            down_left: ANY,
            down_right: ANY,
        },


        // L shapes (corner)
        TerrainRule {
            tile_x: x + 4,
            tile_y: y + 0,

            up: GROUND,
            down: BORDER,
            left: GROUND,
            right: BORDER,

            up_left: ANY,
            up_right: ANY,
            down_left: ANY,
            down_right: GROUND,
        },

        TerrainRule {
            tile_x: x + 6,
            tile_y: y + 0,

            up: GROUND,
            down: BORDER,
            left: BORDER,
            right: GROUND,

            up_left: ANY,
            up_right: ANY,
            down_left: GROUND,
            down_right: ANY,
        },

        TerrainRule {
            tile_x: x + 4,
            tile_y: y + 2,

            up: BORDER,
            down: GROUND,
            left: GROUND,
            right: BORDER,

            up_left: ANY,
            up_right: GROUND,
            down_left: ANY,
            down_right: ANY,
        },

        TerrainRule {
            tile_x: x + 6,
            tile_y: y + 2,

            up: BORDER,
            down: GROUND,
            left: BORDER,
            right: GROUND,

            up_left: GROUND,
            up_right: ANY,
            down_left: ANY,
            down_right: ANY,
        },


        // T shapes
        TerrainRule {
            tile_x: x + 1,
            tile_y: y + 0,

            up: GROUND,
            down: WATER,
            left: BORDER,
            right: BORDER,

            up_left: ANY,
            up_right: ANY,
            down_left: WATER,
            down_right: WATER,
        },

        TerrainRule {
            tile_x: x + 2,
            tile_y: y + 1,

            up: BORDER,
            down: BORDER,
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

            up: BORDER,
            down: BORDER,
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
            left: BORDER,
            right: BORDER,

            up_left: WATER,
            up_right: WATER,
            down_left: ANY,
            down_right: ANY,
        },


        // T shapes (corner)
        TerrainRule {
            tile_x: x + 5,
            tile_y: y + 0,

            up: GROUND,
            down: WATER,
            left: BORDER,
            right: BORDER,

            up_left: ANY,
            up_right: ANY,
            down_left: GROUND,
            down_right: GROUND,
        },

        TerrainRule {
            tile_x: x + 6,
            tile_y: y + 1,

            up: BORDER,
            down: BORDER,
            left: WATER,
            right: GROUND,

            up_left: GROUND,
            up_right: ANY,
            down_left: GROUND,
            down_right: ANY,
        },

        TerrainRule {
            tile_x: x + 4,
            tile_y: y + 1,

            up: BORDER,
            down: BORDER,
            left: GROUND,
            right: WATER,

            up_left: ANY,
            up_right: GROUND,
            down_left: ANY,
            down_right: GROUND,
        },

        TerrainRule {
            tile_x: x + 5,
            tile_y: y + 2,

            up: WATER,
            down: GROUND,
            left: BORDER,
            right: BORDER,

            up_left: GROUND,
            up_right: GROUND,
            down_left: ANY,
            down_right: ANY,
        },

        TerrainRule {
            tile_x: x + 9,
            tile_y: y + 0,

            up: GROUND,
            down: WATER,
            left: BORDER,
            right: BORDER,

            up_left: ANY,
            up_right: ANY,
            down_left: WATER,
            down_right: GROUND,
        },

        TerrainRule {
            tile_x: x + 10,
            tile_y: y + 0,

            up: GROUND,
            down: WATER,
            left: BORDER,
            right: BORDER,

            up_left: ANY,
            up_right: ANY,
            down_left: GROUND,
            down_right: WATER,
        },

        TerrainRule {
            tile_x: x + 8,
            tile_y: y + 0,

            up: BORDER,
            down: BORDER,
            left: WATER,
            right: GROUND,

            up_left: WATER,
            up_right: ANY,
            down_left: GROUND,
            down_right: ANY,
        },

        TerrainRule {
            tile_x: x + 8,
            tile_y: y + 1,

            up: BORDER,
            down: BORDER,
            left: WATER,
            right: GROUND,

            up_left: GROUND,
            up_right: ANY,
            down_left: WATER,
            down_right: ANY,
        },

        TerrainRule {
            tile_x: x + 7,
            tile_y: y + 0,

            up: BORDER,
            down: BORDER,
            left: GROUND,
            right: WATER,

            up_left: ANY,
            up_right: WATER,
            down_left: ANY,
            down_right: GROUND,
        },

        TerrainRule {
            tile_x: x + 7,
            tile_y: y + 1,

            up: BORDER,
            down: BORDER,
            left: GROUND,
            right: WATER,

            up_left: ANY,
            up_right: GROUND,
            down_left: ANY,
            down_right: WATER,
        },

        TerrainRule {
            tile_x: x + 9,
            tile_y: y + 1,

            up: WATER,
            down: GROUND,
            left: BORDER,
            right: BORDER,

            up_left: WATER,
            up_right: GROUND,
            down_left: ANY,
            down_right: ANY,
        },

        TerrainRule {
            tile_x: x + 10,
            tile_y: y + 1,

            up: WATER,
            down: GROUND,
            left: BORDER,
            right: BORDER,

            up_left: GROUND,
            up_right: WATER,
            down_left: ANY,
            down_right: ANY,
        },


        // All
        TerrainRule {
            tile_x: x + 1,
            tile_y: y + 1,

            up: WATER,
            down: WATER,
            left: WATER,
            right: WATER,

            up_left: WATER,
            up_right: WATER,
            down_left: WATER,
            down_right: WATER,
        },


        // All (1 corner)
        TerrainRule {
            tile_x: x + 15,
            tile_y: y + 0,

            up: WATER,
            down: WATER,
            left: WATER,
            right: WATER,

            up_left: WATER,
            up_right: WATER,
            down_left: WATER,
            down_right: GROUND,
        },

        TerrainRule {
            tile_x: x + 17,
            tile_y: y + 0,

            up: WATER,
            down: WATER,
            left: WATER,
            right: WATER,

            up_left: WATER,
            up_right: WATER,
            down_left: GROUND,
            down_right: WATER,
        },

        TerrainRule {
            tile_x: x + 17,
            tile_y: y + 1,

            up: WATER,
            down: WATER,
            left: WATER,
            right: WATER,

            up_left: GROUND,
            up_right: WATER,
            down_left: WATER,
            down_right: WATER,
        },

        TerrainRule {
            tile_x: x + 15,
            tile_y: y + 1,

            up: WATER,
            down: WATER,
            left: WATER,
            right: WATER,

            up_left: WATER,
            up_right: GROUND,
            down_left: WATER,
            down_right: WATER,
        },


        // All (2 corner)
        TerrainRule {
            tile_x: x + 13,
            tile_y: y + 0,

            up: WATER,
            down: WATER,
            left: WATER,
            right: WATER,

            up_left: WATER,
            up_right: GROUND,
            down_left: WATER,
            down_right: GROUND,
        },

        TerrainRule {
            tile_x: x + 14,
            tile_y: y + 0,

            up: WATER,
            down: WATER,
            left: WATER,
            right: WATER,

            up_left: GROUND,
            up_right: WATER,
            down_left: GROUND,
            down_right: WATER,
        },

        TerrainRule {
            tile_x: x + 16,
            tile_y: y + 0,

            up: WATER,
            down: WATER,
            left: WATER,
            right: WATER,

            up_left: WATER,
            up_right: WATER,
            down_left: GROUND,
            down_right: GROUND,
        },

        TerrainRule {
            tile_x: x + 16,
            tile_y: y + 1,

            up: WATER,
            down: WATER,
            left: WATER,
            right: WATER,

            up_left: GROUND,
            up_right: GROUND,
            down_left: WATER,
            down_right: WATER,
        },

        TerrainRule {
            tile_x: x + 13,
            tile_y: y + 1,

            up: WATER,
            down: WATER,
            left: WATER,
            right: WATER,

            up_left: WATER,
            up_right: GROUND,
            down_left: GROUND,
            down_right: WATER,
        },

        TerrainRule {
            tile_x: x + 14,
            tile_y: y + 1,

            up: WATER,
            down: WATER,
            left: WATER,
            right: WATER,

            up_left: GROUND,
            up_right: WATER,
            down_left: WATER,
            down_right: GROUND,
        },


        // All (3 corner)
        TerrainRule {
            tile_x: x + 11,
            tile_y: y + 0,

            up: WATER,
            down: WATER,
            left: WATER,
            right: WATER,

            up_left: GROUND,
            up_right: GROUND,
            down_left: WATER,
            down_right: GROUND,
        },

        TerrainRule {
            tile_x: x + 12,
            tile_y: y + 0,

            up: WATER,
            down: WATER,
            left: WATER,
            right: WATER,

            up_left: GROUND,
            up_right: GROUND,
            down_left: GROUND,
            down_right: WATER,
        },

        TerrainRule {
            tile_x: x + 11,
            tile_y: y + 1,

            up: WATER,
            down: WATER,
            left: WATER,
            right: WATER,

            up_left: WATER,
            up_right: GROUND,
            down_left: GROUND,
            down_right: GROUND,
        },

        TerrainRule {
            tile_x: x + 12,
            tile_y: y + 1,

            up: WATER,
            down: WATER,
            left: WATER,
            right: WATER,

            up_left: GROUND,
            up_right: WATER,
            down_left: GROUND,
            down_right: GROUND,
        },


        // All (4 corner)
        TerrainRule {
            tile_x: x + 5,
            tile_y: y + 1,

            up: WATER,
            down: WATER,
            left: WATER,
            right: WATER,

            up_left: GROUND,
            up_right: GROUND,
            down_left: GROUND,
            down_right: GROUND,
        },
    ].into_iter()
}
