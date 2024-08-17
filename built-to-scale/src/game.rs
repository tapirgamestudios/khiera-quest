use agb::{display::affine::AffineMatrix, fixnum::Vector2D};
use util::{Collider, Number};

struct Offset {
    space: AffineMatrix,
}

struct Player {
    space: AffineMatrix,
}

struct Game {
    screen_space_offset: Offset,
    player: Player,
    terrain: Terrain,
}

struct Terrain {
    // todo
}

impl Terrain {
    fn gravity(&self, position: Vector2D<Number>) -> Vector2D<Number> {
        todo!()
    }

    fn colliders(&self, position: Vector2D<Number>) -> impl Iterator<Item = Collider> {
        todo!();

        [].into_iter()
    }
}
