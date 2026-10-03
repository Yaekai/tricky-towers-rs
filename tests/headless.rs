//! Runs the core loop without a window or GPU and checks that bricks fall,
//! land, stack, get counted as dropped, and that modes, spells and AI work.

use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use tricky_towers::TowerCorePlugin;
use tricky_towers::config::{GameConfig, ModeKind, PuzzleRules, SpellKind, SurvivalRules};
use tricky_towers::gameplay::{
    ActiveBrick, Brick, Match, Phase, PlayerInput, Tower, TowerState, TowerStatus,
};
use tricky_towers::spells::{Stone, VineJoint};

const STEP: f32 = 1.0 / 64.0;

fn headless_app(config: GameConfig) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin, AssetPlugin::default()))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            STEP,
        )))
        .insert_resource(config)
        .add_plugins(TowerCorePlugin);
    app.finish();
    app.cleanup();
    // Let the first fixed step build the towers.
    while app
        .world_mut()
        .query::<&Tower>()
        .iter(app.world())
        .next()
        .is_none()
    {
        app.update();
    }
    app
}

fn run_seconds(app: &mut App, seconds: f32, mut each_frame: impl FnMut(&mut App)) {
    for _ in 0..(seconds / STEP) as usize {
        each_frame(app);
        app.update();
    }
}

fn tower(app: &mut App, index: usize) -> TowerState {
    let world = app.world_mut();
    world
        .query::<(&Tower, &TowerState)>()
        .iter(world)
        .find(|(t, _)| t.index == index)
        .map(|(_, s)| s.clone())
        .expect("tower exists")
}

fn human_input(app: &mut App) -> Mut<'_, PlayerInput> {
    let world = app.world_mut();
    let entity = world
        .query::<(Entity, &Tower)>()
        .iter(world)
        .find(|(_, t)| t.human)
        .unwrap()
        .0;
    world.get_mut::<PlayerInput>(entity).unwrap()
}

fn fast_config() -> GameConfig {
    let mut config = GameConfig {
        fall_speed: 8.0,
        mode: ModeKind::Survival,
        survival: SurvivalRules {
            lives: 3,
            bricks_to_place: 100,
        },
        ..default()
    };
    config.magic.enabled = false;
    config
}

#[test]
fn bricks_land_and_stack_on_the_platform() {
    let mut app = headless_app(fast_config());
    // A single column of bricks topples eventually, so only build a few.
    run_seconds(&mut app, 4.0, |_| {});

    let state = tower(&mut app, 0);
    assert!(
        state.bricks_placed >= 3,
        "expected several landings, got {state:?}"
    );
    assert!(
        state.bricks_placed <= 8,
        "bricks are landing on nothing: {state:?}"
    );
    assert_eq!(
        state.bricks_dropped, 0,
        "nothing should fall off when dropped straight down"
    );
    assert!(state.height >= 2.0, "tower should grow, got {state:?}");

    // Exactly one brick is under player control at a time.
    let world = app.world_mut();
    let active = world
        .query_filtered::<(), With<ActiveBrick>>()
        .iter(world)
        .count();
    assert_eq!(active, 1);
    let bricks = world.query::<&Brick>().iter(world).count();
    assert_eq!(bricks as u32, state.bricks_placed + 1);
}

#[test]
fn steering_off_the_edge_costs_lives() {
    let mut app = headless_app(fast_config());
    run_seconds(&mut app, 12.0, |app| human_input(app).left_held = true);

    let state = tower(&mut app, 0);
    assert!(
        state.bricks_dropped >= 1,
        "expected dropped bricks, got {state:?}"
    );
    assert!(state.lives.unwrap() < 3);
}

#[test]
fn losing_every_life_ends_survival() {
    let mut app = headless_app(fast_config());
    run_seconds(&mut app, 30.0, |app| human_input(app).left_held = true);

    assert_eq!(tower(&mut app, 0).status, TowerStatus::Out);
    let game = app.world().resource::<Match>();
    assert_eq!(game.phase, Phase::Over);
    assert_eq!(game.winner, None);
}

#[test]
fn collapsing_tower_does_not_cascade() {
    // Regression: after a collapse, new bricks used to spawn inside falling
    // ones and "land" instantly, placing dozens of bricks per second.
    let mut app = headless_app(fast_config());
    run_seconds(&mut app, 15.0, |_| {});
    let state = tower(&mut app, 0);
    let max_reasonable = (15.0 * 8.0 / 4.0) as u32; // fall speed 8 cells/s, ~4+ cells per brick
    assert!(
        state.bricks_placed <= max_reasonable,
        "too many landings: {state:?}"
    );
}

#[test]
fn puzzle_is_won_by_staying_under_the_line() {
    let mut config = fast_config();
    config.mode = ModeKind::Puzzle;
    config.puzzle = PuzzleRules {
        bricks: 2,
        line_height: 6.0,
        settle_seconds: 1.0,
    };
    let mut app = headless_app(config);
    // Put the first brick on the left and the second on the right.
    let (mut frame, mut presses) = (0, [0; 2]);
    run_seconds(&mut app, 12.0, |app| {
        frame += 1;
        let placed = tower(app, 0).bricks_placed as usize;
        let mut input = human_input(app);
        if placed < 2 && presses[placed] < 5 && frame % 4 == 0 {
            presses[placed] += 1;
            if placed == 0 {
                input.left_pressed = true;
            } else {
                input.right_pressed = true;
            }
        }
    });

    let state = tower(&mut app, 0);
    assert_eq!(state.bricks_placed, 2, "{state:?}");
    assert_eq!(state.status, TowerStatus::Won, "{state:?}");
    let game = app.world().resource::<Match>();
    assert_eq!((game.phase, game.winner), (Phase::Over, Some(0)));
}

#[test]
fn puzzle_fails_when_crossing_the_line() {
    let mut config = fast_config();
    config.mode = ModeKind::Puzzle;
    config.puzzle = PuzzleRules {
        bricks: 10,
        line_height: 3.0,
        settle_seconds: 1.0,
    };
    let mut app = headless_app(config);
    run_seconds(&mut app, 20.0, |_| {});
    assert_eq!(tower(&mut app, 0).status, TowerStatus::Out);
    assert_eq!(app.world().resource::<Match>().phase, Phase::Over);
}

#[test]
fn ai_opponents_build_towers() {
    let mut config = fast_config();
    config.mode = ModeKind::Race;
    config.opponents = 2;
    let mut app = headless_app(config);
    run_seconds(&mut app, 20.0, |_| {});

    for index in 1..=2 {
        let state = tower(&mut app, index);
        assert!(
            state.bricks_placed >= 5,
            "wizard {index} barely played: {state:?}"
        );
        assert!(
            state.height >= 3.0,
            "wizard {index} tower is too low: {state:?}"
        );
        assert!(
            state.bricks_dropped * 2 <= state.bricks_placed,
            "wizard {index} drops too many bricks: {state:?}"
        );
    }
}

#[test]
fn spells_are_granted_and_cast() {
    let mut config = fast_config();
    config.magic.enabled = true;
    config.magic.bricks_per_spell = 2;
    config.magic.light = vec![SpellKind::Stone];
    let mut app = headless_app(config);
    let mut cast = false;
    run_seconds(&mut app, 8.0, |app| {
        if tower(app, 0).spell.is_some() && !cast {
            cast = true;
            human_input(app).cast_pressed = true;
        }
    });
    assert!(cast, "no spell was granted");
    let world = app.world_mut();
    let stones = world
        .query_filtered::<(), With<Stone>>()
        .iter(world)
        .count();
    assert!(stones >= 1, "Stone spell turned nothing to stone");
}

#[test]
fn vines_join_touching_bricks() {
    let mut config = fast_config();
    config.magic.enabled = true;
    config.magic.bricks_per_spell = 3;
    config.magic.light = vec![SpellKind::Vines];
    let mut app = headless_app(config);
    // Cast as soon as the spell arrives.
    for _ in 0..(8.0 / STEP) as usize {
        if tower(&mut app, 0).spell.is_some() {
            break;
        }
        app.update();
    }
    assert!(tower(&mut app, 0).spell.is_some(), "no spell was granted");
    human_input(&mut app).cast_pressed = true;
    app.update();
    // The vines must hold the bricks where they are, not fling them about.
    let dropped = tower(&mut app, 0).bricks_dropped;
    run_seconds(&mut app, 2.0, |_| {});
    let state = tower(&mut app, 0);
    assert_eq!(
        state.bricks_dropped, dropped,
        "vined tower fell apart: {state:?}"
    );
    let world = app.world_mut();
    let vines = world.query::<&VineJoint>().iter(world).count();
    assert!(vines >= 1, "Vines joined nothing");
    let fastest = world
        .query::<&avian2d::prelude::LinearVelocity>()
        .iter(world)
        .map(|v| v.0.length())
        .fold(0.0, f32::max);
    assert!(
        fastest < 32.0 * 20.0,
        "something is flying at {fastest} units/s"
    );
}

#[test]
fn dark_spells_hit_an_opponent() {
    let mut config = fast_config();
    config.mode = ModeKind::Race;
    config.opponents = 1;
    config.magic.enabled = true;
    config.magic.bricks_per_spell = 2;
    config.magic.light = vec![];
    config.magic.dark = vec![SpellKind::Enlarge];
    let mut app = headless_app(config);
    run_seconds(&mut app, 10.0, |app| {
        if tower(app, 0).spell.is_some() {
            human_input(app).cast_pressed = true;
        }
    });
    let log = app.world().resource::<tricky_towers::spells::SpellLog>();
    assert!(
        log.0
            .iter()
            .any(|c| c.caster == 0 && c.target == 1 && c.spell == SpellKind::Enlarge),
        "expected you to cast Enlarge on wizard 1, log: {:?}",
        log.0
    );
    let world = app.world_mut();
    let big = world.query::<&Brick>().iter(world).any(|b| b.scale > 1.0);
    assert!(big, "no enlarged brick appeared");
}
