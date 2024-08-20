use agb::{display::object::Graphics, include_aseprite, include_background_gfx, include_wav};

static SPRITES: &Graphics = include_aseprite!(
    "gfx/astronaut.aseprite",
    "gfx/bubble.aseprite",
    "gfx/powerups.aseprite",
    "gfx/collideable-objects.aseprite"
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
        SLIME_MOON,
        PLATFORM_UP_RIGHT,
    ]
);

pub static JUMP_SOUND: &[u8] = include_wav!("game-sfx/jump.wav");
pub static POWER_UP_SOUND: &[u8] = include_wav!("game-sfx/power_up.wav");
pub static DASH_SOUND: &[u8] = include_wav!("game-sfx/dash.wav");
pub static LAND_GROUND: &[u8] = include_wav!("game-sfx/land_ground.wav");
pub static RECOVERY_SOUND: &[u8] = include_wav!("game-sfx/recovery.wav");
