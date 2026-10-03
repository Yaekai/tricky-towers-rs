//! Runs the core loop without a window or GPU and checks that bricks fall,
//! land, stack and get counted as dropped.

use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use tricky_towers::TowerCorePlugin;
use tricky_towers::config::{GameConfig, ModeConfig};
use tricky_towers::gameplay::{ActiveBrick, Brick, PlayerInput, Round};

const STEP: f32 = 1.0 / 64.0;

fn headless_app(config: GameConfig) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin, AssetPlugin::default()))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(STEP)))
        .insert_resource(config)
        .add_plugins(TowerCorePlugin);
    app.finish();
    app.cleanup();
    app
}

fn run_seconds(app: &mut App, seconds: f32, mut each_frame: impl FnMut(&mut App)) {
    for _ in 0..(seconds / STEP) as usize {
        each_frame(app);
        app.update();
    }
}

fn fast_config() -> GameConfig {
    GameConfig {
        fall_speed: 8.0,
        mode: ModeConfig::Survival {
            lives: 3,
            bricks_to_place: 100,
        },
        ..default()
    }
}

#[test]
fn bricks_land_and_stack_on_the_platform() {
    let mut app = headless_app(fast_config());
    // A single column of bricks topples eventually, so only build a few.
    run_seconds(&mut app, 4.0, |_| {});

    let round = app.world().resource::<Round>().clone();
    assert!(round.bricks_placed >= 3, "expected several landings, got {round:?}");
    assert!(round.bricks_placed <= 8, "bricks are landing on nothing: {round:?}");
    assert_eq!(round.bricks_dropped, 0, "nothing should fall off when dropped straight down");
    assert!(round.tower_height >= 2.0, "tower should grow, got {round:?}");

    // Exactly one brick is under player control at a time.
    let world = app.world_mut();
    let active = world.query_filtered::<(), With<ActiveBrick>>().iter(world).count();
    assert_eq!(active, 1);
    let bricks = world.query::<&Brick>().iter(world).count();
    assert_eq!(bricks as u32, round.bricks_placed + 1);
}

#[test]
fn steering_off_the_edge_costs_lives() {
    let mut app = headless_app(fast_config());
    run_seconds(&mut app, 12.0, |app| {
        app.world_mut().resource_mut::<PlayerInput>().left_held = true;
    });

    let round = app.world().resource::<Round>().clone();
    assert!(round.bricks_dropped >= 1, "expected dropped bricks, got {round:?}");
    assert!(round.lives.unwrap() < 3);
}

#[test]
fn collapsing_tower_does_not_cascade() {
    // Regression: after a collapse, new bricks used to spawn inside falling
    // ones and "land" instantly, placing dozens of bricks per second.
    let mut app = headless_app(fast_config());
    run_seconds(&mut app, 15.0, |_| {});
    let round = app.world().resource::<Round>().clone();
    let max_reasonable = (15.0 * 8.0 / 4.0) as u32; // fall speed 8 cells/s, ~4+ cells per brick
    assert!(round.bricks_placed <= max_reasonable, "too many landings: {round:?}");
}
