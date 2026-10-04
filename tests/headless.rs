//! Runs the core loop without a window or GPU and checks that bricks fall,
//! land, stack, get counted as dropped, and that modes, spells and AI work.

use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use tricky_towers::TowerCorePlugin;
use tricky_towers::config::{
    AwardRule, Curse, DarkSet, GameConfig, ModeKind, PuzzleRules, SpellKind, SurvivalRules,
};
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
        mode: ModeKind::Survival,
        spawn_gap: 6.0,
        survival: SurvivalRules {
            lives: 3,
            bricks_to_place: 100,
            drop_speed: 8.0,
            ..default()
        },
        ..default()
    };
    config.race.drop_speed = 8.0;
    config.puzzle.drop_speed = 8.0;
    config.puzzle.line_random = 0.0;
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
fn a_lost_life_gives_a_moment_of_grace() {
    let mut config = fast_config();
    config.survival.invincible_seconds = 1000.0;
    let mut app = headless_app(config);
    run_seconds(&mut app, 12.0, |app| human_input(app).left_held = true);
    let state = tower(&mut app, 0);
    assert!(state.bricks_dropped >= 2, "{state:?}");
    assert_eq!(state.lives, Some(2), "only the first drop costs a life");
}

#[test]
fn survival_ends_after_the_last_brick_and_a_countdown() {
    let mut config = fast_config();
    config.survival.bricks_to_place = 3;
    config.survival.finish_countdown = 1.0;
    let mut app = headless_app(config);
    let finished = run_until(&mut app, 10.0, |app| {
        tower(app, 0).status == TowerStatus::Won
    });
    let state = tower(&mut app, 0);
    assert!(finished, "{state:?}");
    assert_eq!(state.bricks_used(), 3, "{state:?}");
}

#[test]
fn a_dropped_puzzle_brick_lowers_the_laser() {
    let mut config = fast_config();
    config.mode = ModeKind::Puzzle;
    config.puzzle.line_height = 20.0;
    let mut app = headless_app(config);
    run_until(&mut app, 8.0, |app| {
        human_input(app).left_held = true;
        tower(app, 0).bricks_dropped >= 1
    });
    let state = tower(&mut app, 0);
    assert_eq!(
        state.status,
        TowerStatus::Building,
        "a drop ended the puzzle"
    );
    assert!(state.laser(app.world().resource::<GameConfig>()) < 20.0);
}

#[test]
fn puzzle_laser_is_shared_and_changes_each_round() {
    let mut config = fast_config();
    config.mode = ModeKind::Puzzle;
    config.opponents = 1;
    config.puzzle.line_random = 4.0;
    let mut app = headless_app(config);
    let first = (tower(&mut app, 0).laser_base, tower(&mut app, 1).laser_base);
    assert_eq!(first.0, first.1, "everyone gets the same laser");
    assert!((3.2..=7.2).contains(&first.0), "{first:?}");
    app.world_mut()
        .resource_mut::<tricky_towers::gameplay::MatchRequest>()
        .pending = true;
    run_seconds(&mut app, 0.1, |_| {});
    assert_ne!(
        tower(&mut app, 0).laser_base,
        first.0,
        "same laser two rounds running"
    );
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
        ..config.puzzle.clone()
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
        ..config.puzzle.clone()
    };
    let mut app = headless_app(config);
    run_seconds(&mut app, 20.0, |_| {});
    assert_eq!(tower(&mut app, 0).status, TowerStatus::Out);
    assert_eq!(app.world().resource::<Match>().phase, Phase::Over);
}

#[test]
fn several_players_get_their_own_towers() {
    let mut config = fast_config();
    config.mode = ModeKind::Race;
    config.players = 2;
    config.opponents = 1;
    let mut app = headless_app(config);
    let world = app.world_mut();
    let mut towers: Vec<(usize, Option<usize>)> = world
        .query::<&Tower>()
        .iter(world)
        .map(|t| (t.index, t.player))
        .collect();
    towers.sort();
    assert_eq!(towers, vec![(0, Some(0)), (1, Some(1)), (2, None)]);
    // Player 2 steers their own brick through their own input.
    run_seconds(&mut app, 0.1, |_| {});
    let world = app.world_mut();
    let (entity, active) = world
        .query::<(Entity, &Tower, &TowerState)>()
        .iter(world)
        .find(|(_, t, _)| t.player == Some(1))
        .map(|(e, _, s)| (e, s.active.unwrap()))
        .unwrap();
    let x = |app: &App| {
        app.world()
            .get::<avian2d::prelude::Position>(active)
            .unwrap()
            .0
            .x
    };
    let start = x(&app);
    app.world_mut()
        .get_mut::<PlayerInput>(entity)
        .unwrap()
        .right_pressed = true;
    run_seconds(&mut app, 0.05, |_| {});
    assert!(x(&app) > start, "player 2's brick did not move");
}

#[test]
fn ai_opponents_build_towers() {
    let mut config = fast_config();
    config.mode = ModeKind::Race;
    config.opponents = 2;
    // The real drop speed; at test speed the AI can't steer in time.
    config.race.drop_speed = 3.0;
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

/// Magic where every spell line is one cell apart and always offers `light`
/// and the curse `dark`.
fn magic_config(light: SpellKind, dark: &[SpellKind]) -> GameConfig {
    let mut config = fast_config();
    config.magic.enabled = true;
    config.magic.line_spacing = 1.0;
    config.magic.award_rules = vec![AwardRule {
        light: vec![(light, 1)],
        dark: Some("test".into()),
        ..default()
    }];
    config.magic.dark_sets = vec![DarkSet {
        name: "test".into(),
        curses: vec![Curse::new(dark, 1, None)],
    }];
    config
}

/// Steps until `done` holds, for at most `seconds`; returns whether it did.
fn run_until(app: &mut App, seconds: f32, mut done: impl FnMut(&mut App) -> bool) -> bool {
    for _ in 0..(seconds / STEP) as usize {
        if done(app) {
            return true;
        }
        app.update();
    }
    done(app)
}

/// Steps until your light slot holds `spell`.
fn wait_for_light(app: &mut App, spell: SpellKind) {
    for _ in 0..(10.0 / STEP) as usize {
        if tower(app, 0).light_spell == Some(spell) {
            return;
        }
        app.update();
    }
    panic!("{spell:?} was never granted: {:?}", tower(app, 0));
}

/// Steps until a new brick is in your hands.
fn wait_for_next_brick(app: &mut App) {
    let first = tower(app, 0).active;
    for _ in 0..(10.0 / STEP) as usize {
        if tower(app, 0).active.is_some_and(|a| Some(a) != first) {
            return;
        }
        app.update();
    }
    panic!("no new brick: {:?}", tower(app, 0));
}

fn count<F: bevy::ecs::query::QueryFilter>(app: &mut App) -> usize {
    let world = app.world_mut();
    world.query_filtered::<(), F>().iter(world).count()
}

fn log(app: &App) -> Vec<tricky_towers::spells::SpellCast> {
    app.world()
        .resource::<tricky_towers::spells::SpellLog>()
        .0
        .clone()
}

fn your_tower_mut(app: &mut App) -> Mut<'_, TowerState> {
    let world = app.world_mut();
    let entity = world
        .query::<(Entity, &Tower)>()
        .iter(world)
        .find(|(_, t)| t.human)
        .unwrap()
        .0;
    world.get_mut::<TowerState>(entity).unwrap()
}

#[test]
fn petrify_turns_the_brick_in_hand_to_stone() {
    let mut app = headless_app(magic_config(SpellKind::Petrify, &[]));
    wait_for_light(&mut app, SpellKind::Petrify);
    human_input(&mut app).cast_light_pressed = true;
    run_seconds(&mut app, 0.1, |_| {});
    assert_eq!(tower(&mut app, 0).light_spell, None, "slot not emptied");
    assert!(
        run_until(&mut app, 3.0, |app| count::<With<Stone>>(app) >= 1),
        "nothing turned to stone"
    );
}

#[test]
fn a_spell_line_gives_a_pair_and_casting_one_drops_the_other() {
    let mut config = magic_config(SpellKind::Undo, &[SpellKind::Large]);
    config.mode = ModeKind::Race;
    config.opponents = 1;
    let mut app = headless_app(config);
    wait_for_light(&mut app, SpellKind::Undo);
    let you = tower(&mut app, 0);
    assert_eq!(
        you.dark_spell.map(|c| c.spells),
        Some(vec![SpellKind::Large])
    );
    human_input(&mut app).cast_light_pressed = true;
    run_seconds(&mut app, 0.05, |_| {});
    let you = tower(&mut app, 0);
    assert!(
        you.light_spell.is_none() && you.dark_spell.is_none(),
        "{you:?}"
    );
}

#[test]
fn no_curses_without_opponents() {
    let mut app = headless_app(magic_config(SpellKind::Ivy, &[SpellKind::Fast]));
    wait_for_light(&mut app, SpellKind::Ivy);
    assert!(tower(&mut app, 0).dark_spell.is_none());
}

#[test]
fn ivy_vines_the_brick_to_what_it_lands_on() {
    let mut app = headless_app(magic_config(SpellKind::Ivy, &[]));
    wait_for_light(&mut app, SpellKind::Ivy);
    human_input(&mut app).cast_light_pressed = true;
    let dropped = tower(&mut app, 0).bricks_dropped;
    assert!(
        run_until(&mut app, 3.0, |app| count::<With<VineJoint>>(app) >= 1),
        "Ivy joined nothing"
    );
    run_seconds(&mut app, 1.0, |_| {});
    let state = tower(&mut app, 0);
    assert_eq!(
        state.bricks_dropped, dropped,
        "vined tower fell apart: {state:?}"
    );
    let world = app.world_mut();
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
fn ivy_onto_stone_petrifies_the_cluster() {
    // Regression: joining a stone (static) brick panicked inside avian
    // ("Neither body ... is in an island").
    let mut app = headless_app(magic_config(SpellKind::Petrify, &[]));
    wait_for_light(&mut app, SpellKind::Petrify);
    human_input(&mut app).cast_light_pressed = true;
    assert!(
        run_until(&mut app, 3.0, |app| count::<With<Stone>>(app) >= 1),
        "the petrified brick is not stone"
    );
    let stones = count::<With<Stone>>(&mut app);
    let ivy = tricky_towers::gameplay::BrickMod {
        ivy: true,
        ..default()
    };
    your_tower_mut(&mut app).queue_mod(0, &ivy);
    assert!(
        run_until(&mut app, 4.0, |app| count::<With<Stone>>(app) > stones),
        "the Ivy brick on stone did not petrify"
    );
    run_seconds(&mut app, 0.5, |_| {});
}

#[test]
fn curses_hit_every_opponent() {
    let mut config = magic_config(SpellKind::Ivy, &[SpellKind::Large]);
    config.mode = ModeKind::Race;
    config.opponents = 2;
    let mut app = headless_app(config);
    for _ in 0..(10.0 / STEP) as usize {
        if tower(&mut app, 0).dark_spell.is_some() {
            human_input(&mut app).cast_dark_pressed = true;
            run_seconds(&mut app, 0.05, |_| {});
            break;
        }
        app.update();
    }
    for target in [1, 2] {
        assert!(
            log(&app)
                .iter()
                .any(|c| c.caster == 0 && c.target == target && c.spell == SpellKind::Large),
            "expected Large on wizard {target}, log: {:?}",
            log(&app)
        );
    }
    let world = app.world_mut();
    let big = world.query::<&Brick>().iter(world).any(|b| b.scale > 1.0);
    assert!(big, "no large brick appeared");
}

/// Casts `spell` on your own tower directly, as if from a spell line.
fn cast_now(app: &mut App, light: SpellKind, dark: Option<SpellKind>) {
    {
        let mut state = your_tower_mut(app);
        state.light_spell = Some(light);
        state.dark_spell = dark.map(|s| Curse::new(&[s], 1, None));
    }
    let mut input = human_input(app);
    if dark.is_some() {
        input.cast_dark_pressed = true;
    } else {
        input.cast_light_pressed = true;
    }
    // Let one fixed step run.
    run_seconds(app, 0.03, |_| {});
}

#[test]
fn undo_zaps_your_last_brick() {
    let mut config = fast_config();
    config.magic.enabled = true;
    let mut app = headless_app(config);
    run_seconds(&mut app, 2.0, |_| {});
    let before = tower(&mut app, 0);
    let last = before.last_placed.expect("a brick was placed");
    cast_now(&mut app, SpellKind::Undo, None);
    assert!(app.world().get_entity(last).is_ok(), "gone before the zap");
    run_seconds(&mut app, 1.0, |_| {});
    let after = tower(&mut app, 0);
    assert!(
        app.world().get_entity(last).is_err(),
        "last brick still there"
    );
    assert_eq!(
        after.bricks_dropped, before.bricks_dropped,
        "Undo counted as a drop"
    );
}

#[test]
fn extra_life_orb_gives_a_life() {
    let mut config = fast_config();
    config.magic.enabled = true;
    let mut app = headless_app(config);
    run_seconds(&mut app, 0.5, |_| {});
    let lives = tower(&mut app, 0).lives.unwrap();
    cast_now(&mut app, SpellKind::ExtraLife, None);
    // Move the orb into the falling brick's path.
    let active = tower(&mut app, 0).active.unwrap();
    let at = app
        .world()
        .get::<avian2d::prelude::Position>(active)
        .unwrap()
        .0;
    let world = app.world_mut();
    let mut orb = world
        .query_filtered::<&mut Transform, With<tricky_towers::spells::LifeOrb>>()
        .single_mut(world)
        .expect("an orb appeared");
    orb.translation = at.extend(0.0);
    run_seconds(&mut app, 0.1, |_| {});
    assert_eq!(tower(&mut app, 0).lives, Some(lives + 1));
    assert_eq!(count::<With<tricky_towers::spells::LifeOrb>>(&mut app), 0);
}

#[test]
fn minibase_adds_a_stone_ledge() {
    let mut config = fast_config();
    config.magic.enabled = true;
    let mut app = headless_app(config);
    run_seconds(&mut app, 0.5, |_| {});
    cast_now(&mut app, SpellKind::Minibase, None);
    run_seconds(&mut app, 0.1, |_| {});
    assert_eq!(count::<(With<Stone>, With<Brick>)>(&mut app), 1);
}

#[test]
fn auto_build_places_stone_bricks_by_itself() {
    let mut config = fast_config();
    config.magic.enabled = true;
    let mut app = headless_app(config);
    run_seconds(&mut app, 0.2, |_| {});
    cast_now(&mut app, SpellKind::AutoBuild, None);
    wait_for_next_brick(&mut app);
    run_seconds(&mut app, 0.5, |_| {});
    let state = tower(&mut app, 0);
    assert!(state.bricks_placed >= 6, "{state:?}");
    assert_eq!(count::<With<Stone>>(&mut app), 5);
}

#[test]
fn large_grows_the_brick_in_hand() {
    let mut config = fast_config();
    config.mode = ModeKind::Race;
    config.opponents = 1;
    config.magic.enabled = true;
    let mut app = headless_app(config);
    run_seconds(&mut app, 0.1, |_| {});
    cast_now(&mut app, SpellKind::Ivy, Some(SpellKind::Large));
    // The curse lands on wizard 1's brick in hand.
    let theirs = tower(&mut app, 1).active.unwrap();
    assert_eq!(app.world().get::<Brick>(theirs).unwrap().scale, 2.0);
}

#[test]
fn mystery_reshapes_the_brick_in_hand() {
    let mut config = fast_config();
    config.mode = ModeKind::Race;
    config.opponents = 1;
    config.magic.enabled = true;
    let mut app = headless_app(config);
    run_seconds(&mut app, 0.1, |_| {});
    cast_now(&mut app, SpellKind::Ivy, Some(SpellKind::Mystery));
    let theirs = tower(&mut app, 1).active.unwrap();
    let brick = app.world().get::<Brick>(theirs).unwrap();
    assert!(brick.cells.len() >= 8, "not a Mystery shape: {brick:?}");
    assert!(brick.mass > 1.0);
}

#[test]
fn bubble_lifts_the_last_brick() {
    let mut config = fast_config();
    config.mode = ModeKind::Race;
    config.opponents = 1;
    // A brick order where the wizard's last brick is free to rise.
    config.race.same_bricks = true;
    config.magic.enabled = true;
    let mut app = headless_app(config);
    run_seconds(&mut app, 2.5, |_| {});
    let last = tower(&mut app, 1)
        .last_placed
        .expect("wizard 1 placed a brick");
    let y = |app: &App| {
        app.world()
            .get::<avian2d::prelude::Position>(last)
            .unwrap()
            .0
            .y
    };
    let before = y(&app);
    cast_now(&mut app, SpellKind::Ivy, Some(SpellKind::Bubble));
    assert!(
        app.world()
            .get::<tricky_towers::spells::Bubbled>(last)
            .is_some()
    );
    let rose = run_until(&mut app, 1.0, |app| y(app) > before + 4.0);
    assert!(rose, "bubble did not lift: {before} -> {}", y(&app));
}

#[test]
fn ghost_carries_off_the_last_brick() {
    let mut config = fast_config();
    config.mode = ModeKind::Race;
    config.opponents = 1;
    config.magic.enabled = true;
    config.magic.ghost_seconds = 1.0;
    let mut app = headless_app(config);
    run_seconds(&mut app, 2.5, |_| {});
    let last = tower(&mut app, 1)
        .last_placed
        .expect("wizard 1 placed a brick");
    let x = |app: &App| {
        app.world()
            .get::<avian2d::prelude::Position>(last)
            .unwrap()
            .0
            .x
    };
    let before = x(&app);
    cast_now(&mut app, SpellKind::Ivy, Some(SpellKind::Ghost));
    assert!(
        app.world()
            .get::<tricky_towers::spells::Ghosted>(last)
            .is_some()
    );
    run_seconds(&mut app, 0.9, |_| {});
    assert!(
        (x(&app) - before).abs() > 32.0,
        "the brick was not carried sideways"
    );
    run_seconds(&mut app, 0.3, |_| {});
    assert!(
        app.world()
            .get::<tricky_towers::spells::Ghosted>(last)
            .is_none(),
        "never dropped"
    );
}

/// Angle of your falling brick, in degrees.
fn active_angle(app: &mut App) -> Option<f32> {
    let active = tower(app, 0).active?;
    app.world()
        .get::<avian2d::prelude::Rotation>(active)
        .map(|r| r.as_degrees())
}

#[test]
fn bricks_rotate_both_ways() {
    let mut app = headless_app(fast_config());
    run_seconds(&mut app, 0.05, |_| {});
    human_input(&mut app).rotate_pressed = true;
    run_seconds(&mut app, 0.05, |_| {});
    let cw = active_angle(&mut app).unwrap();
    assert!((cw + 90.0).abs() < 1.0, "clockwise turn gave {cw}");
    human_input(&mut app).rotate_ccw_pressed = true;
    run_seconds(&mut app, 0.05, |_| {});
    let back = active_angle(&mut app).unwrap();
    assert!(back.abs() < 1.0, "turning back gave {back}");
}

#[test]
fn no_rotate_and_auto_rotate_curses() {
    let mut app = headless_app(fast_config());
    let curse = tricky_towers::gameplay::BrickMod {
        no_rotate: true,
        auto_rotate: Some(0.2),
        ..default()
    };
    your_tower_mut(&mut app).queue_mod(0, &curse);
    wait_for_next_brick(&mut app);
    let start = active_angle(&mut app).unwrap();
    human_input(&mut app).rotate_pressed = true;
    run_seconds(&mut app, 0.1, |_| {});
    assert!(
        (active_angle(&mut app).unwrap() - start).abs() < 1.0,
        "a cursed brick turned by hand"
    );
    run_seconds(&mut app, 0.15, |_| {});
    assert!(
        (active_angle(&mut app).unwrap() - start).abs() > 45.0,
        "Auto Rotate brick did not turn"
    );
}

#[test]
fn holding_down_accelerates() {
    let mut config = fast_config();
    config.survival.drop_speed = 1.0;
    config.spawn_gap = 40.0;
    let mut app = headless_app(config);
    let y = |app: &mut App| {
        let active = tower(app, 0).active.unwrap();
        app.world()
            .get::<avian2d::prelude::Position>(active)
            .unwrap()
            .0
            .y
    };
    let fall = |app: &mut App| {
        let a = y(app);
        run_seconds(app, 0.1, |app| human_input(app).fast_fall_held = true);
        a - y(app)
    };
    let first = fall(&mut app);
    let second = fall(&mut app);
    let third = fall(&mut app);
    assert!(
        second > first && third > second,
        "no speed-up: {first} {second} {third}"
    );
}

#[test]
fn moves_repeat_after_a_delay() {
    let mut config = fast_config();
    config.survival.drop_speed = 0.5;
    let mut app = headless_app(config);
    run_seconds(&mut app, 0.05, |_| {});
    let x = |app: &mut App| {
        let active = tower(app, 0).active.unwrap();
        app.world()
            .get::<avian2d::prelude::Position>(active)
            .unwrap()
            .0
            .x
    };
    let start = x(&mut app);
    human_input(&mut app).right_pressed = true;
    run_seconds(&mut app, 0.15, |app| human_input(app).right_held = true);
    assert_eq!(
        x(&mut app) - start,
        16.0,
        "one half-cell step before the repeat"
    );
    run_seconds(&mut app, 0.2, |app| human_input(app).right_held = true);
    assert!(x(&mut app) - start >= 48.0, "held move never repeated");
}

#[test]
fn race_finish_comes_down_late_in_the_race() {
    let mut config = fast_config();
    config.mode = ModeKind::Race;
    config.race.finish_drop_after = 0.5;
    config.race.finish_drop_speed = 4.0;
    let mut app = headless_app(config);
    run_seconds(&mut app, 1.5, |_| {});
    let game = app.world().resource::<Match>();
    assert!(game.finish_height < 45.0 - 2.0, "{game:?}");
}

#[test]
fn race_spell_line_slides_down_after_waiting() {
    let mut config = fast_config();
    config.mode = ModeKind::Race;
    config.magic.enabled = true;
    config.magic.line_wait = 0.2;
    config.magic.line_spacing = 20.0;
    config.magic.line_rush_beyond = 100.0;
    let mut app = headless_app(config);
    run_seconds(&mut app, 0.1, |_| {});
    let first = tower(&mut app, 0).spell_line.expect("a spell line");
    run_seconds(&mut app, 1.0, |_| {});
    let later = tower(&mut app, 0).spell_line.expect("a spell line");
    assert!(later < first, "line stayed at {first}");
}

#[test]
#[ignore = "slow: about two minutes of AI matches"]
fn stress_ai_spells() {
    for seed in 1..13u64 {
        let mut config = fast_config();
        config.race.drop_speed = 4.0;
        config.survival.drop_speed = 4.0;
        config.seed = seed * 7777;
        config.mode = if seed % 2 == 0 {
            ModeKind::Survival
        } else {
            ModeKind::Race
        };
        config.opponents = 3;
        config.magic.enabled = true;
        config.magic.line_spacing = 2.0;
        let mut app = headless_app(config);
        run_seconds(&mut app, 90.0, |app| {
            if app.world().resource::<Match>().phase == Phase::Over {
                app.world_mut()
                    .resource_mut::<tricky_towers::gameplay::MatchRequest>()
                    .pending = true;
            }
        });
        eprintln!(
            "seed {seed} ok, spells {:?}",
            app.world()
                .resource::<tricky_towers::spells::SpellLog>()
                .0
                .len()
        );
    }
}

#[test]
fn difficulty_sets_the_originals_values() {
    use tricky_towers::config::{Difficulty, PuzzleFloor};
    let mut config = GameConfig {
        difficulty: Some(Difficulty::Easy),
        ..GameConfig::default()
    };
    config.apply_difficulty();
    assert_eq!(config.race.finish_height, 26.0);
    assert_eq!(config.survival.bricks_to_place, 33);
    assert!(!config.race.dark_magic);
    assert_eq!(config.puzzle.start_spell, Some(SpellKind::Undo));
    config.difficulty = Some(Difficulty::Pro);
    config.apply_difficulty();
    assert_eq!(config.race.finish_height, 38.0);
    assert!(config.survival.curse_waves && config.race.same_bricks);
    assert_eq!(config.puzzle.floor, PuzzleFloor::Pro);
}

#[test]
fn puzzle_floors_follow_each_difficulty() {
    use tricky_towers::config::PuzzleFloor;
    use tricky_towers::gameplay::puzzle_floor;
    let mut config = GameConfig::default();
    for seed in 1..20u64 {
        let mut roll = seed;
        config.puzzle.floor = PuzzleFloor::Easy;
        let easy = puzzle_floor(&config, &mut roll);
        assert_eq!(easy.len(), 5);
        assert!(easy.iter().all(|&(_, top, _)| (-2.0..=0.0).contains(&top)));
        config.puzzle.floor = PuzzleFloor::Normal;
        let normal = puzzle_floor(&config, &mut roll);
        assert!((2..=6).contains(&normal.len()));
        assert!(
            normal
                .iter()
                .all(|&(_, top, _)| (-4.0..=0.0).contains(&top))
        );
        config.puzzle.floor = PuzzleFloor::Pro;
        let pro = puzzle_floor(&config, &mut roll);
        assert_eq!(pro.iter().filter(|c| c.1 == 0.0).count(), 3);
        assert!(pro.iter().all(|c| c.0.abs() <= 2.0));
    }
}

#[test]
fn survival_pro_waves_curse_new_bricks() {
    let mut config = fast_config();
    config.survival.curse_waves = true;
    config.survival.dark_magic = false;
    config.survival.waves.every = (0.5, 0.5);
    let mut app = headless_app(config);
    let cursed = run_until(&mut app, 3.0, |app| {
        let mut towers = app
            .world_mut()
            .query::<&tricky_towers::gameplay::TowerState>();
        towers
            .iter(app.world())
            .any(|s| s.wave_mod.is_some() || s.mist_timer > 0.0)
    });
    assert!(cursed, "no curse wave arrived");
}

#[test]
fn endless_pacer_stops_eases_down_and_speeds_up_at_checkpoints() {
    use tricky_towers::config::EndlessRules;
    use tricky_towers::gameplay::{advance_pacer, pacer_checkpoint};
    let rules = EndlessRules::default();
    let mut state = TowerState::new(&GameConfig::default(), 1);
    // Slides in from -15 to -5 over the first second, then rises at 0.2/s.
    assert_eq!(state.pacer, -15.0);
    for _ in 0..50 {
        advance_pacer(&rules, &mut state, 0.02);
    }
    assert!((state.pacer + 5.0).abs() < 0.05, "{}", state.pacer);
    advance_pacer(&rules, &mut state, 1.0);
    assert!((state.pacer + 4.8).abs() < 0.05, "{}", state.pacer);

    // A checkpoint: 2 cells down over 1 s, a 2 s wait, then 0.05 faster.
    let at = state.pacer;
    pacer_checkpoint(&rules, &mut state);
    advance_pacer(&rules, &mut state, 1.0);
    assert!((state.pacer - (at - 2.0)).abs() < 1e-4, "{}", state.pacer);
    advance_pacer(&rules, &mut state, 2.0);
    assert!(
        (state.pacer - (at - 2.0)).abs() < 1e-4,
        "moved while waiting"
    );
    advance_pacer(&rules, &mut state, 1.0);
    assert!(
        (state.pacer - (at - 2.0 + 0.25)).abs() < 1e-4,
        "{}",
        state.pacer
    );
    // The step grows by 0.01 per checkpoint, up to 0.1; the speed tops out at 1.5.
    assert!((state.pacer_step - 0.06).abs() < 1e-6);
    for _ in 0..100 {
        pacer_checkpoint(&rules, &mut state);
    }
    assert_eq!((state.pacer_step, state.pacer_speed), (0.1, 1.5));

    // Spell lines every 10 cells, the gaps one longer each past 150.
    let lines: Vec<f32> = (0..17).map(|n| rules.line_height(n)).collect();
    assert_eq!(lines[0], 10.0);
    assert_eq!(lines[14], 150.0);
    assert_eq!(&lines[15..], &[161.0, 173.0]);
    assert_eq!(rules.score(4.21), 43);
}

#[test]
fn endless_race_ends_when_the_line_catches_the_tower() {
    let mut config = fast_config();
    config.mode = ModeKind::Endless;
    config.endless.intro_seconds = 0.0;
    config.endless.start_height = -0.5;
    config.endless.start_speed = 2.0;
    let mut app = headless_app(config);
    // An empty base is caught within a second.
    run_seconds(&mut app, 1.0, |_| {});
    assert_eq!(tower(&mut app, 0).status, TowerStatus::Out);
    assert_eq!(app.world().resource::<Match>().phase, Phase::Over);
}

#[test]
fn race_pro_wind_blows_one_way_and_puzzle_lanes_are_wider() {
    use tricky_towers::config::Difficulty;
    let mut config = fast_config();
    config.mode = ModeKind::Race;
    config.difficulty = Some(Difficulty::Pro);
    let mut app = headless_app(config.clone());
    let wind = app.world().resource::<Match>().wind;
    assert_eq!(wind.abs(), 1.0, "wind {wind}");
    // Placed bricks drift downwind; nothing breaks.
    run_seconds(&mut app, 4.0, |_| {});
    let world = app.world_mut();
    let drift: Vec<f32> = world
        .query::<(&Brick, &avian2d::prelude::LinearVelocity)>()
        .iter(world)
        .map(|(_, v)| v.0.x)
        .collect();
    assert!(drift.iter().all(|v| v.is_finite()));

    config.difficulty = Some(Difficulty::Normal);
    let app = headless_app(config.clone());
    assert_eq!(app.world().resource::<Match>().wind, 0.0);

    // Puzzle: 1.2x the normal limit, or the tower's widest brick plus 1.5.
    config.mode = ModeKind::Puzzle;
    assert!((config.lane_reach(0.0) - 8.4).abs() < 1e-4);
    assert_eq!(config.lane_reach(10.0), 11.5);
}
