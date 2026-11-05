//! DEBUG-TOOL: timed automation must still control a paused match.

use std::time::Duration;

use bevy::app::AppExit;
use bevy::ecs::message::MessageCursor;
use bevy::input::InputPlugin;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use tricky_towers::TowerCorePlugin;
use tricky_towers::config::{GameConfig, ModeKind, SpellKind};
use tricky_towers::debug::{DebugCommand, DebugPicture, DebugQueue, DebugScript, ScriptStep};
use tricky_towers::gameplay::{Match, Phase, Tower, TowerState};
use tricky_towers::menu::{MenuPlugin, PauseMenu};

#[test]
fn a_script_can_picture_resume_or_quit_while_game_time_is_paused() {
    for resume in [false, true] {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, TransformPlugin, AssetPlugin::default(), InputPlugin))
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(1.0 / 64.0)))
            .insert_resource(GameConfig { mode: ModeKind::Race, ..default() })
            .add_plugins((TowerCorePlugin, MenuPlugin));
        app.finish();
        app.cleanup();
        for _ in 0..4 {
            app.update();
        }
        assert_eq!(app.world().resource::<Match>().phase, Phase::Playing);
        let tower = app.world_mut().query_filtered::<Entity, With<Tower>>().single(app.world()).unwrap();
        let mut steps = vec![
            ScriptStep { at: 0.1, run: DebugCommand::Tap { key: "Escape".into() } },
            ScriptStep { at: 0.3, run: DebugCommand::GiveLight { tower: 0, spell: SpellKind::Ivy } },
            ScriptStep { at: 0.5, run: DebugCommand::Picture { name: "paused".into() } },
        ];
        if resume {
            steps.push(ScriptStep { at: 0.8, run: DebugCommand::Tap { key: "Escape".into() } });
        }
        steps.push(ScriptStep { at: if resume { 1.3 } else { 0.8 }, run: DebugCommand::Quit });
        app.insert_resource(DebugScript::new(steps));
        let started = app.world().resource::<Time<Real>>().elapsed_secs();
        let mut picture_reader = MessageCursor::<DebugPicture>::default();
        let mut exit_reader = MessageCursor::<AppExit>::default();
        let mut pictures = Vec::new();
        let mut frozen = None;
        let mut quit = false;
        for _ in 0..128 {
            app.update();
            let world = app.world();
            let paused = world.resource::<PauseMenu>().is_active();
            let virtual_time = world.resource::<Time<Virtual>>().elapsed_secs_f64();
            if paused {
                assert_eq!(world.resource::<Match>().phase, Phase::Playing);
                assert_eq!(*frozen.get_or_insert(virtual_time), virtual_time, "the script's clock must not advance the game");
                if world.resource::<Time<Real>>().elapsed_secs() - started > 0.4 {
                    assert!(world.get::<TowerState>(tower).unwrap().light_spell.is_none(), "gameplay commands wait for a fixed step");
                }
            }
            pictures.extend(picture_reader.read(world.resource::<Messages<DebugPicture>>())
                .map(|picture| (picture.0.clone(), paused)));
            if exit_reader.read(world.resource::<Messages<AppExit>>()).any(|exit| *exit == AppExit::Success) {
                quit = true;
                break;
            }
        }
        assert!(quit, "Quit must be serviced even when FixedUpdate is stopped");
        assert!(app.world().resource::<DebugScript>().finished());
        assert_eq!(pictures, [("paused".to_string(), true)], "Picture runs exactly once during pause");
        let world = app.world();
        if resume {
            assert!(!world.resource::<PauseMenu>().is_active());
            assert!(world.resource::<Time<Virtual>>().elapsed_secs_f64() > frozen.unwrap());
            assert_eq!(world.get::<TowerState>(tower).unwrap().light_spell, Some(SpellKind::Ivy));
            assert!(world.resource::<DebugQueue>().0.is_empty());
        } else {
            assert!(world.resource::<PauseMenu>().is_active());
            assert_eq!(world.resource::<DebugQueue>().0.len(), 1, "the queued gameplay command remains unexecuted");
        }
    }
}
