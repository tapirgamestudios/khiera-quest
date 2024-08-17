use agb::{display::object::Graphics, include_aseprite};

static SPRITES: &Graphics = include_aseprite!("gfx/astronaut.aseprite");

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

extract_statics!(SPRITES, [IDLE, WALK, JUMP]);
