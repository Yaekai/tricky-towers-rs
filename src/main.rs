use bevy::prelude::*;

use tricky_towers::TowerCorePlugin;
use tricky_towers::art::ArtPlugin;
use tricky_towers::config::{DEFAULT_CONFIG_PATH, Difficulty, GameConfig, ModeKind};
use tricky_towers::effects::EffectsPlugin;
use tricky_towers::input::KeyboardInputPlugin;
use tricky_towers::letters::LettersPlugin;
use tricky_towers::menu::MenuPlugin;
use tricky_towers::results::ResultsPlugin;
use tricky_towers::rival::RivalPlugin;
use tricky_towers::visuals::VisualsPlugin;
use tricky_towers::wizards::WizardsPlugin;

// DEBUG-TOOL: not in the original; remove before the final build (docs/release-checklist.md).
// The command-line options (`--play`, `--difficulty`, `--offscreen`, `--shots`, `--menu`, a
// config file path) are development tools for scripted checks and pictures.
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
            "endless-survival" => Some(ModeKind::EndlessSurvival),
            "endless-puzzle" => Some(ModeKind::EndlessPuzzle),
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
                        "--play"
                            | "--difficulty"
                            | "--offscreen"
                            | "--shots"
                            | "--size"
                            | "--menu"
                            | "--script"
                            | "--trials"
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
    #[cfg(feature = "debug-panel")]
    let windowed = offscreen.is_none();
    let shots: Vec<f32> = value("--shots")
        .map(|s| s.split(',').filter_map(|t| t.trim().parse().ok()).collect())
        .unwrap_or_else(|| vec![5.0, 20.0]);

    // DEBUG-TOOL: not in the original; remove before the final build (docs/release-checklist.md).
    let script = value("--script").map(|path| {
        tricky_towers::debug::DebugScript::load(std::path::Path::new(&path))
            .unwrap_or_else(|err| panic!("can't read script {path}: {err}"))
    });

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
        tricky_towers::casts::CastsPlugin,
        tricky_towers::packs::PacksPlugin,
        tricky_towers::weather::WeatherPlugin,
        tricky_towers::particles::ParticlesPlugin,
        tricky_towers::hud::HudPlugin,
        RivalPlugin,
    ));
    app.add_plugins((
        tricky_towers::game_info::GameInfoPlugin,
        tricky_towers::intro::IntroPlugin,
        tricky_towers::cup_winner::CupWinnerPlugin,
    ));
    // Endless scores are kept between runs, except in offscreen picture runs.
    if offscreen.is_none() {
        let file = tricky_towers::single::save_dir().join("leaderboard.ron");
        app.insert_resource(tricky_towers::single::Leaderboard::load(file));
        let file = tricky_towers::single::save_dir().join("stats.ron");
        app.insert_resource(tricky_towers::stats::Stats::load(file));
        let file = tricky_towers::single::save_dir().join("keys.ron");
        app.insert_resource(tricky_towers::keybinding::KeyBindings::load(file));
        let file = tricky_towers::single::save_dir().join("settings.ron");
        app.insert_resource(tricky_towers::settings::Settings::load(file));
        let file = tricky_towers::single::save_dir().join("xp.ron");
        app.insert_resource(tricky_towers::cups::XpBook::load(file));
        let file = tricky_towers::single::save_dir().join("trials.ron");
        app.insert_resource(tricky_towers::trials::TrialProgress::load(file));
    }
    // `--trials file.ron`: trial progress from this file, for scripted pictures of the tier unlock.
    if let Some(file) = value("--trials") {
        app.insert_resource(tricky_towers::trials::TrialProgress::load(file.into()));
    }
    // `--menu title|players|modes|single|endless|trials|settings|pause` opens on that menu screen, for scripted pictures.
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
            "trials" => MenuScreen::Trials,
            "settings" => MenuScreen::Settings,
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
            // `--size 3440x1440`: picture size (1280x720 if left out).
            size: value("--size")
                .and_then(|s| {
                    let (w, h) = s.split_once('x')?;
                    Some(UVec2::new(w.parse().ok()?, h.parse().ok()?))
                })
                .unwrap_or(UVec2::new(1280, 720)),
            scripted: script.is_some(),
        });
    }
    // DEBUG-TOOL: the F1 debug panel, in a window only.
    #[cfg(feature = "debug-panel")]
    if windowed {
        app.add_plugins(tricky_towers::debug_panel::DebugPanelPlugin);
    }
    // `--script file.ron`: a timed list of debug commands (src/debug.rs).
    if let Some(script) = script {
        app.insert_resource(script);
    }
    app.run();
}
