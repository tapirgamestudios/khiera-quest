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

    for (chunk_pos, chunk) in map_tiles.chunks() {
        let mut chunk_data = vec![];

        for y in 0..ChunkData::HEIGHT as i32 {
            for x in 0..ChunkData::WIDTH as i32 {
                if let Some(tile) = chunk.get_tile(x, y) {
                    chunk_data.push(tile.id() as u16);
                } else {
                    chunk_data.push(u16::MAX);
                }
            }
        }

        tiles.insert(chunk_pos, chunk_data);
    }

    tiles
}
