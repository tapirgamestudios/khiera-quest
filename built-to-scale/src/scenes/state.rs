use agb::{
    display::{
        affine::AffineMatrix,
        object::{
            AffineMatrixInstance, AffineMode, OamIterator, ObjectUnmanaged, Sprite, SpriteLoader,
        },
        tiled::{AffineMap, MapLoan, VRamManager},
    },
    fixnum::Vector2D,
    input::{Button, ButtonController, Tri},
};
use util::Number;

pub struct Update<'a> {
    button: &'a ButtonController,

    planet_background: &'a mut AffineMap,
    vram_manager: &'a mut VRamManager,
}

impl<'a> Update<'a> {
    pub fn new(
        button: &'a ButtonController,
        planet_background: &'a mut AffineMap,
        vram_manager: &'a mut VRamManager,
    ) -> Self {
        Self {
            button,
            planet_background,
            vram_manager,
        }
    }

    pub fn affine_map(&'a mut self) -> (&'a mut AffineMap, &'a mut VRamManager) {
        (self.planet_background, self.vram_manager)
    }
}

impl Update<'_> {
    pub fn button_x_tri(&self) -> Tri {
        self.button.x_tri()
    }

    pub fn jump_just_pressed(&self) -> bool {
        self.button.is_just_pressed(Button::A)
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

    pub fn display(
        &mut self,
        sprite: &'static Sprite,
        affine: &AffineMatrix,
        position: Vector2D<Number>,
        hflip: bool,
    ) {
        let object = self.object(sprite, *affine, position, hflip);
        self.oam_iter.set_next(&object);
    }
}

pub struct Transition {}
