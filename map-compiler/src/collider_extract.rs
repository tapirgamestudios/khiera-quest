use std::collections::HashMap;

use agb_fixnum::Vector2D;
use nalgebra::Vector2;
use tiled::{Map, ObjectLayer};
use util::{Circle, Collider, ColliderKind, Line, Number};

use crate::BOX_SIZE;

fn occupied_boxes<F>(collider: &Collider, mut f: F)
where
    F: FnMut(i32, i32),
{
    match &collider.kind {
        ColliderKind::Circle(circle) => {
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

pub fn spacial_colliders(colliders: &[Collider]) -> HashMap<(i32, i32), Vec<usize>> {
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

                colliders.push(Collider {
                    kind: ColliderKind::Circle(Circle {
                        position,
                        radius: Number::from_f32(*width / 2.),
                    }),
                    gravitational,
                });
            }
            tiled::ObjectShape::Polygon { points } | tiled::ObjectShape::Polyline { points } => {
                let origin = Vector2::new(object.x, object.y);
                let mut modified_points = Vec::new();

                let mut do_line_work = |a: (f32, f32), o: (f32, f32), b: (f32, f32)| {
                    let a = Vector2::new(a.0, a.1) + origin;
                    let o = Vector2::new(o.0, o.1) + origin;
                    let b = Vector2::new(b.0, b.1) + origin;

                    modified_points.push(rounded_line_collider(
                        a,
                        o,
                        b,
                        2.,
                        gravitational,
                        &mut colliders,
                    ));
                };

                do_line_work(points[points.len() - 1], points[0], points[1]);

                for x in points.windows(3) {
                    let [a, o, b] = x else { panic!() };
                    do_line_work(*a, *o, *b);
                }

                if matches!(&object.shape, tiled::ObjectShape::Polygon { .. }) {
                    do_line_work(
                        points[points.len() - 2],
                        points[points.len() - 1],
                        points[0],
                    );
                }

                let mut current = modified_points[0].1;

                for (new_end, next_start) in modified_points.iter().skip(1) {
                    colliders.push(get_line_collider(current, *new_end, gravitational));

                    current = *next_start;
                }

                if matches!(&object.shape, tiled::ObjectShape::Polygon { .. }) {
                    colliders.push(get_line_collider(
                        current,
                        modified_points[0].0,
                        gravitational,
                    ));
                }
            }
            _ => unimplemented!("Use of unsupported shape, {:?}", object.shape),
        }
    }

    colliders
}

pub fn extract_colliders(map: &Map) -> Vec<Collider> {
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

// pushes the circle that should be added, and returns the replacement end / start positions (so where line ao and ob should actually finish)
fn rounded_line_collider(
    a: Vector2<f32>,
    o: Vector2<f32>,
    b: Vector2<f32>,
    radius: f32,
    gravitational: bool,
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

    colliders.push(Collider {
        kind: ColliderKind::Circle(Circle {
            position: to_vec(circle_center),
            radius: Number::from_f32(radius),
        }),
        gravitational,
    });

    (o + p1, o + p2)
}

fn to_vec(a: Vector2<f32>) -> Vector2D<Number> {
    (Number::from_f32(a.x), Number::from_f32(a.y)).into()
}

fn get_line_collider(start: Vector2<f32>, end: Vector2<f32>, gravitational: bool) -> Collider {
    let normalized = (end - start).normalize();
    let normal = Vector2::new(normalized.y, -normalized.x);
    let length = (start - end).magnitude();

    Collider {
        kind: ColliderKind::Line(Line {
            start: to_vec(start),
            end: to_vec(end),
            normal: to_vec(normal),
            length: Number::from_f32(length),
        }),
        gravitational,
    }
}
