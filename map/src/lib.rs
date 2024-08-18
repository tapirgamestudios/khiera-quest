#![no_std]
use util::{Collider, ScrollStop};

mod map {
    use agb_fixnum::Vector2D;
    use util::*;

    include!(concat!(env!("OUT_DIR"), "/map.rs"));
}

pub use map::{CAMERA_START, START_POINT};

pub fn get_nearby(x: i32, y: i32) -> &'static [&'static Collider] {
    let x = x / map::BOX_SIZE;
    let y = y / map::BOX_SIZE;

    map::NEARBY_COLLIDERS
        .get(&[x, y])
        .copied()
        .unwrap_or_default()
}

pub fn get_scroll_stop(x: i32, y: i32) -> Option<&'static ScrollStop> {
    let x = x / map::SCROLL_STOP_BOX;
    let y = y / map::SCROLL_STOP_BOX;

    map::SCROLL_STOPS.get(&[x, y])
}

#[derive(Clone, Copy)]
pub enum MapTileSet {
    Planets,
    Planets2,
    Platforms,
}

#[derive(Copy, Clone)]
pub struct MapTileSetting {
    pub tile_id: u16,
    pub hflip: bool,
    pub vflip: bool,
    pub map_tile_set: MapTileSet,
}

pub const BLANK_TILE: MapTileSetting = MapTileSetting {
    tile_id: (1 << 10) - 1,
    hflip: false,
    vflip: false,
    map_tile_set: MapTileSet::Planets,
};

static ALL_TRANSPARENT: &[MapTileSetting] = &[BLANK_TILE; 64];

pub fn get_planet_tile_chunk(x: i32, y: i32) -> &'static [MapTileSetting] {
    match map::PLANET_MAP_TILES.get(&[x, y]) {
        Some(tiles) => tiles,
        None => ALL_TRANSPARENT,
    }
}

pub fn get_platform_tile_chunk(x: i32, y: i32) -> &'static [MapTileSetting] {
    match map::PLATFORM_MAP_TILES.get(&[x, y]) {
        Some(tiles) => tiles,
        None => ALL_TRANSPARENT,
    }
}
