use std::{error::Error, path::Path};

use agb_fixnum::Vector2D;
use proc_macro2::TokenStream;
use quote::quote;
use tiled::Loader;
use util::{ColliderKind, Number};

mod collider_extract;
mod maptile_extract;

pub fn compile_map(path: impl AsRef<Path>) -> Result<String, Box<dyn Error>> {
    let mut loader = Loader::new();
    let map = loader.load_tmx_map(path)?;

    let colliders = collider_extract::extract_colliders(&map);
    let spacial_colliders = collider_extract::spacial_colliders(&colliders);

    let tiles = maptile_extract::extract_tiles(&map);

    let colliders_quote = colliders.iter().map(|x| {
        fn quote_vec(vector: Vector2D<Number>) -> TokenStream {
            let x = vector.x.to_raw();
            let y = vector.y.to_raw();

            quote! {
                Vector2D::new(Number::from_raw(#x), Number::from_raw(#y))
            }
        }

        let kind = match &x.kind {
            ColliderKind::Circle(c) => {
                let position = quote_vec(c.position);
                let r = c.radius.to_raw();
                quote! {
                    ColliderKind::Circle(Circle {
                        position: #position,
                        radius: Number::from_raw(#r),
                    })
                }
            }
            ColliderKind::Line(line) => {
                let start = quote_vec(line.start);
                let end = quote_vec(line.end);
                let normal = quote_vec(line.normal);

                let length = line.length.to_raw();

                quote! {ColliderKind::Line(Line {
                    start: #start,
                    end: #end,
                    normal: #normal,
                    length: Number::from_raw(#length),
                })}
            }
            ColliderKind::Arc(s) => {
                let center = quote_vec(s.circle.position);
                let r = s.circle.radius.to_raw();
                let circle = quote! {
                    Circle {
                        position: #center,
                        radius: Number::from_raw(#r),
                    }
                };

                let start_pos = quote_vec(s.start_pos);
                let end_pos = quote_vec(s.end_pos);

                quote! {
                    ColliderKind::Arc(Arc {
                        circle: #circle,
                        start_pos: #start_pos,
                        end_pos: #end_pos,
                    })
                }
            }
        };
        let gravitational = x.gravitational;
        quote! {
            Collider {
                kind: #kind,
                gravitational: #gravitational
            }
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
            .map(|tile_setting| {
                if tile_setting.tile_id == u16::MAX {
                    quote!(super::BLANK_TILE)
                } else {
                    let tile_id = tile_setting.tile_id;
                    let hflip = tile_setting.hflip;
                    let vflip = tile_setting.vflip;
                    quote!(
                        super::MapTileSetting {
                            tile_id: #tile_id,
                            hflip: #hflip,
                            vflip: #vflip,
                        }
                    )
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
            pub static MAP_TILES: phf::Map<[i32; 2], &'static [super::MapTileSetting]> =
        },
        maptile_phf_code
    ))
}

const BOX_SIZE: i32 = 512;
