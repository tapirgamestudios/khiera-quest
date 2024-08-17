#![no_std]
use agb_fixnum::{Num, Vector2D};

pub type Number = Num<i32, 8>;

pub enum Collider {
    Circle(Circle),
    Line(Line),
}

impl Collider {
    pub fn collides_circle(&self, circle: &Circle) -> bool {
        match self {
            Collider::Circle(this) => this.collides_circle(circle),
            Collider::Line(this) => this.collides_circle(circle),
        }
    }

    pub fn normal_circle(&self, circle: &Circle) -> Vector2D<Number> {
        match self {
            Collider::Circle(this) => this.normal_point(circle.position),
            Collider::Line(this) => this.normal,
        }
    }

    pub fn overshoot(&self, circle: &Circle) -> Vector2D<Number> {
        match self {
            Collider::Circle(this) => this.overshoot_circle(circle),
            Collider::Line(this) => this.overshoot_circle(circle),
        }
    }

    pub fn closest_point(&self, point: Vector2D<Number>) -> Vector2D<Number> {
        match self {
            Collider::Circle(this) => this.closest_point(point),
            Collider::Line(this) => this.closest_point(point),
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
        let distance = (circle.position - self.position).fast_magnitude();
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
}

impl Line {
    #[inline(never)]
    pub fn collides_circle(&self, circle: &Circle) -> bool {
        let c = (self.start.x - circle.position.x).floor();
        let d = (self.start.y - circle.position.y).floor();

        let m = (self.end.x - self.start.x).floor();
        let n = (self.end.y - self.start.y).floor();

        let r = circle.radius.floor();

        let discriminant = r * r * (m * m + n * n) - (m * d - n * c) * (m * d - n * c);

        if discriminant < 0 {
            return false;
        }

        let lower_bound_of_sqrt = m * c + n * d;
        let upper_bound_of_sqrt = m * m + n * n + lower_bound_of_sqrt;

        let lower_bound_of_sqrt_squared = lower_bound_of_sqrt * lower_bound_of_sqrt;
        let upper_bound_of_sqrt_squared = upper_bound_of_sqrt * upper_bound_of_sqrt;

        if upper_bound_of_sqrt <= 0 {
            // both will be negative, so only the negative discriminant will be useful in the non-squared case
            return lower_bound_of_sqrt_squared >= discriminant
                && discriminant >= upper_bound_of_sqrt_squared;
        }

        if lower_bound_of_sqrt >= 0 {
            // both will be positive, so only the positive discriminant will be useful in the non-squared case
            return lower_bound_of_sqrt_squared <= discriminant
                && discriminant <= upper_bound_of_sqrt_squared;
        }

        // lower bound is negative and upper bound is positive. Therefore automatically positive square root will
        // be bigger than the lower bound, and the negative square root will be smaller than the upper bound.

        // so need to check that the discriminant itself is less than either the upper bound squared or the lower bound squared
        discriminant <= upper_bound_of_sqrt_squared || discriminant <= lower_bound_of_sqrt_squared
    }

    pub fn overshoot_circle(&self, circle: &Circle) -> Vector2D<Number> {
        // from https://en.wikipedia.org/wiki/Distance_from_a_point_to_a_line#Line_defined_by_two_points
        let x1 = self.start.x;
        let x2 = self.end.x;
        let x0 = circle.position.x;

        let y1 = self.start.y;
        let y2 = self.end.y;
        let y0 = circle.position.y;

        let line_length = (self.end - self.start).fast_magnitude();

        let distance = ((y2 - y1) * x0 - (x2 - x1) * y0 + x2 * y1 - y2 * x1).abs() / line_length;

        let amount = circle.radius - distance;
        self.normal * amount
    }

    pub fn closest_point(&self, point: Vector2D<Number>) -> Vector2D<Number> {
        // translate so that we are working from the origin
        let x = self.end - self.start;
        let p = point - self.start;

        let x_magnitude = x.fast_magnitude();

        // closest point on the infinite line
        let y = x / x_magnitude * (p.dot(x));

        let discriminant = x.dot(y);

        if discriminant < 0.into() {
            self.start
        } else if discriminant > 1.into() {
            self.end
        } else {
            y + self.start
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
}
