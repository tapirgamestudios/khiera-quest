#![no_std]

use agb_fixnum::{Num, Vector2D};

pub type Number = Num<i32, 8>;

#[derive(Clone, Debug)]
pub enum ColliderKind {
    Circle(Circle),
    Line(Line),
    Segment(Segment),
}

#[derive(Clone, Debug)]
pub struct Collider {
    pub kind: ColliderKind,
    pub gravitational: bool,
}

impl Collider {
    pub fn collides_circle(&self, circle: &Circle) -> bool {
        match &self.kind {
            ColliderKind::Circle(this) => this.collides_circle(circle),
            ColliderKind::Line(this) => this.collides_circle(circle),
            ColliderKind::Segment(this) => this.collides_circle(circle),
        }
    }

    pub fn normal_circle(&self, circle: &Circle) -> Vector2D<Number> {
        match &self.kind {
            ColliderKind::Circle(this) => this.normal_point(circle.position),
            ColliderKind::Line(this) => this.normal,
            ColliderKind::Segment(this) => this.normal_point(circle.position),
        }
    }

    pub fn overshoot(&self, circle: &Circle) -> Vector2D<Number> {
        match &self.kind {
            ColliderKind::Circle(this) => this.overshoot_circle(circle),
            ColliderKind::Line(this) => this.overshoot_circle(circle),
            ColliderKind::Segment(this) => this.overshoot_circle(circle),
        }
    }

    pub fn closest_point(&self, point: Vector2D<Number>) -> Vector2D<Number> {
        match &self.kind {
            ColliderKind::Circle(this) => this.closest_point(point),
            ColliderKind::Line(this) => this.closest_point(point),
            ColliderKind::Segment(this) => this.closest_point(point),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RealSpace(pub Vector2D<Number>);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ScreenSpace(pub Vector2D<Number>);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Circle {
    pub position: Vector2D<Number>,
    pub radius: Number,
}

impl Circle {
    pub fn collides_circle(&self, circle: &Circle) -> bool {
        let distance = self.radius + circle.radius;

        (self.position - circle.position).magnitude_squared() <= distance * distance
    }

    pub fn collides_line(&self, line: &Line) -> bool {
        line.collides_circle(self)
    }

    pub fn overshoot_circle(&self, circle: &Circle) -> Vector2D<Number> {
        let distance = (circle.position - self.position).magnitude();
        let magnitude = self.radius + circle.radius - distance;

        self.normal_point(circle.position) * magnitude
    }

    pub fn normal_point(&self, point: Vector2D<Number>) -> Vector2D<Number> {
        (point - self.position).fast_normalise()
    }

    pub fn closest_point(&self, point: Vector2D<Number>) -> Vector2D<Number> {
        self.normal_point(point) * self.radius + self.position
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Line {
    pub start: Vector2D<Number>,
    pub end: Vector2D<Number>,
    pub normal: Vector2D<Number>,
    pub length: Number,
}

impl Line {
    pub fn collides_circle(&self, circle: &Circle) -> bool {
        let closest_point = self.closest_point(circle.position);

        (closest_point - circle.position).magnitude_squared() <= circle.radius * circle.radius
    }

    pub fn overshoot_circle(&self, circle: &Circle) -> Vector2D<Number> {
        // from https://en.wikipedia.org/wiki/Distance_from_a_point_to_a_line#Line_defined_by_two_points
        let x1 = self.start.x;
        let x2 = self.end.x;
        let x0 = circle.position.x;

        let y1 = self.start.y;
        let y2 = self.end.y;
        let y0 = circle.position.y;

        let distance = ((y2 - y1) * x0 - (x2 - x1) * y0 + x2 * y1 - y2 * x1).abs() / self.length;

        let amount = circle.radius - distance;
        self.normal * amount
    }

    pub fn closest_point(&self, point: Vector2D<Number>) -> Vector2D<Number> {
        // translate so that we are working from the origin
        let x = self.end - self.start;
        let p = point - self.start;

        let x_magnitude_sq = self.length * self.length;

        // if y = the point on the line closest to p, then x.y = x.p due to the projection
        let discriminant = x.dot(p);

        if discriminant <= 0.into() {
            self.start
        } else if discriminant >= x_magnitude_sq {
            self.end
        } else {
            // now we actually have to scale it. y = x.p * x / (x.x)
            let offset = x * discriminant / x_magnitude_sq.floor();

            self.start + offset.change_base()
        }
    }
}

#[derive(Clone, Debug)]
pub struct Segment {
    pub circle: Circle,
    pub start_angle: Num<i32, 8>,
    pub end_angle: Num<i32, 8>,
}
impl Segment {
    fn collides_circle(&self, circle: &Circle) -> bool {
        todo!()
    }

    fn normal_point(&self, position: Vector2D<Num<i32, 8>>) -> Vector2D<Num<i32, 8>> {
        todo!()
    }

    fn overshoot_circle(&self, circle: &Circle) -> Vector2D<Num<i32, 8>> {
        todo!()
    }

    fn closest_point(&self, point: Vector2D<Num<i32, 8>>) -> Vector2D<Num<i32, 8>> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
}
