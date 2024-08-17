#![no_std]
use util::Collider;

mod map {
    use agb_fixnum::Vector2D;
    use util::*;

    include!(concat!(env!("OUT_DIR"), "/map.rs"));
}

pub fn get_nearby(x: i32, y: i32) -> &'static [&'static Collider] {
    let x = x / map::BOX_SIZE;
    let y = y / map::BOX_SIZE;
    match map::NEARBY_COLLIDERS.get(&[x, y]) {
        Some(&x) => x,
        None => &[],
    }
}
