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
use tricky_towers::spells::{IvyCluster, Stone};

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
    // The bricks over the laser are zapped away, like the original's.
    let mut zapped = false;
    run_seconds(&mut app, 20.0, |app| {
        let world = app.world_mut();
        zapped |= world
            .query::<&tricky_towers::spells::Zapped>()
            .iter(world)
            .next()
            .is_some();
    });
    assert_eq!(tower(&mut app, 0).status, TowerStatus::Out);
    assert_eq!(app.world().resource::<Match>().phase, Phase::Over);
    assert!(zapped);
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
        run_until(&mut app, 3.0, |app| count::<With<IvyCluster>>(app) >= 1),
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
fn ivy_onto_stone_makes_the_cluster_static() {
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
    // As in the original, the cluster holds still like the stone, without
    // turning its other bricks to stone.
    assert!(
        run_until(&mut app, 4.0, |app| {
            let world = app.world_mut();
            world
                .query_filtered::<&avian2d::prelude::RigidBody, With<IvyCluster>>()
                .iter(world)
                .any(|b| *b == avian2d::prelude::RigidBody::Static)
        }),
        "the Ivy brick on stone did not make a static cluster"
    );
    assert_eq!(count::<With<Stone>>(&mut app), stones);
    run_seconds(&mut app, 0.5, |_| {});
}

#[test]
fn ivy_casts_stacking_up_do_not_panic() {
    // Regression: a fourth Ivy brick joining three clusters panicked inside avian
    // ("Neither body ... is in an island") with a stale contact on a released brick.
    for seed in [7777, 23331] {
        let mut config = magic_config(SpellKind::Ivy, &[]);
        config.seed = seed;
        let mut app = headless_app(config);
        for _ in 0..4 {
            wait_for_next_brick(&mut app);
            cast_now(&mut app, SpellKind::Ivy, None);
            run_seconds(&mut app, 1.5, |_| {});
        }
    }
}

#[test]
fn ivy_clusters_loosened_by_bubbles_and_ghosts_do_not_panic() {
    for seed in [7777, 23331, 4242] {
        let mut config = magic_config(SpellKind::Ivy, &[]);
        config.seed = seed;
        let mut app = headless_app(config);
        for round in 0..6 {
            if tower(&mut app, 0).status == TowerStatus::Out {
                break;
            }
            wait_for_next_brick(&mut app);
            let loosener = if round % 2 == 0 {
                SpellKind::Bubble
            } else {
                SpellKind::Ghost
            };
            cast_now(&mut app, SpellKind::Ivy, Some(loosener));
            run_seconds(&mut app, 1.5, |_| {});
        }
    }
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
    run_seconds(&mut app, 2.0, |_| {});
    // Cast right after the wizard places a brick, so it's still the last one.
    let earlier = tower(&mut app, 1).last_placed;
    let placed = run_until(&mut app, 3.0, |app| {
        let now = tower(app, 1).last_placed;
        now.is_some() && now != earlier
    });
    assert!(placed, "wizard 1 placed no brick");
    let last = tower(&mut app, 1).last_placed.unwrap();
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
    // Slow enough that the brick is still in hand after a full swing.
    let mut config = fast_config();
    config.survival.drop_speed = 1.0;
    config.spawn_gap = 20.0;
    let mut app = headless_app(config);
    let curse = tricky_towers::gameplay::BrickMod {
        no_rotate: true,
        auto_rotate: Some(1.65),
        ..default()
    };
    // Curse the brick in hand straight away.
    run_seconds(&mut app, 0.05, |_| {});
    let active = tower(&mut app, 0).active.unwrap();
    app.world_mut().get_mut::<ActiveBrick>(active).unwrap().mods = curse;
    let start = active_angle(&mut app).unwrap();
    human_input(&mut app).rotate_pressed = true;
    run_seconds(&mut app, 0.1, |_| {});
    // Only the wind-back (at most 15°), never a 90° turn by hand.
    assert!(
        (active_angle(&mut app).unwrap() - start).abs() <= 15.5,
        "a cursed brick turned by hand"
    );
    run_seconds(&mut app, 0.7, |_| {});
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
    config.survival.waves.gap = (1, 1);
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
fn curse_waves_deal_each_level_without_repeats_and_move_on_after_two() {
    use tricky_towers::spells::WaveDeck;
    let rules = tricky_towers::config::WaveRules::default();
    let mut deck = WaveDeck::default();
    let mut n = 0u64;
    let mut roll = || {
        n += 7;
        n
    };
    let drawn: Vec<_> = (0..10)
        .map(|_| deck.draw(&rules, &mut roll).unwrap())
        .collect();
    let levels: Vec<usize> = drawn.iter().map(|d| d.0).collect();
    assert_eq!(
        levels,
        [0, 0, 1, 1, 2, 2, 3, 3, 3, 3],
        "two waves a level, then level 4 repeats"
    );
    assert_ne!(drawn[0], drawn[1], "no repeats within a level");
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

#[test]
fn placings_put_the_winner_first_and_the_first_out_last() {
    use tricky_towers::gameplay::{TowerStatus, placings};
    let config = GameConfig::default();
    let game = Match {
        mode: ModeKind::Race,
        winner: Some(2),
        ..default()
    };
    let mut states: Vec<TowerState> = (0..4).map(|i| TowerState::new(&config, i)).collect();
    states[0].status = TowerStatus::Out;
    states[0].out_at = Some(10.0);
    states[1].height = 4.0;
    states[3].status = TowerStatus::Out;
    states[3].out_at = Some(20.0);
    let list: Vec<_> = states.iter().enumerate().collect();
    assert_eq!(placings(&game, &list), vec![2, 1, 3, 0]);
}

#[test]
fn a_lost_brick_reports_where_it_hit_the_water() {
    use tricky_towers::gameplay::BrickLost;
    let mut app = headless_app(fast_config());
    let mut seen: Vec<BrickLost> = Vec::new();
    run_until(&mut app, 8.0, |app| {
        human_input(app).left_held = true;
        let messages = app.world().resource::<Messages<BrickLost>>();
        seen.extend(messages.iter_current_update_messages().copied());
        !seen.is_empty()
    });
    let lost = seen.first().expect("a steered-off brick is reported");
    assert_eq!(lost.tower, 0);
    let config = app.world().resource::<GameConfig>();
    assert!(lost.at.y <= -config.cells(config.kill_depth));
    assert!(lost.at.x < 0.0, "it went off the left side: {lost:?}");
    // It floats in the water a while, out of play, then goes.
    let mut sunk = app.world_mut().query::<&tricky_towers::gameplay::Sunk>();
    assert_eq!(sunk.iter(app.world()).count(), 1);
    let float = app.world().resource::<GameConfig>().water.float_seconds;
    run_seconds(&mut app, float + 0.5, |app| {
        human_input(app).left_held = false
    });
    let mut sunk = app.world_mut().query::<&tricky_towers::gameplay::Sunk>();
    assert!(sunk.iter(app.world()).count() <= 1, "the first one is gone");
}

#[test]
fn a_ghost_wave_carries_off_a_brick_with_every_new_one() {
    use tricky_towers::config::Wave;
    let mut config = fast_config();
    config.survival.curse_waves = true;
    config.survival.dark_magic = false;
    config.survival.waves.gap = (1, 2);
    config.survival.waves.levels = vec![vec![Wave {
        curses: vec![SpellKind::Ghost],
        seconds: (30, 31),
        bricks: 0,
    }]];
    let mut app = headless_app(config);
    let mut ghosted = std::collections::HashSet::new();
    let enough = run_until(&mut app, 20.0, |app| {
        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<tricky_towers::spells::Ghosted>>();
        ghosted.extend(query.iter(app.world()));
        ghosted.len() >= 3
    });
    assert!(enough, "only {} bricks ghosted in one wave", ghosted.len());
}

#[test]
fn a_curse_wave_holds_the_brick_in_hand_for_a_moment() {
    use avian2d::prelude::Position;
    use tricky_towers::config::Wave;
    let mut config = fast_config();
    config.survival.curse_waves = true;
    config.survival.dark_magic = false;
    config.survival.waves.gap = (1, 2);
    config.survival.waves.levels = vec![vec![Wave {
        curses: vec![SpellKind::Large],
        seconds: (30, 31),
        bricks: 0,
    }]];
    let cell = config.cell_size;
    let shake = config.survival.waves.shake;
    let mut app = headless_app(config);
    // Rising first, then held for the last second.
    let held = run_until(&mut app, 5.0, |app| {
        let state = tower(app, 0);
        state.active.is_some() && state.wave_hold > 0.1 && state.wave_hold < 0.6
    });
    assert!(held, "no wave held the brick");
    let state = tower(&mut app, 0);
    let brick = state.active.unwrap();
    let (_, top) = state.wave_lift;
    let y = app.world().get::<Position>(brick).unwrap().0.y;
    assert!(
        (y - top).abs() <= cell * shake + 0.01,
        "the brick sits at {y}, not near the top {top}"
    );
    run_seconds(&mut app, 0.3, |_| {});
    let later = app.world().get::<Position>(brick).unwrap().0.y;
    assert!(
        (later - top).abs() <= cell * shake + 0.01,
        "the held brick moved from {y} to {later}"
    );
    // Only the picture shakes: the body sits exactly at the top, still.
    assert_eq!(later, top, "the body itself shook");
    let control = app.world().get::<ActiveBrick>(brick).unwrap();
    assert!(
        control.picture_shake > 0.0,
        "the picture should shake while held"
    );
}

#[test]
fn bases_have_the_originals_tops() {
    // The castle's top is three merlons with holes between; both bases are 6 wide.
    let config = fast_config();
    let boxes = config.base_boxes(ModeKind::Survival);
    let tops = boxes.iter().filter(|b| b.y.1 >= 0.0).count();
    assert_eq!(tops, 3, "three merlons reach the top");
    assert_eq!(config.base_half_width(ModeKind::Survival), 3.0);
    assert_eq!(config.base_half_width(ModeKind::Race), 3.0);
    assert_eq!(config.base_half_width(ModeKind::Endless), 3.0);
}

#[test]
fn a_losing_tower_falls_apart_before_the_popup() {
    use tricky_towers::collapse::{Collapse, CollapseStage, all_done};
    let mut app = headless_app(fast_config());
    // Build a little, then steer every brick off the edge until the lives run out.
    run_seconds(&mut app, 3.0, |_| {});
    let built = tower(&mut app, 0).bricks_placed;
    assert!(
        built >= 2,
        "expected a few bricks first: {:?}",
        tower(&mut app, 0)
    );
    assert!(run_until(&mut app, 30.0, |app| {
        human_input(app).left_held = true;
        app.world().resource::<Match>().phase == Phase::Over
    }));
    let at_end = tower(&mut app, 0);
    let done = |app: &mut App| {
        let game = app.world().resource::<Match>().clone();
        let world = app.world_mut();
        all_done(
            &game,
            world
                .query::<(&Tower, &TowerState, Option<&Collapse>)>()
                .iter(world),
        )
    };
    assert!(!done(&mut app), "a loser must play its end sequence first");
    human_input(&mut app).left_held = false;

    // Wait, blasts, rumble: about 1.5 + a little + 2.5 seconds.
    assert!(
        run_until(&mut app, 8.0, |app| done(app)),
        "the collapse never finished"
    );
    let world = app.world_mut();
    let stage = world.query::<&Collapse>().single(world).unwrap().stage;
    assert_eq!(stage, CollapseStage::Done);
    // The wizard was out, so every brick blew up or sank; none stand.
    assert_eq!(count::<With<Brick>>(&mut app), 0);
    // Falling apart doesn't change the final score.
    let after = tower(&mut app, 0);
    assert_eq!(after.lives, at_end.lives);
    assert_eq!(after.bricks_dropped, at_end.bricks_dropped);
}

#[test]
fn the_winner_keeps_its_tower_while_the_loser_falls_apart() {
    use tricky_towers::collapse::{Collapse, CollapseStage};
    let mut config = fast_config();
    config.opponents = 1;
    config.survival.drop_speed = 3.0;
    let mut app = headless_app(config);
    // You throw every brick away; the computer wizard is the last one standing.
    assert!(run_until(&mut app, 40.0, |app| {
        human_input(app).left_held = true;
        app.world().resource::<Match>().phase == Phase::Over
    }));
    assert_eq!(app.world().resource::<Match>().winner, Some(1));
    human_input(&mut app).left_held = false;
    run_seconds(&mut app, 8.0, |_| {});

    let world = app.world_mut();
    let towers: Vec<(usize, Entity, Option<CollapseStage>)> = world
        .query::<(Entity, &Tower, Option<&Collapse>)>()
        .iter(world)
        .map(|(e, t, c)| (t.index, e, c.map(|c| c.stage)))
        .collect();
    let (_, winner, winner_stage) = towers.iter().find(|t| t.0 == 1).copied().unwrap();
    let (_, _, loser_stage) = towers.iter().find(|t| t.0 == 0).copied().unwrap();
    assert_eq!(winner_stage, None, "the winner plays no end sequence");
    assert_eq!(loser_stage, Some(CollapseStage::Done));
    // The winner's bricks are all still there and solid.
    let standing = world
        .query_filtered::<&Brick, Without<avian2d::prelude::Sensor>>()
        .iter(world)
        .filter(|b| b.tower == winner)
        .count();
    assert!(
        standing >= 3,
        "the winner's tower should stand: {standing} bricks"
    );
}

#[test]
fn endless_survival_has_no_brick_limit_and_ends_with_the_lives() {
    let mut config = fast_config();
    config.mode = ModeKind::EndlessSurvival;
    config.endless_play.survival_drop_speed = 8.0;
    config.endless_play.survival_waves.levels.clear();
    let lives = config.endless_play.survival_lives;
    let mut app = headless_app(config);
    run_seconds(&mut app, 5.0, |_| {});
    let state = tower(&mut app, 0);
    assert!(state.bricks_placed > 3, "no brick limit: {state:?}");
    assert_eq!(state.lives, Some(lives));
    assert_eq!(app.world().resource::<Match>().phase, Phase::Playing);
    assert_eq!(
        app.world().resource::<GameConfig>().survival.drop_speed,
        8.0
    );

    assert!(run_until(&mut app, 40.0, |app| {
        human_input(app).left_held = true;
        app.world().resource::<Match>().phase == Phase::Over
    }));
    let state = tower(&mut app, 0);
    assert_eq!(state.status, TowerStatus::Out);
    assert_eq!(state.lives, Some(0));
    // The score is the bricks still in the tower.
    assert_eq!(
        state.endless_score(),
        state.bricks_placed - state.tower_losses
    );
}

#[test]
fn endless_survival_waves_last_a_number_of_bricks() {
    use tricky_towers::config::Wave;
    let mut config = fast_config();
    config.mode = ModeKind::EndlessSurvival;
    config.endless_play.survival_drop_speed = 8.0;
    let waves = &mut config.endless_play.survival_waves;
    waves.gap = (3, 4);
    waves.lift = 0.0;
    waves.hold = 0.0;
    waves.levels = vec![vec![Wave {
        curses: vec![SpellKind::Large],
        seconds: (0, 0),
        bricks: 2,
    }]];
    let mut app = headless_app(config);
    // The brick in hand is the first of the two.
    assert!(run_until(&mut app, 5.0, |app| tower(app, 0)
        .wave_mod
        .is_some()));
    assert_eq!(tower(&mut app, 0).wave_bricks, 1);
    wait_for_next_brick(&mut app);
    let state = tower(&mut app, 0);
    assert_eq!(state.wave_bricks, 0);
    assert_eq!(state.wave_mod, None, "the second brick was the last");
}

#[test]
fn endless_puzzle_levels_raise_the_laser_and_a_lost_brick_ends_the_run() {
    let mut config = fast_config();
    config.mode = ModeKind::EndlessPuzzle;
    config.magic.enabled = true;
    let rules = &mut config.endless_play;
    rules.puzzle_drop_speed = 8.0;
    rules.laser_start = 30.0;
    rules.first_bricks = 2;
    rules.bricks_per_level = 1;
    let rules = rules.clone();
    let mut app = headless_app(config);
    let laser = |app: &mut App| {
        let config = app.world().resource::<GameConfig>().clone();
        tower(app, 0).laser(&config)
    };
    assert_eq!(laser(&mut app), 30.0);
    assert_eq!(tower(&mut app, 0).lives, Some(1));
    assert_eq!(tower(&mut app, 0).light_spell, None);

    // Two bricks, then a level-up: the laser rises and Undo is given.
    assert!(run_until(&mut app, 20.0, |app| tower(app, 0)
        .level_up
        .is_some()));
    assert_eq!(tower(&mut app, 0).bricks_used(), 2);
    assert!(
        tower(&mut app, 0).active.is_none(),
        "no brick during a level-up"
    );
    assert!(run_until(&mut app, 5.0, |app| tower(app, 0).puzzle_level == 2));
    let state = tower(&mut app, 0);
    assert_eq!(laser(&mut app), 30.0 + rules.laser_rise);
    assert_eq!(state.light_spell, Some(SpellKind::Undo));
    assert_eq!(state.level_end, 2 + rules.level_bricks(2));
    assert_eq!(rules.level_bricks(2), 3);

    // Steering a brick off the edge costs the only life.
    assert!(run_until(&mut app, 20.0, |app| {
        human_input(app).left_held = true;
        app.world().resource::<Match>().phase == Phase::Over
    }));
    let state = tower(&mut app, 0);
    assert_eq!(state.status, TowerStatus::Out);
    assert!(state.bricks_dropped >= 1);
}

fn trial_config(trial: tricky_towers::config::TrialDef) -> GameConfig {
    let mut config = fast_config();
    config.trials = vec![trial];
    config.trial = Some(0);
    config
}

#[test]
fn a_race_trial_is_lost_when_its_clock_runs_out() {
    use tricky_towers::config::TrialDef;
    let mut app = headless_app(trial_config(TrialDef {
        name: "clock".into(),
        mode: ModeKind::Race,
        finish_height: Some(40.0),
        seconds: Some(2.0),
        ..default()
    }));
    assert_eq!(app.world().resource::<Match>().mode, ModeKind::Race);
    assert_eq!(
        app.world().resource::<GameConfig>().race.finish_height,
        40.0
    );
    run_seconds(&mut app, 1.5, |_| {});
    assert_eq!(tower(&mut app, 0).status, TowerStatus::Building);
    run_seconds(&mut app, 1.0, |_| {});
    assert_eq!(tower(&mut app, 0).status, TowerStatus::Out);
    assert_eq!(app.world().resource::<Match>().phase, Phase::Over);
}

#[test]
fn the_enemy_wizard_casts_on_its_schedule() {
    use tricky_towers::config::{CastWhen, EnemyCast, TrialDef};
    use tricky_towers::spells::ENEMY_WIZARD;
    let mut config = trial_config(TrialDef {
        name: "enemy".into(),
        mode: ModeKind::Survival,
        bricks: Some(40),
        enemy: vec![
            EnemyCast {
                when: CastWhen::Every(1.0),
                spells: vec![SpellKind::Large],
            },
            EnemyCast {
                when: CastWhen::At(0.5),
                spells: vec![SpellKind::Slow],
            },
            EnemyCast {
                when: CastWhen::BricksUsed(0.1),
                spells: vec![SpellKind::Ice],
            },
        ],
        ..default()
    });
    config.survival.drop_speed = 3.0;
    let mut app = headless_app(config);
    let mut casts: Vec<SpellKind> = Vec::new();
    let mut seen = 0;
    run_seconds(&mut app, 3.2, |app| {
        let log = app.world().resource::<tricky_towers::spells::SpellLog>();
        let fresh = (log.1 - seen) as usize;
        for cast in log.0.iter().rev().take(fresh).rev() {
            assert_eq!(cast.caster, ENEMY_WIZARD);
            casts.push(cast.spell);
        }
        seen = log.1;
    });
    let count = |spell| casts.iter().filter(|&&s| s == spell).count();
    assert_eq!(count(SpellKind::Large), 3, "every second: {casts:?}");
    assert_eq!(count(SpellKind::Slow), 1, "once, at 0.5 s: {casts:?}");
    // 10% of 40 bricks is 4 used.
    assert_eq!(
        count(SpellKind::Ice),
        (tower(&mut app, 0).bricks_used() >= 4) as usize
    );
}

#[test]
fn a_trial_deals_its_fixed_brick_order() {
    use tricky_towers::config::TrialDef;
    use tricky_towers::gameplay::{ActiveBrick, Brick};
    let order = ["I", "O", "T", "S"];
    let mut app = headless_app(trial_config(TrialDef {
        name: "order".into(),
        mode: ModeKind::Survival,
        bricks: Some(10),
        order: order.iter().map(|s| s.to_string()).collect(),
        ..default()
    }));
    let names: Vec<String> = app
        .world()
        .resource::<GameConfig>()
        .pieces
        .iter()
        .map(|p| p.name.clone())
        .collect();
    let mut dealt: Vec<(Entity, String)> = Vec::new();
    run_until(&mut app, 30.0, |app| {
        let world = app.world_mut();
        for (entity, brick) in world
            .query_filtered::<(Entity, &Brick), With<ActiveBrick>>()
            .iter(world)
        {
            let name = names[brick.piece].clone();
            if dealt.last().map(|(e, _)| *e) != Some(entity) {
                dealt.push((entity, name));
            }
        }
        dealt.len() >= order.len()
    });
    let dealt: Vec<&str> = dealt.iter().map(|(_, n)| n.as_str()).collect();
    assert_eq!(dealt, order);
}

#[test]
fn the_enemy_wizard_casts_at_heights_and_bricks() {
    use tricky_towers::config::{CastWhen, EnemyCast, TrialDef};
    let mut config = trial_config(TrialDef {
        name: "triggers".into(),
        mode: ModeKind::Survival,
        bricks: Some(40),
        enemy: vec![
            EnemyCast {
                when: CastWhen::Height(0.0),
                spells: vec![SpellKind::Slow],
            },
            EnemyCast {
                when: CastWhen::Brick(3),
                spells: vec![SpellKind::Ice],
            },
            EnemyCast {
                when: CastWhen::Height(500.0),
                spells: vec![SpellKind::Large],
            },
        ],
        ..default()
    });
    config.survival.drop_speed = 8.0;
    let mut app = headless_app(config);
    run_until(&mut app, 2.0, |app| !log(app).is_empty());
    assert_eq!(log(&app)[0].spell, SpellKind::Slow);
    assert!(run_until(&mut app, 30.0, |app| log(app)
        .iter()
        .any(|c| c.spell == SpellKind::Ice)));
    // The third brick is the one in hand when Ice lands.
    assert_eq!(tower(&mut app, 0).bricks_used(), 2);
    run_seconds(&mut app, 3.0, |_| {});
    let spells: Vec<SpellKind> = log(&app).iter().map(|c| c.spell).collect();
    assert_eq!(spells, [SpellKind::Slow, SpellKind::Ice], "each fires once");
}

#[test]
fn trial_spell_lines_give_their_spells_in_order_then_stop() {
    use tricky_towers::config::{TrialDef, TrialLines};
    let mut config = trial_config(TrialDef {
        name: "lines".into(),
        mode: ModeKind::Survival,
        bricks: Some(60),
        lines: Some(TrialLines {
            every: 1.0,
            spells: vec![(SpellKind::Ivy, false), (SpellKind::Petrify, true)],
        }),
        ..default()
    });
    config.magic.enabled = true;
    let mut app = headless_app(config);
    assert!(run_until(&mut app, 30.0, |app| tower(app, 0).lines_reached >= 1));
    assert_eq!(tower(&mut app, 0).light_spell, Some(SpellKind::Ivy));
    assert_eq!(tower(&mut app, 0).dark_spell, None);
    // The second line's spell is cast as soon as it's collected.
    assert!(run_until(&mut app, 30.0, |app| log(app)
        .iter()
        .any(|c| c.spell == SpellKind::Petrify)));
    assert_eq!(tower(&mut app, 0).light_spell, None);
    run_seconds(&mut app, 5.0, |_| {});
    let state = tower(&mut app, 0);
    assert_eq!(state.lines_reached, 2);
    assert_eq!(state.spell_line, None);
}

#[test]
fn winning_a_trial_marks_it_done() {
    use tricky_towers::config::TrialDef;
    use tricky_towers::trials::TrialProgress;
    let mut app = headless_app(trial_config(TrialDef {
        name: "short puzzle".into(),
        mode: ModeKind::Puzzle,
        bricks: Some(2),
        laser: Some(8.0),
        ..default()
    }));
    assert_eq!(app.world().resource::<GameConfig>().puzzle.line_random, 0.0);
    // A dropped brick fails a puzzle trial, so lay them side by side.
    let (mut frame, mut presses) = (0, [0; 2]);
    assert!(run_until(&mut app, 20.0, |app| {
        frame += 1;
        let placed = tower(app, 0).bricks_placed as usize;
        let mut input = human_input(app);
        if placed < 2 && presses[placed] < 3 && frame % 4 == 0 {
            presses[placed] += 1;
            if placed == 0 {
                input.left_pressed = true;
            } else {
                input.right_pressed = true;
            }
        }
        app.world().resource::<Match>().phase == Phase::Over
    }));
    assert_eq!(
        tower(&mut app, 0).status,
        TowerStatus::Won,
        "{:?}",
        tower(&mut app, 0)
    );
    let progress = app.world().resource::<TrialProgress>();
    assert!(progress.is_done("short puzzle"));
}

#[test]
fn a_cup_plays_matches_until_someone_reaches_the_target() {
    use tricky_towers::config::CupSize;
    use tricky_towers::cups::{Cup, XpBook};
    use tricky_towers::gameplay::MatchRequest;
    let mut config = fast_config();
    config.opponents = 1;
    config.survival.drop_speed = 3.0;
    config.cups.sizes = vec![CupSize {
        name: "Test Cup".into(),
        target: 6,
        trophy: "trophy".into(),
    }];
    let mut app = headless_app(config);
    {
        let mut cup = app.world_mut().resource_mut::<Cup>();
        cup.pick = Some(0);
        cup.begin();
    }
    // You throw every brick away, so the computer wizard wins both matches.
    for round in 1..=2 {
        assert!(run_until(&mut app, 40.0, |app| {
            human_input(app).left_held = true;
            app.world().resource::<Match>().phase == Phase::Over
        }));
        run_seconds(&mut app, 0.1, |_| {});
        let cup = app.world().resource::<Cup>().clone();
        assert_eq!(cup.played, round);
        if round == 1 {
            assert!(cup.in_progress());
            app.world_mut().resource_mut::<MatchRequest>().pending = true;
            run_seconds(&mut app, 0.1, |_| {});
        }
    }
    let cup = app.world().resource::<Cup>().clone();
    assert!(!cup.in_progress());
    assert_eq!(
        cup.points,
        vec![4, 6],
        "2 + 2 for you, 3 + 3 for the wizard"
    );
    assert_eq!(cup.standings, Some(vec![1, 0]));
    // Local cups give no XP.
    assert!(app.world().resource::<XpBook>().players.is_empty());
}

#[test]
fn auto_rotate_winds_back_swings_and_waits() {
    use tricky_towers::gameplay::auto_rotate_angle;
    let magic = GameConfig::default().magic;
    let cycle = magic.auto_rotate_interval;
    let at = |t: f32| auto_rotate_angle(&magic, cycle, t);
    assert_eq!(at(0.0), 0.0);
    assert!((at(0.1) - -7.5).abs() < 1e-3, "half the wind-back");
    assert!((at(0.2) - -15.0).abs() < 1e-3, "wound back 15°");
    assert!((at(0.65) - 90.0).abs() < 1e-3, "swung 105°, 90° net");
    assert!((at(1.6) - 90.0).abs() < 1e-3, "waiting");
    assert!((at(cycle + 0.65) - 180.0).abs() < 1e-3, "the next cycle");
}

#[test]
fn no_rotate_refuses_the_turn_with_a_shake() {
    let mut app = headless_app(magic_config(SpellKind::Ivy, &[SpellKind::NoRotate]));
    wait_for_next_brick(&mut app);
    let active = tower(&mut app, 0).active.unwrap();
    app.world_mut()
        .get_mut::<ActiveBrick>(active)
        .unwrap()
        .mods
        .no_rotate = true;
    let before = *app
        .world()
        .get::<avian2d::prelude::Rotation>(active)
        .unwrap();
    human_input(&mut app).rotate_pressed = true;
    run_seconds(&mut app, 0.05, |_| {});
    let control = app.world().get::<ActiveBrick>(active).unwrap();
    assert!(control.refused > 0.0, "the refused turn shakes");
    let after = *app
        .world()
        .get::<avian2d::prelude::Rotation>(active)
        .unwrap();
    assert_eq!(before.as_radians(), after.as_radians());
}

#[test]
fn debug_commands_and_scripts_drive_a_match() {
    use tricky_towers::debug::{DebugCommand, DebugQueue, DebugScript, ScriptStep};
    let mut app = headless_app(fast_config());
    // Commands straight away: set your lives and put a light spell in your hand.
    app.world_mut().resource_mut::<DebugQueue>().0.extend([
        DebugCommand::SetLives { tower: 0, lives: 1 },
        DebugCommand::GiveLight {
            tower: 0,
            spell: SpellKind::Petrify,
        },
    ]);
    run_seconds(&mut app, 0.05, |_| {});
    let state = tower(&mut app, 0);
    assert_eq!(state.lives, Some(1));
    assert_eq!(state.light_spell, Some(SpellKind::Petrify));

    // A script: restart as a race with one opponent, curse everyone, then win.
    let script: Vec<ScriptStep> = ron::from_str(
        "[
            (at: 0.1, run: Start(mode: Race, opponents: Some(1))),
            (at: 0.5, run: Curse(spells: [Large])),
            (at: 0.6, run: Press(tower: 0, key: RotateRight)),
            (at: 1.0, run: Win(tower: 0)),
        ]",
    )
    .expect("script parses");
    app.insert_resource(DebugScript::new(script));
    run_seconds(&mut app, 0.3, |_| {});
    assert_eq!(app.world().resource::<Match>().mode, ModeKind::Race);
    assert_eq!(count::<With<Tower>>(&mut app), 2);
    run_seconds(&mut app, 1.0, |_| {});
    let game = app.world().resource::<Match>();
    assert_eq!((game.phase, game.winner), (Phase::Over, Some(0)));
    let log = app.world().resource::<tricky_towers::spells::SpellLog>();
    assert!(log.0.iter().any(|c| c.spell == SpellKind::Large));
}

#[test]
fn endless_puzzle_level_ups_put_down_four_blocks_by_the_original_rule() {
    use tricky_towers::gameplay::Obstacle;
    let mut config = fast_config();
    config.mode = ModeKind::EndlessPuzzle;
    let rules = &mut config.endless_play;
    rules.puzzle_drop_speed = 8.0;
    rules.laser_start = 30.0;
    rules.first_bricks = 2;
    let mut app = headless_app(config);
    assert!(run_until(&mut app, 30.0, |app| count::<With<Obstacle>>(
        app
    ) == 4));
    // All four come from distinct spots, at the original's columns and above the tower.
    let world = app.world_mut();
    let spots: Vec<Vec2> = world
        .query_filtered::<&Transform, With<Obstacle>>()
        .iter(world)
        .map(|t| t.translation.truncate())
        .collect();
    let config = app.world().resource::<GameConfig>().clone();
    for (i, a) in spots.iter().enumerate() {
        let x = a.x / config.cell_size;
        assert!(
            (x.abs() - x.abs().floor() - 0.5).abs() < 0.01 && x.abs() <= 6.5,
            "{x}"
        );
        assert!(spots[i + 1..].iter().all(|b| b.distance(*a) > 1.0));
    }
}

#[test]
fn obstacle_spots_follow_the_weights_and_never_repeat() {
    let rules = tricky_towers::config::EndlessPlay::default();
    let mut seed = 7u64;
    let mut far = 0;
    for _ in 0..200 {
        let spots = rules.obstacle_spots(2.3, 10.2, || {
            (tricky_towers::gameplay::xorshift(&mut seed) % 10_000) as f32 / 10_000.0
        });
        assert_eq!(spots.len(), 4);
        for (i, s) in spots.iter().enumerate() {
            assert!(s.y >= 3.5 && s.y <= 9.5, "rows 4 to 10: {s}");
            assert!(spots[i + 1..].iter().all(|o| o != s));
            far += (s.x.abs() == 6.5) as u32;
        }
    }
    // The outer columns carry most of the weight.
    assert!(far > 400, "{far}");
}

#[test]
fn the_survival_winner_steers_a_roof_onto_the_tower_before_winner_shows() {
    use tricky_towers::gameplay::{RoofPiece, RoofStage};
    let mut config = fast_config();
    config.survival.bricks_to_place = 3;
    config.survival.finish_countdown = 0.5;
    let mut app = headless_app(config);
    assert!(run_until(&mut app, 15.0, |app| tower(app, 0).roof
        == RoofStage::Steering));
    assert_eq!(count::<With<RoofPiece>>(&mut app), 1);
    assert!(tower(&mut app, 0).active.is_none());
    // Held down, it lands on the tower and rests 0.7 s.
    assert!(run_until(&mut app, 20.0, |app| {
        human_input(app).fast_fall_held = true;
        tower(app, 0).roof == RoofStage::Done
    }));
    let world = app.world_mut();
    let y = world
        .query_filtered::<&Transform, With<RoofPiece>>()
        .iter(world)
        .next()
        .expect("the roof stays on the tower")
        .translation
        .y;
    let state = tower(&mut app, 0);
    assert!(
        y > 0.0 && y / app.world().resource::<GameConfig>().cell_size < state.height + 10.0,
        "{y} {state:?}"
    );
}

#[test]
fn a_trial_has_its_own_floor_lane_and_rival_tower() {
    use tricky_towers::gameplay::{Platform, RivalLane};
    let mut config = fast_config();
    config.opponents = 2;
    // SR_10 (rival tower 1) and SP_20 (FLOOR_PUZZLE_BASIC).
    config.trial = config.trials.iter().position(|t| t.name == "SR_10");
    let mut app = headless_app(config.clone());
    let world = app.world_mut();
    let lanes: Vec<(f32, Option<u32>)> = world
        .query::<&RivalLane>()
        .iter(world)
        .map(|l| (l.origin_x, l.rival))
        .collect();
    assert_eq!(lanes.len(), 1);
    assert_eq!(lanes[0].1, Some(1));
    let towers: Vec<f32> = world
        .query::<&Tower>()
        .iter(world)
        .map(|t| t.origin_x)
        .collect();
    assert_eq!(
        towers.len(),
        1,
        "a trial is one player, whatever the lobby says"
    );
    assert!(towers[0] < 0.0 && lanes[0].0 > 0.0);

    config.trial = config.trials.iter().position(|t| t.name == "SP_20");
    let floor = config
        .trial_floor()
        .expect("SP_20 has its own floor")
        .clone();
    assert_eq!(floor.boxes.len(), 5);
    let mut app = headless_app(config);
    let world = app.world_mut();
    // No random Puzzle columns: the floor is the platform's own boxes.
    assert_eq!(
        world
            .query::<&tricky_towers::gameplay::FloorBlock>()
            .iter(world)
            .count(),
        0
    );
    assert_eq!(world.query::<&Platform>().iter(world).count(), 1);
}

#[test]
fn ivy_bricks_are_drawn_where_their_bodies_are() {
    let mut app = headless_app(magic_config(SpellKind::Ivy, &[]));
    wait_for_light(&mut app, SpellKind::Ivy);
    human_input(&mut app).cast_light_pressed = true;
    assert!(run_until(&mut app, 3.0, |app| count::<With<IvyCluster>>(
        app
    ) >= 1));
    run_seconds(&mut app, 1.0, |_| {});
    let world = app.world_mut();
    for (p, g) in world
        .query_filtered::<(&avian2d::prelude::Position, &GlobalTransform), (With<Brick>, With<ChildOf>)>()
        .iter(world)
    {
        assert!(p.0.distance(g.translation().truncate()) < 1.0, "{p:?} drawn at {g:?}");
    }
}

#[test]
fn two_large_curses_stack_to_2_5() {
    let mut config = fast_config();
    config.mode = ModeKind::Race;
    config.opponents = 1;
    config.magic.enabled = true;
    let mut app = headless_app(config);
    run_seconds(&mut app, 0.1, |_| {});
    cast_now(&mut app, SpellKind::Ivy, Some(SpellKind::Large));
    let brick = tower(&mut app, 1).active.unwrap();
    assert_eq!(app.world().get::<Brick>(brick).unwrap().scale, 2.0);
    cast_now(&mut app, SpellKind::Ivy, Some(SpellKind::Large));
    run_seconds(&mut app, 0.1, |_| {});
    assert_eq!(tower(&mut app, 1).active, Some(brick));
    let world = app.world();
    assert_eq!(world.get::<Brick>(brick).unwrap().scale, 2.5);
    assert_eq!(
        world
            .get::<tricky_towers::gameplay::ActiveBrick>(brick)
            .unwrap()
            .mods
            .large,
        2
    );
}
