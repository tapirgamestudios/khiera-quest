use agb::{
    display::affine::AffineMatrix,
    fixnum::{num, Vector2D},
};
use util::{Circle, Collider, Number};

use crate::resources;

use super::Scene;

struct Offset {
    space: AffineMatrix,
}

struct Player {
    angle: AffineMatrix,
    speed: Vector2D<Number>,
    position: Vector2D<Number>,
    on_ground: bool,
}

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

    fn get_normal(&self) -> Vector2D<Number> {
        (self.angle.b, -self.angle.a).into()
    }

    fn handle_direction_input(&mut self, x: i32) {
        if x != 0 {
            let acceleration: Vector2D<Number> = Vector2D::new(0.into(), Number::new(x) / 8);
            let normal = self.get_normal();

            let rotated_acceleration = (
                normal.x * acceleration.x - normal.y * acceleration.y,
                normal.y * acceleration.x + normal.x * acceleration.y,
            )
                .into();

            agb::println!("{:?}", rotated_acceleration);

            self.speed += rotated_acceleration;
        }
    }
}

pub struct Game {
    screen_space_offset: Offset,
    player: Player,
    terrain: Terrain,
}

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
                on_ground: false,
            },
            terrain: Terrain {},
        }
    }

    fn handle_direction_input(&mut self, x: i32) {
        self.player.handle_direction_input(x);
    }

    fn handle_jump_input(&mut self) {
        //todo
    }

    fn handle_collider_collisions(&mut self) -> bool {
        let colliders = self.terrain.colliders(self.player.position);
        let player_circle = Circle {
            position: self.player.position,
            radius: 8.into(),
        };

        for collider in colliders {
            if collider.collides_circle(&player_circle) {
                let normal = collider.normal_circle(&player_circle);
                self.player.set_angle_from_normal(normal);

                self.player.speed -= normal * normal.dot(self.player.speed);
                self.player.speed *= num!(0.8);
                return true;
            }
        }
        false
    }

    fn physics_frame(&mut self) {
        // todo, set the player angle
        let gravity = self.terrain.gravity(self.player.position);

        self.player.speed += gravity;

        self.player.on_ground = self.handle_collider_collisions();

        self.player.position += self.player.speed;
    }
}

impl Scene for Game {
    fn transition(&mut self, transition: &mut super::Transition) -> Option<super::TransitionScene> {
        None
    }

    fn update(&mut self, update: &mut super::Update) {
        let button_press = update.button_x_tri();
        self.handle_direction_input(button_press as i32);
        self.physics_frame();
    }

    fn display(&mut self, display: &mut super::Display) {
        display.display(
            resources::IDLE.sprite(0),
            &self.player.angle,
            self.player.position,
        );
    }
}

struct Terrain {
    // todo
}

impl Terrain {
    fn gravity(&self, position: Vector2D<Number>) -> Vector2D<Number> {
        (Vector2D::<Number>::from((100, 100)) - position).fast_normalise()
    }

    fn colliders(&self, position: Vector2D<Number>) -> impl Iterator<Item = Collider> {
        [Collider::Circle(Circle {
            position: (100, 100).into(),
            radius: 32.into(),
        })]
        .into_iter()
    }
}