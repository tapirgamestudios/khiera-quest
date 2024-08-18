use std::collections::HashMap;

use tiled::{ChunkData, Map, TileLayer};

pub fn extract_tiles(map: &Map) -> HashMap<(i32, i32), Vec<u16>> {
    // if this changes, then 😭
    assert_eq!(ChunkData::HEIGHT, 16);
    assert_eq!(ChunkData::WIDTH, 16);
    assert_eq!(ChunkData::TILE_COUNT, 256);

    let mut tiles = HashMap::new();

    let Some(TileLayer::Infinite(map_tiles)) = map.layers().find_map(|x| x.as_tile_layer()) else {
        panic!("May layer not valid")
    };

    for ((super_chunk_x, super_chunk_y), chunk) in map_tiles.chunks() {
        // internally split these into 8x8 chunks
        for chunk_y in 0..2 {
            for chunk_x in 0..2 {
                let mut chunk_data = vec![];

                for y in chunk_y * 8..(chunk_y + 1) * 8 {
                    for x in chunk_x * 8..(chunk_x + 1) * 8 {
                        if let Some(tile) = chunk.get_tile(x, y) {
                            chunk_data.push(tile.id() as u16);
                        } else {
                            chunk_data.push(u16::MAX);
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
