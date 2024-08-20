mod powerups;

use core::fmt::Write;

use agb::{
    display::{
        affine::AffineMatrix,
        object::{ObjectTextRender, PaletteVram, Size, Sprite, TextAlignment},
        HEIGHT, WIDTH,
    },
    fixnum::{num, Num, Rect, Vector2D},
};

use alloc::vec::Vec;
use map::{Path, PowerUpKind};
use powerups::PowerUpObject;
use util::{Circle, Collider, Number};

use crate::{
    mission_logs::MISSION_LOGS,
    resources::{self, BUBBLE, BUBBLE_POP, FONT, TEXT_PALETTE},
};

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

    fn handle_direction_input(&mut self, x: i32, is_dashing: bool, update: &mut Update) {
        if x != 0 {
            let dash =
                if self.can_dash && is_dashing && self.dash_state == DashState::Available && x != 0
                {
                    self.dash_state = DashState::Used;
                    update.play_sfx(resources::DASH_SOUND);
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

    /// returns whether or not the jump actually happened
    fn handle_jump_input(&mut self) -> bool {
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

            return true;
        }

        false
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
            PowerUpKind::Dash => self.can_dash = true,
            PowerUpKind::DoubleJump => {
                self.max_jumps += 1;
                self.jumps_remaining += 1;
            }
        }
    }
}

pub struct Game {
    game: GamePart,
    terrain: Terrain,
    mission_log: MissionLogPlayer,
}

struct GamePart {
    camera: Camera,
    player: Player,
    last_gravity_source: Option<Collider>,
    player_state: PlayerState,

    powerups: Vec<PowerUpObject>,
}

impl GamePart {
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

            powerups: map::POWER_UPS.iter().map(PowerUpObject::new).collect(),
        }
    }

    fn handle_direction_input(&mut self, x: i32, is_dashing: bool, update: &mut Update) {
        self.player.handle_direction_input(x, is_dashing, update);
    }

    /// returns whether or not the jump actually happened
    fn handle_jump_input(&mut self) -> bool {
        self.player.handle_jump_input()
    }

    fn handle_player_death(&mut self, update: &mut Update, terrain: &Terrain) {
        update.play_sfx(resources::RECOVERY_SOUND);

        let point_to_recover_to = map::get_recovery_point(self.player.position);
        self.player_state = PlayerState::Recovering(RecoveringState {
            recover_to: point_to_recover_to,
            starting_from: self.player.position,
            starting_reverse_local_gravity: (self.player.position
                - self
                    .last_gravity_source
                    .as_ref()
                    .unwrap()
                    .closest_point(self.player.position))
            .fast_normalise(),
            destination_reverse_local_gravity: (point_to_recover_to
                - get_gravity_source(terrain.colliders(point_to_recover_to), point_to_recover_to)
                    .1)
                .fast_normalise(),
            time: 0,
        });
    }

    /// returns the cosine of the smallest angle of collision if there is one. So None = not touching the ground
    fn handle_collider_collisions(
        &mut self,
        update: &mut Update,
        colliders: DynamicAndStaticColliders,
        terrain: &Terrain,
    ) -> Option<Number> {
        let mut max_angle = None;

        for collider in colliders.iter() {
            let player_circle = Circle {
                position: self.player.position,
                radius: 8.into(),
            };
            if collider.collides_circle(&player_circle) {
                if collider.tag.is_kills_player() {
                    self.handle_player_death(update, terrain);
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

                    self.player.position += overshoot + collider.velocity;
                }
            }
        }

        max_angle
    }

    fn get_gravity_source(&mut self, colliders: DynamicAndStaticColliders) -> Vector2D<Number> {
        if colliders.is_empty() {
            let source = self
                .last_gravity_source
                .as_ref()
                .expect("We should have a gravity source if we're in empty space");
            source.closest_point(self.player.position)
        } else {
            let (gravity_source_collider, gravity_source_position) =
                get_gravity_source(colliders, self.player.position);

            self.last_gravity_source = Some(gravity_source_collider.clone());
            gravity_source_position
        }
    }

    fn physics_frame(&mut self, update: &mut Update, terrain: &Terrain) {
        let colliders = terrain.colliders(self.player.position);
        let gravity_source = self.get_gravity_source(colliders);

        let gravity_direction = (gravity_source - self.player.position).fast_normalise();

        let gravity = if self.player.jump_state == JumpState::Jumping && update.jump_pressed() {
            gravity_direction / 128
        } else {
            gravity_direction / 10
        };

        let old_speed = self.player.speed;
        let was_on_ground = self.player.is_on_ground();

        self.player.speed += gravity;
        self.player.position += self.player.speed;

        self.player.ground_state = match self.handle_collider_collisions(update, colliders, terrain)
        {
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

        let is_on_ground = self.player.is_on_ground();
        if !was_on_ground && is_on_ground && old_speed.dot(gravity_direction) > 1.into() {
            update.play_sfx(resources::LAND_GROUND);
        }

        if self.player.speed.magnitude_squared() < num!(0.005) {
            self.player.speed = (0, 0).into();
        }

        self.player.update_facing(-gravity_direction);
    }

    fn update_camera(&mut self) {
        let camera_size = (64, 32).into();
        let target_position =
            self.player.position + self.player.get_normal() * 32 + self.player.speed * 64;
        let camera_rect = Rect::new(self.camera.position - camera_size / 2, camera_size);

        let camera_destination = if !camera_rect.contains_point(target_position) {
            let mut offset = (target_position - self.camera.position) / 60;
            if offset.magnitude_squared() > (6 * 6).into() {
                offset = offset.normalise() * 6;
            }

            self.camera.position + offset
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
    colliders: DynamicAndStaticColliders<'_>,
    position: Vector2D<Number>,
) -> (&Collider, Vector2D<Number>) {
    colliders
        .iter()
        .filter(|x| x.tag.is_gravitational())
        .map(|collider| (collider, collider.closest_point(position)))
        .min_by_key(|&(_, closest_point)| (closest_point - position).magnitude_squared())
        .unwrap()
}

impl GamePart {
    fn update(&mut self, update: &mut Update, terrain: &Terrain) {
        match &mut self.player_state {
            PlayerState::Playing {
                remaining_pop_time, ..
            } => {
                *remaining_pop_time = remaining_pop_time.saturating_sub(1);

                let button_press = update.button_x_tri();
                self.handle_direction_input(button_press as i32, update.is_dash_pressed(), update);
                self.physics_frame(update, terrain);

                if update.jump_just_pressed() && self.handle_jump_input() {
                    update.play_sfx(resources::JUMP_SOUND);
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
            if let Some(powerup) = powerup.update(self.player.position, update) {
                self.player.apply_powerup(powerup);

                return false;
            }

            true
        });

        if self.player.position.y < (-140).into() {
            update.play_space_music();
        }
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
                    self.player.rendered_position() - self.camera.position,
                    self.player.facing != PlayerFacing::Right,
                );

                if *remaining_pop_time > 0 {
                    let idx =
                        ((BUBBLE_POP.sprites().len() as i32 * 2 - *remaining_pop_time as i32) / 2)
                            .try_into()
                            .unwrap_or_default();
                    display.display_regular(
                        BUBBLE_POP.animation_sprite(idx),
                        *pop_location - (16, 16).into() - self.camera.position,
                    );
                }
            }
            PlayerState::Recovering(state) => {
                let idx = state.time as usize / 2;
                display.display_regular(
                    BUBBLE.animation_sprite(idx),
                    self.player.position - (16, 16).into() - self.camera.position,
                );
            }
        }

        for powerup in self.powerups.iter() {
            powerup.display(self.camera.position, display);
        }
    }
}

impl Game {
    pub fn new() -> Self {
        Self {
            game: GamePart::new(),
            terrain: Terrain {
                loaded_dynamic_colliders: Vec::new(),
            },
            mission_log: MissionLogPlayer::new(),
        }
    }
}

impl Scene for Game {
    fn transition(&mut self, transition: &mut super::Transition) -> Option<super::TransitionScene> {
        None
    }

    fn update(&mut self, update: &mut Update) {
        self.terrain.update(self.game.player.position);
        self.game.update(update, &self.terrain);
        self.mission_log.update(self.game.player.position);
    }

    fn display(&mut self, display: &mut super::Display) {
        self.game.display(display);
        self.mission_log.display(display);
        self.terrain.display(display, self.game.camera.position);
    }
}

#[derive(Copy, Clone)]
struct DynamicAndStaticColliders<'a> {
    static_colliders: &'static [&'static Collider],
    dynamic_colliders: &'a [DynamicCollider],
}

impl<'a> DynamicAndStaticColliders<'a> {
    fn iter(&self) -> impl Iterator<Item = &'a Collider> {
        self.static_colliders.iter().copied().chain(
            self.dynamic_colliders
                .iter()
                .flat_map(|x| x.colliders.iter()),
        )
    }

    fn is_empty(&self) -> bool {
        self.static_colliders.is_empty()
            && !self
                .dynamic_colliders
                .iter()
                .any(|x| x.colliders[0].tag.is_gravitational())
    }
}

struct DynamicCollider {
    path: &'static Path,
    current_path_element_idx: usize,
    current_position: Vector2D<Number>,
    colliders: Vec<Collider>,
    direction: PathDirection,
    path_index_timer: Num<i32, 24>,
}

enum PathDirection {
    Forwards,
    Backwards,
}

struct Terrain {
    loaded_dynamic_colliders: Vec<DynamicCollider>,
}

impl Terrain {
    fn colliders(&self, position: Vector2D<Number>) -> DynamicAndStaticColliders {
        DynamicAndStaticColliders {
            static_colliders: map::get_nearby(position.x.floor(), position.y.floor()),
            dynamic_colliders: &self.loaded_dynamic_colliders,
        }
    }

    fn load_paths(&mut self, player_position: Vector2D<Number>) {
        let should_be_loaded_paths =
            map::get_paths(player_position.x.floor(), player_position.y.floor());
        // remove non active paths
        self.loaded_dynamic_colliders.retain(|x| {
            should_be_loaded_paths
                .iter()
                .any(|&p| core::ptr::eq(x.path, p))
        });
        let paths_to_load: Vec<_> = should_be_loaded_paths
            .iter()
            .copied()
            .filter(|&path| {
                !self
                    .loaded_dynamic_colliders
                    .iter()
                    .any(|x| core::ptr::eq(x.path, path))
            })
            .collect();

        // load now active paths
        for to_be_loaded in paths_to_load {
            let colliders = to_be_loaded.colliders.to_vec();
            self.loaded_dynamic_colliders.push(DynamicCollider {
                path: to_be_loaded,
                current_path_element_idx: 0,
                colliders,
                current_position: to_be_loaded.points[0].point,
                path_index_timer: 0.into(),
                direction: PathDirection::Forwards,
            });
        }
    }

    fn update_paths(&mut self) {
        for loaded in self.loaded_dynamic_colliders.iter_mut() {
            let (from, to, frames) = match loaded.direction {
                PathDirection::Forwards => {
                    let from = &loaded.path.points[loaded.current_path_element_idx];
                    let to = &loaded.path.points
                        [(loaded.current_path_element_idx + 1) % loaded.path.points.len()];
                    (from.point, to.point, from.incrementer)
                }
                PathDirection::Backwards => {
                    let from = &loaded.path.points[loaded.current_path_element_idx];
                    let to = &loaded.path.points[(loaded.current_path_element_idx as isize - 1)
                        .rem_euclid(loaded.path.points.len() as isize)
                        as usize];
                    (from.point, to.point, to.incrementer)
                }
            };

            loaded.path_index_timer += frames;
            if loaded.path_index_timer >= 1.into() {
                loaded.path_index_timer -= 1;
                match loaded.direction {
                    PathDirection::Forwards => {
                        loaded.current_path_element_idx += 1;
                        if !loaded.path.complete {
                            if loaded.current_path_element_idx == loaded.path.points.len() - 1 {
                                loaded.direction = PathDirection::Backwards;
                            }
                        } else if loaded.current_path_element_idx == loaded.path.points.len() {
                            loaded.current_path_element_idx = 0;
                        }
                    }
                    PathDirection::Backwards => {
                        loaded.current_path_element_idx -= 1;
                        if loaded.current_path_element_idx == 0 {
                            loaded.direction = PathDirection::Forwards;
                        }
                    }
                }
            } else {
                let next_position = from * (-loaded.path_index_timer + 1).change_base()
                    + to * loaded.path_index_timer.change_base();
                let velocity = next_position - loaded.current_position;
                loaded.current_position += velocity;
                for collider in loaded.colliders.iter_mut() {
                    collider.apply_velocity(velocity);
                }
            }
        }
    }

    fn update(&mut self, player_position: Vector2D<Number>) {
        self.load_paths(player_position);
        self.update_paths();
    }

    fn display(&self, display: &mut super::Display, camera_position: Vector2D<Number>) {
        let camera = Rect::new(
            camera_position - (WIDTH / 2 + 32, HEIGHT / 2 + 32).into(),
            (WIDTH + 64, HEIGHT + 64).into(),
        );
        for collider in self.loaded_dynamic_colliders.iter() {
            if camera.contains_point(collider.current_position) {
                let image = convert_sprite(collider.path.image);
                let image_size = image.size().to_width_height();
                let image_size = Vector2D::new(image_size.0 as i32, image_size.1 as i32);
                display.display_regular(
                    image,
                    collider.current_position - camera_position - image_size.change_base() / 2,
                );
            }
        }
    }
}

fn convert_sprite(sprite: map::DynamicColliderImage) -> &'static Sprite {
    match sprite {
        map::DynamicColliderImage::SLIME_MOON => resources::SLIME_MOON.sprite(0),
        map::DynamicColliderImage::PLATFORM_UP_RIGHT => resources::PLATFORM_UP_RIGHT.sprite(0),
        // map::DynamicColliderImage::PLATFORM_VERTICAL => resources::PLATFORM_VERTICAL.sprite(0),
        map::DynamicColliderImage::SQUARE_PLATFORM => resources::SQUARE_PLATFORM.sprite(0),
        map::DynamicColliderImage::ASTEROID1 => resources::ASTEROID1.sprite(0),
        map::DynamicColliderImage::ASTEROID2 => resources::ASTEROID2.sprite(0),
        map::DynamicColliderImage::ASTEROID3 => resources::ASTEROID3.sprite(0),
        map::DynamicColliderImage::DIAGONAL_PLATFORM => resources::DIAGONAL_PLATFORM.sprite(0),
    }
}

struct MissionLogPlayer {
    playing_mission_log: Option<ObjectTextRender<'static>>,
    currently_playing_mission_log_timer: u32,
    encountered_mission_logs: u32,
    palette: PaletteVram,
}

impl MissionLogPlayer {
    fn new() -> Self {
        Self {
            playing_mission_log: None,
            currently_playing_mission_log_timer: 0,
            encountered_mission_logs: 0,
            palette: PaletteVram::new(&TEXT_PALETTE).unwrap(),
        }
    }

    fn update(&mut self, player_position: Vector2D<Number>) {
        let floored = player_position.floor();
        if let Some(playing_log) = self.playing_mission_log.as_mut() {
            playing_log.next_letter_group();
            playing_log.update((WIDTH / 4, HEIGHT / 4));
            self.currently_playing_mission_log_timer += 1;
            if self.currently_playing_mission_log_timer > 8 * 60 {
                self.playing_mission_log = None;
            }
        } else {
            let active = MISSION_LOGS.iter().enumerate().find(|(idx, x)| {
                self.encountered_mission_logs & (1 << idx) == 0
                    && (x.point - floored).magnitude_squared() < 64 * 64
            });
            if let Some((idx, log)) = active {
                // mark as encountered
                self.encountered_mission_logs |= 1 << idx;

                self.currently_playing_mission_log_timer = 0;

                let mut render = ObjectTextRender::new(&FONT, Size::S32x32, self.palette.clone());

                let _ = render.write_str(log.text);
                let _ = render.write_char('\n');

                render.layout((WIDTH / 2, HEIGHT), TextAlignment::Left, 3);
                self.playing_mission_log = Some(render);
            }
        }
    }

    fn display(&mut self, display: &mut super::Display) {
        if let Some(render) = self.playing_mission_log.as_mut() {
            render.commit(display.oam());
        }
    }
}
