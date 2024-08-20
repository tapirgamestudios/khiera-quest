use agb::fixnum::Vector2D;

pub struct MissionLog {
    pub point: Vector2D<i32>,
    pub text: &'static str,
}

pub static MISSION_LOGS: &[MissionLog] = &[MissionLog {
    point: Vector2D::new(-192, 88),
    text: "MISSION LOG:\n\nVENTURE FORTH AND CROSS THE BRIMSTONE PIT",
}];
