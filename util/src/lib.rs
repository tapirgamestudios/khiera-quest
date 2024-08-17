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
            Collider::Circle(this) => (circle.position - this.position).fast_normalise(),
            Collider::Line(line) => line.normal,
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
        (self.position - circle.position).magnitude_squared()
            <= self.radius * self.radius + circle.radius * circle.radius
    }

    pub fn collides_line(&self, line: &Line) -> bool {
        line.collides_circle(self)
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
        let m = (self.start.x - circle.position.x).floor();
        let n = (self.start.y - circle.position.y).floor();

        let c = (self.end.x - self.start.x).floor();
        let d = (self.end.y - self.start.y).floor();

        let r = circle.radius.floor();

        let discriminant = r * r * (m * m + n * n) - (m * d - n * c) * (m * d - n * c);

        if discriminant < 0.into() {
            return false;
        }

        let lower_bound_of_sqrt = m * c + n * d;
        let upper_bound_of_sqrt = m * m + n * n + lower_bound_of_sqrt;

        agb::println!(
            "{} {} {}",
            lower_bound_of_sqrt,
            discriminant,
            upper_bound_of_sqrt
        );

        let lower_bound_of_sqrt_squared = lower_bound_of_sqrt * lower_bound_of_sqrt;
        let upper_bound_of_sqrt_squared = upper_bound_of_sqrt * upper_bound_of_sqrt;

        if upper_bound_of_sqrt <= 0.into() {
            // both will be negative, so only the negative discriminant will be useful in the non-squared case
            return lower_bound_of_sqrt_squared >= discriminant
                && discriminant >= upper_bound_of_sqrt_squared;
        }

        if lower_bound_of_sqrt >= 0.into() {
            // both will be positive, so only the positive discriminant will be useful in the non-squared case
            return lower_bound_of_sqrt_squared <= discriminant
                && discriminant <= upper_bound_of_sqrt_squared;
        }

        // lower bound is negative and upper bound is positive. Therefore automatically positive square root will
        // be bigger than the lower bound, and the negative square root will be smaller than the upper bound.

        // so need to check that the discriminant itself is less than either the upper bound squared or the lower bound squared
        discriminant <= upper_bound_of_sqrt_squared || discriminant <= lower_bound_of_sqrt_squared
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
}
