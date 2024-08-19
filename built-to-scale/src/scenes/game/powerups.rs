use agb::{
    display::{object::Sprite, HEIGHT, WIDTH},
    fixnum::{num, Vector2D},
};
use map::{PowerUp, PowerUpKind};
use util::Number;

use crate::{
    resources,
    scenes::{Display, Update},
};

fn powerup_sprite(powerup: PowerUpKind, frame_amount: usize) -> &'static Sprite {
    match powerup {
        PowerUpKind::JumpBoost => resources::JUMP_BOOST.animation_sprite(frame_amount),
        PowerUpKind::Dash => resources::DASH.animation_sprite(frame_amount),
        PowerUpKind::DoubleJump => resources::DOUBLE_JUMP.animation_sprite(frame_amount),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PowerUpState {
    Idle,
    Collecting,
}

pub struct PowerUpObject {
    location: Vector2D<Number>,
    kind: PowerUpKind,
    frame: usize,
    state: PowerUpState,
}

impl PowerUpObject {
    pub fn new(powerup: &PowerUp) -> Self {
        Self {
            kind: powerup.kind,
            location: powerup.location,
            frame: 0,
            state: PowerUpState::Idle,
        }
    }

    // If none, then do nothing
    // If some, then give the player the power up returned
    pub fn update(
        &mut self,
        player_location: Vector2D<Number>,
        update: &mut Update,
    ) -> Option<PowerUpKind> {
        self.frame += 1;

        if self.state == PowerUpState::Collecting && self.frame > 32 {
            return Some(self.kind);
        }

        let player_distance_sq = (player_location - self.location).magnitude_squared();

        if self.state == PowerUpState::Idle && player_distance_sq < (16 * 16).into() {
            self.state = PowerUpState::Collecting;
            self.frame = 0;

            update.play_sfx(resources::POWER_UP_SOUND);
        }

        None
    }

    pub fn display(&self, camera_location: Vector2D<Number>, display: &mut Display) {
        if (self.location - camera_location).magnitude_squared() > (WIDTH * WIDTH / 3).into() {
            return; // don't need to render
        }

        let frame_amount = match self.state {
            PowerUpState::Idle => self.frame / 8,
            PowerUpState::Collecting => self.frame / 2,
        };

        display.display_regular(
            powerup_sprite(self.kind, frame_amount),
            self.location - camera_location - (num!(8.), num!(8.)).into(),
        );
    }
}
