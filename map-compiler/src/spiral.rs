enum Side {
    Center,
    Top,
    Right,
    Bottom,
    Left,
}

pub struct SpiralIterator {
    side: Side,
    side_length: i32,
    x: i32,
    y: i32,
    center_x: i32,
    center_y: i32,
}

impl SpiralIterator {
    pub fn new(center: (i32, i32)) -> Self {
        Self {
            side: Side::Center,
            side_length: 1,
            x: 0,
            y: 0,
            center_x: center.0,
            center_y: center.1,
        }
    }
}

impl Iterator for SpiralIterator {
    type Item = (i32, i32);

    fn next(&mut self) -> Option<Self::Item> {
        match self.side {
            Side::Center => self.side = Side::Top,
            Side::Top => {
                self.x += 1;
                if self.x == self.side_length {
                    self.side = Side::Right;
                }
            }
            Side::Right => {
                self.y += 1;
                if self.y == self.side_length {
                    self.side = Side::Bottom
                }
            }
            Side::Bottom => {
                self.x -= 1;
                if self.x == -self.side_length {
                    self.side = Side::Left;
                }
            }
            Side::Left => {
                self.y -= 1;
                if self.y == -self.side_length {
                    self.side_length += 1;
                    self.side = Side::Top;
                }
            }
        }

        Some((self.center_x + self.x, self.center_y + self.y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_spiral_iterator() {
        let mut iter = SpiralIterator::new((0, 0));

        assert_eq!(iter.next(), Some((0, 0)));

        assert_eq!(iter.next(), Some((1, 0)));
        assert_eq!(iter.next(), Some((1, 1)));
        assert_eq!(iter.next(), Some((0, 1)));
        assert_eq!(iter.next(), Some((-1, 1)));
        assert_eq!(iter.next(), Some((-1, 0)));
        assert_eq!(iter.next(), Some((-1, -1)));
        assert_eq!(iter.next(), Some((0, -1)));
        assert_eq!(iter.next(), Some((1, -1)));

        assert_eq!(iter.next(), Some((2, -1)));
        assert_eq!(iter.next(), Some((2, 0)));
        assert_eq!(iter.next(), Some((2, 1)));
        assert_eq!(iter.next(), Some((2, 2)));
        assert_eq!(iter.next(), Some((1, 2)));
        assert_eq!(iter.next(), Some((0, 2)));
        assert_eq!(iter.next(), Some((-1, 2)));
        assert_eq!(iter.next(), Some((-2, 2)));
        assert_eq!(iter.next(), Some((-2, 1)));
        assert_eq!(iter.next(), Some((-2, 0)));
        assert_eq!(iter.next(), Some((-2, -1)));
        assert_eq!(iter.next(), Some((-2, -2)));
        assert_eq!(iter.next(), Some((-1, -2)));
        assert_eq!(iter.next(), Some((0, -2)));
        assert_eq!(iter.next(), Some((1, -2)));
        assert_eq!(iter.next(), Some((2, -2)));
    }
}
