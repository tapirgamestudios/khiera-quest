#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

use agb::{
    display::{
        tiled::{AffineBackgroundSize, TiledMap},
        Priority,
    },
    input::ButtonController,
    interrupt::VBlank,
};
use scenes::{Display, SceneManager, Update};

extern crate alloc;

mod resources;
mod scenes;

#[agb::entry]
fn main(gba: agb::Gba) -> ! {
    entry(gba);
}

fn entry(mut gba: agb::Gba) -> ! {
    let mut scene = SceneManager::new();

    let (mut unmanaged, mut loader) = gba.display.object.get_unmanaged();
    let (tiles, mut vram) = gba.display.video.tiled1();

    vram.set_background_palettes(resources::bg::PALETTES);

    let mut planet_background = tiles.affine(Priority::P0, AffineBackgroundSize::Background64x64);
    planet_background.set_visible(true);

    let vblank = VBlank::get();

    let mut button_controller = ButtonController::new();

    loop {
        button_controller.update();

        scene.frame(&mut Update::new(
            &button_controller,
            &mut planet_background,
            &mut vram,
        ));
        vblank.wait_for_vblank();
        scene.display(&mut Display::new(unmanaged.iter(), &mut loader));

        planet_background.commit(&mut vram);
    }
}
