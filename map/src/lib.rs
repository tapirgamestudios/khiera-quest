#![no_std]
use alloc::vec::Vec;
use spiral::SpiralIterator;
use util::Collider;

extern crate alloc;

mod spiral;

mod map {
    use agb_fixnum::Vector2D;
    use util::*;

    include!(concat!(env!("OUT_DIR"), "/map.rs"));
}

fn shell_size_iterator() -> impl Iterator<Item = usize> {
    let mut n = 3;
    let mut previous = 0;

    core::iter::from_fn(move || {
        let this = n * n - previous;
        previous = n * n;
        n += 2;
        Some(this)
    })
}

pub fn get_nearby(x: i32, y: i32) -> Vec<&'static Collider> {
    let x = x / map::BOX_SIZE;
    let y = y / map::BOX_SIZE;

    let mut spiral_iterator = SpiralIterator::new((x, y));
    let mut shell_size_iterator = shell_size_iterator();

    let mut colliders = Vec::new();

    while colliders.is_empty() {
        for (x, y) in (&mut spiral_iterator).take(shell_size_iterator.next().unwrap()) {
            if let Some(&region_colliders) = map::NEARBY_COLLIDERS.get(&[x, y]) {
                colliders.extend_from_slice(region_colliders);
            }
        }
    }

    colliders
}

static ALL_TRANSPARENT: &[u16] = &[(1 << 10) - 1; 64];

pub fn get_tile_chunk(x: i32, y: i32) -> &'static [u16] {
    match map::MAP_TILES.get(&[x, y]) {
        Some(tiles) => tiles,
        None => ALL_TRANSPARENT,
    }
}
