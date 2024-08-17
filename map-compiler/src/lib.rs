use std::{collections::HashMap, error::Error, path::Path};

use proc_macro2::TokenStream;
use quote::quote;
use tiled::{Loader, Map};
use util::{Circle, Collider, Number};

pub fn compile_map(path: impl AsRef<Path>) -> Result<String, Box<dyn Error>> {
    let mut loader = Loader::new();
    let map = loader.load_tmx_map(path)?;
    let colliders = extract_colliders(&map);
    let spacial_colliders = spacial_colliders(&colliders);

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
        Collider::Line(_) => todo!(),
    });

    let mut phf = phf_codegen::Map::new();

    for (key, colliders) in spacial_colliders.iter() {
        let x = key.0;
        let y = key.1;
        let colliders = colliders.iter().map(|idx| quote! { &COLLIDERS [#idx] });
        let entry = quote! {&[#(#colliders),*]}.to_string();
        phf.entry([x, y], &entry);
    }

    let build = phf.build();
    Ok(format!(
        "{}{};",
        quote! {
            pub const BOX_SIZE: i32 = #BOX_SIZE;

            static COLLIDERS: &[Collider] = &[#(#colliders_quote),*];

            pub static NEARBY_COLLIDERS: phf::Map<[i32; 2], &'static [&'static Collider]> =
        },
        build
    ))
}

const BOX_SIZE: i32 = 128;

fn occupied_boxes<F>(collider: &Collider, mut f: F)
where
    F: FnMut(i32, i32),
{
    match collider {
        Collider::Circle(circle) => {
            let position = circle.position.floor();
            let radius = circle.radius.floor();

            let min_box_x = (position.x - radius) / BOX_SIZE;
            let max_box_x = (position.x + radius + BOX_SIZE - 1) / BOX_SIZE;
            let min_box_y = (position.y - radius) / BOX_SIZE;
            let max_box_y = (position.y + radius + BOX_SIZE - 1) / BOX_SIZE;

            let in_circle = |x: i32, y: i32| {
                (position - (x, y).into()).magnitude_squared() <= (radius + 8) * (radius + 8)
            };
            for x in min_box_x..=max_box_x {
                for y in min_box_y..=max_box_y {
                    if in_circle(x * BOX_SIZE, y * BOX_SIZE)
                        || in_circle((x + 1) * BOX_SIZE, (y + 1) * BOX_SIZE)
                        || ((x * BOX_SIZE)..((x + 1) * BOX_SIZE)).contains(&position.x)
                        || ((y * BOX_SIZE)..((y + 1) * BOX_SIZE)).contains(&position.y)
                    {
                        f(x, y)
                    }
                }
            }
        }
        Collider::Line(line) => {
            let start = line.start.floor();
            let end = line.end.floor();
            let mut current_box = (i32::MIN, i32::MIN);
            for (x, y) in bresenham::Bresenham::new(
                (start.x as isize, start.y as isize),
                (end.x as isize, end.y as isize),
            ) {
                let this_box = (x as i32 / BOX_SIZE, y as i32 / BOX_SIZE);
                if current_box != this_box {
                    f(this_box.0, this_box.1);
                    current_box = this_box;
                }
            }
        }
    }
}

fn spacial_colliders(colliders: &[Collider]) -> HashMap<(i32, i32), Vec<usize>> {
    let mut hs: HashMap<(i32, i32), Vec<usize>> = HashMap::new();

    for (idx, collider) in colliders.iter().enumerate() {
        occupied_boxes(collider, |x, y| {
            hs.entry((x, y)).or_default().push(idx);
        });
    }

    hs
}

fn extract_colliders(map: &Map) -> Vec<Collider> {
    let mut colliders = Vec::new();

    let objects = map.layers().find_map(|x| x.as_object_layer()).unwrap();
    for object in objects.objects() {
        match &object.shape {
            tiled::ObjectShape::Rect { width, height } => todo!(),
            tiled::ObjectShape::Ellipse { width, height } => {
                assert_eq!(
                    width, height,
                    "width and height of ellipse must be the same, ie we must have a circle"
                );

                let position = (
                    Number::from_f32(object.x + *width / 2.),
                    Number::from_f32(object.y + *width / 2.),
                )
                    .into();

                colliders.push(Collider::Circle(Circle {
                    position,
                    radius: Number::from_f32(*width / 2.),
                }));
            }
            tiled::ObjectShape::Polyline { points } => todo!(),
            tiled::ObjectShape::Polygon { points } => todo!(),
            _ => unimplemented!("Use of unsupported shape, {:?}", object.shape),
        }
    }

    colliders
}
