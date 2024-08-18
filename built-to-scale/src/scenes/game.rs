use core::cmp::Ordering;

use agb::{
    display::{
        affine::AffineMatrix,
        object::Sprite,
        tiled::{AffineMap, TiledMap, VRamManager},
    },
    fixnum::{num, Rect, Vector2D},
};

use alloc::vec::Vec;
use util::{Circle, Collider, Line, Number};

use crate::resources;

use super::{Scene, Update};

struct Offset {
    space: AffineMatrix,
}
impl Offset {
    fn center(&self) -> Vector2D<Number> {
        (self.space.x, self.space.y).into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlayerFacing {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlayerState {
    OnGround,
    Jumping,
    Falling,
}

struct Player {
    angle: AffineMatrix,
    speed: Vector2D<Number>,
    position: Vector2D<Number>,
    state: PlayerState,
    facing: PlayerFacing,

    frame: usize,
}

const JUMP_SPEED: i32 = 5;

impl Player {
    fn set_angle_from_normal(&mut self, normal: Vector2D<Number>) {
        self.angle = AffineMatrix {
            a: -normal.y,
            b: normal.x,
            c: -normal.x,
            d: -normal.y,
            x: 0.into(),
            y: 0.into(),
        };
    }

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
                Vector2D::new(0.into(), Number::new(x) / 20)
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

            self.speed += normal * JUMP_SPEED;
            self.position += self.speed;

            self.state = PlayerState::Jumping;
            self.frame = 0;
        }
    }

    fn rendered_position(&self) -> Vector2D<Number> {
        self.position - (8, 8).into()
    }

    fn frame(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        if self.state == PlayerState::Jumping && self.frame > 32 {
            self.state = PlayerState::Falling;
        }
    }

    fn is_on_ground(&self) -> bool {
        self.state == PlayerState::OnGround
    }

    fn sprite(&self) -> &'static Sprite {
        match self.state {
            PlayerState::OnGround => {
                if self.speed.magnitude_squared() < num!(0.1) {
                    resources::IDLE.sprite(0)
                } else {
                    resources::WALK.animation_sprite(self.frame / 16)
                }
            }
            PlayerState::Jumping => resources::JUMP.animation_sprite(self.frame / 16),
            PlayerState::Falling => resources::FALL.sprite(0),
        }
    }
}

pub struct Game {
    screen_space_offset: Offset,
    player: Player,
    terrain: Terrain,

    has_drawn_background: bool,
}

const CHUNK_SIZE: i32 = 8;

impl Game {
    pub fn new() -> Self {
        Self {
            screen_space_offset: Offset {
                space: AffineMatrix::identity(),
            },
            player: Player {
                angle: AffineMatrix::identity(),
                speed: (0, 0).into(),
                position: (0, 0).into(),
                state: PlayerState::Falling,
                facing: PlayerFacing::Right,

                frame: 0,
            },
            terrain: Terrain {},
            has_drawn_background: false,
        }
    }

    fn handle_direction_input(&mut self, x: i32) {
        self.player.handle_direction_input(x);
    }

    fn handle_jump_input(&mut self) {
        self.player.handle_jump_input();
    }

    fn speculate_collision_with_displacement(
        &self,
        colliders: &[&Collider],
        displacement: Vector2D<Number>,
    ) -> bool {
        let player_circle = Circle {
            position: self.player.position + displacement,
            radius: 8.into(),
        };
        for collider in colliders {
            if collider.collides_circle(&player_circle) {
                return true;
            }
        }

        false
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
                self.player.set_angle_from_normal(normal);

                self.player.speed -= normal * normal.dot(self.player.speed);
                let overshoot = collider.overshoot(&player_circle);
                on_ground = true;

                self.player.position += overshoot;
            }
        }
        on_ground
    }

    fn physics_frame(&mut self) {
        let mut colliders = self.terrain.colliders(self.player.position);

        // put the circles first
        colliders.sort_unstable_by(|a, b| match (a, b) {
            (Collider::Circle(_), _) => Ordering::Less,
            (_, Collider::Circle(_)) => Ordering::Greater,
            (_, _) => Ordering::Equal,
        });

        // work out the gravity to use
        let gravity_direction = (colliders
            .iter()
            .map(|collider| collider.closest_point(self.player.position))
            .min_by_key(|&closest_point| (closest_point - self.player.position).magnitude_squared())
            .unwrap_or_default()
            - self.player.position)
            .fast_normalise();

        let gravity = gravity_direction / 10;
        if !self.speculate_collision_with_displacement(&colliders, gravity) {
            self.player.speed += gravity;
        }

        if self.handle_collider_collisions(&colliders) {
            self.player.state = PlayerState::OnGround;
        }

        if !self.player.is_on_ground() {
            self.player.speed *= num!(0.9);
        } else {
            self.player.speed *= num!(0.8);
        }

        if self.player.speed.magnitude_squared() == 0.into() {
            self.player.speed = (0, 0).into();
        }

        self.player.update_facing(-gravity_direction);

        self.player.position += self.player.speed;
    }

    fn update_map(&self, (affine_map, vram): (&mut AffineMap, &mut VRamManager)) {
        // 145 = ceil(hypot(WIDTH / 2, HEIGHT / 2))
        const SCREEN_RADIUS: i32 = 145;
        let min_pos =
            self.screen_space_offset.center().floor() - (SCREEN_RADIUS, SCREEN_RADIUS).into();
        let max_pos =
            self.screen_space_offset.center().floor() + (SCREEN_RADIUS, SCREEN_RADIUS).into();

        // divide by 8 for the tile size
        let min_chunk = min_pos / 8 / CHUNK_SIZE;
        let max_chunk = max_pos / 8 / CHUNK_SIZE;

        let rendered_bounds = Rect::new(min_chunk, max_chunk - min_chunk);

        for chunk in rendered_bounds.iter() {
            let map_offset = (Vector2D::from(chunk) + max_chunk) * CHUNK_SIZE;

            self.render_chunk(
                affine_map,
                vram,
                chunk,
                Vector2D::new(
                    map_offset.x.rem_euclid(64) as u16,
                    map_offset.y.rem_euclid(64) as u16,
                ),
            );
        }

        affine_map.set_transform(self.screen_space_offset.space.try_to_background().unwrap());
    }

    fn render_chunk(
        &self,
        affine_map: &mut AffineMap,
        vram: &mut VRamManager,
        chunk: (i32, i32),
        map_offset: Vector2D<u16>,
    ) {
        let chunk = map::get_tile_chunk(chunk.0, chunk.1);

        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let pos = map_offset + (x as u16, y as u16).into();

                affine_map.set_tile(
                    vram,
                    pos,
                    &resources::bg::planets.tiles,
                    chunk[(x + y * CHUNK_SIZE) as usize],
                );
            }
        }
    }
}

impl Scene for Game {
    fn transition(&mut self, transition: &mut super::Transition) -> Option<super::TransitionScene> {
        None
    }

    fn update<'a>(&mut self, update: &'a mut Update<'a>) {
        let button_press = update.button_x_tri();
        self.handle_direction_input(button_press as i32);
        self.physics_frame();

        if update.jump_just_pressed() {
            self.handle_jump_input();
        }

        self.player.frame();

        if !self.has_drawn_background {
            self.update_map(update.affine_map());
            self.has_drawn_background = true;
        }
    }

    fn display(&mut self, display: &mut super::Display) {
        display.display(
            self.player.sprite(),
            &self.player.angle,
            self.player.rendered_position(),
            self.player.facing != PlayerFacing::Right,
        );
    }
}

struct Terrain {
    // todo
}

impl Terrain {
    fn colliders<'b>(&self, position: Vector2D<Number>) -> Vec<&'b Collider> {
        map::get_nearby(position.x.floor(), position.y.floor())
    }
}
