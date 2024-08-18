#![no_std]
#![no_main]
#![feature(int_roundings)]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

use agb::{
    display::{
        tiled::{
            InfiniteScrolledMap, PartialUpdateStatus, RegularBackgroundSize, TileFormat, TiledMap,
        },
        Priority, HEIGHT, WIDTH,
    },
    fixnum::Vector2D,
    input::ButtonController,
    interrupt::VBlank,
};
use alloc::boxed::Box;
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
    let (tiles, mut vram) = gba.display.video.tiled0();

    vram.set_background_palettes(resources::bg::PALETTES);

    let mut planet_background = tiles.background(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::EightBpp,
    );
    planet_background.set_visible(true);

    let mut infinite_scrolled_map = InfiniteScrolledMap::new(
        planet_background,
        Box::new(|pos| {
            let chunk = Vector2D::new(pos.x.div_floor(8), pos.y.div_floor(8));
            let chunk_data = map::get_tile_chunk(chunk.x, chunk.y);
            let chunk_x = pos.x.rem_euclid(8);
            let chunk_y = pos.y.rem_euclid(8);

            let tile_idx = chunk_data[(chunk_x + chunk_y * 8) as usize];

            (
                &resources::bg::planets.tiles,
                resources::bg::planets.tile_settings[tile_idx as usize],
            )
        }),
    );

    infinite_scrolled_map.set_visible(true);
    infinite_scrolled_map.init(&mut vram, (-WIDTH / 2, -HEIGHT / 2).into(), &mut || {});

    let vblank = VBlank::get();

    let mut button_controller = ButtonController::new();

    loop {
        button_controller.update();

        {
            let mut update = Update::new(&button_controller);

            scene.frame(&mut update);

            if let Some(new_pos) = update.new_pos() {
                while infinite_scrolled_map.set_pos(&mut vram, new_pos) != PartialUpdateStatus::Done
                {
                }
            }
        }
        vblank.wait_for_vblank();
        scene.display(&mut Display::new(unmanaged.iter(), &mut loader));

        infinite_scrolled_map.commit(&mut vram);
    }
}
