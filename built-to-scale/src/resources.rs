use agb::{
    display::{object::Graphics, palette16::Palette16, Font},
    include_aseprite, include_background_gfx, include_font, include_wav,
};

static SPRITES: &Graphics = include_aseprite!(
    "gfx/astronaut.aseprite",
    "gfx/bubble.aseprite",
    "gfx/powerups.aseprite",
    "gfx/collideable-objects.aseprite"
);

pub static FONT: Font = include_font!("fnt/Dungeon Puzzler Font.ttf", 8);

include_background_gfx!(backgrounds, "000000",
    dummy => deduplicate "gfx/sprites.aseprite",
    planets => 256 deduplicate "gfx/background.aseprite",
    planets2 => 256 deduplicate "gfx/background2.aseprite",
    platforms => 256 deduplicate "gfx/platforms.aseprite",

    stars => deduplicate "gfx/stars.aseprite",
);

pub mod bg {
    use super::backgrounds;

    pub use backgrounds::*;
}

macro_rules! extract_statics {
    (
        $sprites:ident, [
            $($name:tt),+ $(,)?
        ] $(,)?
    ) => {
        $(
            pub static $name: &agb::display::object::Tag = $sprites.tags().get(stringify!($name));
        )+

    };
}

extract_statics!(
    SPRITES,
    [
        IDLE,
        WALK,
        JUMP,
        FALL,
        BUBBLE,
        BUBBLE_POP,
        JUMP_BOOST,
        DOUBLE_JUMP,
        DASH,
        SLIME_MOON,
        PLATFORM_UP_RIGHT,
        PLATFORM_VERTICAL,
        ROUND_PLATFORM,
        SQUARE_PLATFORM,
        ASTEROID1,
        ASTEROID2,
        ASTEROID3,
        DIAGONAL_PLATFORM,
    ]
);

pub static JUMP_SOUND: &[u8] = include_wav!("game-sfx/jump.wav");
pub static POWER_UP_SOUND: &[u8] = include_wav!("game-sfx/power_up.wav");
pub static DASH_SOUND: &[u8] = include_wav!("game-sfx/dash.wav");
pub static LAND_GROUND: &[u8] = include_wav!("game-sfx/land_ground.wav");
pub static RECOVERY_SOUND: &[u8] = include_wav!("game-sfx/recovery.wav");

pub static TEXT_PALETTE: Palette16 = Palette16::new([
    0, 0xFFFF, 0, 0xFFFF, 0x2b9f, 0x321f, 0x2b6c, 0x7ee6, 0x3619, 0x2485, 0, 0, 0, 0, 0, 0,
]);
