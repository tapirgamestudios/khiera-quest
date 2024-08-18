use std::collections::HashMap;

use tiled::{ChunkData, InfiniteTileLayer, Map, TileLayer};

pub enum GameTileSet {
    Planets,
    Platforms,
}

pub struct TileSetting {
    pub tileset: GameTileSet,
    pub hflip: bool,
    pub vflip: bool,
    pub tile_id: u16,
}

pub fn extract_tiles(map_tiles: &InfiniteTileLayer) -> HashMap<(i32, i32), Vec<TileSetting>> {
    // if this changes, then 😭
    assert_eq!(ChunkData::HEIGHT, 16);
    assert_eq!(ChunkData::WIDTH, 16);
    assert_eq!(ChunkData::TILE_COUNT, 256);

    let mut tiles = HashMap::new();

    for ((super_chunk_x, super_chunk_y), chunk) in map_tiles.chunks() {
        // internally split these into 8x8 chunks
        for chunk_y in 0..2 {
            for chunk_x in 0..2 {
                let mut chunk_data = vec![];

                for y in chunk_y * 8..(chunk_y + 1) * 8 {
                    for x in chunk_x * 8..(chunk_x + 1) * 8 {
                        if let Some(tile) = chunk.get_tile(x, y) {
                            chunk_data.push(TileSetting {
                                tileset: match tile.get_tileset().name.as_str() {
                                    "planets" => GameTileSet::Planets,
                                    "platforms" => GameTileSet::Platforms,
                                    name => panic!("Unknown tile set {name}"),
                                },
                                tile_id: tile.id() as u16,
                                hflip: tile.flip_h,
                                vflip: tile.flip_v,
                            });
                        } else {
                            chunk_data.push(TileSetting {
                                tileset: GameTileSet::Planets,
                                tile_id: u16::MAX,
                                hflip: false,
                                vflip: false,
                            });
                        }
                    }
                }

                tiles.insert(
                    (super_chunk_x * 2 + chunk_x, super_chunk_y * 2 + chunk_y),
                    chunk_data,
                );
            }
        }
    }

    tiles
}
