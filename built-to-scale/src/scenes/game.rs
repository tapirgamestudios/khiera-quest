use agb::{
    display::{affine::AffineMatrix, object::Sprite, HEIGHT, WIDTH},
    fixnum::{num, Rect, Vector2D},
};

use util::{Circle, Collider, Number};

use crate::resources;

use super::{Scene, Update};

struct Camera {
    position: Vector2D<Number>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlayerFacing {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JumpState {
    OnGround,
    Jumping,
    Falling,
}

struct Player {
    angle: AffineMatrix,
    speed: Vector2D<Number>,
    position: Vector2D<Number>,
    jump_state: JumpState,
    facing: PlayerFacing,

    frame: usize,
}

impl Player {
    fn update_facing(&mut self, direction: Vector2D<Number>) {
        let target_angle = AffineMatrix {
            a: -direction.y,
            b: direction.x,
            c: -direction.x,
            d: -direction.y,
            x: 0.into(),
            y: 0.into(),
        };

        self.angle = target_angle;
    }

    fn get_normal(&self) -> Vector2D<Number> {
        (self.angle.b, -self.angle.a).into()
    }

    fn handle_direction_input(&mut self, x: i32) {
        if x != 0 {
            let acceleration: Vector2D<Number> = if self.is_on_ground() {
                Vector2D::new(0.into(), Number::new(x) / 8)
            } else {
                Vector2D::new(0.into(), Number::new(x) / 10)
            };

            let normal = self.get_normal();

            let rotated_acceleration = (
                normal.x * acceleration.x - normal.y * acceleration.y,
                normal.y * acceleration.x + normal.x * acceleration.y,
            )
                .into();

            self.speed += rotated_acceleration;

            if x < 0 {
                self.facing = PlayerFacing::Left;
            } else {
                self.facing = PlayerFacing::Right;
            }
        }
    }

    fn handle_jump_input(&mut self) {
        if self.is_on_ground() {
            let normal = self.get_normal();

            self.speed += normal * num!(4.);
            self.position += self.speed;

            self.jump_state = JumpState::Jumping;
            self.frame = 0;
        }
    }

    fn rendered_position(&self) -> Vector2D<Number> {
        self.position - (8, 8).into()
    }

    fn frame(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        if self.jump_state == JumpState::Jumping && self.frame > 32 {
            self.jump_state = JumpState::Falling;
        }
    }

    fn is_on_ground(&self) -> bool {
        self.jump_state == JumpState::OnGround
    }

    fn sprite(&self) -> &'static Sprite {
        match self.jump_state {
            JumpState::OnGround => {
                if self.speed.magnitude_squared() < num!(0.1) {
                    resources::IDLE.sprite(0)
                } else {
                    resources::WALK.animation_sprite(self.frame / 8)
                }
            }
            JumpState::Jumping => resources::JUMP.animation_sprite(self.frame / 16),
            JumpState::Falling => resources::FALL.sprite(0),
        }
    }
}

pub struct Game {
    camera: Camera,
    player: Player,
    terrain: Terrain,
}

impl Game {
    pub fn new() -> Self {
        Self {
            camera: Camera {
                position: map::CAMERA_START,
            },
            player: Player {
                angle: AffineMatrix::identity(),
                speed: (0, 0).into(),
                position: map::START_POINT,
                jump_state: JumpState::Falling,
                facing: PlayerFacing::Right,

                frame: 0,
            },
            terrain: Terrain {},
        }
    }

    fn handle_direction_input(&mut self, x: i32) {
        self.player.handle_direction_input(x);
    }

    fn handle_jump_input(&mut self) {
        self.player.handle_jump_input();
    }

    fn handle_collider_collisions(&mut self, colliders: &[&Collider]) -> bool {
        let mut on_ground = false;
        for collider in colliders {
            let player_circle = Circle {
                position: self.player.position,
                radius: 8.into(),
            };
            if collider.collides_circle(&player_circle) {
                let normal = collider.normal_circle(&player_circle);

                let dot = normal.dot(self.player.speed);
                if dot < 0.into() {
                    self.player.speed -= normal * dot;
                }

                if self.player.get_normal().dot(normal) > num!(0.8) {
                    on_ground = true;
                }

                let overshoot = collider.overshoot(&player_circle);

                self.player.position += overshoot;
            }
        }
        on_ground
    }

    fn physics_frame(&mut self, jump_pressed: bool) {
        let colliders = self.terrain.colliders(self.player.position);

        // work out the gravity to use
        let gravity_source = colliders
            .iter()
            .filter(|x| x.gravitational)
            .map(|collider| collider.closest_point(self.player.position))
            .min_by_key(|&closest_point| {
                (closest_point - self.player.position).magnitude_squared()
            });

        let gravity_direction = if let Some(gravity_source) = gravity_source {
            (gravity_source - self.player.position).fast_normalise()
        } else {
            (0, 0).into()
        };

        let gravity = if self.player.jump_state == JumpState::Jumping && jump_pressed {
            gravity_direction / 128
        } else if self.player.jump_state == JumpState::OnGround {
            gravity_direction / 10
        } else {
            gravity_direction / 5
        };

        self.player.speed += gravity;

        if self.handle_collider_collisions(colliders) {
            self.player.jump_state = JumpState::OnGround;
            self.player.speed *= num!(0.8);
        } else {
            self.player.speed *= num!(0.9);
        }

        if self.player.speed.magnitude_squared() < num!(0.005) {
            self.player.speed = (0, 0).into();
        }

        self.player.update_facing(-gravity_direction);

        self.player.position += self.player.speed;
    }

    fn update_camera(&mut self) {
        let camera_size = (64, 32).into();
        let target_position =
            self.player.position + self.player.get_normal() * 32 + self.player.speed * 64;
        let camera_rect = Rect::new(self.camera.position - camera_size / 2, camera_size);

        if !camera_rect.contains_point(target_position) {
            self.camera.position += (target_position - self.camera.position).fast_normalise();
        }
        if let Some(scroll_stop) = map::get_scroll_stop(
            self.camera.position.x.floor(),
            self.camera.position.y.floor(),
        ) {
            if let Some(x_min) = scroll_stop.minimum_x {
                self.camera.position.x = self.camera.position.x.max(x_min);
            }
            if let Some(y_min) = scroll_stop.minimum_y {
                self.camera.position.y = self.camera.position.y.max(y_min);
            }
            if let Some(x_max) = scroll_stop.maximum_x {
                self.camera.position.x = self.camera.position.x.min(x_max);
            }
            if let Some(y_max) = scroll_stop.maximum_y {
                self.camera.position.y = self.camera.position.y.min(y_max);
            }
        }
    }
}

impl Scene for Game {
    fn transition(&mut self, transition: &mut super::Transition) -> Option<super::TransitionScene> {
        None
    }

    fn update(&mut self, update: &mut Update) {
        let button_press = update.button_x_tri();
        self.handle_direction_input(button_press as i32);
        self.physics_frame(update.jump_pressed());

        if update.jump_just_pressed() {
            self.handle_jump_input();
        }

        self.player.frame();
        self.update_camera();

        update.set_pos(
            (self.camera.position + (num!(0.5), num!(0.5)).into()).floor()
                - (WIDTH / 2, HEIGHT / 2).into(),
        );
    }

    fn display(&mut self, display: &mut super::Display) {
        display.display(
            self.player.sprite(),
            &self.player.angle,
            self.player.rendered_position() - self.camera.position
                + (num!(0.5), num!(0.5)).into()
                + (WIDTH / 2, HEIGHT / 2).into(),
            self.player.facing != PlayerFacing::Right,
        );
    }
}

struct Terrain {
    // todo
}

impl Terrain {
    fn colliders(&self, position: Vector2D<Number>) -> &'static [&'static Collider] {
        map::get_nearby(position.x.floor(), position.y.floor())
    }
}
