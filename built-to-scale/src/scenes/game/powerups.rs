use agb::{
    display::{object::Sprite, HEIGHT, WIDTH},
    fixnum::{num, Vector2D},
};
use util::Number;

use crate::{resources, scenes::Display};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PowerUp {
    JumpBoost,
}

impl PowerUp {
    fn sprite(self, frame_amount: usize) -> &'static Sprite {
        match self {
            PowerUp::JumpBoost => resources::JUMP_BOOST.animation_sprite(frame_amount),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PowerUpState {
    Idle,
    Collecting,
}

pub struct PowerUpObject {
    location: Vector2D<Number>,
    kind: PowerUp,
    frame: usize,
    state: PowerUpState,
}

impl PowerUpObject {
    pub fn new(kind: PowerUp, location: Vector2D<Number>) -> Self {
        Self {
            kind,
            location,
            frame: 0,
            state: PowerUpState::Idle,
        }
    }

    // If none, then do nothing
    // If some, then give the player the power up returned
    pub fn update(&mut self, player_location: Vector2D<Number>) -> Option<PowerUp> {
        self.frame += 1;

        if self.state == PowerUpState::Collecting && self.frame > 32 {
            return Some(self.kind);
        }

        let player_distance_sq = (player_location - self.location).magnitude_squared();

        if self.state == PowerUpState::Idle && player_distance_sq < (16 * 16).into() {
            self.state = PowerUpState::Collecting;
            self.frame = 0;
        }

        None
    }

    pub fn display(&self, camera_location: Vector2D<Number>, display: &mut Display) {
        if (self.location - camera_location).magnitude_squared() > (WIDTH * WIDTH).into() {
            return; // don't need to render
        }

        let frame_amount = match self.state {
            PowerUpState::Idle => self.frame / 8,
            PowerUpState::Collecting => self.frame / 2,
        };

        display.display_regular(
            self.kind.sprite(frame_amount),
            self.location - camera_location - (num!(16.), num!(16.)).into()
                + (WIDTH / 2, HEIGHT / 2).into(),
        );
    }
}
