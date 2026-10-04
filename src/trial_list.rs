//! The original's 50 trials, in the order the game lists them, from the PC readout
//! (docs/research/trials.md). Generated from that table; edit the table and regenerate
//! rather than editing by hand. Floors other than the mode's default and the race
//! trials' rival tower pictures aren't built yet.

use bevy::prelude::*;

use crate::config::{CastWhen, EnemyCast, ModeKind, SpellKind, TrialDef, TrialLines};

/// Lives in the last Master trial ("the game's maximum lives"; the number isn't read, ?).
pub const MAX_TRIAL_LIVES: u32 = 5;

pub fn original_trials() -> Vec<TrialDef> {
    vec![
        // Rookie (0 medals) 1: reach 14 cells in 60 s
        TrialDef {
            name: "SR_10".into(),
            mode: ModeKind::Race,
            finish_height: Some(14.0),
            seconds: Some(60.0),
            enemy: vec![EnemyCast {
                when: CastWhen::Height(11.0),
                spells: vec![SpellKind::Large],
            }],
            ..default()
        },
        // Rookie (0 medals) 2: place 15 bricks, 3 lives
        TrialDef {
            name: "SS_17".into(),
            mode: ModeKind::Survival,
            bricks: Some(15),
            lives: Some(3),
            drop_speed: Some(3.0),
            enemy: vec![
                EnemyCast {
                    when: CastWhen::Brick(5),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(10),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(15),
                    spells: vec![SpellKind::Large],
                },
            ],
            ..default()
        },
        // Rookie (0 medals) 3: place 9 bricks under a laser at 2.2, 1 life
        TrialDef {
            name: "SP_20".into(),
            mode: ModeKind::Puzzle,
            bricks: Some(9),
            laser: Some(2.2),
            order: vec![
                "O".into(),
                "Z".into(),
                "S".into(),
                "T".into(),
                "T".into(),
                "O".into(),
                "T".into(),
                "T".into(),
                "I".into(),
            ],
            ..default()
        },
        // Rookie (0 medals) 4: place 6 bricks under a laser at 5.2, 1 life
        TrialDef {
            name: "SP_16".into(),
            mode: ModeKind::Puzzle,
            bricks: Some(6),
            laser: Some(5.2),
            order: vec![
                "J".into(),
                "I".into(),
                "S".into(),
                "Z".into(),
                "L".into(),
                "J".into(),
            ],
            ..default()
        },
        // Rookie (0 medals) 5: reach 16 cells in 60 s
        TrialDef {
            name: "SR_11".into(),
            mode: ModeKind::Race,
            finish_height: Some(16.0),
            seconds: Some(60.0),
            enemy: vec![
                EnemyCast {
                    when: CastWhen::Height(4.0),
                    spells: vec![SpellKind::Grassy],
                },
                EnemyCast {
                    when: CastWhen::Height(8.0),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Height(11.0),
                    spells: vec![SpellKind::Grassy],
                },
            ],
            ..default()
        },
        // Rookie (0 medals) 6: place 20 bricks, 3 lives
        TrialDef {
            name: "SS_10".into(),
            mode: ModeKind::Survival,
            bricks: Some(20),
            lives: Some(3),
            drop_speed: Some(3.0),
            lines: Some(TrialLines {
                every: 12.0,
                spells: vec![(SpellKind::Petrify, false)],
            }),
            enemy: vec![
                EnemyCast {
                    when: CastWhen::Brick(5),
                    spells: vec![SpellKind::Mystery],
                },
                EnemyCast {
                    when: CastWhen::Brick(10),
                    spells: vec![SpellKind::Mystery],
                },
                EnemyCast {
                    when: CastWhen::Brick(13),
                    spells: vec![SpellKind::Mystery],
                },
                EnemyCast {
                    when: CastWhen::Brick(20),
                    spells: vec![SpellKind::Large, SpellKind::Mystery],
                },
            ],
            ..default()
        },
        // Rookie (0 medals) 7: place 7 bricks under a laser at 1.2, 1 life
        TrialDef {
            name: "SP_15".into(),
            mode: ModeKind::Puzzle,
            bricks: Some(7),
            laser: Some(1.2),
            order: vec![
                "O".into(),
                "Z".into(),
                "S".into(),
                "L".into(),
                "S".into(),
                "J".into(),
                "Z".into(),
            ],
            ..default()
        },
        // Rookie (0 medals) 8: reach 22 cells in 70 s
        TrialDef {
            name: "SR_18".into(),
            mode: ModeKind::Race,
            finish_height: Some(22.0),
            seconds: Some(70.0),
            lines: Some(TrialLines {
                every: 9.0,
                spells: vec![(SpellKind::Petrify, false)],
            }),
            enemy: vec![
                EnemyCast {
                    when: CastWhen::Height(5.0),
                    spells: vec![SpellKind::Grassy],
                },
                EnemyCast {
                    when: CastWhen::Height(9.0),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Height(10.0),
                    spells: vec![SpellKind::Grassy],
                },
                EnemyCast {
                    when: CastWhen::Height(15.0),
                    spells: vec![SpellKind::Slow],
                },
            ],
            ..default()
        },
        // Rookie (0 medals) 9: place 6 bricks under a laser at 2.2, 1 life
        TrialDef {
            name: "SP_10".into(),
            mode: ModeKind::Puzzle,
            bricks: Some(6),
            laser: Some(2.2),
            order: vec![
                "T".into(),
                "J".into(),
                "L".into(),
                "I".into(),
                "L".into(),
                "O".into(),
            ],
            ..default()
        },
        // Rookie (0 medals) 10: place 26 bricks, 3 lives
        TrialDef {
            name: "SS_11".into(),
            mode: ModeKind::Survival,
            bricks: Some(26),
            lives: Some(3),
            drop_speed: Some(3.0),
            lines: Some(TrialLines {
                every: 9.0,
                spells: vec![(SpellKind::Petrify, true)],
            }),
            enemy: vec![
                EnemyCast {
                    when: CastWhen::Brick(1),
                    spells: vec![SpellKind::Mist],
                },
                EnemyCast {
                    when: CastWhen::Brick(11),
                    spells: vec![SpellKind::Mist],
                },
                EnemyCast {
                    when: CastWhen::Brick(19),
                    spells: vec![SpellKind::Mist],
                },
                EnemyCast {
                    when: CastWhen::Brick(23),
                    spells: vec![SpellKind::Mist],
                },
                EnemyCast {
                    when: CastWhen::Brick(24),
                    spells: vec![SpellKind::Mystery],
                },
                EnemyCast {
                    when: CastWhen::Brick(25),
                    spells: vec![SpellKind::Mystery],
                },
                EnemyCast {
                    when: CastWhen::Brick(26),
                    spells: vec![SpellKind::Mystery],
                },
            ],
            ..default()
        },
        // Apprentice (8) 1: place 6 bricks under a laser at 5.2, 1 life
        TrialDef {
            name: "SP_21".into(),
            mode: ModeKind::Puzzle,
            bricks: Some(6),
            laser: Some(5.2),
            order: vec![
                "I".into(),
                "L".into(),
                "Z".into(),
                "T".into(),
                "J".into(),
                "I".into(),
            ],
            ..default()
        },
        // Apprentice (8) 2: reach 24 cells in 75 s
        TrialDef {
            name: "SR_22".into(),
            mode: ModeKind::Race,
            finish_height: Some(24.0),
            seconds: Some(75.0),
            drop_speed: Some(4.0),
            lines: Some(TrialLines {
                every: 12.0,
                spells: vec![(SpellKind::Ivy, true)],
            }),
            enemy: vec![
                EnemyCast {
                    when: CastWhen::Brick(10),
                    spells: vec![SpellKind::Bubble],
                },
                EnemyCast {
                    when: CastWhen::Brick(20),
                    spells: vec![SpellKind::Bubble],
                },
                EnemyCast {
                    when: CastWhen::Brick(30),
                    spells: vec![SpellKind::Bubble],
                },
                EnemyCast {
                    when: CastWhen::Brick(40),
                    spells: vec![SpellKind::Bubble],
                },
                EnemyCast {
                    when: CastWhen::Brick(50),
                    spells: vec![SpellKind::Bubble],
                },
                EnemyCast {
                    when: CastWhen::Brick(60),
                    spells: vec![SpellKind::Bubble],
                },
                EnemyCast {
                    when: CastWhen::Brick(70),
                    spells: vec![SpellKind::Bubble],
                },
                EnemyCast {
                    when: CastWhen::Brick(80),
                    spells: vec![SpellKind::Bubble],
                },
                EnemyCast {
                    when: CastWhen::Brick(90),
                    spells: vec![SpellKind::Bubble],
                },
                EnemyCast {
                    when: CastWhen::Brick(100),
                    spells: vec![SpellKind::Bubble],
                },
            ],
            ..default()
        },
        // Apprentice (8) 3: place 25 bricks, 3 lives
        TrialDef {
            name: "SS_13".into(),
            mode: ModeKind::Survival,
            bricks: Some(25),
            lives: Some(3),
            drop_speed: Some(4.0),
            lines: Some(TrialLines {
                every: 9.0,
                spells: vec![(SpellKind::Ivy, false)],
            }),
            enemy: vec![
                EnemyCast {
                    when: CastWhen::Brick(1),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(6),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(11),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(14),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(18),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(21),
                    spells: vec![SpellKind::Large, SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(25),
                    spells: vec![SpellKind::Large, SpellKind::NoRotate, SpellKind::Mystery],
                },
            ],
            ..default()
        },
        // Apprentice (8) 4: reach 27 cells in 60 s
        TrialDef {
            name: "SR_14".into(),
            mode: ModeKind::Race,
            finish_height: Some(27.0),
            seconds: Some(60.0),
            drop_speed: Some(4.0),
            lines: Some(TrialLines {
                every: 12.5,
                spells: vec![(SpellKind::Ivy, true)],
            }),
            enemy: vec![
                EnemyCast {
                    when: CastWhen::Height(0.0),
                    spells: vec![SpellKind::Mystery],
                },
                EnemyCast {
                    when: CastWhen::Height(5.0),
                    spells: vec![SpellKind::Bubble],
                },
                EnemyCast {
                    when: CastWhen::Height(9.0),
                    spells: vec![SpellKind::Large, SpellKind::Mystery],
                },
                EnemyCast {
                    when: CastWhen::Height(20.0),
                    spells: vec![SpellKind::Grassy],
                },
            ],
            ..default()
        },
        // Apprentice (8) 5: place 8 bricks under a laser at 5.2, 1 life
        TrialDef {
            name: "SP_12".into(),
            mode: ModeKind::Puzzle,
            bricks: Some(8),
            laser: Some(5.2),
            order: vec![
                "L".into(),
                "J".into(),
                "Z".into(),
                "O".into(),
                "T".into(),
                "Z".into(),
                "O".into(),
                "I".into(),
            ],
            enemy: vec![EnemyCast {
                when: CastWhen::Brick(5),
                spells: vec![SpellKind::Large],
            }],
            ..default()
        },
        // Apprentice (8) 6: place 30 bricks, 3 lives
        TrialDef {
            name: "SS_14".into(),
            mode: ModeKind::Survival,
            bricks: Some(30),
            lives: Some(3),
            drop_speed: Some(4.0),
            lines: Some(TrialLines {
                every: 9.0,
                spells: vec![(SpellKind::Ivy, true)],
            }),
            enemy: vec![
                EnemyCast {
                    when: CastWhen::Brick(1),
                    spells: vec![SpellKind::Fast],
                },
                EnemyCast {
                    when: CastWhen::Brick(11),
                    spells: vec![SpellKind::Fast],
                },
                EnemyCast {
                    when: CastWhen::Brick(21),
                    spells: vec![SpellKind::Fast, SpellKind::Mist],
                },
                EnemyCast {
                    when: CastWhen::Brick(26),
                    spells: vec![SpellKind::Fast],
                },
            ],
            ..default()
        },
        // Apprentice (8) 7: place 7 bricks under a laser at 4.2, 1 life
        TrialDef {
            name: "SP_29".into(),
            mode: ModeKind::Puzzle,
            bricks: Some(7),
            laser: Some(4.2),
            order: vec![
                "L".into(),
                "T".into(),
                "Z".into(),
                "L".into(),
                "Z".into(),
                "S".into(),
                "I".into(),
            ],
            ..default()
        },
        // Apprentice (8) 8: place 7 bricks under a laser at 7.2, 1 life
        TrialDef {
            name: "SP_22".into(),
            mode: ModeKind::Puzzle,
            bricks: Some(7),
            laser: Some(7.2),
            order: vec![
                "Z".into(),
                "T".into(),
                "S".into(),
                "L".into(),
                "I".into(),
                "O".into(),
                "I".into(),
            ],
            enemy: vec![EnemyCast {
                when: CastWhen::Brick(7),
                spells: vec![SpellKind::Large],
            }],
            ..default()
        },
        // Apprentice (8) 9: place 33 bricks, 3 lives
        TrialDef {
            name: "SS_16".into(),
            mode: ModeKind::Survival,
            bricks: Some(33),
            lives: Some(3),
            drop_speed: Some(4.0),
            lines: Some(TrialLines {
                every: 9.0,
                spells: vec![(SpellKind::Ivy, true)],
            }),
            enemy: vec![
                EnemyCast {
                    when: CastWhen::Brick(1),
                    spells: vec![SpellKind::NoRotate, SpellKind::Mystery],
                },
                EnemyCast {
                    when: CastWhen::Brick(7),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(14),
                    spells: vec![SpellKind::Large, SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(21),
                    spells: vec![SpellKind::NoRotate, SpellKind::Mist],
                },
                EnemyCast {
                    when: CastWhen::Brick(29),
                    spells: vec![SpellKind::Fast, SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(31),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(33),
                    spells: vec![SpellKind::NoRotate],
                },
            ],
            ..default()
        },
        // Apprentice (8) 10: place 7 bricks under a laser at 5.2, 1 life
        TrialDef {
            name: "SP_24".into(),
            mode: ModeKind::Puzzle,
            bricks: Some(7),
            laser: Some(5.2),
            order: vec![
                "T".into(),
                "L".into(),
                "S".into(),
                "I".into(),
                "I".into(),
                "J".into(),
                "J".into(),
            ],
            ..default()
        },
        // Pro (17) 1: place 18 bricks, 3 lives
        TrialDef {
            name: "SS_18".into(),
            mode: ModeKind::Survival,
            bricks: Some(18),
            lives: Some(3),
            drop_speed: Some(5.0),
            lines: Some(TrialLines {
                every: 9.0,
                spells: vec![(SpellKind::Ivy, false), (SpellKind::Minibase, false)],
            }),
            enemy: vec![
                EnemyCast {
                    when: CastWhen::Brick(2),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(4),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(6),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(8),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(10),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(12),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(14),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(16),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(18),
                    spells: vec![SpellKind::Large],
                },
            ],
            ..default()
        },
        // Pro (17) 2: place 7 bricks under a laser at 0.2, 1 life
        TrialDef {
            name: "SP_18".into(),
            mode: ModeKind::Puzzle,
            bricks: Some(7),
            laser: Some(0.2),
            order: vec![
                "Z".into(),
                "O".into(),
                "S".into(),
                "Z".into(),
                "O".into(),
                "O".into(),
                "O".into(),
            ],
            ..default()
        },
        // Pro (17) 3: place 9 bricks under a laser at 4.2, 1 life
        TrialDef {
            name: "SP_11".into(),
            mode: ModeKind::Puzzle,
            bricks: Some(9),
            laser: Some(4.2),
            order: vec![
                "J".into(),
                "S".into(),
                "J".into(),
                "T".into(),
                "Z".into(),
                "T".into(),
                "I".into(),
                "I".into(),
                "J".into(),
            ],
            ..default()
        },
        // Pro (17) 4: place 33 bricks, 3 lives
        TrialDef {
            name: "SS_12".into(),
            mode: ModeKind::Survival,
            bricks: Some(33),
            lives: Some(3),
            drop_speed: Some(5.0),
            lines: Some(TrialLines {
                every: 9.0,
                spells: vec![(SpellKind::Minibase, false), (SpellKind::Ivy, false)],
            }),
            enemy: vec![
                EnemyCast {
                    when: CastWhen::Brick(1),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(2),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(3),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(4),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(5),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(6),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(7),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(8),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(9),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(10),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(11),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(12),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(13),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(14),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(15),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(16),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(17),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(18),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(19),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(20),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(21),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(22),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(23),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(24),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(25),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(26),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(27),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(28),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(29),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(30),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(31),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(32),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(33),
                    spells: vec![SpellKind::Large, SpellKind::NoRotate],
                },
            ],
            ..default()
        },
        // Pro (17) 5: place 7 bricks under a laser at 0.35, 1 life
        TrialDef {
            name: "SP_30".into(),
            mode: ModeKind::Puzzle,
            bricks: Some(7),
            laser: Some(0.35),
            order: vec![
                "L".into(),
                "S".into(),
                "T".into(),
                "J".into(),
                "Z".into(),
                "T".into(),
                "I".into(),
            ],
            ..default()
        },
        // Pro (17) 6: reach 32 cells in 97 s
        TrialDef {
            name: "SR_16".into(),
            mode: ModeKind::Race,
            finish_height: Some(32.0),
            seconds: Some(97.0),
            drop_speed: Some(5.0),
            lines: Some(TrialLines {
                every: 12.0,
                spells: vec![(SpellKind::Ivy, false), (SpellKind::Ivy, false)],
            }),
            enemy: vec![
                EnemyCast {
                    when: CastWhen::Height(8.0),
                    spells: vec![SpellKind::Ice],
                },
                EnemyCast {
                    when: CastWhen::Height(17.0),
                    spells: vec![SpellKind::Mystery],
                },
                EnemyCast {
                    when: CastWhen::Height(25.0),
                    spells: vec![SpellKind::Ice],
                },
            ],
            ..default()
        },
        // Pro (17) 7: place 7 bricks under a laser at 0.35, 1 life
        TrialDef {
            name: "SP_17".into(),
            mode: ModeKind::Puzzle,
            bricks: Some(7),
            laser: Some(0.35),
            order: vec![
                "T".into(),
                "I".into(),
                "L".into(),
                "I".into(),
                "J".into(),
                "I".into(),
                "L".into(),
            ],
            enemy: vec![
                EnemyCast {
                    when: CastWhen::Brick(2),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(6),
                    spells: vec![SpellKind::NoRotate],
                },
            ],
            ..default()
        },
        // Pro (17) 8: reach 40 cells in 128 s
        TrialDef {
            name: "SR_17".into(),
            mode: ModeKind::Race,
            finish_height: Some(40.0),
            seconds: Some(128.0),
            drop_speed: Some(5.0),
            lines: Some(TrialLines {
                every: 12.0,
                spells: vec![
                    (SpellKind::Ivy, false),
                    (SpellKind::Minibase, false),
                    (SpellKind::Ivy, false),
                ],
            }),
            enemy: vec![
                EnemyCast {
                    when: CastWhen::Height(7.0),
                    spells: vec![SpellKind::Ice, SpellKind::Mystery],
                },
                EnemyCast {
                    when: CastWhen::Height(18.0),
                    spells: vec![SpellKind::Large, SpellKind::Ice],
                },
                EnemyCast {
                    when: CastWhen::Height(32.0),
                    spells: vec![SpellKind::Ice],
                },
            ],
            ..default()
        },
        // Pro (17) 9: place 8 bricks under a laser at 2.2, 1 life
        TrialDef {
            name: "SP_28".into(),
            mode: ModeKind::Puzzle,
            bricks: Some(8),
            laser: Some(2.2),
            order: vec![
                "L".into(),
                "J".into(),
                "Z".into(),
                "J".into(),
                "S".into(),
                "T".into(),
                "I".into(),
                "I".into(),
            ],
            ..default()
        },
        // Pro (17) 10: place 42 bricks, 3 lives
        TrialDef {
            name: "SS_15".into(),
            mode: ModeKind::Survival,
            bricks: Some(42),
            lives: Some(3),
            drop_speed: Some(5.0),
            lines: Some(TrialLines {
                every: 9.0,
                spells: vec![
                    (SpellKind::Ivy, false),
                    (SpellKind::Minibase, false),
                    (SpellKind::Ivy, false),
                ],
            }),
            enemy: vec![
                EnemyCast {
                    when: CastWhen::Brick(1),
                    spells: vec![SpellKind::Fast, SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(11),
                    spells: vec![SpellKind::Mist],
                },
                EnemyCast {
                    when: CastWhen::Brick(13),
                    spells: vec![SpellKind::Fast],
                },
                EnemyCast {
                    when: CastWhen::Brick(21),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(27),
                    spells: vec![SpellKind::Fast, SpellKind::Mystery],
                },
                EnemyCast {
                    when: CastWhen::Brick(31),
                    spells: vec![SpellKind::Large, SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(39),
                    spells: vec![SpellKind::Fast],
                },
                EnemyCast {
                    when: CastWhen::Brick(42),
                    spells: vec![SpellKind::Large, SpellKind::NoRotate],
                },
            ],
            ..default()
        },
        // Expert (26) 1: place 8 bricks under a laser at 0.2, 1 life
        TrialDef {
            name: "SP_13".into(),
            mode: ModeKind::Puzzle,
            bricks: Some(8),
            laser: Some(0.2),
            order: vec![
                "L".into(),
                "T".into(),
                "S".into(),
                "I".into(),
                "T".into(),
                "O".into(),
                "J".into(),
                "L".into(),
            ],
            ..default()
        },
        // Expert (26) 2: reach 50 cells in 155 s
        TrialDef {
            name: "SR_19".into(),
            mode: ModeKind::Race,
            finish_height: Some(50.0),
            seconds: Some(155.0),
            drop_speed: Some(5.0),
            lines: Some(TrialLines {
                every: 13.0,
                spells: vec![
                    (SpellKind::Minibase, false),
                    (SpellKind::Petrify, false),
                    (SpellKind::Ivy, false),
                ],
            }),
            enemy: vec![
                EnemyCast {
                    when: CastWhen::Height(7.0),
                    spells: vec![SpellKind::AutoRotate],
                },
                EnemyCast {
                    when: CastWhen::Height(12.0),
                    spells: vec![SpellKind::Mystery],
                },
                EnemyCast {
                    when: CastWhen::Height(24.0),
                    spells: vec![SpellKind::Grassy],
                },
                EnemyCast {
                    when: CastWhen::Height(32.0),
                    spells: vec![SpellKind::Slow, SpellKind::AutoRotate],
                },
                EnemyCast {
                    when: CastWhen::Height(34.0),
                    spells: vec![SpellKind::AutoRotate],
                },
                EnemyCast {
                    when: CastWhen::Height(46.0),
                    spells: vec![SpellKind::Large, SpellKind::AutoRotate],
                },
            ],
            ..default()
        },
        // Expert (26) 3: place 9 bricks under a laser at 0.2, 1 life
        TrialDef {
            name: "SP_25".into(),
            mode: ModeKind::Puzzle,
            bricks: Some(9),
            laser: Some(0.2),
            order: vec![
                "S".into(),
                "L".into(),
                "T".into(),
                "J".into(),
                "O".into(),
                "T".into(),
                "O".into(),
                "J".into(),
                "O".into(),
            ],
            enemy: vec![EnemyCast {
                when: CastWhen::Brick(5),
                spells: vec![SpellKind::Large],
            }],
            ..default()
        },
        // Expert (26) 4: place 9 bricks under a laser at 5.2, 1 life
        TrialDef {
            name: "SP_23".into(),
            mode: ModeKind::Puzzle,
            bricks: Some(9),
            laser: Some(5.2),
            order: vec![
                "J".into(),
                "I".into(),
                "L".into(),
                "Z".into(),
                "T".into(),
                "J".into(),
                "I".into(),
                "J".into(),
                "J".into(),
            ],
            ..default()
        },
        // Expert (26) 5: place 40 bricks, 3 lives
        TrialDef {
            name: "SS_21".into(),
            mode: ModeKind::Survival,
            bricks: Some(40),
            lives: Some(3),
            drop_speed: Some(5.0),
            lines: Some(TrialLines {
                every: 9.0,
                spells: vec![(SpellKind::Minibase, false), (SpellKind::Minibase, false)],
            }),
            enemy: vec![
                EnemyCast {
                    when: CastWhen::Brick(1),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(2),
                    spells: vec![SpellKind::Ghost],
                },
                EnemyCast {
                    when: CastWhen::Brick(10),
                    spells: vec![SpellKind::Ghost],
                },
                EnemyCast {
                    when: CastWhen::Brick(24),
                    spells: vec![SpellKind::Ghost],
                },
                EnemyCast {
                    when: CastWhen::Brick(35),
                    spells: vec![SpellKind::Ghost],
                },
                EnemyCast {
                    when: CastWhen::Brick(40),
                    spells: vec![SpellKind::Ghost],
                },
            ],
            ..default()
        },
        // Expert (26) 6: reach 20 cells in 44 s
        TrialDef {
            name: "SR_20".into(),
            mode: ModeKind::Race,
            finish_height: Some(20.0),
            seconds: Some(44.0),
            drop_speed: Some(5.0),
            lines: Some(TrialLines {
                every: 12.0,
                spells: vec![(SpellKind::Ivy, true)],
            }),
            enemy: vec![
                EnemyCast {
                    when: CastWhen::Brick(1),
                    spells: vec![SpellKind::Ice],
                },
                EnemyCast {
                    when: CastWhen::Brick(7),
                    spells: vec![SpellKind::Ice],
                },
                EnemyCast {
                    when: CastWhen::Brick(13),
                    spells: vec![SpellKind::Ice],
                },
                EnemyCast {
                    when: CastWhen::Brick(19),
                    spells: vec![SpellKind::Ice],
                },
                EnemyCast {
                    when: CastWhen::Brick(25),
                    spells: vec![SpellKind::Ice],
                },
                EnemyCast {
                    when: CastWhen::Brick(31),
                    spells: vec![SpellKind::Ice],
                },
                EnemyCast {
                    when: CastWhen::Brick(37),
                    spells: vec![SpellKind::Ice],
                },
                EnemyCast {
                    when: CastWhen::Brick(43),
                    spells: vec![SpellKind::Ice],
                },
                EnemyCast {
                    when: CastWhen::Brick(49),
                    spells: vec![SpellKind::Ice],
                },
                EnemyCast {
                    when: CastWhen::Brick(55),
                    spells: vec![SpellKind::Ice],
                },
                EnemyCast {
                    when: CastWhen::Brick(61),
                    spells: vec![SpellKind::Ice],
                },
                EnemyCast {
                    when: CastWhen::Brick(67),
                    spells: vec![SpellKind::Ice],
                },
                EnemyCast {
                    when: CastWhen::Brick(73),
                    spells: vec![SpellKind::Ice],
                },
                EnemyCast {
                    when: CastWhen::Brick(79),
                    spells: vec![SpellKind::Ice],
                },
            ],
            ..default()
        },
        // Expert (26) 7: place 9 bricks under a laser at 3.2, 1 life
        TrialDef {
            name: "SP_31".into(),
            mode: ModeKind::Puzzle,
            bricks: Some(9),
            laser: Some(3.2),
            order: vec![
                "S".into(),
                "T".into(),
                "J".into(),
                "T".into(),
                "I".into(),
                "T".into(),
                "I".into(),
                "S".into(),
                "I".into(),
            ],
            ..default()
        },
        // Expert (26) 8: reach 56 cells in 119 s
        TrialDef {
            name: "SR_21".into(),
            mode: ModeKind::Race,
            finish_height: Some(56.0),
            seconds: Some(119.0),
            drop_speed: Some(5.0),
            lines: Some(TrialLines {
                every: 15.0,
                spells: vec![
                    (SpellKind::Ivy, false),
                    (SpellKind::Minibase, false),
                    (SpellKind::Ivy, false),
                ],
            }),
            enemy: vec![
                EnemyCast {
                    when: CastWhen::Height(10.0),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Height(20.0),
                    spells: vec![SpellKind::Mystery],
                },
                EnemyCast {
                    when: CastWhen::Height(30.0),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Height(40.0),
                    spells: vec![SpellKind::Mystery],
                },
                EnemyCast {
                    when: CastWhen::Height(50.0),
                    spells: vec![SpellKind::Large, SpellKind::Mystery],
                },
            ],
            ..default()
        },
        // Expert (26) 9: place 8 bricks under a laser at 6.2, 1 life
        TrialDef {
            name: "SP_14".into(),
            mode: ModeKind::Puzzle,
            bricks: Some(8),
            laser: Some(6.2),
            order: vec![
                "L".into(),
                "L".into(),
                "O".into(),
                "I".into(),
                "S".into(),
                "L".into(),
                "I".into(),
                "T".into(),
            ],
            enemy: vec![
                EnemyCast {
                    when: CastWhen::Brick(1),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(6),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(8),
                    spells: vec![SpellKind::NoRotate],
                },
            ],
            ..default()
        },
        // Expert (26) 10: place 50 bricks, 3 lives
        TrialDef {
            name: "SS_06".into(),
            mode: ModeKind::Survival,
            bricks: Some(50),
            lives: Some(3),
            drop_speed: Some(5.0),
            lines: Some(TrialLines {
                every: 9.0,
                spells: vec![
                    (SpellKind::Ivy, false),
                    (SpellKind::Minibase, false),
                    (SpellKind::Ivy, false),
                ],
            }),
            enemy: vec![
                EnemyCast {
                    when: CastWhen::Brick(10),
                    spells: vec![SpellKind::Fast],
                },
                EnemyCast {
                    when: CastWhen::Brick(17),
                    spells: vec![SpellKind::Fast],
                },
                EnemyCast {
                    when: CastWhen::Brick(24),
                    spells: vec![SpellKind::Fast],
                },
                EnemyCast {
                    when: CastWhen::Brick(31),
                    spells: vec![SpellKind::Fast],
                },
                EnemyCast {
                    when: CastWhen::Brick(38),
                    spells: vec![SpellKind::Fast],
                },
                EnemyCast {
                    when: CastWhen::Brick(46),
                    spells: vec![SpellKind::Fast],
                },
                EnemyCast {
                    when: CastWhen::Brick(49),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(50),
                    spells: vec![SpellKind::Large, SpellKind::Mystery],
                },
            ],
            ..default()
        },
        // Master (36) 1: reach 35 cells in 95 s
        TrialDef {
            name: "SR_24".into(),
            mode: ModeKind::Race,
            finish_height: Some(35.0),
            seconds: Some(95.0),
            drop_speed: Some(5.0),
            lines: Some(TrialLines {
                every: 11.0,
                spells: vec![
                    (SpellKind::Ivy, false),
                    (SpellKind::Ivy, false),
                    (SpellKind::Ivy, false),
                ],
            }),
            enemy: vec![
                EnemyCast {
                    when: CastWhen::Brick(5),
                    spells: vec![SpellKind::AutoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(6),
                    spells: vec![SpellKind::AutoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(7),
                    spells: vec![SpellKind::AutoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(8),
                    spells: vec![SpellKind::AutoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(15),
                    spells: vec![SpellKind::AutoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(16),
                    spells: vec![SpellKind::AutoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(17),
                    spells: vec![SpellKind::AutoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(18),
                    spells: vec![SpellKind::AutoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(25),
                    spells: vec![SpellKind::Large, SpellKind::AutoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(29),
                    spells: vec![SpellKind::Large, SpellKind::AutoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(35),
                    spells: vec![SpellKind::AutoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(36),
                    spells: vec![SpellKind::AutoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(37),
                    spells: vec![SpellKind::AutoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(38),
                    spells: vec![SpellKind::AutoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(45),
                    spells: vec![SpellKind::AutoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(46),
                    spells: vec![SpellKind::AutoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(47),
                    spells: vec![SpellKind::AutoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(48),
                    spells: vec![SpellKind::AutoRotate],
                },
            ],
            ..default()
        },
        // Master (36) 2: place 55 bricks, 3 lives
        TrialDef {
            name: "SS_07".into(),
            mode: ModeKind::Survival,
            bricks: Some(55),
            lives: Some(3),
            drop_speed: Some(5.0),
            lines: Some(TrialLines {
                every: 9.0,
                spells: vec![
                    (SpellKind::Undo, false),
                    (SpellKind::Ivy, false),
                    (SpellKind::Minibase, false),
                    (SpellKind::Ivy, false),
                    (SpellKind::Ivy, false),
                ],
            }),
            enemy: vec![
                EnemyCast {
                    when: CastWhen::Brick(1),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(2),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(3),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(4),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(5),
                    spells: vec![SpellKind::Large, SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(6),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(7),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(8),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(9),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(10),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(11),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(12),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(13),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(14),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(15),
                    spells: vec![SpellKind::Fast, SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(16),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(17),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(18),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(19),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(20),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(21),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(22),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(23),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(24),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(25),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(26),
                    spells: vec![SpellKind::Large, SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(27),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(28),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(29),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(30),
                    spells: vec![SpellKind::Large, SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(31),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(32),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(33),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(34),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(35),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(36),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(37),
                    spells: vec![SpellKind::NoRotate, SpellKind::Mystery],
                },
                EnemyCast {
                    when: CastWhen::Brick(38),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(39),
                    spells: vec![SpellKind::NoRotate, SpellKind::Mystery],
                },
                EnemyCast {
                    when: CastWhen::Brick(40),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(41),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(42),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(43),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(44),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(45),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(46),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(47),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(48),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(49),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(50),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(51),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(52),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(53),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(54),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(55),
                    spells: vec![SpellKind::Large, SpellKind::NoRotate, SpellKind::Mystery],
                },
            ],
            ..default()
        },
        // Master (36) 3: place 10 bricks under a laser at 7.2, 1 life
        TrialDef {
            name: "SP_33".into(),
            mode: ModeKind::Puzzle,
            bricks: Some(10),
            laser: Some(7.2),
            order: vec![
                "J".into(),
                "S".into(),
                "O".into(),
                "J".into(),
                "S".into(),
                "I".into(),
                "O".into(),
                "T".into(),
                "L".into(),
                "T".into(),
            ],
            ..default()
        },
        // Master (36) 4: place 7 bricks under a laser at 0.35, 1 life
        TrialDef {
            name: "SP_27".into(),
            mode: ModeKind::Puzzle,
            bricks: Some(7),
            laser: Some(0.35),
            order: vec![
                "L".into(),
                "I".into(),
                "S".into(),
                "Z".into(),
                "I".into(),
                "T".into(),
                "T".into(),
            ],
            ..default()
        },
        // Master (36) 5: reach 30 cells in 45 s
        TrialDef {
            name: "SR_03".into(),
            mode: ModeKind::Race,
            finish_height: Some(30.0),
            seconds: Some(45.0),
            drop_speed: Some(5.0),
            enemy: vec![EnemyCast {
                when: CastWhen::Height(10.0),
                spells: vec![SpellKind::Slow],
            }],
            ..default()
        },
        // Master (36) 6: place 56 bricks, 3 lives
        TrialDef {
            name: "SS_19".into(),
            mode: ModeKind::Survival,
            bricks: Some(56),
            lives: Some(3),
            drop_speed: Some(5.0),
            lines: Some(TrialLines {
                every: 16.0,
                spells: vec![
                    (SpellKind::Undo, false),
                    (SpellKind::Ivy, false),
                    (SpellKind::Petrify, false),
                    (SpellKind::Minibase, false),
                    (SpellKind::Ivy, false),
                ],
            }),
            enemy: vec![
                EnemyCast {
                    when: CastWhen::Brick(2),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(4),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(6),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(8),
                    spells: vec![SpellKind::Large, SpellKind::Mystery],
                },
                EnemyCast {
                    when: CastWhen::Brick(10),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(12),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(14),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(16),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(18),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(20),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(22),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(24),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(26),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(28),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(30),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(32),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(33),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(34),
                    spells: vec![SpellKind::Large, SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(36),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(38),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(40),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(42),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(44),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(46),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(48),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(50),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(52),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(53),
                    spells: vec![SpellKind::Mystery],
                },
                EnemyCast {
                    when: CastWhen::Brick(54),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(55),
                    spells: vec![SpellKind::Mystery],
                },
                EnemyCast {
                    when: CastWhen::Brick(56),
                    spells: vec![SpellKind::Large, SpellKind::Mystery],
                },
            ],
            ..default()
        },
        // Master (36) 7: place 9 bricks under a laser at 7.2, 1 life
        TrialDef {
            name: "SP_19".into(),
            mode: ModeKind::Puzzle,
            bricks: Some(9),
            laser: Some(7.2),
            order: vec![
                "S".into(),
                "L".into(),
                "O".into(),
                "O".into(),
                "T".into(),
                "J".into(),
                "T".into(),
                "L".into(),
                "T".into(),
            ],
            enemy: vec![
                EnemyCast {
                    when: CastWhen::Brick(0),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(9),
                    spells: vec![SpellKind::Large],
                },
            ],
            ..default()
        },
        // Master (36) 8: reach 75 cells in 145 s
        TrialDef {
            name: "SR_23".into(),
            mode: ModeKind::Race,
            finish_height: Some(75.0),
            seconds: Some(145.0),
            drop_speed: Some(5.0),
            lines: Some(TrialLines {
                every: 9.0,
                spells: vec![
                    (SpellKind::Undo, false),
                    (SpellKind::Minibase, false),
                    (SpellKind::Undo, false),
                    (SpellKind::Ivy, false),
                    (SpellKind::Minibase, false),
                    (SpellKind::Undo, false),
                    (SpellKind::Ivy, false),
                ],
            }),
            enemy: vec![
                EnemyCast {
                    when: CastWhen::Height(10.0),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Height(20.0),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Height(30.0),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Height(40.0),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Height(50.0),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Height(60.0),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Height(70.0),
                    spells: vec![SpellKind::Large],
                },
            ],
            ..default()
        },
        // Master (36) 9: place 8 bricks under a laser at 0.35, 1 life
        TrialDef {
            name: "SP_26".into(),
            mode: ModeKind::Puzzle,
            bricks: Some(8),
            laser: Some(0.35),
            order: vec![
                "O".into(),
                "I".into(),
                "L".into(),
                "T".into(),
                "L".into(),
                "T".into(),
                "Z".into(),
                "J".into(),
            ],
            ..default()
        },
        // Master (36) 10: place 99 bricks, max lives lives
        TrialDef {
            name: "SS_20".into(),
            mode: ModeKind::Survival,
            bricks: Some(99),
            lives: Some(MAX_TRIAL_LIVES),
            drop_speed: Some(5.0),
            lines: Some(TrialLines {
                every: 9.0,
                spells: vec![
                    (SpellKind::Ivy, false),
                    (SpellKind::Undo, false),
                    (SpellKind::Petrify, false),
                    (SpellKind::Ivy, false),
                    (SpellKind::Undo, false),
                    (SpellKind::Minibase, false),
                    (SpellKind::Ivy, false),
                ],
            }),
            enemy: vec![
                EnemyCast {
                    when: CastWhen::Brick(5),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(6),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(7),
                    spells: vec![SpellKind::Large, SpellKind::Mystery],
                },
                EnemyCast {
                    when: CastWhen::Brick(13),
                    spells: vec![SpellKind::Mystery],
                },
                EnemyCast {
                    when: CastWhen::Brick(14),
                    spells: vec![SpellKind::Mystery],
                },
                EnemyCast {
                    when: CastWhen::Brick(15),
                    spells: vec![SpellKind::Mystery],
                },
                EnemyCast {
                    when: CastWhen::Brick(28),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(29),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(30),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(31),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(40),
                    spells: vec![SpellKind::Fast, SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(41),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(42),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(43),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(44),
                    spells: vec![SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(50),
                    spells: vec![SpellKind::Fast, SpellKind::Mist],
                },
                EnemyCast {
                    when: CastWhen::Brick(54),
                    spells: vec![SpellKind::Fast],
                },
                EnemyCast {
                    when: CastWhen::Brick(65),
                    spells: vec![SpellKind::Large, SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(67),
                    spells: vec![SpellKind::Large, SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(69),
                    spells: vec![SpellKind::Large, SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(71),
                    spells: vec![SpellKind::Large, SpellKind::NoRotate],
                },
                EnemyCast {
                    when: CastWhen::Brick(90),
                    spells: vec![SpellKind::Fast, SpellKind::Mystery],
                },
                EnemyCast {
                    when: CastWhen::Brick(91),
                    spells: vec![SpellKind::Large],
                },
                EnemyCast {
                    when: CastWhen::Brick(93),
                    spells: vec![SpellKind::Mist],
                },
                EnemyCast {
                    when: CastWhen::Brick(95),
                    spells: vec![SpellKind::Large, SpellKind::NoRotate, SpellKind::Mystery],
                },
                EnemyCast {
                    when: CastWhen::Brick(99),
                    spells: vec![
                        SpellKind::Fast,
                        SpellKind::Large,
                        SpellKind::NoRotate,
                        SpellKind::Mystery,
                    ],
                },
            ],
            ..default()
        },
    ]
}
