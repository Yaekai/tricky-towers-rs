use bevy::prelude::*;

use tricky_towers::TowerCorePlugin;
use tricky_towers::art::ArtPlugin;
use tricky_towers::config::{DEFAULT_CONFIG_PATH, Difficulty, GameConfig, ModeKind};
use tricky_towers::input::KeyboardInputPlugin;
use tricky_towers::letters::LettersPlugin;
use tricky_towers::menu::MenuPlugin;
use tricky_towers::visuals::VisualsPlugin;
use tricky_towers::wizards::WizardsPlugin;

fn main() {
    // An alternative config (a mod) can be passed as an argument, and
    // `--play race|survival|puzzle` skips the menu (handy for scripted checks).
    let args: Vec<String> = std::env::args().skip(1).collect();
    let play = args
        .iter()
        .position(|a| a == "--play")
        .and_then(|i| args.get(i + 1))
        .and_then(|mode| match mode.to_lowercase().as_str() {
            "race" => Some(ModeKind::Race),
            "survival" => Some(ModeKind::Survival),
            "puzzle" => Some(ModeKind::Puzzle),
            "endless" => Some(ModeKind::Endless),
            _ => None,
        });
    let path = args
        .iter()
        .enumerate()
        .find(|(i, a)| {
            !a.starts_with("--")
                && (*i == 0 || (args[i - 1] != "--play" && args[i - 1] != "--difficulty"))
        })
        .map_or_else(|| DEFAULT_CONFIG_PATH.to_string(), |(_, a)| a.clone());
    let mut config = GameConfig {
        start_in_menu: play.is_none(),
        ..GameConfig::load_or_default(&path)
    };
    if let Some(mode) = play {
        config.mode = mode;
    }
    // `--difficulty easy|normal|pro` picks the original's difficulty values.
    let difficulty = args
        .iter()
        .position(|a| a == "--difficulty")
        .and_then(|i| args.get(i + 1))
        .and_then(|d| match d.to_lowercase().as_str() {
            "easy" => Some(Difficulty::Easy),
            "normal" => Some(Difficulty::Normal),
            "pro" => Some(Difficulty::Pro),
            _ => None,
        });
    if difficulty.is_some() {
        config.difficulty = difficulty;
    }

    let mut app = App::new();
    // Original art and sound from your own install, if extracted (see docs/art-layout.md).
    tricky_towers::art::register_source(&mut app);
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Tricky Towers (Rust + Bevy rewrite)".into(),
            resolution: (1280, 720).into(),
            ..default()
        }),
        ..default()
    }))
    .insert_resource(config)
    .add_plugins((
        TowerCorePlugin,
        KeyboardInputPlugin,
        MenuPlugin,
        VisualsPlugin,
        ArtPlugin,
        LettersPlugin,
        WizardsPlugin,
    ))
    .run();
}
