use agb::fixnum::Vector2D;

pub struct MissionLog {
    pub point: Vector2D<i32>,
    pub text: &'static str,
}

pub static MISSION_LOGS: &[MissionLog] = &[
    MissionLog {
        point: Vector2D::new(56, 42),
        text: "MISSION OBJECTIVE:\n\nTeleport the slime planet away from Earth to prevent the invasion.",
    },
    MissionLog {
        point: Vector2D::new(400, 46),
        text: "MISSION LOG:\n\nThe architect approached the slime planet without adequate protection, \
            and has been teleported into the void",
    },
    MissionLog {
        point: Vector2D::new(922, -401),
        text: "MISSION LOG:\n\nWith the help of the evil wizards, we were able to get safety equipment teleported \
            into the heavens. But we don't know where it is...",
    },
    MissionLog {
        point: Vector2D::new(387, -532),
        text: "MISSION LOG:\n\nI always thought the L shape was a measurement error. Seems it actually looks like this",
    },
    MissionLog {
        point: Vector2D::new(111, -1144),
        text: "Now that we have everything we need, we can return to Earth and teleport the slimes away.",
    },
    MissionLog {
        point: Vector2D::new(-66, -1776),
        text: "PROCESSING...\nArchitect technology signatures detected\nTargeting calibration kit located\n\
            WARNING: May cause sudden teleportation."
    },
    MissionLog {
        point: Vector2D::new(-66, -1776),
        text: "TELEPORTATION SUCCESSFUL. SLIME PLANET IS NOW NO LONGER A THREAT.\n\n
            Uhhh... I think I'm stuck here now."
    },
    MissionLog {
        point: Vector2D::new(613, -598),
        text: "MISSION LOG:\n\nThere used to be a planet here. But the archetict blew it up to make travel between projects easier.",
    },
    MissionLog {
        point: Vector2D::new(66, -611),
        text: "MISSION LOG:\n\nI've made it to the slime planet. Strange to think this is the cause of all our problems.\n\
            Need to finish collecting the safety equipment before I can teleport this away from Earth.",
    },
    MissionLog {
        point: Vector2D::new(1427, -957),
        text: "Huh, some boots of dashing... Wonder why these were teleported up here. Well, they'll probably come in handy.",
    },
    MissionLog {
        point: Vector2D::new(1038, -1020),
        text: "MISSION LOG:\n\nSeems there is a second asteroid field here. I think the last piece of equipment I'll need \
            is on the other side of this."
    },
];
