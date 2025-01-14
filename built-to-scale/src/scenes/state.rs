use agb::{
    display::{
        affine::AffineMatrix,
        object::{
            AffineMatrixInstance, AffineMode, OamIterator, ObjectUnmanaged, Sprite, SpriteLoader,
        },
        HEIGHT, WIDTH,
    },
    fixnum::{num, Vector2D},
    input::{Button, ButtonController, Tri},
    sound::mixer::{Mixer, SoundChannel},
};
use util::Number;

pub struct Update<'a, 'b> {
    button: &'a ButtonController,
    new_pos: Option<Vector2D<i32>>,
    mixer: &'a mut Mixer<'b>,
    play_space_music: bool,
}

impl<'a, 'b> Update<'a, 'b> {
    pub fn new(button: &'a ButtonController, mixer: &'a mut Mixer<'b>) -> Self {
        Self {
            button,
            new_pos: None,
            mixer,
            play_space_music: false,
        }
    }

    pub fn set_pos(&mut self, new_pos: Vector2D<i32>) {
        self.new_pos = Some(new_pos);
    }

    pub fn new_pos(&self) -> Option<Vector2D<i32>> {
        self.new_pos
    }

    pub fn play_space_music(&mut self) {
        self.play_space_music = true;
    }

    pub fn should_play_space_music(&self) -> bool {
        self.play_space_music
    }
}

impl Update<'_, '_> {
    pub fn button_x_tri(&self) -> Tri {
        self.button.x_tri()
    }

    pub fn jump_just_pressed(&self) -> bool {
        self.button.is_just_pressed(Button::A)
    }

    pub fn jump_pressed(&self) -> bool {
        self.button.is_pressed(Button::A)
    }

    pub fn is_dash_pressed(&self) -> bool {
        self.button.is_just_pressed(Button::B)
    }

    pub fn play_sfx(&mut self, effect: &'static [u8]) {
        self.mixer.play_sound(SoundChannel::new(effect));
    }
}

pub struct Display<'a, 'b> {
    oam_iter: OamIterator<'b>,
    sprite_loader: &'a mut SpriteLoader,
}

impl<'a, 'b> Display<'a, 'b> {
    pub fn new(oam_iter: OamIterator<'b>, sprite_loader: &'a mut SpriteLoader) -> Self {
        Self {
            oam_iter,
            sprite_loader,
        }
    }
}

impl<'a, 'b> Display<'a, 'b> {
    pub fn oam(&mut self) -> &mut OamIterator<'b> {
        &mut self.oam_iter
    }

    pub fn affine_object(
        &mut self,
        sprite: &'static Sprite,
        affine: AffineMatrix,
        position: Vector2D<Number>,
        hflip: bool,
    ) -> ObjectUnmanaged {
        let mut o = ObjectUnmanaged::new(self.sprite_loader.get_vram_sprite(sprite));

        let affine = if hflip {
            AffineMatrix::from_scale((-1, 1).into()) * affine
        } else {
            affine
        };

        let aff = AffineMatrix::from_translation(position) * affine;

        o.set_affine_matrix(AffineMatrixInstance::new(aff.to_object_wrapping()));
        o.show_affine(AffineMode::Affine);
        o.set_position(aff.position().floor());

        o
    }

    pub fn regular_object(
        &mut self,
        sprite: &'static Sprite,
        position: Vector2D<Number>,
    ) -> ObjectUnmanaged {
        let mut o = ObjectUnmanaged::new(self.sprite_loader.get_vram_sprite(sprite));

        o.show();
        o.set_position(position.floor());

        o
    }

    pub fn display(
        &mut self,
        sprite: &'static Sprite,
        affine: &AffineMatrix,
        position: Vector2D<Number>,
        hflip: bool,
    ) {
        let object = self.affine_object(
            sprite,
            *affine,
            position + (num!(0.5), num!(0.5)).into() + (WIDTH / 2, HEIGHT / 2).into(),
            hflip,
        );
        self.oam_iter.set_next(&object);
    }

    pub fn display_regular(&mut self, sprite: &'static Sprite, position: Vector2D<Number>) {
        let object = self.regular_object(
            sprite,
            position + (num!(0.5), num!(0.5)).into() + (WIDTH / 2, HEIGHT / 2).into(),
        );
        self.oam_iter.set_next(&object);
    }
}

pub struct Transition {}
