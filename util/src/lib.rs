#![no_std]

use agb_fixnum::{Num, Vector2D};

type Number = Num<i32, 8>;

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
        todo!()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Line {
    pub start: Vector2D<Number>,
    pub end: Vector2D<Number>,
}

impl Line {
    pub fn collides_circle(&self, circle: &Circle) -> bool {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
}
