use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};

use agb_fixnum::{Num, Vector2D};
use itertools::Itertools;
use nalgebra::{Vector2, Vector3};
use proc_macro2::TokenStream;
use tiled::{Map, Object, ObjectShape, PropertyValue};
use util::{Arc, Circle, Collider, ColliderKind, ColliderTag, Line, Number};

use quote::{format_ident, quote};

use crate::spiral::{perimeter, SpiralIterator};

/// These control the performance and ROM size
/// The size of a box of colliders in pixels
const BOX_SIZE: i32 = 32;
/// The number of boxes to go out from each inner box
const BOX_DISTANCE_FROM_INNER: usize = 4;

const PLAYER_CIRCLE_APPROX_RADIUS: i32 = 8;

const PATH_BOX_SIZE: i32 = 256;

fn occupied_boxes<F>(collider: &Collider, mut f: F)
where
    F: FnMut(i32, i32),
{
    match &collider.kind {
        ColliderKind::Circle(circle) | ColliderKind::Arc(Arc { circle, .. }) => {
            let position = circle.position.floor();
            let radius = circle.radius.floor();

            let min_box_x = (position.x - radius).div_floor(BOX_SIZE);
            let max_box_x = (position.x + radius + BOX_SIZE - 1).div_floor(BOX_SIZE);
            let min_box_y = (position.y - radius) / BOX_SIZE;
            let max_box_y = (position.y + radius + BOX_SIZE - 1).div_floor(BOX_SIZE);

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
            for (x, y) in boxes_line_crosses_through(line.start.floor(), line.end.floor(), BOX_SIZE)
            {
                f(x, y);
            }
        }
    }
}

fn boxes_line_crosses_through(
    start: Vector2D<i32>,
    end: Vector2D<i32>,
    box_size: i32,
) -> impl Iterator<Item = (i32, i32)> {
    let mut current_box = (i32::MIN, i32::MIN);
    let mut iter = bresenham::Bresenham::new(
        (start.x as isize, start.y as isize),
        (end.x as isize, end.y as isize),
    );

    core::iter::from_fn(move || loop {
        let (x, y) = iter.next()?;
        let this_box = (
            (x as i32).div_floor(box_size),
            (y as i32).div_floor(box_size),
        );
        if current_box != this_box {
            current_box = this_box;
            return Some((this_box.0, this_box.1));
        }
    })
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

#[derive(Clone)]
struct ColliderGroup {
    name: String,
    class: String,
    colliders: Vec<Collider>,
}

fn extract_from_layer<'a>(
    layer: impl Iterator<Item = Object<'a>>,
    tag: ColliderTag,
) -> Vec<ColliderGroup> {
    let mut all_colliders = Vec::new();
    for object in layer {
        let mut colliders = Vec::new();
        match &object.shape {
            tiled::ObjectShape::Rect { width, height } => {
                handle_points_for_collider(
                    object,
                    &[(0., 0.), (*width, 0.), (*width, *height), (0., *height)],
                    &mut colliders,
                    tag,
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
                    tag,
                    velocity: Vector2D::new(0.into(), 0.into()),
                });
            }
            tiled::ObjectShape::Polygon { points } | tiled::ObjectShape::Polyline { points } => {
                handle_points_for_collider(
                    object,
                    points,
                    &mut colliders,
                    tag,
                    matches!(&object.shape, tiled::ObjectShape::Polygon { .. }),
                );
            }
            _ => unimplemented!("Use of unsupported shape, {:?}", object.shape),
        }

        all_colliders.push(ColliderGroup {
            name: object.name.clone(),
            class: object.user_type.clone(),
            colliders,
        })
    }

    all_colliders
}

fn handle_points_for_collider(
    object: tiled::Object,
    points: &[(f32, f32)],
    colliders: &mut Vec<Collider>,
    tag: ColliderTag,
    is_polygon: bool,
) {
    let origin = Vector2::new(object.x, object.y);

    let radius = object
        .properties
        .get("radius")
        .map(|radius| {
            if let PropertyValue::FloatValue(radius) = radius {
                *radius
            } else {
                panic!("Invalid value for radius {radius:?}")
            }
        })
        .unwrap_or(2.);

    if points.len() == 2 {
        colliders.extend(get_line_colliders(
            Vector2::new(points[0].0, points[0].1) + origin,
            Vector2::new(points[1].0, points[1].1) + origin,
            tag,
        ));

        return;
    }

    let mut modified_points = Vec::new();

    let mut do_line_work = |a: (f32, f32), o: (f32, f32), b: (f32, f32)| {
        let a = Vector2::new(a.0, a.1) + origin;
        let o = Vector2::new(o.0, o.1) + origin;
        let b = Vector2::new(b.0, b.1) + origin;

        modified_points.push(rounded_line_collider(a, o, b, radius, tag, colliders));
    };

    for x in points.windows(3) {
        let [a, o, b] = x else { panic!() };
        do_line_work(*a, *o, *b);
    }

    // do the closing part of the polygon
    if is_polygon {
        do_line_work(
            points[points.len() - 2],
            points[points.len() - 1],
            points[0],
        );
        do_line_work(points[points.len() - 1], points[0], points[1]);
    }

    let mut current = modified_points[0].1;
    for (new_end, next_start) in modified_points.iter().skip(1) {
        colliders.extend(get_line_colliders(current, *new_end, tag));

        current = *next_start;
    }

    if is_polygon {
        colliders.extend(get_line_colliders(current, modified_points[0].0, tag));
    } else {
        // need to manually attach the start and end lines
        let start_point = Vector2::new(points[0].0, points[0].1) + origin;
        let end_point =
            Vector2::new(points[points.len() - 1].0, points[points.len() - 1].1) + origin;

        colliders.extend(get_line_colliders(start_point, modified_points[0].0, tag));
        colliders.extend(get_line_colliders(
            modified_points[modified_points.len() - 1].1,
            end_point,
            tag,
        ));
    }
}

fn extract_colliders(map: &Map) -> Vec<ColliderGroup> {
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

    let killision = map
        .layers()
        .filter(|x| x.name == "Killision")
        .find_map(|x| x.as_object_layer())
        .unwrap();

    let mut o = extract_from_layer(
        gravitational_objects.objects(),
        ColliderTag::CollisionGravitational,
    );
    o.extend(extract_from_layer(
        non_gravitattional_objects.objects(),
        ColliderTag::CollisionOnly,
    ));
    o.extend(extract_from_layer(
        killision
            .objects()
            .filter(|x| !matches!(x.shape, tiled::ObjectShape::Point(_, _))),
        ColliderTag::Killision,
    ));

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
        // add the entire box it's in
        this_container.extend(
            spacial_colliders
                .get(&(x, y))
                .map(|x| x.as_slice())
                .unwrap_or_default(),
        );

        // look at the surrounding boxes...
        let surrounding_containers: HashSet<_> = SpiralIterator::new((x, y))
            .take(9)
            .skip(1)
            .flat_map(|(xx, yy)| spacial_colliders.get(&(xx, yy)))
            .flatten()
            .copied()
            .collect();

        // and go over the entire perimeter and check they could collide with the player
        for (xx, yy) in perimeter((x * BOX_SIZE, y * BOX_SIZE), BOX_SIZE) {
            for (collider_idx, collider) in
                surrounding_containers.iter().map(|&x| (x, &colliders[x]))
            {
                if this_container.contains(&collider_idx) {
                    continue;
                }
                if (collider.closest_point((xx, yy).into()) - (xx, yy).into()).magnitude_squared()
                    <= (PLAYER_CIRCLE_APPROX_RADIUS * PLAYER_CIRCLE_APPROX_RADIUS).into()
                {
                    this_container.insert(collider_idx);
                }
            }
        }

        // find closest gravity
        for xx in 0..2 {
            for yy in 0..2 {
                let center_of_box = ((x + xx) * BOX_SIZE, (y + yy) * BOX_SIZE).into();
                let (idx, _) = colliders
                    .iter()
                    .enumerate()
                    .filter(|(_, x)| x.tag.is_gravitational())
                    .map(|(idx, collider)| (idx, collider.closest_point(center_of_box)))
                    .min_by_key(|&(_, closest_point)| {
                        (closest_point - center_of_box).magnitude_squared()
                    })
                    .unwrap();

                this_container.insert(idx);
            }
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

fn extract_recovery_points(map: &Map) -> Vec<Vector2D<Number>> {
    let layer = map
        .layers()
        .filter(|x| x.name == "Killision")
        .find_map(|x| x.as_object_layer())
        .unwrap();

    layer
        .objects()
        .filter_map(|x| {
            if let ObjectShape::Point(x, y) = x.shape {
                Some((Number::from_f32(x), Number::from_f32(y)).into())
            } else {
                None
            }
        })
        .collect()
}

fn quote_vec(vector: Vector2D<Number>) -> TokenStream {
    let x = vector.x.to_raw();
    let y = vector.y.to_raw();

    quote! {
        Vector2D::new(Number::from_raw(#x), Number::from_raw(#y))
    }
}

struct Path {
    name: String,
    points: Vec<Vector2D<Number>>,
    complete: bool,
    speed: f64,
}

fn extract_paths(map: &Map) -> Vec<Path> {
    let path_layer = map
        .layers()
        .filter(|x| x.name == "Paths")
        .find_map(|x| x.as_object_layer())
        .unwrap();

    path_layer
        .objects()
        .map(|object| {
            let is_complete = matches!(object.shape, ObjectShape::Polygon { .. });

            let points = match &object.shape {
                ObjectShape::Polyline { points } => points,
                ObjectShape::Polygon { points } => points,
                _ => panic!("Path should be polyline or polygon"),
            };

            let speed = object
                .properties
                .get("speed")
                .map(|x| match x {
                    PropertyValue::FloatValue(x) => *x as f64,
                    _ => panic!("Speed should be a float"),
                })
                .expect("Moving path should specify a speed");

            Path {
                name: object.name.clone(),
                points: points
                    .iter()
                    .copied()
                    .map(|(x, y)| {
                        (
                            Number::from_f32(x + object.x),
                            Number::from_f32(y + object.y),
                        )
                            .into()
                    })
                    .collect(),
                complete: is_complete,
                speed,
            }
        })
        .collect()
}

fn assemble_dynamic_colliders(map: &Map) -> String {
    let dynamic_colliders: Vec<_> = extract_colliders(map)
        .iter()
        .filter(|&x| !x.name.is_empty())
        .cloned()
        .collect();

    let dynamic_object_images = dynamic_colliders
        .iter()
        .map(|x| x.name.as_str())
        .unique()
        .map(|x| format_ident!("{}", x));

    let paths = extract_paths(map);

    // lookup what index the collider group is stored in
    let collider_group_indexes: HashMap<&str, usize> = dynamic_colliders
        .iter()
        .enumerate()
        .map(|(idx, x)| (x.class.as_str(), idx))
        .collect();

    let dynamic_collider_groups = dynamic_colliders.iter().map(|collider_group| {
        let path = paths
            .iter()
            .find(|x| x.name == collider_group.class)
            .expect("Should be a path for an object");

        let path_points = path
            .points
            .iter()
            .copied()
            .chain(core::iter::once(path.points[0]))
            .tuple_windows()
            .map(|(a, b)| {
                let length = (b - a).magnitude();
                let length = length.to_raw() as f64 / (1 << 8) as f64;
                let time = path.speed * 1. / length;
                let time_fixed = Num::<i32, 24>::from_f64(time);

                (a, time_fixed)
            });
        let points = path_points.map(|(point, frames)| {
            let point = quote_vec(point);
            let incrementer = frames.to_raw();

            quote! {
                PathPoint {
                    point: #point,
                    incrementer: Num::from_raw(#incrementer),
                }
            }
        });

        let colliders = collider_group.colliders.iter().map(quote_collider);
        let complete = path.complete;
        let image = format_ident!("{}", collider_group.name);

        quote! {
            Path{
                points: &[
                    #(#points),*
                ],
                colliders:  &[
                    #(#colliders),*
                ],
                complete: #complete,
                image: DynamicColliderImage::#image,
            }
        }
    });

    let mut phf = phf_codegen::Map::new();

    let mut boxes_path_crosses_idx: HashMap<(i32, i32), Vec<usize>> = HashMap::new();

    for path in paths.iter() {
        // find out which group this path is for
        let collider_group_idx = collider_group_indexes
            .get(path.name.as_str())
            .copied()
            .expect("Find object group for path");
        let boxes_path_goes_through: HashSet<(i32, i32)> = path
            .points
            .windows(2)
            .flat_map(|line| {
                boxes_line_crosses_through(line[0].floor(), line[1].floor(), PATH_BOX_SIZE)
            })
            .collect();

        let mut boxes_surrounding = HashSet::new();

        for (x, y) in boxes_path_goes_through {
            for (x, y) in SpiralIterator::new((x, y)).take(9) {
                boxes_surrounding.insert((x, y));
            }
        }

        for (x, y) in boxes_surrounding {
            boxes_path_crosses_idx
                .entry((x, y))
                .or_default()
                .push(collider_group_idx);
        }
    }

    for ((x, y), path_idx) in boxes_path_crosses_idx {
        let references = path_idx.into_iter().map(|idx| {
            quote! {
                &DYNAMIC_COLLIDER_GROUPS[#idx]
            }
        });
        phf.entry([x, y], &format!("{}", quote! { &[ #(#references),* ] }));
    }

    format!(
        "{}{};\n\n",
        quote! {

            pub static DYNAMIC_COLLIDER_GROUPS: &[Path] = &[
                #(#dynamic_collider_groups),*
            ];

            #[derive(Clone, Copy)]
            pub enum DynamicColliderImage {
                #(#dynamic_object_images),*
            }

            pub const PATH_BOX_SIZE: i32 = #PATH_BOX_SIZE;

            pub static PATH_LOOKUP: phf::Map<[i32; 2], &'static [&'static Path]> =

        },
        phf.build()
    )
}

fn quote_collider(collider: &Collider) -> TokenStream {
    let kind = match &collider.kind {
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
    let tag = match collider.tag {
        util::ColliderTag::CollisionOnly => quote! {
            ColliderTag::CollisionOnly
        },
        util::ColliderTag::CollisionGravitational => quote! {
            ColliderTag::CollisionGravitational
        },
        util::ColliderTag::Killision => quote! {
            ColliderTag::Killision
        },
    };

    let velocity = quote_vec(collider.velocity);

    quote! {
        Collider {
            kind: #kind,
            tag: #tag,
            velocity: #velocity,
        }
    }
}

pub fn assemble_colliders(map: &Map) -> String {
    let colliders: Vec<_> = extract_colliders(map)
        .iter()
        .filter(|&x| x.name.is_empty())
        .cloned()
        .flat_map(|x| x.colliders)
        .collect();
    let spacial_colliders = spacial_colliders(&colliders);
    let boxed_up = get_3_and_first_gravity(&colliders, &spacial_colliders);

    let max_collider_box = boxed_up.iter().max_by_key(|(_, x)| x.len()).unwrap();

    println!(
        "cargo::warning=Maximum number of colliders in a box = {} at ({}, {})",
        max_collider_box.1.len(),
        max_collider_box.0 .0 * BOX_SIZE,
        max_collider_box.0 .1 * BOX_SIZE
    );

    let colliders_quote = colliders.iter().map(quote_collider);

    let mut collider_phf = phf_codegen::Map::new();

    for (key, colliders) in boxed_up.iter() {
        let x = key.0;
        let y = key.1;
        let colliders = colliders.iter().map(|idx| quote! { &COLLIDERS [#idx] });
        let entry = quote! {&[#(#colliders),*]}.to_string();
        collider_phf.entry([x, y], &entry);
    }

    let collider_phf_code = collider_phf.build();

    let recovery_points = extract_recovery_points(map);
    let recovery_points = recovery_points.into_iter().map(quote_vec);

    format!(
        "{}{};\n\n{}",
        quote! {
            pub const BOX_SIZE: i32 = #BOX_SIZE;

            static COLLIDERS: &[Collider] = &[#(#colliders_quote),*];

            pub static RECOVERY_POINTS: &[Vector2D<Number>] = &[
                #(#recovery_points),*
            ];

            pub static NEARBY_COLLIDERS: phf::Map<[i32; 2], &'static [&'static Collider]> =
        },
        collider_phf_code,
        assemble_dynamic_colliders(map),
    )
}

// pushes the circle that should be added, and returns the replacement end / start positions (so where line ao and ob should actually finish)
fn rounded_line_collider(
    a: Vector2<f32>,
    o: Vector2<f32>,
    b: Vector2<f32>,
    mut radius: f32,
    tag: ColliderTag,
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
            velocity: Vector2D::new(0.into(), 0.into()),
            tag,
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
            tag,
            velocity: Vector2D::new(0.into(), 0.into()),
        })
    }

    (o + p1, o + p2)
}

fn to_vec(a: Vector2<f32>) -> Vector2D<Number> {
    (Number::from_f32(a.x), Number::from_f32(a.y)).into()
}

fn get_line_colliders(start: Vector2<f32>, end: Vector2<f32>, tag: ColliderTag) -> Vec<Collider> {
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
            velocity: Vector2D::new(0.into(), 0.into()),
            tag,
        });

        start = end;
        remaining_length -= segment_length;
    }

    ret
}
