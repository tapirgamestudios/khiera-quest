use agb::{display::object::Graphics, include_aseprite, include_background_gfx, include_wav};

static SPRITES: &Graphics = include_aseprite!(
    "gfx/astronaut.aseprite",
    "gfx/bubble.aseprite",
    "gfx/powerups.aseprite"
);

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
    ]
);

pub static JUMP_SOUND: &[u8] = include_wav!("game-sfx/jump.wav");
pub static POWER_UP_SOUND: &[u8] = include_wav!("game-sfx/power_up.wav");
