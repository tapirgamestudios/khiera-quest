mod powerups;

use agb::{
    display::{affine::AffineMatrix, object::Sprite, HEIGHT, WIDTH},
    fixnum::{num, Rect, Vector2D},
};

use alloc::vec::Vec;
use map::PowerUpKind;
use powerups::PowerUpObject;
use util::{Circle, Collider, Number};

use crate::resources::{self, BUBBLE, BUBBLE_POP};

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
    HasJump,
    Jumping,
    Falling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GroundState {
    OnGround,
    InAir,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DashState {
    Available,
    Used,
}

struct RecoveringState {
    recover_to: Vector2D<Number>,
    starting_from: Vector2D<Number>,
    starting_reverse_local_gravity: Vector2D<Number>,
    destination_reverse_local_gravity: Vector2D<Number>,
    time: u32,
}

enum PlayerState {
    Playing {
        remaining_pop_time: u32,
        pop_location: Vector2D<Number>,
    },
    Recovering(RecoveringState),
}

struct Player {
    // the angle the player is facing, corresponding to local gravity
    angle: AffineMatrix,
    // the normal of the surface the player is on, not the gravity source
    surface_normal: Vector2D<Number>,
    speed: Vector2D<Number>,
    position: Vector2D<Number>,
    facing: PlayerFacing,
    ground_state: GroundState,
    ground_speed: Number,
    air_speed: Number,

    can_dash: bool,
    dash_state: DashState,
    max_jumps: usize,
    jumps_remaining: usize,
    jump_state: JumpState,
    jump_speed: Number,

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

    fn handle_direction_input(&mut self, x: i32, is_dashing: bool) {
        if x != 0 {
            let dash =
                if self.can_dash && is_dashing && self.dash_state == DashState::Available && x != 0
                {
                    self.dash_state = DashState::Used;
                    num!(3.)
                } else {
                    num!(0.)
                };

            let (acceleration, normal) = if self.is_on_ground() {
                if self.surface_normal.dot(self.get_normal()) > num!(0.7) {
                    (
                        Vector2D::new(0.into(), Number::new(x)) * (self.ground_speed + dash),
                        self.surface_normal,
                    )
                } else {
                    (
                        Vector2D::new(0.into(), Number::new(x)) * (self.ground_speed + dash),
                        self.get_normal(),
                    )
                }
            } else {
                (
                    Vector2D::new(0.into(), Number::new(x)) * (self.air_speed + dash),
                    self.get_normal(),
                )
            };

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
        if self.jump_state == JumpState::HasJump {
            let normal = self.get_normal();

            let dot = self.speed.dot(normal);
            if dot < 0.into() {
                self.speed -= normal * dot;
            }
            self.speed += normal * self.jump_speed;

            self.position += self.speed;

            self.jump_state = JumpState::Jumping;
            self.jumps_remaining -= 1;
            self.frame = 0;
        }
    }

    fn rendered_position(&self) -> Vector2D<Number> {
        self.position - (8, 8).into()
    }

    fn frame(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        if self.jump_state == JumpState::Jumping && self.frame > 32 {
            if self.jumps_remaining > 0 {
                self.jump_state = JumpState::HasJump;
            } else {
                self.jump_state = JumpState::Falling;
            }
        }
    }

    fn is_on_ground(&self) -> bool {
        self.ground_state == GroundState::OnGround
    }

    fn sprite(&self) -> &'static Sprite {
        match self.jump_state {
            JumpState::HasJump => {
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

    fn apply_powerup(&mut self, powerup: PowerUpKind) {
        match powerup {
            PowerUpKind::JumpBoost => {
                self.jump_speed = num!(3.5);
            }
            PowerUpKind::SpeedBoost => self.can_dash = true,
        }
    }
}

pub struct Game {
    camera: Camera,
    player: Player,
    terrain: Terrain,
    last_gravity_source: Option<&'static Collider>,
    player_state: PlayerState,

    powerups: Vec<PowerUpObject>,
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
                ground_state: GroundState::InAir,
                surface_normal: (0, 0).into(),

                jump_speed: num!(2.2),
                ground_speed: num!(0.25),
                air_speed: num!(0.0625),

                can_dash: false,
                dash_state: DashState::Available,
                jumps_remaining: 1,
                max_jumps: 1,

                frame: 0,
            },
            last_gravity_source: None,
            player_state: PlayerState::Playing {
                remaining_pop_time: 0,
                pop_location: (0, 0).into(),
            },

            terrain: Terrain {},

            powerups: map::POWER_UPS.iter().map(PowerUpObject::new).collect(),
        }
    }

    fn handle_direction_input(&mut self, x: i32, is_dashing: bool) {
        self.player.handle_direction_input(x, is_dashing);
    }

    fn handle_jump_input(&mut self) {
        self.player.handle_jump_input();
    }

    fn handle_player_death(&mut self) {
        let point_to_recover_to = map::get_recovery_point(self.player.position);
        self.player_state = PlayerState::Recovering(RecoveringState {
            recover_to: point_to_recover_to,
            starting_from: self.player.position,
            starting_reverse_local_gravity: (self.player.position
                - self
                    .last_gravity_source
                    .unwrap()
                    .closest_point(self.player.position))
            .fast_normalise(),
            destination_reverse_local_gravity: (point_to_recover_to
                - get_gravity_source(
                    self.terrain.colliders(point_to_recover_to),
                    point_to_recover_to,
                )
                .1)
                .fast_normalise(),
            time: 0,
        });
    }

    /// returns the cosine of the smallest angle of collision if there is one. So None = not touching the ground
    fn handle_collider_collisions(&mut self, colliders: &[&Collider]) -> Option<Number> {
        let mut max_angle = None;

        for collider in colliders {
            let player_circle = Circle {
                position: self.player.position,
                radius: 8.into(),
            };
            if collider.collides_circle(&player_circle) {
                if collider.tag.is_kills_player() {
                    self.handle_player_death();
                } else if collider.tag.is_collision() {
                    let normal = collider.normal_circle(&player_circle);

                    self.player.surface_normal = normal;

                    let dot = normal.dot(self.player.speed);
                    if dot < 0.into() {
                        self.player.speed -= normal * dot;
                    }

                    let cosine_of_floor_angle = self.player.get_normal().dot(normal);
                    // 0.7 is approximately sqrt(2) / 2 which is about 45 degrees
                    max_angle = Some(max_angle.unwrap_or(num!(-1.)).max(cosine_of_floor_angle));

                    let overshoot = collider.overshoot(&player_circle);

                    self.player.position += overshoot;
                }
            }
        }

        max_angle
    }

    fn get_gravity_source(&mut self, colliders: &[&'static Collider]) -> Vector2D<Number> {
        if colliders.is_empty() {
            let source = self
                .last_gravity_source
                .expect("We should have a gravity source if we're in empty space");
            source.closest_point(self.player.position)
        } else {
            let (gravity_source_collider, gravity_source_position) =
                get_gravity_source(colliders, self.player.position);

            self.last_gravity_source = Some(gravity_source_collider);
            gravity_source_position
        }
    }

    fn physics_frame(&mut self, jump_pressed: bool) {
        let colliders: &[&Collider] = self.terrain.colliders(self.player.position);

        let gravity_source = self.get_gravity_source(colliders);

        let gravity_direction = (gravity_source - self.player.position).fast_normalise();

        let gravity = if self.player.jump_state == JumpState::Jumping && jump_pressed {
            gravity_direction / 128
        } else {
            gravity_direction / 10
        };

        self.player.speed += gravity;

        self.player.ground_state = match self.handle_collider_collisions(colliders) {
            Some(value) => {
                if value > num!(0.8) {
                    // approximately < 45 degree angle. So definitely on the ground
                    self.player.jump_state = JumpState::HasJump;
                    self.player.dash_state = DashState::Available;
                    self.player.jumps_remaining = self.player.max_jumps;

                    // Apply a reasonably high amount of friction
                    self.player.speed *= num!(0.8);

                    GroundState::OnGround
                } else if value > num!(0.7) {
                    // just over 45 degrees (since 0.7 ~= sqrt(2) / 2). Still should be considered ground, but apply less friction
                    self.player.jump_state = JumpState::HasJump; // should allow for another jump
                    self.player.dash_state = DashState::Available;
                    self.player.jumps_remaining = self.player.max_jumps;
                    self.player.speed *= num!(0.90);

                    GroundState::OnGround
                } else {
                    // hit something which isn't floor-like
                    self.player.speed *= num!(0.95);
                    GroundState::InAir
                }
            }
            None => {
                // hit something which isn't floor-like
                self.player.speed *= num!(0.95);
                GroundState::InAir
            }
        };

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

        let camera_destination = if !camera_rect.contains_point(target_position) {
            self.camera.position
                + (target_position - self.camera.position).fast_normalise() * num!(1.25)
        } else {
            self.camera.position
        };

        let camera_destination = if let Some(scroll_stop) = map::get_scroll_stop(
            self.camera.position.x.floor(),
            self.camera.position.y.floor(),
        ) {
            let mut camera_destination = camera_destination;
            if let Some(x_min) = scroll_stop.minimum_x {
                if self.camera.position.x >= x_min {
                    camera_destination.x = camera_destination.x.max(x_min);
                }
            }
            if let Some(y_min) = scroll_stop.minimum_y {
                if self.camera.position.y >= y_min {
                    camera_destination.y = camera_destination.y.max(y_min);
                }
            }
            if let Some(x_max) = scroll_stop.maximum_x {
                if self.camera.position.x <= x_max {
                    camera_destination.x = camera_destination.x.min(x_max);
                }
            }
            if let Some(y_max) = scroll_stop.maximum_y {
                if self.camera.position.y <= y_max {
                    camera_destination.y = camera_destination.y.min(y_max);
                }
            }

            camera_destination
        } else {
            camera_destination
        };

        self.camera.position = camera_destination;
    }
}

fn get_gravity_source(
    colliders: &[&'static Collider],
    position: Vector2D<Number>,
) -> (&'static Collider, Vector2D<Number>) {
    colliders
        .iter()
        .copied()
        .filter(|x| x.tag.is_gravitational())
        .map(|collider| (collider, collider.closest_point(position)))
        .min_by_key(|&(_, closest_point)| (closest_point - position).magnitude_squared())
        .unwrap()
}

impl Scene for Game {
    fn transition(&mut self, transition: &mut super::Transition) -> Option<super::TransitionScene> {
        None
    }

    fn update(&mut self, update: &mut Update) {
        match &mut self.player_state {
            PlayerState::Playing {
                remaining_pop_time, ..
            } => {
                *remaining_pop_time = remaining_pop_time.saturating_sub(1);

                let button_press = update.button_x_tri();
                self.handle_direction_input(button_press as i32, update.is_dash_pressed());
                self.physics_frame(update.jump_pressed());

                if update.jump_just_pressed() {
                    self.handle_jump_input();
                }

                self.player.frame();
            }
            PlayerState::Recovering(recover) => {
                recover.time += 1;
                match recover.time {
                    0..16 => {
                        self.player.speed = (0, 0).into();
                    }
                    16..64 => {
                        let time = recover.time as i32 - 16;
                        let time = Number::new(time) / (64 - 16);

                        let start_position_line = recover.starting_from
                            + recover.starting_reverse_local_gravity * time * 30;
                        let ending_position_line = recover.recover_to
                            + recover.destination_reverse_local_gravity * (-time + 1) * 30;

                        let position =
                            start_position_line * (-time + 1) + ending_position_line * time;

                        self.player.position = position;
                    }
                    64..80 => {}
                    80.. => {
                        self.player_state = PlayerState::Playing {
                            remaining_pop_time: BUBBLE_POP.sprites().len() as u32 * 2,
                            pop_location: self.player.position,
                        }
                    }
                }
            }
        }

        self.update_camera();
        update.set_pos(
            (self.camera.position + (num!(0.5), num!(0.5)).into()).floor()
                - (WIDTH / 2, HEIGHT / 2).into(),
        );

        self.powerups.retain_mut(|powerup| {
            if let Some(powerup) = powerup.update(self.player.position) {
                self.player.apply_powerup(powerup);

                return false;
            }

            true
        });
    }

    fn display(&mut self, display: &mut super::Display) {
        match &self.player_state {
            PlayerState::Playing {
                remaining_pop_time,
                pop_location,
            } => {
                display.display(
                    self.player.sprite(),
                    &self.player.angle,
                    self.player.rendered_position() - self.camera.position
                        + (num!(0.5), num!(0.5)).into()
                        + (WIDTH / 2, HEIGHT / 2).into(),
                    self.player.facing != PlayerFacing::Right,
                );

                if *remaining_pop_time > 0 {
                    let idx =
                        ((BUBBLE_POP.sprites().len() as i32 * 2 - *remaining_pop_time as i32) / 2)
                            .try_into()
                            .unwrap_or_default();
                    display.display_regular(
                        BUBBLE_POP.animation_sprite(idx),
                        *pop_location - (16, 16).into() - self.camera.position
                            + (num!(0.5), num!(0.5)).into()
                            + (WIDTH / 2, HEIGHT / 2).into(),
                    );
                }
            }
            PlayerState::Recovering(state) => {
                let idx = state.time as usize / 2;
                display.display_regular(
                    BUBBLE.animation_sprite(idx),
                    self.player.position - (16, 16).into() - self.camera.position
                        + (num!(0.5), num!(0.5)).into()
                        + (WIDTH / 2, HEIGHT / 2).into(),
                );
            }
        }

        for powerup in self.powerups.iter() {
            powerup.display(self.camera.position, display);
        }
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
