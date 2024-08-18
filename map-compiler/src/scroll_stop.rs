use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::quote;
use tiled::Map;
use util::{Number, ScrollStop};

const SCREEN_WIDTH: i32 = 240;
const SCREEN_HEIGHT: i32 = 160;

const SCROLL_BOX_SIZE: i32 = 128;

pub fn get_scroll_stops(map: &Map) -> String {
    let layer = map
        .layers()
        .filter(|x| x.name == "Scroll stops")
        .find_map(|x| x.as_object_layer())
        .unwrap();

    let lines: Vec<_> = layer
        .objects()
        .flat_map(|x| match &x.shape {
            tiled::ObjectShape::Polyline { points } => {
                let points: Vec<_> = points.iter().map(|r| (r.0 + x.x, r.1 + x.y)).collect();
                let lines: Vec<_> = points
                    .windows(2)
                    .map(|x| {
                        let [x, y] = x else { panic!() };
                        [*x, *y]
                    })
                    .collect();
                Some(lines)
            }
            _ => None,
        })
        .flatten()
        .collect();

    let mut map: HashMap<(i32, i32), ScrollStop> = HashMap::new();

    for line in lines {
        assert!(
            line[0].0 == line[1].0 || line[0].1 == line[1].1,
            "Scroll stops should be axis aligned"
        );

        if line[0].0 == line[1].0 {
            // x

            let direction = (line[0].1 - line[1].1).signum() as i32;

            let start = line[0].1.min(line[1].1) as i32;
            let end = line[0].1.max(line[1].1) as i32;

            let x = line[0].0 as i32 / SCROLL_BOX_SIZE;
            let direction_x = x + direction * 3;
            let start_x = x.min(direction_x);
            let end_x = x.max(direction_x);

            let start = start / SCROLL_BOX_SIZE;
            let end = end.div_ceil(SCROLL_BOX_SIZE);
            for y in start..end {
                for x in start_x..end_x {
                    let entry = map.entry((x, y)).or_default();
                    if direction > 0 {
                        entry.minimum_x = Some(Number::from_f32(line[0].0) + SCREEN_WIDTH / 2);
                    } else {
                        entry.maximum_x = Some(Number::from_f32(line[0].0) - SCREEN_WIDTH / 2);
                    }
                }
            }
        } else {
            // y
            let direction = (line[0].0 - line[1].0).signum() as i32;

            let start = line[0].0.min(line[1].0) as i32;
            let end = line[0].0.max(line[1].0) as i32;

            let x = line[0].1 as i32 / SCROLL_BOX_SIZE;
            let direction_x = x + direction.abs() * 3;
            let start_x = x.min(direction_x);
            let end_x = x.max(direction_x);

            let start = start / SCROLL_BOX_SIZE;
            let end = end.div_ceil(SCROLL_BOX_SIZE);
            for x in start..end {
                for y in start_x..end_x {
                    let entry = map.entry((x, y)).or_default();
                    if direction > 0 {
                        entry.maximum_y = Some(Number::from_f32(line[0].1) - SCREEN_HEIGHT / 2);
                    } else {
                        entry.minimum_y = Some(Number::from_f32(line[0].1) + SCREEN_HEIGHT / 2);
                    }
                }
            }
        }
    }

    let mut phf = phf_codegen::Map::new();

    for (coords, entry) in map.iter() {
        let min_x = optional_quote(entry.minimum_x);
        let max_x = optional_quote(entry.maximum_x);
        let min_y = optional_quote(entry.minimum_y);
        let max_y = optional_quote(entry.maximum_y);
        phf.entry(
            [coords.0, coords.1],
            &quote! {
                ScrollStop {
                    minimum_x: #min_x,
                    minimum_y: #min_y,
                    maximum_x: #max_x,
                    maximum_y: #max_y
                }
            }
            .to_string(),
        );
    }

    format!(
        "{}{}",
        quote! {
            pub const SCROLL_STOP_BOX: i32 = #SCROLL_BOX_SIZE;
            pub static SCROLL_STOPS: phf::Map<[i32; 2], ScrollStop> =
        },
        phf.build()
    )
}

fn optional_quote(a: Option<Number>) -> TokenStream {
    if let Some(a) = a {
        let a = a.to_raw();
        quote! {
            Some( Number::from_raw(#a) )
        }
    } else {
        quote! {None}
    }
}
