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
    sound::mixer::Frequency,
};
use agb_tracker::Tracker;
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

    let planet_background = tiles.background(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::EightBpp,
    );

    let mut planet_scrolled_map =
        infinite_scroll_wrapper(planet_background, map::get_planet_tile_chunk);

    planet_scrolled_map.init(
        &mut vram,
        map::CAMERA_START.floor() + (-WIDTH / 2, -HEIGHT / 2).into(),
        &mut || {},
    );

    let platform_background = tiles.background(
        Priority::P1,
        RegularBackgroundSize::Background32x32,
        TileFormat::EightBpp,
    );

    let mut platform_scrolled_map =
        infinite_scroll_wrapper(platform_background, map::get_platform_tile_chunk);

    platform_scrolled_map.init(
        &mut vram,
        map::CAMERA_START.floor() + (-WIDTH / 2, -HEIGHT / 2).into(),
        &mut || {},
    );

    let mut star_background = tiles.background(
        Priority::P2,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );
    for y in 0..32u16 {
        for x in 0..32u16 {
            star_background.set_tile(
                &mut vram,
                (x, y),
                &resources::bg::stars.tiles,
                resources::bg::stars.tile_settings[(y * 32 + x) as usize],
            );
        }
    }

    planet_scrolled_map.commit(&mut vram);
    star_background.commit(&mut vram);
    platform_scrolled_map.commit(&mut vram);

    planet_scrolled_map.set_visible(true);
    star_background.set_visible(true);
    platform_scrolled_map.set_visible(true);

    let vblank = VBlank::get();

    let mut button_controller = ButtonController::new();

    let mut mixer = gba.mixer.mixer(Frequency::Hz32768);
    mixer.enable();
    let mut tracker = Tracker::new(&sfx::BG_MUSIC);

    loop {
        button_controller.update();

        {
            let mut update = Update::new(&button_controller);

            scene.frame(&mut update);

            if let Some(new_pos) = update.new_pos() {
                while planet_scrolled_map.set_pos(&mut vram, new_pos) != PartialUpdateStatus::Done {
                }

                while platform_scrolled_map.set_pos(&mut vram, new_pos) != PartialUpdateStatus::Done
                {
                }

                let star_pos = new_pos / 16;
                star_background.set_scroll_pos((
                    star_pos.x.rem_euclid(32 * 8) as i16,
                    star_pos.y.rem_euclid(32 * 8) as i16,
                ));
            }
        }
        vblank.wait_for_vblank();
        scene.display(&mut Display::new(unmanaged.iter(), &mut loader));

        planet_scrolled_map.commit(&mut vram);
        platform_scrolled_map.commit(&mut vram);
        star_background.commit(&mut vram);

        tracker.step(&mut mixer);
        mixer.frame();
    }
}

fn infinite_scroll_wrapper<'a>(
    planet_background: agb::display::tiled::MapLoan<'a, agb::display::tiled::RegularMap>,
    get_chunk_data: impl Fn(i32, i32) -> &'static [map::MapTileSetting] + 'a,
) -> InfiniteScrolledMap<'a> {
    InfiniteScrolledMap::new(
        planet_background,
        Box::new(move |pos| {
            let chunk = Vector2D::new(pos.x.div_floor(8), pos.y.div_floor(8));
            let chunk_x = pos.x.rem_euclid(8);
            let chunk_y = pos.y.rem_euclid(8);

            let chunk_data = get_chunk_data(chunk.x, chunk.y);
            let map_tile_setting = chunk_data[(chunk_x + chunk_y * 8) as usize];

            let tileset = match map_tile_setting.map_tile_set {
                map::MapTileSet::Planets => &resources::bg::planets,
                map::MapTileSet::Planets2 => &resources::bg::planets2,
                map::MapTileSet::Platforms => &resources::bg::platforms,
            };

            (
                &tileset.tiles,
                tileset.tile_settings[map_tile_setting.tile_id as usize]
                    .hflip(map_tile_setting.hflip)
                    .vflip(map_tile_setting.vflip),
            )
        }),
    )
}
