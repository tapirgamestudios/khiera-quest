use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    ops::ControlFlow,
};

use agb_fixnum::Vector2D;
use nalgebra::{Vector2, Vector3};
use proc_macro2::TokenStream;
use tiled::{Map, ObjectLayer};
use util::{Arc, Circle, Collider, ColliderKind, Line, Number};

use quote::quote;

use crate::spiral::SpiralIterator;

/// These control the performance and ROM size
/// The size of a box of colliders in pixels
const BOX_SIZE: i32 = 32;
/// The number of boxes to go out from each inner box
const BOX_DISTANCE_FROM_INNER: usize = 4;

fn occupied_boxes<F>(collider: &Collider, mut f: F)
where
    F: FnMut(i32, i32),
{
    match &collider.kind {
        ColliderKind::Circle(circle) | ColliderKind::Arc(Arc { circle, .. }) => {
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
        ColliderKind::Line(line) => {
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

fn extract_from_layer(layer: &ObjectLayer, gravitational: bool) -> Vec<Collider> {
    let mut colliders = Vec::new();

    for object in layer.objects() {
        match &object.shape {
            tiled::ObjectShape::Rect { width, height } => {
                handle_points_for_collider(
                    object,
                    &[(0., 0.), (*width, 0.), (*width, *height), (0., *height)],
                    &mut colliders,
                    gravitational,
                    true,
                );
            }
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

                colliders.push(Collider {
                    kind: ColliderKind::Circle(Circle {
                        position,
                        radius: Number::from_f32(*width / 2.),
                    }),
                    gravitational,
                });
            }
            tiled::ObjectShape::Polygon { points } | tiled::ObjectShape::Polyline { points } => {
                handle_points_for_collider(
                    object,
                    points,
                    &mut colliders,
                    gravitational,
                    matches!(&object.shape, tiled::ObjectShape::Polygon { .. }),
                );
            }
            _ => unimplemented!("Use of unsupported shape, {:?}", object.shape),
        }
    }

    colliders
}

fn handle_points_for_collider(
    object: tiled::Object,
    points: &[(f32, f32)],
    colliders: &mut Vec<Collider>,
    gravitational: bool,
    is_polygon: bool,
) {
    let origin = Vector2::new(object.x, object.y);
    if points.len() == 2 {
        colliders.extend(get_line_colliders(
            Vector2::new(points[0].0, points[0].1) + origin,
            Vector2::new(points[1].0, points[1].1) + origin,
            gravitational,
        ));

        return;
    }

    let mut modified_points = Vec::new();

    let mut do_line_work = |a: (f32, f32), o: (f32, f32), b: (f32, f32)| {
        let a = Vector2::new(a.0, a.1) + origin;
        let o = Vector2::new(o.0, o.1) + origin;
        let b = Vector2::new(b.0, b.1) + origin;

        modified_points.push(rounded_line_collider(a, o, b, 2., gravitational, colliders));
    };

    do_line_work(points[points.len() - 1], points[0], points[1]);

    for x in points.windows(3) {
        let [a, o, b] = x else { panic!() };
        do_line_work(*a, *o, *b);
    }

    if is_polygon {
        do_line_work(
            points[points.len() - 2],
            points[points.len() - 1],
            points[0],
        );
    }

    let mut current = modified_points[0].1;
    for (new_end, next_start) in modified_points.iter().skip(1) {
        colliders.extend(get_line_colliders(current, *new_end, gravitational));

        current = *next_start;
    }

    if is_polygon {
        colliders.extend(get_line_colliders(
            current,
            modified_points[0].0,
            gravitational,
        ));
    }
}

fn extract_colliders(map: &Map) -> Vec<Collider> {
    let gravitational_objects = map
        .layers()
        .filter(|x| x.name == "Colliders")
        .find_map(|x| x.as_object_layer())
        .unwrap();

    let non_gravitattional_objects = map
        .layers()
        .filter(|x| x.name == "Colliders No Gravity")
        .find_map(|x| x.as_object_layer())
        .unwrap();

    let mut o = extract_from_layer(&gravitational_objects, true);
    o.extend(extract_from_layer(&non_gravitattional_objects, false));

    o
}

fn coordinates_to_generate_box_list_from<T>(
    spacial_colliders: &HashMap<(i32, i32), T>,
) -> HashSet<(i32, i32)> {
    let mut s = HashSet::new();

    let spiral_side_length = BOX_DISTANCE_FROM_INNER * 2 + 1;

    for (x, y) in spacial_colliders.keys() {
        for (x, y) in SpiralIterator::new((*x, *y)).take(spiral_side_length * spiral_side_length) {
            s.insert((x, y));
        }
    }

    s
}

fn get_3_and_first_gravity(
    colliders: &[Collider],
    spacial_colliders: &HashMap<(i32, i32), Vec<usize>>,
) -> HashMap<(i32, i32), Vec<usize>> {
    let box_list = coordinates_to_generate_box_list_from(spacial_colliders);

    let mut resultant = HashMap::new();

    for (x, y) in box_list.into_iter() {
        let mut this_container: HashSet<usize> = HashSet::new();
        for (x, y) in SpiralIterator::new((x, y)).take(9) {
            this_container.extend(
                spacial_colliders
                    .get(&(x, y))
                    .map(|x| x.as_slice())
                    .unwrap_or_default(),
            );
        }
        if !this_container.iter().any(|&x| colliders[x].gravitational) {
            let center_of_box = (x * BOX_SIZE + BOX_SIZE / 2, y * BOX_SIZE + BOX_SIZE / 2).into();
            let (idx, _) = colliders
                .iter()
                .enumerate()
                .filter(|(_, x)| x.gravitational)
                .map(|(idx, collider)| (idx, collider.closest_point(center_of_box)))
                .min_by_key(|&(_, closest_point)| {
                    (closest_point - center_of_box).magnitude_squared()
                })
                .unwrap();

            this_container.insert(idx);
        }

        let mut this_container_as_vec: Vec<_> = this_container.into_iter().collect();

        this_container_as_vec.sort_by(|&a, &b| {
            let (a, b) = (&colliders[a], &colliders[b]);
            match (&a.kind, &b.kind) {
                (ColliderKind::Circle(_) | ColliderKind::Arc(_), _) => Ordering::Less,
                (_, ColliderKind::Circle(_) | ColliderKind::Arc(_)) => Ordering::Greater,
                (_, _) => Ordering::Equal,
            }
        });

        resultant.insert((x, y), this_container_as_vec);
    }

    resultant
}

pub fn assemble_colliders(map: &Map) -> String {
    let colliders = extract_colliders(map);
    let spacial_colliders = spacial_colliders(&colliders);
    let boxed_up = get_3_and_first_gravity(&colliders, &spacial_colliders);

    println!(
        "cargo::warning=Maximum number of colliders in a box = {}",
        boxed_up.values().map(|x| x.len()).max().unwrap()
    );

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

    for (key, colliders) in boxed_up.iter() {
        let x = key.0;
        let y = key.1;
        let colliders = colliders.iter().map(|idx| quote! { &COLLIDERS [#idx] });
        let entry = quote! {&[#(#colliders),*]}.to_string();
        collider_phf.entry([x, y], &entry);
    }

    let collider_phf_code = collider_phf.build();
    format!(
        "{}{};",
        quote! {
            pub const BOX_SIZE: i32 = #BOX_SIZE;

            static COLLIDERS: &[Collider] = &[#(#colliders_quote),*];

            pub static NEARBY_COLLIDERS: phf::Map<[i32; 2], &'static [&'static Collider]> =
        },
        collider_phf_code,
    )
}

// pushes the circle that should be added, and returns the replacement end / start positions (so where line ao and ob should actually finish)
fn rounded_line_collider(
    a: Vector2<f32>,
    o: Vector2<f32>,
    b: Vector2<f32>,
    mut radius: f32,
    gravitational: bool,
    colliders: &mut Vec<Collider>,
) -> (Vector2<f32>, Vector2<f32>) {
    let x = a - o;
    let y = b - o;

    let x_hat = x.normalize();
    let y_hat = y.normalize();

    let x_hat3 = Vector3::new(x_hat.x, x_hat.y, 0.);
    let y_hat3 = Vector3::new(y_hat.x, y_hat.y, 0.);
    let cross_product = x_hat3.cross(&y_hat3).z;

    if cross_product >= 0. {
        radius *= 5.;
    }

    let c = (x_hat + y_hat).normalize() * radius / ((1. - x_hat.dot(&y_hat)) / 2.).sqrt();

    let p1 = x_hat.dot(&c) * x_hat;
    let p2 = y_hat.dot(&c) * y_hat;

    let circle_center = o + c;

    if cross_product <= 0. {
        colliders.push(Collider {
            kind: ColliderKind::Circle(Circle {
                position: to_vec(circle_center),
                radius: Number::from_f32(radius),
            }),
            gravitational,
        });
    } else {
        colliders.push(Collider {
            kind: ColliderKind::Arc(Arc {
                circle: Circle {
                    position: to_vec(circle_center),
                    radius: Number::from_f32(radius),
                },
                start_pos: to_vec((p1 - c).normalize()),
                end_pos: to_vec((p2 - c).normalize()),
            }),
            gravitational,
        })
    }

    (o + p1, o + p2)
}

fn to_vec(a: Vector2<f32>) -> Vector2D<Number> {
    (Number::from_f32(a.x), Number::from_f32(a.y)).into()
}

fn get_line_colliders(
    start: Vector2<f32>,
    end: Vector2<f32>,
    gravitational: bool,
) -> Vec<Collider> {
    let normalized = (end - start).normalize();
    let normal = Vector2::new(normalized.y, -normalized.x);
    let length = (start - end).magnitude();

    let mut start = start;
    let mut remaining_length = length;

    let mut ret = vec![];

    while remaining_length > 0. {
        let segment_length = remaining_length.min(100.);
        let end = start + normalized * segment_length;

        ret.push(Collider {
            kind: ColliderKind::Line(Line {
                start: to_vec(start),
                end: to_vec(end),
                normal: to_vec(normal),
                length: Number::from_f32(segment_length),
            }),
            gravitational,
        });

        start = end;
        remaining_length -= segment_length;
    }

    ret
}
