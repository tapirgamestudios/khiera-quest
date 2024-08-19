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

pub fn perimeter(top_corner: (i32, i32), size: i32) -> impl Iterator<Item = (i32, i32)> {
    assert!(size > 0);

    (top_corner.0..top_corner.0 + size)
        .map(move |x| (x, top_corner.1))
        .chain((top_corner.1..top_corner.1 + size).map(move |y| (top_corner.0 + size, y)))
        .chain(
            (top_corner.0..=top_corner.0 + size)
                .rev()
                .take(size as usize)
                .map(move |x| (x, top_corner.1 + size)),
        )
        .chain(
            (top_corner.1..=top_corner.1 + size)
                .rev()
                .take(size as usize)
                .map(move |y| (top_corner.0, y)),
        )
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

    #[test]
    fn check_perimeter() {
        let mut perimeter = perimeter((-1, -1), 2);

        assert_eq!(perimeter.next(), Some((-1, -1)));
        assert_eq!(perimeter.next(), Some((0, -1)));
        assert_eq!(perimeter.next(), Some((1, -1)));
        assert_eq!(perimeter.next(), Some((1, 0)));
        assert_eq!(perimeter.next(), Some((1, 1)));
        assert_eq!(perimeter.next(), Some((0, 1)));
        assert_eq!(perimeter.next(), Some((-1, 1)));
        assert_eq!(perimeter.next(), Some((-1, 0)));

        assert_eq!(perimeter.next(), None);
    }

    #[test]
    fn check_perimeter_larger() {
        let mut perimeter = perimeter((0, 0), 4);

        let expected = [
            (0, 0),
            (1, 0),
            (2, 0),
            (3, 0),
            (4, 0),
            (4, 1),
            (4, 2),
            (4, 3),
            (4, 4),
            (3, 4),
            (2, 4),
            (1, 4),
            (0, 4),
            (0, 3),
            (0, 2),
            (0, 1),
        ];

        for (expected, actual) in expected.into_iter().zip(&mut perimeter) {
            assert_eq!(actual, expected);
        }

        assert_eq!(perimeter.next(), None);
    }
}
