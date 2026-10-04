use bevy::prelude::*;

use tricky_towers::TowerCorePlugin;
use tricky_towers::art::ArtPlugin;
use tricky_towers::config::{DEFAULT_CONFIG_PATH, Difficulty, GameConfig, ModeKind};
use tricky_towers::effects::EffectsPlugin;
use tricky_towers::input::KeyboardInputPlugin;
use tricky_towers::letters::LettersPlugin;
use tricky_towers::menu::MenuPlugin;
use tricky_towers::results::ResultsPlugin;
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
                && (*i == 0
                    || !matches!(
                        args[i - 1].as_str(),
                        "--play" | "--difficulty" | "--offscreen" | "--shots" | "--menu"
                    ))
        })
        .map_or_else(|| DEFAULT_CONFIG_PATH.to_string(), |(_, a)| a.clone());
    let mut config = GameConfig {
        start_in_menu: play.is_none(),
        ..tricky_towers::mods::apply_config_mods(
            GameConfig::load_or_default(&path),
            &tricky_towers::mods::mods_dir(),
        )
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

    // `--offscreen <folder>`: no window, no sound; save pictures at the `--shots`
    // seconds (default 5 and 20) into the folder, then quit.
    let value = |flag: &str| {
        args.iter()
            .position(|a| a == flag)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let offscreen = value("--offscreen").map(std::path::PathBuf::from);
    let shots: Vec<f32> = value("--shots")
        .map(|s| s.split(',').filter_map(|t| t.trim().parse().ok()).collect())
        .unwrap_or_else(|| vec![5.0, 20.0]);

    let mut app = App::new();
    // Original art and sound from your own install, if extracted (see docs/art-layout.md).
    tricky_towers::art::register_source(&mut app);
    match &offscreen {
        Some(dir) => {
            let _ = std::fs::create_dir_all(dir);
            app.add_plugins(
                DefaultPlugins
                    .set(WindowPlugin {
                        primary_window: None,
                        exit_condition: bevy::window::ExitCondition::DontExit,
                        ..default()
                    })
                    .disable::<bevy::winit::WinitPlugin>()
                    .disable::<bevy::audio::AudioPlugin>(),
            )
            .add_plugins(bevy::app::ScheduleRunnerPlugin::run_loop(
                std::time::Duration::from_secs_f64(1.0 / 60.0),
            ))
            .insert_resource(tricky_towers::art::Silent);
        }
        None => {
            app.add_plugins(DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Tricky Towers (Rust + Bevy rewrite)".into(),
                    resolution: (1280, 720).into(),
                    ..default()
                }),
                ..default()
            }));
        }
    }
    app.insert_resource(config).add_plugins((
        TowerCorePlugin,
        KeyboardInputPlugin,
        MenuPlugin,
        VisualsPlugin,
        ArtPlugin,
        LettersPlugin,
        WizardsPlugin,
        ResultsPlugin,
        EffectsPlugin,
    ));
    // `--menu title|players|modes|single|endless|pause` opens on that menu screen, for scripted pictures.
    // Player select shows the keyboard player and one AI wizard seated.
    if let Some(menu) = value("--menu") {
        use tricky_towers::menu::MenuScreen;
        use tricky_towers::select::{Lobby, Seat};
        let mut lobby = Lobby::default();
        let screen = match menu.as_str() {
            "players" => {
                lobby.seats[0] = Seat::Keyboard;
                lobby.seats[1] = Seat::Ai;
                MenuScreen::Players
            }
            "single" => MenuScreen::Single,
            "endless" => MenuScreen::Endless,
            "title" => MenuScreen::Main,
            "pause" => MenuScreen::Pause,
            _ => MenuScreen::Modes,
        };
        app.insert_resource(lobby).insert_resource(screen);
    }
    if let Some(dir) = offscreen {
        app.add_plugins(tricky_towers::offscreen::OffscreenPlugin {
            dir,
            shots,
            size: UVec2::new(1280, 720),
        });
    }
    app.run();
}
