#![feature(int_roundings)]

use std::{error::Error, path::Path};

use collider_extract::assemble_colliders;
use proc_macro2::TokenStream;
use quote::quote;
use scroll_stop::get_scroll_stops;
use tiled::{Loader, Map, TileLayer};
use util::Number;

mod collider_extract;
mod maptile_extract;

mod scroll_stop;
mod spiral;

pub fn compile_map(path: impl AsRef<Path>) -> Result<String, Box<dyn Error>> {
    let mut loader = Loader::new();
    let map = loader.load_tmx_map(path)?;

    let planet_maptile_phf = tiles_for_layer(&map, "Planets");
    let platform_maptile_phf = tiles_for_layer(&map, "Platforms");

    let planet_maptile_phf_code = planet_maptile_phf.build();
    let platform_maptile_phf_code = platform_maptile_phf.build();
    Ok(format!(
        "{}\n\n{}{planet_maptile_phf_code};\n\n{}{platform_maptile_phf_code};\n\n{}\n\n{};\n{}",
        assemble_colliders(&map),
        quote! {
            pub static PLANET_MAP_TILES: phf::Map<[i32; 2], &'static [super::MapTileSetting]> =
        },
        quote! {
            pub static PLATFORM_MAP_TILES: phf::Map<[i32; 2], &'static [super::MapTileSetting]> =
        },
        get_start_point(&map),
        get_scroll_stops(&map),
        get_powerups(&map),
    ))
}

fn tiles_for_layer(map: &Map, name: &str) -> phf_codegen::Map<[i32; 2]> {
    let infinite_map = map
        .layers()
        .find_map(|layer| {
            if let Some(TileLayer::Infinite(infinite_layer)) = layer.as_tile_layer() {
                if layer.name == name {
                    return Some(infinite_layer);
                }
            }
            None
        })
        .expect("Could not find '{name}' map");

    let tiles = maptile_extract::extract_tiles(&infinite_map);

    let mut maptile_phf = phf_codegen::Map::new();

    for (key, tiles) in tiles {
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
                    let map_tile_set = match tile_setting.tileset {
                        maptile_extract::GameTileSet::Planets => quote!(super::MapTileSet::Planets),
                        maptile_extract::GameTileSet::Platforms => {
                            quote!(super::MapTileSet::Platforms)
                        }
                        maptile_extract::GameTileSet::Planets2 => {
                            quote!(super::MapTileSet::Planets2)
                        }
                    };

                    quote!(
                        super::MapTileSetting {
                            tile_id: #tile_id,
                            hflip: #hflip,
                            vflip: #vflip,
                            map_tile_set: #map_tile_set,
                        }
                    )
                }
            })
            .collect();

        maptile_phf.entry([x, y], &quote! { &[#(#tile_settings),*] }.to_string());
    }

    maptile_phf
}

fn get_start_point(map: &Map) -> TokenStream {
    let layer = map
        .layers()
        .filter(|x| x.name == "Start")
        .find_map(|x| x.as_object_layer())
        .unwrap();
    let start_object = layer.objects().find(|x| x.name == "PLAYER").unwrap();
    let camera = layer.objects().find(|x| x.name == "CAMERA").unwrap();

    let x = Number::from_f32(start_object.x).to_raw();
    let y = Number::from_f32(start_object.y).to_raw();

    let cx = Number::from_f32(camera.x).to_raw();
    let cy = Number::from_f32(camera.y).to_raw();

    quote! {
        pub const START_POINT: Vector2D<Number> = Vector2D::new(Number::from_raw(#x), Number::from_raw(#y));
        pub const CAMERA_START: Vector2D<Number> = Vector2D::new(Number::from_raw(#cx), Number::from_raw(#cy));

    }
}

fn get_powerups(map: &Map) -> TokenStream {
    let layer = map
        .layers()
        .find_map(|x| {
            if x.name == "Items" {
                x.as_object_layer()
            } else {
                None
            }
        })
        .unwrap();

    let powerups = layer.objects().map(|obj| {
        let x = Number::from_f32(obj.x).to_raw();
        let y = Number::from_f32(obj.y).to_raw();

        let powerup = match obj.name.as_str() {
            "Jump Boost" => quote! { PowerUpKind::JumpBoost },
            "Dash" => quote! { PowerUpKind::Dash },
            "Double Jump" => quote! { PowerUpKind::DoubleJump },
            boost_name => todo!("Unknown boost {boost_name}"),
        };

        quote! {
            PowerUp {
                kind: #powerup,
                location: Vector2D::new(Number::from_raw(#x), Number::from_raw(#y)),
            }
        }
    });

    quote! {
        pub static POWER_UPS: &[PowerUp] = &[#(#powerups),*];
    }
}
