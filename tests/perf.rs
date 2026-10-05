//! DEBUG-TOOL: frame-time probe, not part of the game (docs/release-checklist.md).
//! `cargo test --release --test perf -- --ignored --nocapture` prints the CPU time
//! of one 60 fps frame per mode with four AI towers, for the core loop alone
//! (physics, rules, AI) and for the whole game minus the GPU (visuals, HUD, text).

use std::time::{Duration, Instant};

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use tricky_towers::TowerCorePlugin;
use tricky_towers::config::{DEFAULT_CONFIG_PATH, Difficulty, GameConfig, ModeKind};
use tricky_towers::gameplay::{AiControlled, Brick, Match, MatchRequest, Phase, Tower};

const FRAME: f32 = 1.0 / 60.0;
const SECONDS: f32 = 60.0;
const MODES: [ModeKind; 3] = [ModeKind::Race, ModeKind::Survival, ModeKind::Puzzle];

fn config(mode: ModeKind) -> GameConfig {
    let mut config = GameConfig::load_or_default(format!(
        "{}/{DEFAULT_CONFIG_PATH}",
        env!("CARGO_MANIFEST_DIR")
    ));
    config.mode = mode;
    config.start_in_menu = false;
    config.difficulty = Some(Difficulty::Normal);
    config.players = 1;
    config.opponents = 3;
    config.seed = 4242;
    config
}

fn core_app(config: GameConfig) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin, AssetPlugin::default()))
        .insert_resource(config)
        .add_plugins(TowerCorePlugin);
    app
}

fn full_app(config: GameConfig) -> App {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: bevy::window::ExitCondition::DontExit,
                ..default()
            })
            .set(bevy::render::RenderPlugin {
                render_creation: bevy::render::settings::WgpuSettings {
                    backends: None,
                    ..default()
                }
                .into(),
                ..default()
            })
            .disable::<bevy::winit::WinitPlugin>()
            .disable::<bevy::audio::AudioPlugin>(),
    )
    .insert_resource(tricky_towers::art::Silent)
    .insert_resource(config)
    .add_plugins((
        TowerCorePlugin,
        tricky_towers::input::KeyboardInputPlugin,
        tricky_towers::menu::MenuPlugin,
        tricky_towers::visuals::VisualsPlugin,
        tricky_towers::art::ArtPlugin,
        tricky_towers::letters::LettersPlugin,
        tricky_towers::wizards::WizardsPlugin,
        tricky_towers::results::ResultsPlugin,
        tricky_towers::effects::EffectsPlugin,
        tricky_towers::casts::CastsPlugin,
        tricky_towers::packs::PacksPlugin,
        tricky_towers::weather::WeatherPlugin,
        tricky_towers::particles::ParticlesPlugin,
        tricky_towers::hud::HudPlugin,
    ));
    app
}

/// Runs `SECONDS` of 60 fps frames with every tower played by the AI and
/// prints mean, 99th percentile and worst frame.
fn measure(label: &str, mode: ModeKind, mut app: App) {
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
        FRAME,
    )));
    app.finish();
    app.cleanup();
    let mut times = Vec::new();
    let mut bricks = 0;
    for _ in 0..(SECONDS / FRAME) as usize {
        let world = app.world_mut();
        let humans: Vec<Entity> = world
            .query_filtered::<Entity, (With<Tower>, Without<AiControlled>)>()
            .iter(world)
            .collect();
        for entity in humans {
            world.entity_mut(entity).insert(AiControlled);
        }
        if world.get_resource::<Match>().is_some_and(|m| m.phase == Phase::Over) {
            world.resource_mut::<MatchRequest>().pending = true;
        }
        let start = Instant::now();
        app.update();
        times.push(start.elapsed());
        let world = app.world_mut();
        bricks = bricks.max(world.query::<&Brick>().iter(world).count());
    }
    times.sort();
    let mean = times.iter().sum::<Duration>() / times.len() as u32;
    let p99 = times[times.len() * 99 / 100];
    let worst = *times.last().unwrap();
    eprintln!(
        "{label:5} {mode:?}: mean {:.2} ms, p99 {:.2} ms, worst {:.2} ms, up to {bricks} bricks",
        mean.as_secs_f64() * 1e3,
        p99.as_secs_f64() * 1e3,
        worst.as_secs_f64() * 1e3,
    );
}

#[test]
#[ignore = "DEBUG-TOOL: frame-time probe, run by hand"]
fn frame_times() {
    for mode in MODES {
        measure("core", mode, core_app(config(mode)));
    }
    for mode in MODES {
        measure("full", mode, full_app(config(mode)));
    }
}
