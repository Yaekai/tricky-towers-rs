//! The menus: the original's title screen of tiles (Local Battle, Single
//! Player, ...), the pause menu, and the way between them and the player and
//! mode select screens (src/select.rs).

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::ecs::hierarchy::ChildSpawnerCommands;

use crate::art::Art;
use crate::config::GameConfig;
use crate::gameplay::{Match, MatchRequest, Phase};
use crate::select::{
    Lobby, modes_input, players_input, show_select_screens, spawn_select_screens, step_back,
};
use crate::single::{
    Leaderboard, SingleMenu, endless_input, record_endless_scores, show_carousel, show_list_ribbon,
    show_single_screens, single_input, spawn_single_screens,
};
use crate::start::{StartClock, show_start_screen, spawn_start_screen, start_input};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            crate::popup::PopupPlugin,
            crate::cups_menu::CupsMenuPlugin,
            crate::settings::SettingsPlugin,
            crate::navbar::NavBarPlugin,
        ))
        .init_resource::<MenuScreen>()
        .init_resource::<MainMenu>()
        .init_resource::<PauseMenu>()
        .init_resource::<StartClock>()
        .init_resource::<Lobby>()
        .init_resource::<SingleMenu>()
        .init_resource::<Leaderboard>()
        .init_resource::<crate::visuals::MenuLook>()
        .init_resource::<crate::trials_menu::TrialsMenu>()
        .add_systems(
            Startup,
            (
                spawn_start_screen,
                spawn_main_menu,
                spawn_select_screens,
                spawn_single_screens,
                crate::trials_menu::spawn_trials_screen,
                spawn_pause_menu,
            ),
        )
        .add_systems(
            Update,
            (
                // Each screen reads its keys before the screen that leads to it,
                // so one press never acts on two screens.
                menu_escape,
                modes_input,
                players_input,
                endless_input,
                crate::trials_menu::trials_input,
                single_input,
                crate::settings::settings_input,
                main_menu_input,
                start_input,
                pause_input,
                show_start_screen,
                show_select_screens,
                show_single_screens,
                show_carousel,
                show_list_ribbon,
                crate::trials_menu::show_trials_screen,
                crate::settings::show_settings_screen,
                show_pause_menu,
                show_main_menu,
                spin_tile_rays,
            )
                .chain()
                .in_set(MenuSystems),
        )
        .add_systems(
            Update,
            (record_endless_scores, crate::single::turn_single_rays),
        );
    }
}

fn menu_escape(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mut game: ResMut<Match>,
    mut screen: ResMut<MenuScreen>,
    mut pause: ResMut<PauseMenu>,
    mut lobby: ResMut<Lobby>,
    mut binds: ResMut<crate::keybinding::KeybindingMenu>,
    mut stats: ResMut<crate::stats::StatsMenu>,
    mut credits: ResMut<crate::credits::CreditsMenu>,
    mut single: ResMut<SingleMenu>,
) {
    let Some(keys) = keys else { return };
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    match (game.phase, *screen) {
        // Pause the match.
        (Phase::Playing | Phase::Intro, _) => {
            pause.resume_to = Some(game.phase);
            pause.selected = 0;
            game.phase = Phase::Menu;
            *screen = MenuScreen::Pause;
        }
        // Back to the match we paused.
        (Phase::Menu, MenuScreen::Pause) => {
            if let Some(phase) = pause.resume_to.take() {
                game.phase = phase;
            }
        }
        // Leave a finished match, or step back a screen.
        // Back on Select Players undoes the keyboard seat's last step first.
        (Phase::Menu, MenuScreen::Players)
            if lobby
                .seats
                .iter()
                .position(|s| *s == crate::select::Seat::Keyboard)
                .is_some_and(|seat| crate::select::step_back_seat(&mut lobby, seat)) => {}
        (Phase::Over, _) | (Phase::Menu, MenuScreen::Players) => {
            pause.resume_to = None;
            game.phase = Phase::Menu;
            *screen = MenuScreen::Main;
        }
        (Phase::Menu, MenuScreen::Modes) => step_back(&mut lobby, &mut screen),
        // Back on Single Player undoes the last choice first.
        (Phase::Menu, MenuScreen::Single) => {
            if !crate::single::step_back(&mut single) {
                *screen = MenuScreen::Main;
            }
        }
        (Phase::Menu, MenuScreen::Endless | MenuScreen::Trials) => *screen = MenuScreen::Single,
        (Phase::Menu, MenuScreen::Main) => *screen = MenuScreen::Start,
        // The explanation popup waits for Continue.
        (Phase::Menu, MenuScreen::Explain) => {}
        // A popup over Settings closes first.
        (Phase::Menu, MenuScreen::Settings) => {
            if credits.open {
                credits.open = false;
            } else if stats.open {
                stats.open = false;
            } else if !crate::keybinding::keybinding_escape(&mut binds) {
                *screen = MenuScreen::Main;
            }
        }
        (Phase::Menu, MenuScreen::Start) => {}
    }
}

/// The menu systems. Anything else that changes [`MenuScreen`] on a key
/// press runs after them, so the same press isn't read twice.
#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub struct MenuSystems;

/// Which menu screen is up while the game is in the menu.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuScreen {
    /// "Press Enter to start" under the logo, before the title tiles.
    #[default]
    Start,
    /// The title tiles.
    Main,
    /// "Select Players": seats for people and AI wizards.
    Players,
    /// "Select Mode": mode tiles with their difficulty.
    Modes,
    /// Single Player: the player's seat over Trials and Endless.
    Single,
    /// Endless: Race, Survival and Puzzle beside the leaderboard.
    Endless,
    /// Trials: the list of single-player challenges.
    Trials,
    /// Settings: volume, music and full screen.
    Settings,
    /// "Game Paused" over a match: Resume, Retry, Quit.
    Pause,
    /// The explanation popup before a match (explain.rs).
    Explain,
}

/// The pause menu's highlighted option and the phase Resume goes back to.
#[derive(Resource, Default)]
pub struct PauseMenu {
    pub selected: usize,
    pub resume_to: Option<Phase>,
}

/// The original's pause options, top to bottom.
pub const PAUSE_OPTIONS: [&str; 3] = ["Resume", "Retry", "Quit"];

/// The pause options shown: a cup match has no Retry (CupMatchFlowController's
/// allowRetry), so Quit moves up into its place.
pub fn pause_options(cup: Option<&crate::cups::Cup>) -> Vec<&'static str> {
    let retry = !cup.is_some_and(|c| c.in_progress());
    PAUSE_OPTIONS.into_iter().filter(|o| retry || *o != "Retry").collect()
}

#[derive(Component)]
struct PauseRoot;

/// Layout of the pause screen in pixels of a 1920x1080 screen (PausePopup): the
/// title and the options share a column 840 wide centred 480 px right of the
/// screen centre (the left half holds the controls); the title's and first
/// option's heights over the centre, the row pitch and text size, and the bar
/// behind the picked option.
const PAUSE_COLUMN: (f32, f32) = (1020.0, 840.0);
const PAUSE_TITLE_Y: f32 = 260.0;
const PAUSE_ITEMS_Y: f32 = 120.0;
const PAUSE_ROW: (f32, f32) = (60.0, 30.0);
const PAUSE_BAR: f32 = 28.0;
/// The picked option is yellow, the others pink (Max's screenshots).
const PAUSE_PICKED: Color = Color::srgb(1.0, 0.85, 0.25);
const PAUSE_OTHER: Color = Color::srgb(1.0, 0.31, 0.55);

#[derive(Component)]
struct PauseBar(usize);

/// An option's words, coloured by whether it is picked.
#[derive(Component)]
struct PauseWords(usize);

/// The keyboard controls panel in the pause screen's left half (the original's
/// ControlsExplanationContentKeyboard, in pixels of a 1920x1080 screen): the panel,
/// then each section's top below the panel's top, its height, its title and its key
/// caps as (action, offset from the section's centre, arrows: -1 left, 1 right,
/// 2 down, 0 none).
const CONTROLS_PANEL: ((f32, f32), (f32, f32)) = ((480.0, 860.0), (480.0, 554.0));
const CONTROLS_INSET: f32 = 50.35;
type KeyCap = (crate::keybinding::Action, (f32, f32), i32);
const CONTROLS_SECTIONS: [(f32, f32, &str, &[KeyCap]); 5] = {
    use crate::keybinding::Action::*;
    [
        (
            70.0,
            155.0,
            "Move Brick",
            &[
                (MoveLeft, (-80.0, 25.0), -1),
                (MoveRight, (80.0, 25.0), 1),
                (Drop, (0.0, -25.0), 2),
            ],
        ),
        (260.0, 80.0, "Rotate Brick", &[(TurnRight, (0.0, 0.0), 0)]),
        (
            // Nudge's rect is placed by its middle (pivot 0.5), 425 under the top.
            425.0 - 94.9 / 2.0,
            94.9,
            "Nudge Brick",
            &[
                (NudgeLeft, (-80.0, -10.0), -1),
                (NudgeRight, (80.0, -10.0), 1),
            ],
        ),
        (
            535.0,
            100.0,
            "Light Magic",
            &[(LightMagic, (0.0, -10.0), 0)],
        ),
        (704.0, 100.0, "Dark Magic", &[(DarkMagic, (0.0, -10.0), 0)]),
    ]
};

/// A key's name on its cap as the original writes it (Unity's KeyCode names).
fn cap_name(name: &str) -> &str {
    match name {
        "Left" => "LeftArrow",
        "Right" => "RightArrow",
        "Up" => "UpArrow",
        "Down" => "DownArrow",
        name => name,
    }
}

/// The keyboard controls panel centred at `centre` (px from the top-left), or at
/// the pause screen's place.
pub(crate) fn spawn_controls_panel(
    root: &mut ChildSpawnerCommands,
    art: Option<&Art>,
    keys: &crate::keybinding::KeyBindings,
    centre: Option<(f32, f32)>,
) -> Entity {
    use crate::popup::px;
    let image = |name: &str| art.and_then(|a| a.image(name));
    let ((w, h), at) = CONTROLS_PANEL;
    let (cx, cy) = centre.unwrap_or(at);
    let mut panel = root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(cx - w / 2.0),
            top: px(cy - h / 2.0),
            width: px(w),
            height: px(h),
            ..default()
        },
        BackgroundColor(Color::srgba(0.85, 0.87, 1.0, 0.95)),
    ));
    if let Some(art) = image("menu_leaderboard_panel") {
        panel.insert((
            crate::select::sliced(art, [0.0, 0.0, 37.0, 51.0]),
            BackgroundColor(Color::NONE),
        ));
    }
    panel.with_children(|p| {
        // The title's middle stands 7 px over the panel's top (its rect has no height).
        p.spawn(Node {
            position_type: PositionType::Absolute,
            top: px(-7.0 - 30.0),
            width: Val::Percent(100.0),
            height: px(60.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|t| {
            t.spawn(crate::select::label_with(
                46.0 / 10.8,
                "Controls",
                crate::letters::TextLook::GOLD,
            ));
        });
        for (top, height, title, caps) in CONTROLS_SECTIONS {
            let mut section = p.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(CONTROLS_INSET),
                    width: px(w - 2.0 * CONTROLS_INSET),
                    top: px(top),
                    height: px(height),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.75, 0.78, 0.95, 0.9)),
            ));
            if let Some(art) = image("popup_text_background") {
                section.insert((
                    crate::select::sliced(art, [2.0, 1.0, 0.0, 0.0]),
                    BackgroundColor(Color::NONE),
                ));
            }
            section.with_children(|s| {
                // The magic sections carry their spell slot behind the title.
                let slot = match title {
                    "Light Magic" => image("spell_slot_light"),
                    "Dark Magic" => image("spell_slot_dark"),
                    _ => None,
                };
                if let Some(art) = slot {
                    s.spawn((
                        ImageNode::new(art),
                        Node {
                            position_type: PositionType::Absolute,
                            left: px((w - 2.0 * CONTROLS_INSET) / 2.0 - 35.0),
                            top: px(-45.0),
                            width: px(70.0),
                            height: px(70.0),
                            ..default()
                        },
                    ));
                }
                let magic = title.ends_with("Magic");
                let up = if magic { 14.0 } else { 10.0 };
                s.spawn(Node {
                    position_type: PositionType::Absolute,
                    top: px(-up - 29.5),
                    width: Val::Percent(100.0),
                    height: px(59.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|l| {
                    let look = if magic {
                        crate::letters::TextLook::GOLD
                    } else {
                        crate::letters::TextLook {
                            face: Some(Color::srgb(0.918, 0.933, 1.0)),
                            ..crate::letters::TextLook::TITLE
                        }
                    };
                    l.spawn(crate::select::label_with(26.0 / 10.8, title, look));
                });
                let half = (w - 2.0 * CONTROLS_INSET) / 2.0;
                for &(action, (dx, dy), arrow) in caps {
                    let name = cap_name(crate::keybinding::key_name(keys.key(action)));
                    s.spawn(Node {
                        position_type: PositionType::Absolute,
                        left: px(half + dx - 50.0),
                        top: px(height / 2.0 - dy - 12.5),
                        width: px(100.0),
                        height: px(25.0),
                        ..default()
                    })
                    .with_children(|g| {
                        let mut back = g.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(-10.0),
                                top: px(-9.0),
                                width: px(120.0),
                                height: px(43.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.95, 0.95, 0.95, 0.9)),
                        ));
                        if let Some(art) = image("menu_kbkey_background") {
                            back.insert((
                                crate::select::sliced(art, [21.0, 21.0, 0.0, 0.0]),
                                BackgroundColor(Color::NONE),
                            ));
                        }
                        g.spawn(Node {
                            position_type: PositionType::Absolute,
                            top: px(12.5 - 7.0 - 8.0),
                            width: Val::Percent(100.0),
                            height: px(16.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_child((
                            Text::new(name),
                            TextFont {
                                font_size: FontSize::Vh(16.0 / 10.8),
                                ..default()
                            },
                            TextColor(Color::srgb(0.42, 0.33, 0.55)),
                            crate::letters::TextLook {
                                body: false,
                                face: None,
                                outline: None,
                                low: None,
                                outline_edge: 0.0,
                            },
                        ));
                        // Nudge has two arrows a side, the others one.
                        let nudge = title == "Nudge Brick";
                        let arrows: &[(f32, f32, f32)] = match (arrow, nudge) {
                            (-1, false) => &[(-29.0, 1.5, 0.0)],
                            (-1, true) => &[(-28.3, 1.5, 0.0), (-37.3, 1.5, 0.0)],
                            (1, false) => &[(116.0, 1.5, 180.0)],
                            (1, true) => &[(115.7, 1.5, 180.0), (124.7, 1.5, 180.0)],
                            (2, _) => &[(32.5 + 4.5, 43.0 - 4.5, -90.0)],
                            _ => &[],
                        };
                        if let Some(art) = image("menu_explanation_popup_label_arrow") {
                            for &(left, top, turn) in arrows {
                                g.spawn((
                                    ImageNode::new(art.clone()),
                                    UiTransform::from_rotation(Rot2::degrees(turn)),
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: px(left),
                                        top: px(top),
                                        width: px(13.0),
                                        height: px(22.0),
                                        ..default()
                                    },
                                ));
                            }
                        }
                    });
                }
            });
        }
    });
    panel.id()
}

/// "GAME PAUSED" and its options centred in the right half, over a darkened
/// match (the original's PausePopup).
fn spawn_pause_menu(
    mut commands: Commands,
    art: Option<Res<Art>>,
    keys: Option<Res<crate::keybinding::KeyBindings>>,
) {
    use crate::popup::{BAR_SLICE, px, sliced};
    let bar = art.as_ref().and_then(|a| a.image("menu_item_selected"));
    let text = |size: f32, value: &str, look: crate::letters::TextLook| {
        (
            Text::new(value.to_uppercase()),
            TextFont {
                font_size: FontSize::Vh(size / 10.8),
                ..default()
            },
            TextColor(Color::WHITE),
            TextLayout::justify(Justify::Center),
            look,
        )
    };
    // Heights over the centre become tops in a 1080-high screen.
    let top = |y: f32, h: f32| px(540.0 - y - h / 2.0);
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            // PausePopup's Panel, (8, 5, 24) at 0.847 over the grey; Unity blends that in
            // gamma, Bevy in linear light, where 0.95 gives about the same darkness.
            BackgroundColor(Color::srgba(8.0 / 255.0, 5.0 / 255.0, 24.0 / 255.0, 0.95)),
            GlobalZIndex(12),
            Visibility::Hidden,
            PauseRoot,
        ))
        .with_children(|root| {
            let defaults = crate::keybinding::KeyBindings::default();
            spawn_controls_panel(
                root,
                art.as_deref(),
                keys.as_deref().unwrap_or(&defaults),
                None,
            );
            root.spawn((Node {
                position_type: PositionType::Absolute,
                left: px(PAUSE_COLUMN.0),
                width: px(PAUSE_COLUMN.1),
                top: top(PAUSE_TITLE_Y, 100.0),
                height: px(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },))
                .with_children(|t| {
                    // EnzoBlackBlueBlackOutline: a dark face with a pale blue outline.
                    t.spawn(text(
                        75.0,
                        "Game Paused",
                        crate::letters::TextLook {
                            body: false,
                            face: Some(Color::srgb(0.145, 0.051, 0.157)),
                            outline: Some(Color::srgb(0.714, 0.773, 0.996)),
                            low: None,
                            // Dilate and outline 0.15.
                            outline_edge: 0.5 - 0.3 * 0.8333 / 2.0,
                        },
                    ));
                });
            for (i, option) in PAUSE_OPTIONS.iter().enumerate() {
                let y = PAUSE_ITEMS_Y - PAUSE_ROW.0 * i as f32;
                let mut row = root.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(PAUSE_COLUMN.0 + PAUSE_COLUMN.1 / 2.0 - 120.0),
                        width: px(240.0),
                        top: top(y, PAUSE_BAR),
                        height: px(PAUSE_BAR),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    PauseBar(i),
                ));
                if let Some(bar) = &bar {
                    let mut node = sliced(bar.clone(), BAR_SLICE);
                    node.color = Color::NONE;
                    row.insert(node);
                }
                row.with_children(|r| {
                    r.spawn((
                        text(PAUSE_ROW.1, option, crate::letters::TextLook::OUTLINED),
                        PauseWords(i),
                    ));
                });
            }
        });
}

/// Up/Down pick an option, Enter or A takes it. Resume goes back to the
/// match, Retry starts it again, Quit leaves for the title tiles.
fn pause_input(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    gamepads: Query<&Gamepad>,
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut game: ResMut<Match>,
    mut screen: ResMut<MenuScreen>,
    mut pause: ResMut<PauseMenu>,
    mut request: ResMut<MatchRequest>,
    cup: Option<Res<crate::cups::Cup>>,
    mut last_cursor: Local<Option<Vec2>>,
) {
    // Start on any pad pauses, as Esc does.
    if matches!(game.phase, Phase::Playing | Phase::Intro)
        && gamepads
            .iter()
            .any(|pad| pad.just_pressed(GamepadButton::Start))
    {
        pause.resume_to = Some(game.phase);
        pause.selected = 0;
        game.phase = Phase::Menu;
        *screen = MenuScreen::Pause;
        return;
    }
    if game.phase != Phase::Menu || *screen != MenuScreen::Pause {
        return;
    }
    let options = pause_options(cup.as_deref());
    let count = options.len();
    let (mut up, mut down, mut pick) = (false, false, false);
    if let Some(keys) = keys {
        // DEBUG-TOOL: not in the original; remove before the final build (docs/release-checklist.md).
        // WASD and the numpad arrows: scripted presses on Windows and habit (see menu.rs).
        up |= keys.any_just_pressed([KeyCode::ArrowUp, KeyCode::KeyW, KeyCode::Numpad8]);
        down |= keys.any_just_pressed([KeyCode::ArrowDown, KeyCode::KeyS, KeyCode::Numpad2]);
        pick |= keys.any_just_pressed([KeyCode::Enter, KeyCode::Space, KeyCode::NumpadEnter]);
    }
    // The mouse: an option under the cursor is selected as it moves, and a click picks it.
    if let Ok(window) = windows.single() {
        let cursor = window.cursor_position();
        let over = cursor
            .and_then(|c| pause_row_at(c, window.size()))
            .filter(|&i| i < count);
        if let Some(i) = over
            && cursor != *last_cursor
        {
            pause.selected = i;
        }
        *last_cursor = cursor;
        if let Some(i) = over
            && mouse.as_deref().is_some_and(|m| m.just_pressed(MouseButton::Left))
        {
            pause.selected = i;
            pick = true;
        }
    }
    for pad in &gamepads {
        up |= pad.just_pressed(GamepadButton::DPadUp);
        down |= pad.just_pressed(GamepadButton::DPadDown);
        pick |= pad.just_pressed(GamepadButton::South);
        // Start resumes, as Esc does.
        if pad.just_pressed(GamepadButton::Start) {
            pause.selected = 0;
            pick = true;
        }
    }
    if up {
        pause.selected = (pause.selected + count - 1) % count;
    }
    if down {
        pause.selected = (pause.selected + 1) % count;
    }
    if !pick {
        return;
    }
    match options[pause.selected.min(count - 1)] {
        "Resume" => {
            if let Some(phase) = pause.resume_to.take() {
                game.phase = phase;
            }
        }
        "Retry" => {
            pause.resume_to = None;
            request.pending = true;
            request.retry = true;
        }
        _ => {
            pause.resume_to = None;
            *screen = MenuScreen::Main;
        }
    }
}

#[allow(clippy::type_complexity)]
fn show_pause_menu(
    game: Res<Match>,
    screen: Res<MenuScreen>,
    pause: Res<PauseMenu>,
    mut root: Query<&mut Visibility, With<PauseRoot>>,
    mut bars: Query<(&PauseBar, Option<&mut ImageNode>, &mut BackgroundColor)>,
    mut words: Query<(
        &PauseWords,
        &mut TextColor,
        &mut Text,
        &mut crate::letters::TextLook,
        &mut TextFont,
        &mut UiTransform,
    )>,
    mut rows: Query<(&PauseBar, &mut Visibility), Without<PauseRoot>>,
    cup: Option<Res<crate::cups::Cup>>,
    grey: Option<ResMut<crate::post::GreyTarget>>,
    mut was_shown: Local<bool>,
) {
    let options = pause_options(cup.as_deref());
    for (bar, mut v) in &mut rows {
        v.set_if_neq(if bar.0 < options.len() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    for (item, _, mut text, mut look, mut font, mut lift) in &mut words {
        // PopupListItem: the picked option in EnzoBlackGold46 at 46 px on its bar, the
        // others in EnzoBlackRed46 at 36 px, 4 px up.
        let picked = item.0 == pause.selected;
        let (want_look, size, up) = if picked {
            (crate::letters::TextLook::GOLD, 46.0, 0.0)
        } else {
            (crate::letters::TextLook::PINK, 36.0, 4.0)
        };
        look.set_if_neq(want_look);
        let want_size = FontSize::Vh(size / 10.8);
        if font.font_size != want_size {
            font.font_size = want_size;
        }
        let want_lift = Val2::new(Val::Px(0.0), Val::Vh(-up / 10.8));
        if lift.translation != want_lift {
            lift.translation = want_lift;
        }
        if let Some(want) = options.get(item.0).map(|o| o.to_uppercase())
            && text.0 != want
        {
            text.0 = want;
        }
    }
    for (item, mut color, ..) in &mut words {
        let want = if item.0 == pause.selected {
            PAUSE_PICKED
        } else {
            PAUSE_OTHER
        };
        if color.0 != want {
            color.0 = want;
        }
    }
    let shown = game.phase == Phase::Menu && *screen == MenuScreen::Pause;
    // Behind the pause the world turns grey (PausePopup's BlackWhite background); the cup
    // standings set it too, so only a change of the pause writes it.
    if let Some(mut grey) = grey
        && shown != *was_shown
    {
        grey.0 = if shown { 1.0 } else { 0.0 };
    }
    *was_shown = shown;
    for mut v in &mut root {
        v.set_if_neq(if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    for (bar, image, mut colour) in &mut bars {
        let picked = bar.0 == pause.selected;
        match image {
            Some(mut image) => {
                let want = if picked { Color::WHITE } else { Color::NONE };
                if image.color != want {
                    image.color = want;
                }
            }
            None => {
                let want = if picked {
                    Color::srgba(1.0, 0.75, 0.2, 0.55)
                } else {
                    Color::NONE
                };
                if colour.0 != want {
                    colour.0 = want;
                }
            }
        }
    }
}

/// The highlighted title tile.
#[derive(Resource, Default)]
pub struct MainMenu {
    pub selected: usize,
}

/// One title tile: its name, label, size and centre in pixels of a
/// 1920×1080 screen measured from the middle with y up, and the colour it
/// falls back to when its art isn't staged.
pub struct Tile {
    pub key: &'static str,
    pub label: &'static str,
    pub size: (f32, f32),
    pub at: (f32, f32),
    pub color: (f32, f32, f32),
    /// Where the art over the tile sits: its centre's offset from the tile's
    /// centre and its size, when it isn't the tile's own size.
    pub front: Option<((f32, f32), (f32, f32))>,
    /// The picked tile's header bar (TopBar): its left edge from the tile's, how far its
    /// top stands over the tile's top, its width and height.
    pub bar: (f32, f32, f32, f32),
    /// The picked tile's corner frame (SelectedFrame): its centre's offset from the tile's
    /// and its size; the corners sit in its corners.
    pub frame: ((f32, f32), (f32, f32)),
}

/// The original's title tiles in its order, with its sizes, centres and front
/// art placement (docs/research/menus.md).
pub const TILES: [Tile; 7] = [
    Tile {
        key: "local",
        label: "Local Battle",
        size: (437.0, 335.0),
        at: (-249.0, 225.5),
        color: (0.85, 0.35, 0.2),
        front: Some(((-5.0, -25.0), (538.0, 272.0))),
        bar: (-12.5, 10.4, 456.0, 45.0),
        frame: ((1.7, 4.0), (507.69, 395.68)),
    },
    Tile {
        key: "single",
        label: "Single Player",
        size: (435.0, 335.0),
        at: (-246.0, -155.5),
        color: (0.25, 0.6, 0.3),
        front: Some(((17.0, -42.0), (488.0, 235.0))),
        bar: (-8.0, 15.3, 449.0, 45.0),
        frame: ((1.0, 4.0), (502.0, 400.0)),
    },
    Tile {
        key: "online",
        label: "Online Battle",
        size: (441.0, 335.0),
        at: (278.0, 232.5),
        color: (0.25, 0.45, 0.85),
        front: Some(((-21.0, -19.0), (467.0, 283.0))),
        bar: (-8.0, 17.0, 455.0, 45.0),
        frame: ((1.0, 5.0), (502.3, 400.92)),
    },
    Tile {
        key: "shop",
        label: "Shop",
        size: (450.0, 165.0),
        at: (277.0, -62.5),
        color: (0.85, 0.65, 0.15),
        front: Some(((-20.0, -9.0), (483.0, 128.0))),
        bar: (-8.0, 6.0, 464.0, 32.0),
        frame: ((0.0, 1.8), (508.0, 209.7)),
    },
    Tile {
        key: "help",
        label: "Help",
        size: (197.0, 150.0),
        at: (146.0, -250.0),
        color: (0.55, 0.35, 0.75),
        front: None,
        bar: (-10.0, 5.5, 210.0, 26.0),
        frame: ((-0.5, 2.39), (254.92, 195.71)),
    },
    Tile {
        key: "settings",
        label: "Settings",
        size: (197.0, 150.0),
        at: (396.0, -251.0),
        color: (0.4, 0.45, 0.5),
        front: Some(((0.0, 0.0), (181.0, 134.0))),
        bar: (-10.0, 5.5, 210.0, 26.0),
        frame: ((-1.6, 3.39), (258.6, 195.71)),
    },
    Tile {
        key: "quit",
        label: "Exit Game",
        size: (436.0, 68.0),
        at: (0.0, -406.0),
        color: (0.7, 0.2, 0.25),
        front: None,
        bar: (-10.0, 41.1, 456.0, 26.0),
        frame: ((0.0, 5.0), (456.0, 78.0)),
    },
];

const SCREEN: (f32, f32) = (1920.0, 1080.0);
/// How far a tile's name sits over its top edge (Max's title clip).
const LABEL_UP: f32 = 10.0;
/// The rotating sun-ray layers behind a selected tile's art.
const RAY_LAYERS: usize = 5;
/// The rays' square in the MainMenu scene, cut to its tile by a mask.
const RAY_SIZE: f32 = 556.0;
/// Without the turning material the rays are cut down to this share of the tile's shorter side.
const RAY_FRACTION: f32 = 0.75;

#[derive(Component)]
struct MainMenuRoot;

/// A tile's parts, by tile index.
#[derive(Component)]
struct TileBack(usize);
#[derive(Component)]
struct TileChosen {
    /// Shown when the tile is picked, else when it isn't.
    picked: bool,
}
#[derive(Component)]
struct TileLabel;
/// A corner of the picked tile's frame, and which way it pulses (UI y is down).
#[derive(Component)]
struct TileCorner(Vec2);
#[derive(Component)]
struct TileRays {
    layer: usize,
}

fn percent_node(size: (f32, f32), at: (f32, f32)) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Percent(50.0 + (at.0 - size.0 / 2.0) / SCREEN.0 * 100.0),
        top: Val::Percent(50.0 - (at.1 + size.1 / 2.0) / SCREEN.1 * 100.0),
        width: Val::Percent(size.0 / SCREEN.0 * 100.0),
        height: Val::Percent(size.1 / SCREEN.1 * 100.0),
        ..default()
    }
}

/// A child filling its tile.
fn fill() -> Node {
    Node {
        position_type: PositionType::Absolute,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..default()
    }
}

/// The title screen. Each tile shows `menu_tile_<key>_deselected` with its grey
/// `menu_tile_<key>_front_deselected` art until it is picked; then
/// `menu_tile_<key>_selected` with the turning `menu_sunrays_1..5`, its
/// `menu_tile_<key>_front` art (bigger than the tile, poking out), the
/// `menu_tile_<key>_overlay` frame, the `menu_tile_header` bar with the label
/// and `menu_corner_large|small` pulsing in its corners. Missing art falls back
/// to plain colours.
fn spawn_main_menu(
    mut commands: Commands,
    art: Option<Res<Art>>,
    mut turning: Option<ResMut<Assets<crate::ui_material::TurningPicture>>>,
) {
    let image = |name: &str| art.as_ref().and_then(|a| a.image(name));
    let mut root = commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        // The live Race scene shows through (visuals::MenuScene).
        GlobalZIndex(11),
        Visibility::Hidden,
        MainMenuRoot,
    ));
    root.with_children(|parent| {
        for (i, tile) in TILES.iter().enumerate() {
            let (r, g, b) = tile.color;
            let small = tile.size.1 < 200.0;
            let (w, h) = tile.size;
            // A child `size` pixels big whose centre sits `offset` from the tile's.
            let inside = |size: (f32, f32), offset: (f32, f32)| Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(((w - size.0) / 2.0 + offset.0) / w * 100.0),
                top: Val::Percent(((h - size.1) / 2.0 - offset.1) / h * 100.0),
                width: Val::Percent(size.0 / w * 100.0),
                height: Val::Percent(size.1 / h * 100.0),
                ..default()
            };
            let shown_when = |picked: bool| {
                let visibility = if picked {
                    Visibility::Hidden
                } else {
                    Visibility::Inherited
                };
                (TileChosen { picked }, visibility)
            };
            let key = tile.key;
            parent
                .spawn((percent_node(tile.size, tile.at), TileBack(i)))
                .with_children(|t| {
                    // Grey when not picked.
                    let mut back = t.spawn((fill(), shown_when(false)));
                    match image(&format!("menu_tile_{key}_deselected")) {
                        Some(art) => back.insert(ImageNode::new(art)),
                        None => back.insert(BackgroundColor(Color::srgb(
                            0.35 + r * 0.15,
                            0.35 + g * 0.15,
                            0.35 + b * 0.15,
                        ))),
                    };
                    // Coloured, with the rays turning inside the tile.
                    let colour = Node {
                        overflow: Overflow::clip(),
                        ..fill()
                    };
                    let mut on = t.spawn((colour, shown_when(true)));
                    match image(&format!("menu_tile_{key}_selected")) {
                        Some(art) => on.insert(ImageNode::new(art)),
                        None => on.insert(BackgroundColor(Color::srgb(r, g, b))),
                    };
                    // The rays (556 px squares) turn inside the tile, cut to it as the
                    // original's mask does: the turning material when there is one, else a
                    // square cut down to fit (a turned picture isn't clipped).
                    on.with_children(|on| {
                        for layer in 0..RAY_LAYERS {
                            let Some(art) = image(&format!("menu_sunrays_{}", layer + 1)) else {
                                continue;
                            };
                            match turning.as_mut() {
                                Some(turning) => {
                                    let share = Vec2::new(RAY_SIZE / w, RAY_SIZE / h);
                                    on.spawn((
                                        MaterialNode(turning.add(
                                            crate::ui_material::TurningPicture::new(art, share),
                                        )),
                                        fill(),
                                        TileRays { layer },
                                    ));
                                }
                                None => {
                                    let ray = w.min(h) * RAY_FRACTION;
                                    on.spawn((
                                        ImageNode::new(art),
                                        inside((ray, ray), (0.0, 0.0)),
                                        UiTransform::default(),
                                        TileRays { layer },
                                    ));
                                }
                            }
                        }
                    });
                    // The art over the tile, grey or in colour, and the frame.
                    let front = || match tile.front {
                        Some((offset, size)) => inside(size, offset),
                        None => fill(),
                    };
                    // The picked tile's Foreground frame comes before its wizards, which
                    // stand over it and past the tile's sides (MainMenu scene order).
                    if let Some(art) = image(&format!("menu_tile_{key}_overlay")) {
                        t.spawn((ImageNode::new(art), fill(), shown_when(true)));
                    }
                    for (part, picked) in [("_front_deselected", false), ("_front", true)] {
                        if let Some(art) = image(&format!("menu_tile_{key}{part}")) {
                            t.spawn((ImageNode::new(art), front(), shown_when(picked)));
                        }
                    }
                    let corner = if small {
                        "menu_corner_small"
                    } else {
                        "menu_corner_large"
                    };
                    if let Some(art) = image(corner) {
                        let (cw, ch) = if small { (38.0, 34.0) } else { (69.0, 68.0) };
                        let (offset, size) = tile.frame;
                        t.spawn((inside(size, offset), shown_when(true)))
                            .with_children(|corners| {
                                for (flip_x, flip_y) in
                                    [(false, false), (true, false), (false, true), (true, true)]
                                {
                                    let mut node = Node {
                                        position_type: PositionType::Absolute,
                                        width: Val::Percent(cw / size.0 * 100.0),
                                        height: Val::Percent(ch / size.1 * 100.0),
                                        ..default()
                                    };
                                    if flip_x {
                                        node.right = Val::Px(0.0);
                                    }
                                    if flip_y {
                                        node.bottom = Val::Px(0.0);
                                    }
                                    let side = |flip: bool| if flip { 1.0 } else { -1.0 };
                                    corners.spawn((
                                        ImageNode {
                                            image: art.clone(),
                                            flip_x,
                                            flip_y,
                                            ..default()
                                        },
                                        node,
                                        UiTransform::default(),
                                        TileCorner(Vec2::new(side(flip_x), side(flip_y))),
                                    ));
                                }
                            });
                    }
                    // The Help tile's big question mark.
                    if tile.key == "help" {
                        t.spawn(Node {
                            position_type: PositionType::Absolute,
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|q| {
                            // The original's "?" is dark with a white outline, 130.5 px.
                            q.spawn(crate::select::label_with(
                                130.5 / SCREEN.1 * 100.0,
                                "?",
                                crate::letters::TextLook {
                                    body: false,
                                    face: Some(Color::srgb(0.07, 0.055, 0.12)),
                                    outline: Some(Color::WHITE),
                                    low: None,
                                    outline_edge: 0.0,
                                },
                            ));
                        });
                    }
                    // The header bar on the picked tile, big or small.
                    if let Some(art) = image("menu_tile_header") {
                        let (left, up, bw, bh) = tile.bar;
                        t.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Percent(left / w * 100.0),
                                top: Val::Percent(-up / h * 100.0),
                                width: Val::Percent(bw / w * 100.0),
                                height: Val::Percent(bh / h * 100.0),
                                ..default()
                            },
                            ImageNode::new(art),
                            TileLabel,
                            Visibility::Hidden,
                        ));
                    }
                    // Every tile's name, always shown as in the original (MainMenu scene):
                    // centred 10 px over the tile's top edge, in outlined capitals, 46 px
                    // on the big tiles and the shop, 36 on the others.
                    // The picked tile's name is gold (EnzoBlackGold46), the others milky white
                    // (EnzoBlackMilkyWhiteOutline46); each sits in its own box, since letter
                    // text hides with its parent.
                    let label_box = 60.0;
                    let px = if small && tile.key != "shop" { 36.0 } else { 46.0 };
                    for picked in [false, true] {
                        let look = if picked {
                            crate::letters::TextLook::GOLD
                        } else {
                            crate::letters::TextLook::TITLE
                        };
                        t.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                width: Val::Percent(100.0),
                                top: Val::Percent(-(LABEL_UP + label_box / 2.0) / h * 100.0),
                                height: Val::Percent(label_box / h * 100.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            shown_when(picked),
                        ))
                        .with_child((
                            Text::new(tile.label.to_uppercase()),
                            TextFont {
                                font_size: FontSize::Vh(px / SCREEN.1 * 100.0),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            TextLayout::justify(Justify::Center),
                            look,
                        ));
                    }
                });
        }
    });
}

/// The title tile under a cursor at `cursor` (top-left origin, in a window `window` big).
fn tile_at(cursor: Vec2, window: Vec2) -> Option<usize> {
    let p = Vec2::new(cursor.x / window.x * 1920.0, cursor.y / window.y * 1080.0);
    TILES.iter().position(|t| {
        let centre = Vec2::new(960.0 + t.at.0, 540.0 - t.at.1);
        let half = Vec2::new(t.size.0, t.size.1) * 0.5;
        ((p - centre).abs().cmple(half)).all()
    })
}

/// The pause option under a cursor (top-left origin) in a window `window` big.
fn pause_row_at(cursor: Vec2, window: Vec2) -> Option<usize> {
    let p = Vec2::new(
        cursor.x / window.x * 1920.0 - 960.0,
        540.0 - cursor.y / window.y * 1080.0,
    );
    if (p.x - 480.0).abs() > 120.0 {
        return None;
    }
    (0..PAUSE_OPTIONS.len()).find(|&i| {
        (p.y - (PAUSE_ITEMS_Y - PAUSE_ROW.0 * i as f32)).abs() <= PAUSE_BAR / 2.0
    })
}

/// The tile from `from` in direction `dir`: the nearest one ahead, with
/// sideways distance counting double. Tiles more sideways than ahead (twice
/// over) don't count, so Up from Local Battle stays put.
fn tile_towards(from: usize, dir: Vec2) -> usize {
    let at = |i: usize| Vec2::new(TILES[i].at.0, TILES[i].at.1);
    let here = at(from);
    (0..TILES.len())
        .filter(|&i| {
            let d = at(i) - here;
            let ahead = d.dot(dir);
            i != from && ahead > 1.0 && ahead * 2.0 > (d - dir * ahead).length()
        })
        .min_by(|&a, &b| {
            let cost = |i: usize| {
                let d = at(i) - here;
                let ahead = d.dot(dir);
                ahead + (d - dir * ahead).length() * 2.0
            };
            cost(a).total_cmp(&cost(b))
        })
        .unwrap_or(from)
}

/// Arrows or a d-pad move between the tiles; Enter, Space or A picks one.
/// Local Battle opens player select, Single Player the mode select with
/// Endless Race, and Quit closes the game. The other tiles aren't built yet.
#[allow(clippy::too_many_arguments)]
fn main_menu_input(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    gamepads: Query<&Gamepad>,
    game: Res<Match>,
    mut screen: ResMut<MenuScreen>,
    mut menu: ResMut<MainMenu>,
    mut lobby: ResMut<Lobby>,
    mut exit: MessageWriter<AppExit>,
    mut last_cursor: Local<Option<Vec2>>,
) {
    if game.phase != Phase::Menu || *screen != MenuScreen::Main {
        return;
    }
    let mut dir = Vec2::ZERO;
    let mut pick = false;
    // The mouse: the tile under the cursor is highlighted when it moves, and a click picks it.
    if let Ok(window) = windows.single() {
        let cursor = window.cursor_position();
        let over = cursor.and_then(|c| tile_at(c, window.size()));
        if over.is_some() && cursor != *last_cursor {
            menu.selected = over.unwrap_or(menu.selected);
        }
        *last_cursor = cursor;
        if let Some(i) = over
            && mouse.is_some_and(|m| m.just_pressed(MouseButton::Left))
        {
            menu.selected = i;
            pick = true;
        }
    }
    if let Some(keys) = keys {
        let pressed = |list: [KeyCode; 3]| keys.any_just_pressed(list);
        // DEBUG-TOOL: not in the original; remove before the final build (docs/release-checklist.md).
        // WASD and the numpad arrows (scripted key presses on Windows) also work in
        // menus, here and in select.rs, popup.rs and the pause menu.
        if pressed([KeyCode::ArrowUp, KeyCode::KeyW, KeyCode::Numpad8]) {
            dir = Vec2::Y;
        }
        if pressed([KeyCode::ArrowDown, KeyCode::KeyS, KeyCode::Numpad2]) {
            dir = Vec2::NEG_Y;
        }
        if pressed([KeyCode::ArrowLeft, KeyCode::KeyA, KeyCode::Numpad4]) {
            dir = Vec2::NEG_X;
        }
        if pressed([KeyCode::ArrowRight, KeyCode::KeyD, KeyCode::Numpad6]) {
            dir = Vec2::X;
        }
        pick |= keys.any_just_pressed([KeyCode::Enter, KeyCode::Space, KeyCode::NumpadEnter]);
    }
    for pad in &gamepads {
        let buttons = [
            (GamepadButton::DPadUp, Vec2::Y),
            (GamepadButton::DPadDown, Vec2::NEG_Y),
            (GamepadButton::DPadLeft, Vec2::NEG_X),
            (GamepadButton::DPadRight, Vec2::X),
        ];
        for (button, d) in buttons {
            if pad.just_pressed(button) {
                dir = d;
            }
        }
        pick |= pad.any_just_pressed([GamepadButton::South, GamepadButton::Start]);
    }
    if dir != Vec2::ZERO {
        menu.selected = tile_towards(menu.selected, dir);
    }
    if !pick {
        return;
    }
    match TILES[menu.selected].key {
        "local" => {
            lobby.single = false;
            *screen = MenuScreen::Players;
        }
        "single" => {
            *screen = MenuScreen::Single;
        }
        "settings" => {
            *screen = MenuScreen::Settings;
        }
        "quit" => {
            exit.write(AppExit::Success);
        }
        _ => {}
    }
}

#[allow(clippy::type_complexity)]
fn show_main_menu(
    game: Res<Match>,
    screen: Res<MenuScreen>,
    menu: Res<MainMenu>,
    mut root: Query<&mut Visibility, With<MainMenuRoot>>,
    mut parts: Query<
        (&mut Visibility, Option<&TileChosen>, Option<&TileLabel>),
        (
            Without<MainMenuRoot>,
            Or<(With<TileChosen>, With<TileLabel>)>,
        ),
    >,
    tiles: Query<(&TileBack, &Children)>,
) {
    let shown = game.phase == Phase::Menu && *screen == MenuScreen::Main;
    for mut v in &mut root {
        v.set_if_neq(if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    if !shown {
        return;
    }
    for (back, children) in &tiles {
        let on = back.0 == menu.selected;
        for &child in children {
            let Ok((mut v, chosen, label)) = parts.get_mut(child) else {
                continue;
            };
            let want = match (chosen, label) {
                (Some(chosen), _) => chosen.picked == on,
                (None, Some(_)) => on,
                (None, None) => continue,
            };
            v.set_if_neq(if want {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            });
        }
    }
}

/// The picked tile's first ray layers turn and its corners pulse outward and
/// back, as `GameConfig::menu` sets.
fn spin_tile_rays(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut rays: Query<
        (
            &TileRays,
            Option<&mut UiTransform>,
            Option<&MaterialNode<crate::ui_material::TurningPicture>>,
        ),
        Without<TileCorner>,
    >,
    mut corners: Query<(&TileCorner, &mut UiTransform), Without<TileRays>>,
    mut turning: Option<ResMut<Assets<crate::ui_material::TurningPicture>>>,
) {
    let t = time.elapsed_secs();
    let look = &config.menu;
    for (rays, transform, picture) in &mut rays {
        let Some(&(degrees, seconds)) = look.ray_turns.get(rays.layer) else {
            continue;
        };
        let loop_at = (t / seconds.max(0.01)).fract();
        let turn = Rot2::degrees(degrees * loop_at);
        match (picture, turning.as_mut(), transform) {
            (Some(picture), Some(turning), _) => {
                crate::ui_material::set_turn(turning, &picture.0, turn.as_radians())
            }
            (_, _, Some(mut transform)) => transform.rotation = turn,
            _ => {}
        }
    }
    let (reach, seconds) = look.corner_pulse;
    // Out at the start of each loop, back at rest halfway, out again: linear.
    let out = reach * (2.0 * (t / seconds.max(0.01)).fract() - 1.0).abs();
    // Pixels of a 1080-high screen, as a share of the view height.
    let vh = out / SCREEN.1 * 100.0;
    for (corner, mut transform) in &mut corners {
        transform.translation = Val2::new(Val::Vh(corner.0.x * vh), Val::Vh(corner.0.y * vh));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cursor_over_a_title_tile_picks_it() {
        let window = Vec2::new(1920.0, 1080.0);
        let centre = |i: usize| {
            let t = &TILES[i];
            Vec2::new(960.0 + t.at.0, 540.0 - t.at.1)
        };
        for i in 0..TILES.len() {
            assert_eq!(tile_at(centre(i), window), Some(i), "tile {i}");
        }
        assert_eq!(tile_at(Vec2::new(5.0, 5.0), window), None);
        // The same point in a half-size window is the same tile.
        assert_eq!(tile_at(centre(0) * 0.5, window * 0.5), Some(0));
    }
}
