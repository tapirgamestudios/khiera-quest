#![no_std]
#![feature(int_roundings)]
use agb_fixnum::Vector2D;
use util::{Collider, Number, ScrollStop};

mod map {
    use super::*;
    use agb_fixnum::Vector2D;
    use util::*;

    include!(concat!(env!("OUT_DIR"), "/map.rs"));
}

pub use map::{CAMERA_START, START_POINT};

pub fn get_recovery_point(position: Vector2D<Number>) -> Vector2D<Number> {
    map::RECOVERY_POINTS
        .iter()
        .copied()
        .min_by_key(|&x| (x - position).magnitude_squared())
        .unwrap()
}

pub fn get_nearby(x: i32, y: i32) -> &'static [&'static Collider] {
    let x = x.div_floor(map::BOX_SIZE);
    let y = y.div_floor(map::BOX_SIZE);

    map::NEARBY_COLLIDERS
        .get(&[x, y])
        .copied()
        .unwrap_or_default()
}

pub fn get_scroll_stop(x: i32, y: i32) -> Option<&'static ScrollStop> {
    let x = x.div_floor(map::SCROLL_STOP_BOX);
    let y = y.div_floor(map::SCROLL_STOP_BOX);

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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PowerUpKind {
    JumpBoost,
    SpeedBoost,
}

pub struct PowerUp {
    pub location: Vector2D<Number>,
    pub kind: PowerUpKind,
}

pub use map::POWER_UPS;
