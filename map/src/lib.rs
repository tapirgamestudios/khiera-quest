use util::Collider;

mod map {
    include!(concat!(env!("OUT_DIR"), "/map.rs"));
}

pub fn get_nearby(x: i32, y: i32) -> &'static [&'static Collider] {
    match map::NEARBY_COLLIDERS.get(&[x, y]) {
        Some(&x) => x,
        None => &[],
    }
}
