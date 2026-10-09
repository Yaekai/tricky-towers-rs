//! Drives the menus with simulated key presses.

use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use tricky_towers::TowerCorePlugin;
use tricky_towers::config::{Difficulty, GameConfig, ModeKind};
use tricky_towers::gameplay::{Match, Phase};
use tricky_towers::menu::{MainMenu, MenuPlugin, MenuScreen};
use tricky_towers::select::{Lobby, Seat};

fn app() -> App {
    app_with(|_| {})
}

/// The menu app, with `extra` plugins added before it starts.
fn app_with(extra: impl FnOnce(&mut App)) -> App {
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
    extra(&mut app);
    app.finish();
    app.cleanup();
    app.update();
    // Past "Press Enter to start" to the title tiles.
    assert_eq!(*app.world().resource::<MenuScreen>(), MenuScreen::Start);
    tap(&mut app, KeyCode::Enter);
    assert_eq!(*app.world().resource::<MenuScreen>(), MenuScreen::Main);
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

/// Local Battle, join with the keyboard, on to the modes, take the first mode,
/// difficulty and single match, play.
fn start(app: &mut App) {
    // Title, seat, confirm the wizard, confirm the pack, Select Mode, then its columns.
    for _ in 0..8 {
        tap(app, KeyCode::Enter);
    }
}

#[test]
fn local_battle_seats_players_then_starts_the_picked_mode() {
    let mut app = app();
    let screen = |app: &App| *app.world().resource::<MenuScreen>();
    assert_eq!(app.world().resource::<Match>().phase, Phase::Menu);
    // Local Battle is the highlighted title tile; it opens player select.
    tap(&mut app, KeyCode::Enter);
    assert_eq!(screen(&app), MenuScreen::Players);
    // Enter seats the keyboard player, Tab adds an AI wizard.
    tap(&mut app, KeyCode::Enter);
    tap(&mut app, KeyCode::Tab);
    tap(&mut app, KeyCode::Tab);
    tap(&mut app, KeyCode::Backspace);
    assert_eq!(
        app.world().resource::<Lobby>().seats[..2],
        [Seat::Keyboard, Seat::Ai]
    );
    assert_eq!(screen(&app), MenuScreen::Players);
    // Enter confirms the wizard, then the brick pack, then goes on.
    tap(&mut app, KeyCode::Enter);
    tap(&mut app, KeyCode::Enter);
    assert_eq!(app.world().resource::<Lobby>().confirmed[0], [true, true]);
    assert_eq!(screen(&app), MenuScreen::Players);
    tap(&mut app, KeyCode::Enter);
    assert_eq!(screen(&app), MenuScreen::Modes);
    // Race, Survival, Puzzle down the mode column; Enter moves to the
    // difficulty column, where Down goes from Normal to Special.
    tap(&mut app, KeyCode::ArrowDown);
    tap(&mut app, KeyCode::ArrowDown);
    tap(&mut app, KeyCode::Enter);
    assert_eq!(app.world().resource::<Lobby>().column, 1);
    tap(&mut app, KeyCode::ArrowDown);
    // Esc steps back to the mode column, Right comes back.
    tap(&mut app, KeyCode::Escape);
    assert_eq!(screen(&app), MenuScreen::Modes);
    assert_eq!(app.world().resource::<Lobby>().column, 0);
    tap(&mut app, KeyCode::ArrowRight);
    tap(&mut app, KeyCode::Enter);
    // On to the third column (single match or a cup), and play.
    assert_eq!(app.world().resource::<Lobby>().column, 2);
    tap(&mut app, KeyCode::Enter);
    let game = app.world().resource::<Match>();
    assert_eq!((game.phase, game.mode), (Phase::Playing, ModeKind::Puzzle));
    let config = app.world().resource::<GameConfig>();
    assert_eq!((config.players, config.opponents), (1, 1));
    assert_eq!(config.difficulty, Some(Difficulty::Pro));
}

#[test]
fn single_player_opens_endless_and_plays_endless_race() {
    let mut app = app();
    let screen = |app: &App| *app.world().resource::<MenuScreen>();
    tap(&mut app, KeyCode::ArrowDown);
    tap(&mut app, KeyCode::Enter);
    assert_eq!(screen(&app), MenuScreen::Single);
    tap(&mut app, KeyCode::Escape);
    assert_eq!(screen(&app), MenuScreen::Main);
    tap(&mut app, KeyCode::Enter);
    // Trials opens its list; Esc comes back.
    tap(&mut app, KeyCode::ArrowUp);
    tap(&mut app, KeyCode::Enter);
    assert_eq!(screen(&app), MenuScreen::Trials);
    tap(&mut app, KeyCode::Escape);
    assert_eq!(screen(&app), MenuScreen::Single);
    tap(&mut app, KeyCode::ArrowDown);
    tap(&mut app, KeyCode::Enter);
    assert_eq!(screen(&app), MenuScreen::Endless);
    tap(&mut app, KeyCode::Escape);
    assert_eq!(screen(&app), MenuScreen::Single);
    tap(&mut app, KeyCode::Enter);
    tap(&mut app, KeyCode::Enter);
    let game = app.world().resource::<Match>();
    assert_eq!((game.phase, game.mode), (Phase::Playing, ModeKind::Endless));
    let config = app.world().resource::<GameConfig>();
    assert_eq!((config.players, config.opponents), (1, 0));
}

#[test]
fn endless_survival_and_puzzle_start_from_the_endless_screen() {
    // The original lists Race, Puzzle, Survival.
    for (downs, mode) in [(1, ModeKind::EndlessPuzzle), (2, ModeKind::EndlessSurvival)] {
        let mut app = app();
        // Title → Single Player → Endless.
        tap(&mut app, KeyCode::ArrowDown);
        tap(&mut app, KeyCode::Enter);
        tap(&mut app, KeyCode::Enter);
        assert_eq!(*app.world().resource::<MenuScreen>(), MenuScreen::Endless);
        for _ in 0..downs {
            tap(&mut app, KeyCode::ArrowDown);
        }
        tap(&mut app, KeyCode::Enter);
        let game = app.world().resource::<Match>();
        assert_eq!((game.phase, game.mode), (Phase::Playing, mode));
        let config = app.world().resource::<GameConfig>();
        assert_eq!((config.players, config.opponents), (1, 0));
    }
}

#[test]
fn arrows_walk_the_title_tiles_and_escape_comes_back() {
    let mut app = app();
    let selected = |app: &App| app.world().resource::<MainMenu>().selected;
    tap(&mut app, KeyCode::ArrowDown);
    assert_eq!(selected(&app), 1, "Single Player is under Local Battle");
    tap(&mut app, KeyCode::ArrowRight);
    assert_ne!(selected(&app), 1, "right leaves the left column");
    tap(&mut app, KeyCode::ArrowDown);
    tap(&mut app, KeyCode::ArrowDown);
    tap(&mut app, KeyCode::ArrowDown);
    assert_eq!(selected(&app), 6, "Quit is at the bottom");
    tap(&mut app, KeyCode::ArrowUp);
    tap(&mut app, KeyCode::ArrowUp);
    tap(&mut app, KeyCode::ArrowUp);
    tap(&mut app, KeyCode::ArrowLeft);
    tap(&mut app, KeyCode::ArrowUp);
    assert_eq!(selected(&app), 0);
    tap(&mut app, KeyCode::Enter);
    assert_eq!(*app.world().resource::<MenuScreen>(), MenuScreen::Players);
    tap(&mut app, KeyCode::Escape);
    assert_eq!(*app.world().resource::<MenuScreen>(), MenuScreen::Main);
    assert_eq!(app.world().resource::<Match>().phase, Phase::Menu);
}

#[test]
fn escape_pauses_and_the_pause_menu_resumes_retries_or_quits() {
    let mut app = app();
    start(&mut app);
    assert_eq!(app.world().resource::<Match>().phase, Phase::Playing);
    let screen = |app: &App| *app.world().resource::<MenuScreen>();
    let phase = |app: &App| app.world().resource::<Match>().phase;

    tap(&mut app, KeyCode::Escape);
    assert_eq!(
        (phase(&app), screen(&app)),
        (Phase::Menu, MenuScreen::Pause)
    );
    tap(&mut app, KeyCode::Enter);
    assert_eq!(phase(&app), Phase::Playing, "Resume");

    tap(&mut app, KeyCode::Escape);
    let round = app.world().resource::<Match>().round;
    tap(&mut app, KeyCode::ArrowDown);
    tap(&mut app, KeyCode::Enter);
    let game = app.world().resource::<Match>();
    assert_eq!(game.phase, Phase::Playing, "Retry");
    assert_ne!(game.round, round, "Retry starts a new round");

    tap(&mut app, KeyCode::Escape);
    tap(&mut app, KeyCode::ArrowUp);
    tap(&mut app, KeyCode::Enter);
    assert_eq!(
        (phase(&app), screen(&app)),
        (Phase::Menu, MenuScreen::Main),
        "Quit"
    );
    tap(&mut app, KeyCode::Escape);
    assert_eq!(
        phase(&app),
        Phase::Menu,
        "Esc on the title screen doesn't resume"
    );
}

#[test]
fn game_completed_popup_rematches_selects_a_mode_or_goes_to_the_title() {
    use tricky_towers::popup::Popup;
    let mut app = app();
    app.world_mut()
        .resource_mut::<GameConfig>()
        .moments
        .popup_after = 0.1;
    start(&mut app);
    let end = |app: &mut App| {
        let mut game = app.world_mut().resource_mut::<Match>();
        game.phase = Phase::Over;
        game.winner = Some(0);
        // The winner's roof is already on.
        let world = app.world_mut();
        for mut state in world
            .query::<&mut tricky_towers::gameplay::TowerState>()
            .iter_mut(world)
        {
            state.roof = tricky_towers::gameplay::RoofStage::Done;
        }
        for _ in 0..12 {
            app.update();
        }
        assert!(app.world().resource::<Popup>().shown);
    };
    end(&mut app);
    let round = app.world().resource::<Match>().round;
    tap(&mut app, KeyCode::Enter);
    let game = app.world().resource::<Match>();
    assert_eq!(game.phase, Phase::Playing, "Rematch");
    assert_ne!(game.round, round);
    assert!(!app.world().resource::<Popup>().shown);

    end(&mut app);
    tap(&mut app, KeyCode::ArrowDown);
    tap(&mut app, KeyCode::Enter);
    assert_eq!(
        *app.world().resource::<MenuScreen>(),
        MenuScreen::Modes,
        "Select Mode"
    );
    for _ in 0..3 {
        tap(&mut app, KeyCode::Enter);
    }
    end(&mut app);
    tap(&mut app, KeyCode::ArrowUp);
    tap(&mut app, KeyCode::Enter);
    assert_eq!(
        app.world().resource::<Match>().phase,
        Phase::Menu,
        "Main menu"
    );
    assert_eq!(*app.world().resource::<MenuScreen>(), MenuScreen::Main);
}

#[test]
fn trials_list_plays_the_picked_trial() {
    let mut app = app();
    let screen = |app: &App| *app.world().resource::<MenuScreen>();
    // Title → Single Player → Trials (above Endless, where the cursor starts).
    tap(&mut app, KeyCode::ArrowDown);
    tap(&mut app, KeyCode::Enter);
    tap(&mut app, KeyCode::ArrowUp);
    tap(&mut app, KeyCode::Enter);
    assert_eq!(screen(&app), MenuScreen::Trials);
    // The second sample trial is Survival.
    tap(&mut app, KeyCode::ArrowDown);
    tap(&mut app, KeyCode::Enter);
    let game = app.world().resource::<Match>();
    assert_eq!(
        (game.phase, game.mode),
        (Phase::Playing, ModeKind::Survival)
    );
    let config = app.world().resource::<GameConfig>();
    assert_eq!(config.trial, Some(1));
    assert_eq!((config.players, config.opponents), (1, 0));
    assert_eq!(
        config.survival.bricks_to_place,
        config.trials[1].bricks.unwrap()
    );
}

#[test]
fn the_cup_column_picks_a_cup_and_starting_begins_it() {
    use tricky_towers::cups::Cup;
    let mut app = app();
    tap(&mut app, KeyCode::Enter);
    tap(&mut app, KeyCode::Enter);
    tap(&mut app, KeyCode::Tab);
    tap(&mut app, KeyCode::Enter); // wizard
    tap(&mut app, KeyCode::Enter); // brick pack
    tap(&mut app, KeyCode::Enter);
    assert_eq!(*app.world().resource::<MenuScreen>(), MenuScreen::Modes);
    // The third column: single match → Tiny → Big → Super → single match → Tiny.
    tap(&mut app, KeyCode::ArrowRight);
    tap(&mut app, KeyCode::ArrowRight);
    let count = app.world().resource::<GameConfig>().cups.sizes.len();
    for _ in 0..count + 2 {
        tap(&mut app, KeyCode::ArrowDown);
    }
    assert_eq!(app.world().resource::<Cup>().pick, Some(0));
    tap(&mut app, KeyCode::Enter);
    assert_eq!(app.world().resource::<Match>().phase, Phase::Playing);
    let cup = app.world().resource::<Cup>();
    assert_eq!((cup.size, cup.played), (Some(0), 0));
    assert!(cup.in_progress());
}

#[test]
fn settings_change_with_left_and_right_and_escape_goes_back() {
    use tricky_towers::menu::TILES;
    use tricky_towers::settings::{Row, Settings, SettingsMenu};
    let mut app = app();
    let settings_tile = TILES.iter().position(|t| t.key == "settings").unwrap();
    app.world_mut().resource_mut::<MainMenu>().selected = settings_tile;
    tap(&mut app, KeyCode::Enter);
    assert_eq!(*app.world().resource::<MenuScreen>(), MenuScreen::Settings);
    let music = app.world().resource::<Settings>().music;
    // Down to Music, Left lowers it a step.
    tap(&mut app, KeyCode::ArrowDown);
    tap(&mut app, KeyCode::ArrowLeft);
    let after = app.world().resource::<Settings>().music;
    assert!((after - (music - 0.1)).abs() < 1e-4, "{music} -> {after}");
    // Enter on Explanations toggles it (on by default).
    tap(&mut app, KeyCode::ArrowDown);
    tap(&mut app, KeyCode::Enter);
    assert!(!app.world().resource::<Settings>().explanations);
    // Right on Keybinding (no value) turns to page 2, on the same row: Apply.
    for _ in 0..3 {
        tap(&mut app, KeyCode::ArrowDown);
    }
    tap(&mut app, KeyCode::ArrowRight);
    let menu = app.world().resource::<SettingsMenu>();
    assert_eq!((menu.page, menu.row()), (1, Row::Apply));
    tap(&mut app, KeyCode::ArrowDown);
    assert_eq!(
        app.world().resource::<SettingsMenu>().row(),
        Row::Fullscreen
    );
    // Page 2 waits for Apply.
    tap(&mut app, KeyCode::Enter);
    assert!(app.world().resource::<Settings>().fullscreen);
    tap(&mut app, KeyCode::ArrowUp);
    assert_eq!(app.world().resource::<SettingsMenu>().row(), Row::Apply);
    tap(&mut app, KeyCode::Enter);
    assert!(!app.world().resource::<Settings>().fullscreen);
    tap(&mut app, KeyCode::Escape);
    assert_eq!(*app.world().resource::<MenuScreen>(), MenuScreen::Main);
}

#[test]
fn each_seat_picks_its_wizard_and_brick_pack() {
    use tricky_towers::config::BrickPack;
    use tricky_towers::gameplay::{Tower, TowerSkin};
    let mut app = app();
    tap(&mut app, KeyCode::Enter); // title → Select Players
    tap(&mut app, KeyCode::Enter); // seat the keyboard player
    // Character: two to the right (wizard 1 → 3); then the brick pack: one to the right.
    tap(&mut app, KeyCode::ArrowRight);
    tap(&mut app, KeyCode::ArrowRight);
    tap(&mut app, KeyCode::ArrowDown); // confirm the wizard: the cursor moves to the packs
    tap(&mut app, KeyCode::ArrowRight);
    let lobby = app.world().resource::<Lobby>();
    assert_eq!(lobby.skins[0], (2, BrickPack::Christmas));
    // Up undoes the wizard's confirmation: Left/Right turn the wizard again.
    tap(&mut app, KeyCode::ArrowUp);
    tap(&mut app, KeyCode::ArrowLeft);
    assert_eq!(app.world().resource::<Lobby>().skins[0], (1, BrickPack::Christmas));
    tap(&mut app, KeyCode::ArrowRight);
    tap(&mut app, KeyCode::ArrowDown);
    tap(&mut app, KeyCode::ArrowDown); // confirm the pack: Left/Right do nothing now
    tap(&mut app, KeyCode::ArrowRight);
    assert_eq!(app.world().resource::<Lobby>().skins[0], (2, BrickPack::Christmas));
    tap(&mut app, KeyCode::Tab); // an AI wizard keeps its seat's default
    tap(&mut app, KeyCode::Enter); // on to Select Mode
    tap(&mut app, KeyCode::Enter);
    tap(&mut app, KeyCode::Enter);
    tap(&mut app, KeyCode::Enter); // play
    assert_eq!(app.world().resource::<Match>().phase, Phase::Playing);
    let world = app.world_mut();
    let mut skins: Vec<(usize, TowerSkin)> = world
        .query::<(&Tower, &TowerSkin)>()
        .iter(world)
        .map(|(t, s)| (t.index, *s))
        .collect();
    skins.sort_by_key(|(i, _)| *i);
    assert_eq!(
        skins[0].1,
        TowerSkin {
            wizard: 2,
            pack: BrickPack::Christmas
        }
    );
    assert_eq!(
        skins[1].1,
        TowerSkin {
            wizard: 1,
            pack: BrickPack::Default
        }
    );
}

#[test]
fn an_earned_tier_plays_its_unlock_once_then_opens() {
    use tricky_towers::trials::TrialProgress;
    use tricky_towers::trials_menu::TrialsMenu;
    let mut app = app();
    // Eight trials beaten: Apprentice (8 medals) opens.
    {
        let names: Vec<String> = app.world().resource::<GameConfig>().trials[..8]
            .iter()
            .map(|t| t.name.clone())
            .collect();
        app.world_mut().resource_mut::<TrialProgress>().done = names;
    }
    tap(&mut app, KeyCode::ArrowDown);
    tap(&mut app, KeyCode::Enter);
    tap(&mut app, KeyCode::ArrowUp);
    tap(&mut app, KeyCode::Enter);
    assert_eq!(*app.world().resource::<MenuScreen>(), MenuScreen::Trials);
    let menu = app.world().resource::<TrialsMenu>();
    assert_eq!(menu.unlock.map(|(tier, _)| tier), Some(1));
    // The screen turns to the tier's first trial, and keys wait for the unlock.
    assert_eq!(menu.selected, 10);
    tap(&mut app, KeyCode::ArrowDown);
    assert_eq!(app.world().resource::<TrialsMenu>().selected, 10);
    for _ in 0..(5.0 * 64.0) as usize {
        app.update();
    }
    assert_eq!(app.world().resource::<TrialsMenu>().unlock, None);
    assert_eq!(app.world().resource::<TrialProgress>().tiers_shown, 2);
    tap(&mut app, KeyCode::ArrowDown);
    assert_eq!(app.world().resource::<TrialsMenu>().selected, 11);
    // Back and in again: it doesn't play twice.
    tap(&mut app, KeyCode::Escape);
    tap(&mut app, KeyCode::Enter);
    assert_eq!(*app.world().resource::<MenuScreen>(), MenuScreen::Trials);
    assert_eq!(app.world().resource::<TrialsMenu>().unlock, None);
}

#[test]
fn keybinding_rebinds_a_key_and_escape_closes_the_list_first() {
    use tricky_towers::keybinding::{Action, KeyBindings, KeybindingMenu};
    use tricky_towers::menu::TILES;
    use tricky_towers::settings::{Row, SettingsMenu};
    let mut app = app();
    let settings_tile = TILES.iter().position(|t| t.key == "settings").unwrap();
    app.world_mut().resource_mut::<MainMenu>().selected = settings_tile;
    tap(&mut app, KeyCode::Enter);
    // Keybinding is page 1's last row.
    for _ in 0..5 {
        tap(&mut app, KeyCode::ArrowDown);
    }
    assert_eq!(
        app.world().resource::<SettingsMenu>().row(),
        Row::Keybinding
    );
    tap(&mut app, KeyCode::Enter);
    assert!(app.world().resource::<KeybindingMenu>().open);
    // Drop is the third action: Enter waits, S binds it.
    tap(&mut app, KeyCode::ArrowDown);
    tap(&mut app, KeyCode::ArrowDown);
    tap(&mut app, KeyCode::Enter);
    assert!(app.world().resource::<KeybindingMenu>().waiting);
    tap(&mut app, KeyCode::KeyS);
    assert_eq!(
        app.world().resource::<KeyBindings>().key(Action::Drop),
        KeyCode::KeyS
    );
    assert!(!app.world().resource::<KeybindingMenu>().waiting);
    // Esc closes the list, then leaves Settings.
    tap(&mut app, KeyCode::Escape);
    assert!(!app.world().resource::<KeybindingMenu>().open);
    assert_eq!(*app.world().resource::<MenuScreen>(), MenuScreen::Settings);
    tap(&mut app, KeyCode::Escape);
    assert_eq!(*app.world().resource::<MenuScreen>(), MenuScreen::Main);
}

#[test]
fn tab_opens_game_info_in_a_match_and_pausing_closes_it() {
    use tricky_towers::game_info::{GameInfo, GameInfoPlugin};
    let mut app = app_with(|app| {
        app.add_plugins(GameInfoPlugin);
    });
    start(&mut app);
    assert_eq!(app.world().resource::<Match>().phase, Phase::Playing);
    tap(&mut app, KeyCode::Tab);
    assert!(app.world().resource::<GameInfo>().open);
    tap(&mut app, KeyCode::Tab);
    assert!(!app.world().resource::<GameInfo>().open);
    tap(&mut app, KeyCode::Tab);
    tap(&mut app, KeyCode::Escape);
    assert!(!app.world().resource::<GameInfo>().open);
}

#[test]
fn a_finished_battle_counts_in_stats_and_the_stats_popup_opens_from_settings() {
    use tricky_towers::menu::TILES;
    use tricky_towers::settings::{Row, SettingsMenu};
    use tricky_towers::stats::{Stats, StatsMenu};
    let mut app = app();
    start(&mut app);
    // Player 1 wins: the match ends.
    {
        let world = app.world_mut();
        let mut q = world.query::<(
            &tricky_towers::gameplay::Tower,
            &mut tricky_towers::gameplay::TowerState,
        )>();
        for (tower, mut state) in q.iter_mut(world) {
            if tower.is_first_player() {
                state.status = tricky_towers::gameplay::TowerStatus::Won;
            }
        }
    }
    for _ in 0..64 {
        app.update();
    }
    assert_eq!(app.world().resource::<Match>().phase, Phase::Over);
    app.update();
    let stats = app.world().resource::<Stats>();
    let mode = app.world().resource::<Match>().mode;
    assert_eq!(stats.get(mode, None), (1, 1), "{:?}", stats.records);
    // Back to the title, into Settings, and Stats (page 1's fourth row).
    tap(&mut app, KeyCode::Escape);
    let settings_tile = TILES.iter().position(|t| t.key == "settings").unwrap();
    app.world_mut().resource_mut::<MainMenu>().selected = settings_tile;
    tap(&mut app, KeyCode::Enter);
    for _ in 0..3 {
        tap(&mut app, KeyCode::ArrowDown);
    }
    assert_eq!(app.world().resource::<SettingsMenu>().row(), Row::Stats);
    tap(&mut app, KeyCode::Enter);
    assert!(app.world().resource::<StatsMenu>().open);
    tap(&mut app, KeyCode::Escape);
    assert!(!app.world().resource::<StatsMenu>().open);
    assert_eq!(*app.world().resource::<MenuScreen>(), MenuScreen::Settings);
}
