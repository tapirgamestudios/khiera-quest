use std::{error::Error, path::Path};

use quote::quote;
use tiled::Loader;
use util::Collider;

mod collider_extract;
mod maptile_extract;

pub fn compile_map(path: impl AsRef<Path>) -> Result<String, Box<dyn Error>> {
    let mut loader = Loader::new();
    let map = loader.load_tmx_map(path)?;

    let colliders = collider_extract::extract_colliders(&map);
    let spacial_colliders = collider_extract::spacial_colliders(&colliders);

    let tiles = maptile_extract::extract_tiles(&map);

    let colliders_quote = colliders.iter().map(|x| match x {
        Collider::Circle(c) => {
            let x = c.position.x.to_raw();
            let y = c.position.y.to_raw();
            let r = c.radius.to_raw();
            quote! {
                Collider::Circle(Circle {
                    position: Vector2D::new(Number::from_raw(#x), Number::from_raw(#y)),
                    radius: Number::from_raw(#r),
                })
            }
        }
        Collider::Line(line) => {
            let sx = line.start.x.to_raw();
            let sy = line.start.y.to_raw();
            let ex = line.end.x.to_raw();
            let ey = line.end.y.to_raw();

            let nx = line.normal.x.to_raw();
            let ny = line.normal.y.to_raw();

            quote! {Collider::Line(Line {
                start: Vector2D::new(Number::from_raw(#sx), Number::from_raw(#sy)),
                end: Vector2D::new(Number::from_raw(#ex), Number::from_raw(#ey)),
                normal: Vector2D::new(Number::from_raw(#nx), Number::from_raw(#ny)),
            })}
        }
    });

    let mut collider_phf = phf_codegen::Map::new();

    for (key, colliders) in spacial_colliders.iter() {
        let x = key.0;
        let y = key.1;
        let colliders = colliders.iter().map(|idx| quote! { &COLLIDERS [#idx] });
        let entry = quote! {&[#(#colliders),*]}.to_string();
        collider_phf.entry([x, y], &entry);
    }

    let mut maptile_phf = phf_codegen::Map::new();

    for (key, tiles) in tiles.iter() {
        let x = key.0;
        let y = key.1;

        let tile_settings: Vec<_> = tiles
            .iter()
            .map(|&tid| {
                if tid == u16::MAX {
                    const TRANSPARENT_TILE_INDEX: u16 = (1 << 10) - 1;
                    quote!(#TRANSPARENT_TILE_INDEX)
                } else {
                    quote!(#tid)
                }
            })
            .collect();

        maptile_phf.entry([x, y], &quote! { &[#(#tile_settings),*] }.to_string());
    }

    let collider_phf_code = collider_phf.build();
    let maptile_phf_code = maptile_phf.build();

    Ok(format!(
        "{}{};\n\n{}{};",
        quote! {
            pub const BOX_SIZE: i32 = #BOX_SIZE;

            static COLLIDERS: &[Collider] = &[#(#colliders_quote),*];

            pub static NEARBY_COLLIDERS: phf::Map<[i32; 2], &'static [&'static Collider]> =
        },
        collider_phf_code,
        quote! {
            pub static MAP_TILES: phf::Map<[i32; 2], &'static [u16]> =
        },
        maptile_phf_code
    ))
}

const BOX_SIZE: i32 = 128;
