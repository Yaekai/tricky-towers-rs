use bevy::prelude::*;

use tricky_towers::TowerCorePlugin;
use tricky_towers::config::{DEFAULT_CONFIG_PATH, GameConfig};
use tricky_towers::input::KeyboardInputPlugin;
use tricky_towers::visuals::VisualsPlugin;

fn main() {
    // An alternative config (a mod) can be passed as the first argument.
    let path = std::env::args().nth(1).unwrap_or_else(|| DEFAULT_CONFIG_PATH.to_string());
    let config = GameConfig::load_or_default(&path);

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Tricky Towers (Rust + Bevy rewrite)".into(),
                resolution: (720, 960).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(config)
        .add_plugins((TowerCorePlugin, KeyboardInputPlugin, VisualsPlugin))
        .run();
}
