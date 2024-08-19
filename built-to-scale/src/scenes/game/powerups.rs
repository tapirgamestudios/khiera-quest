use agb::{
    display::{
        object::{Sprite, SpriteLoader, SpriteVram},
        HEIGHT, WIDTH,
    },
    fixnum::{num, Vector2D},
};
use util::Number;

use crate::{resources, scenes::Display};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PowerUp {
    JumpBoost,
}

impl PowerUp {
    fn sprite(self) -> &'static Sprite {
        match self {
            PowerUp::JumpBoost => resources::JUMP_BOOST.sprite(0),
        }
    }
}

pub struct PowerUpObject {
    location: Vector2D<Number>,
    kind: PowerUp,
}

impl PowerUpObject {
    pub fn new(kind: PowerUp, location: Vector2D<Number>) -> Self {
        Self { kind, location }
    }

    // If none, then do nothing
    // If some, then give the player the power up returned
    pub fn update(&self, player_location: Vector2D<Number>) -> Option<PowerUp> {
        if (player_location - self.location).magnitude_squared() < (16 * 16).into() {
            Some(self.kind)
        } else {
            None
        }
    }

    pub fn display(&self, camera_location: Vector2D<Number>, display: &mut Display) {
        if (self.location - camera_location).magnitude_squared() > (WIDTH * WIDTH).into() {
            return; // don't need to render
        }

        display.display_regular(
            self.kind.sprite(),
            self.location - camera_location - (num!(16.), num!(16.)).into()
                + (WIDTH / 2, HEIGHT / 2).into(),
        );
    }
}
