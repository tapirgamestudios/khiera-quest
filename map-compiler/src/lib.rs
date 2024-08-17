use std::{collections::HashMap, error::Error, path::Path};

use agb_fixnum::Vector2D;
use nalgebra::Vector2;
use quote::quote;
use tiled::{Loader, Map};
use util::{Circle, Collider, Line, Number};

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
            tiled::ObjectShape::Polygon { points } => {
                let origin = Vector2::new(object.x, object.y);
                let mut modified_points = Vec::new();

                let mut do_line_work = |a: (f32, f32), o: (f32, f32), b: (f32, f32)| {
                    let a = Vector2::new(a.0, a.1) + origin;
                    let o = Vector2::new(o.0, o.1) + origin;
                    let b = Vector2::new(b.0, b.1) + origin;

                    modified_points.push(rounded_line_collider(a, o, b, 10., &mut colliders));
                };

                do_line_work(points[points.len() - 1], points[0], points[1]);

                for x in points.windows(3) {
                    let [a, o, b] = x else { panic!() };
                    do_line_work(*a, *o, *b);
                }

                do_line_work(
                    points[points.len() - 2],
                    points[points.len() - 1],
                    points[0],
                );

                let mut current = modified_points[0].1;

                for (new_end, next_start) in modified_points.iter().skip(1) {
                    colliders.push(get_line_collider(current, *new_end));

                    current = *next_start;
                }

                colliders.push(get_line_collider(current, modified_points[0].0));
            }
            // tiled::ObjectShape::Polyline { points } | tiled::ObjectShape::Polygon { points } => {
            //     for x in points.windows(2) {
            //         let [start, end] = x else { panic!() };
            //         let start = (start.0 + object.x, start.1 + object.y);
            //         let end = (end.0 + object.x, end.1 + object.y);
            //         let vector = (end.0 - start.0, end.1 - start.1);
            //         let magnitude = (vector.0 * vector.0 + vector.1 * vector.1).sqrt();
            //         let normal = (vector.0 / magnitude, vector.1 / magnitude);

            //         colliders.push(Collider::Line(Line {
            //             start: (Number::from_f32(start.0), Number::from_f32(start.1)).into(),
            //             end: (Number::from_f32(end.0), Number::from_f32(end.1)).into(),
            //             normal: (Number::from_f32(normal.1), Number::from_f32(-normal.0)).into(),
            //         }))
            //     }
            // }
            _ => unimplemented!("Use of unsupported shape, {:?}", object.shape),
        }
    }

    println!("{colliders:#?}");

    colliders
}

// pushes the circle that should be added, and returns the replacement end / start positions (so where line ao and ob should actually finish)
fn rounded_line_collider(
    a: Vector2<f32>,
    o: Vector2<f32>,
    b: Vector2<f32>,
    radius: f32,
    colliders: &mut Vec<Collider>,
) -> (Vector2<f32>, Vector2<f32>) {
    let x = a - o;
    let y = b - o;

    let x_hat = x.normalize();
    let y_hat = y.normalize();

    let c = (x_hat + y_hat).normalize() * radius / ((1. - x_hat.dot(&y_hat)) / 2.).sqrt();

    let p1 = x_hat.dot(&c) * x_hat;
    let p2 = y_hat.dot(&c) * y_hat;

    let circle_center = o + c;

    colliders.push(Collider::Circle(Circle {
        position: to_vec(circle_center),
        radius: Number::from_f32(radius),
    }));

    (o + p1, o + p2)
}

fn to_vec(a: Vector2<f32>) -> Vector2D<Number> {
    (Number::from_f32(a.x), Number::from_f32(a.y)).into()
}

fn get_line_collider(start: Vector2<f32>, end: Vector2<f32>) -> Collider {
    let normalized = (end - start).normalize();
    let normal = Vector2::new(normalized.y, -normalized.x);

    Collider::Line(Line {
        start: to_vec(start),
        end: to_vec(end),
        normal: to_vec(normal),
    })
}
