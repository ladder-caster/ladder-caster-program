use crate::{Tile, TileType};
use crate::utils::{MAX_LEVEL_0_BASED, MINIMUM_TILE_LIFE, RandomGenerator};

pub fn cycle_tile(tile: Option<Tile>, level: u8, rand: &mut RandomGenerator) -> Tile {
    match tile {
        None => {
            let random_life = rand.random_within_range::<u8, 1>(0, 3);

            Tile {
                tile_type: rand.random_enum_within_range::<TileType>(0, 2),
                life: MINIMUM_TILE_LIFE + random_life,
                is_first_time_spawning: true,
            }
        }
        Some(tile) => {
            //if resource tile, then return crafting / if crafting tile then return resource
            match tile.tile_type {
                TileType::Crafting | TileType::Legendary => {
                    let random_life = rand.random_within_range::<u8, 1>(0, 3);

                    Tile {
                        tile_type: rand.random_enum_within_range::<TileType>(0, 2),
                        life: MINIMUM_TILE_LIFE + random_life,
                        is_first_time_spawning: false,
                    }
                }
                _ => {
                    let mut _feature: TileType;

                    //Crafting tile level 30 are always legendary, if not they are normal crafting
                    if level == MAX_LEVEL_0_BASED || tile.is_first_time_spawning {
                        _feature = TileType::Legendary;
                    } else {
                        _feature = TileType::Crafting;
                    }

                    Tile {
                        tile_type: _feature,
                        life: 1,
                        is_first_time_spawning: false,
                    }
                }
            }
        }
    }
}

pub fn get_highest_level_and_column(map: &[[Option<Tile>; 3]; 30]) -> (u8, u8) {
    let mut highest_level: usize = 0;
    let mut highest_column: usize = 0;

    for i in 0..map.len() {
        for j in 0..map[i].len() {
            //Works because goes in order
            if map[i][j] != None {
                highest_level = i;
                highest_column = j;
            }
        }
    }

    (highest_level as u8, highest_column as u8)
}

pub fn get_current_tile(map: &[[Option<Tile>; 3]; 30], dest_level: u8, dest_column: u8) -> Option<&Tile> {
    match map.get(dest_level as usize) {
        Some(lvl) => match lvl.get(dest_column as usize) {
            Some(tile) => tile.as_ref(),
            None => None
        },
        None => None
    }
}

pub fn get_current_tile_feature(map: &[[Option<TileType>; 3]; 30], dest_level: u8, dest_column: u8) -> Option<&TileType> {
    match map.get(dest_level as usize) {
        Some(lvl) => match lvl.get(dest_column as usize) {
            Some(tile_type) => tile_type.as_ref(),
            None => None
        },
        None => None
    }
}