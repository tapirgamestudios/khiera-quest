use agb::{
    display::{
        affine::AffineMatrix,
        object::{
            AffineMatrixInstance, AffineMode, OamIterator, ObjectUnmanaged, Sprite, SpriteLoader,
        },
    },
    fixnum::Vector2D,
    input::{ButtonController, Tri},
};
use util::Number;

pub struct Update<'a> {
    button: &'a ButtonController,
}

impl<'a> Update<'a> {
    pub fn new(button: &'a ButtonController) -> Self {
        Self { button }
    }
}

impl Update<'_> {
    pub fn button_x_tri(&self) -> Tri {
        self.button.x_tri()
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
    pub fn object(
        &mut self,
        sprite: &'static Sprite,
        affine: &AffineMatrix,
        position: Vector2D<Number>,
    ) -> ObjectUnmanaged {
        let mut o = ObjectUnmanaged::new(self.sprite_loader.get_vram_sprite(sprite));
        let aff = *affine * AffineMatrix::from_translation(position);
        o.set_affine_matrix(AffineMatrixInstance::new(aff.to_object_wrapping()));
        o.show_affine(AffineMode::Affine);

        o
    }

    pub fn display(
        &mut self,
        sprite: &'static Sprite,
        affine: &AffineMatrix,
        position: Vector2D<Number>,
    ) {
        let object = self.object(sprite, affine, position);
        self.oam_iter.set_next(&object);
    }
}

pub struct Transition {}
