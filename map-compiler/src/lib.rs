use std::{error::Error, path::Path};

use collider_extract::assemble_colliders;
use quote::quote;
use tiled::Loader;

mod collider_extract;
mod maptile_extract;

mod spiral;

pub fn compile_map(path: impl AsRef<Path>) -> Result<String, Box<dyn Error>> {
    let mut loader = Loader::new();
    let map = loader.load_tmx_map(path)?;

    let tiles = maptile_extract::extract_tiles(&map);

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

    let maptile_phf_code = maptile_phf.build();
    Ok(format!(
        "{}\n\n{}{};",
        assemble_colliders(&map),
        quote! {
            pub static MAP_TILES: phf::Map<[i32; 2], &'static [super::MapTileSetting]> =
        },
        maptile_phf_code
    ))
}
