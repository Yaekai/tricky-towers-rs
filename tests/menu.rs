//! Drives the mode menu with simulated key presses.

use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use tricky_towers::TowerCorePlugin;
use tricky_towers::config::{GameConfig, ModeKind};
use tricky_towers::gameplay::{Match, Phase};
use tricky_towers::menu::MenuPlugin;

fn app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin, AssetPlugin::default()))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / 64.0,
        )))
        .insert_resource(GameConfig {
            start_in_menu: true,
            ..default()
        })
        .init_resource::<ButtonInput<KeyCode>>()
        .add_plugins((TowerCorePlugin, MenuPlugin));
    app.finish();
    app.cleanup();
    app.update();
    app
}

fn tap(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(key);
    app.update();
    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.release(key);
    keys.clear();
    for _ in 0..3 {
        app.update();
    }
}

#[test]
fn enter_starts_the_highlighted_mode() {
    let mut app = app();
    assert_eq!(app.world().resource::<Match>().phase, Phase::Menu);
    tap(&mut app, KeyCode::ArrowDown);
    tap(&mut app, KeyCode::ArrowDown);
    assert_eq!(app.world().resource::<GameConfig>().mode, ModeKind::Puzzle);
    tap(&mut app, KeyCode::Enter);
    let game = app.world().resource::<Match>();
    assert_eq!((game.phase, game.mode), (Phase::Playing, ModeKind::Puzzle));
}
