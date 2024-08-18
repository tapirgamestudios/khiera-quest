use agb::{display::object::Graphics, include_aseprite, include_background_gfx};

static SPRITES: &Graphics = include_aseprite!("gfx/astronaut.aseprite");

include_background_gfx!(backgrounds, "000000",
    dummy => deduplicate "gfx/sprites.aseprite",
    planets => 256 deduplicate "gfx/background.aseprite",
    platforms => 256 deduplicate "gfx/platforms.aseprite",
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

extract_statics!(SPRITES, [IDLE, WALK, JUMP, FALL]);
