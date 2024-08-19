use agb::{
    display::{
        affine::AffineMatrix,
        object::{
            AffineMatrixInstance, AffineMode, OamIterator, ObjectUnmanaged, Sprite, SpriteLoader,
        },
    },
    fixnum::Vector2D,
    input::{Button, ButtonController, Tri},
};
use util::Number;

pub struct Update<'a> {
    button: &'a ButtonController,
    new_pos: Option<Vector2D<i32>>,
}

impl<'a> Update<'a> {
    pub fn new(button: &'a ButtonController) -> Self {
        Self {
            button,
            new_pos: None,
        }
    }

    pub fn set_pos(&mut self, new_pos: Vector2D<i32>) {
        self.new_pos = Some(new_pos);
    }

    pub fn new_pos(&self) -> Option<Vector2D<i32>> {
        self.new_pos
    }
}

impl Update<'_> {
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

impl Display<'_, '_> {
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
        let object = self.affine_object(sprite, *affine, position, hflip);
        self.oam_iter.set_next(&object);
    }

    pub fn display_regular(&mut self, sprite: &'static Sprite, position: Vector2D<Number>) {
        let object = self.regular_object(sprite, position);
        self.oam_iter.set_next(&object);
    }
}

pub struct Transition {}
