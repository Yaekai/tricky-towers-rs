//! The menus: the original's title screen of tiles (Local Battle, Single
//! Player, ...) and behind it the mode menu, where you pick the mode, how many
//! people play and how many AI wizards join.

use bevy::prelude::*;

use crate::art::Art;
use crate::config::{Difficulty, GameConfig, ModeKind};
use crate::gameplay::{Match, MatchRequest, Phase};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(crate::popup::PopupPlugin)
            .init_resource::<MenuScreen>()
            .init_resource::<MainMenu>()
            .init_resource::<PauseMenu>()
            .add_systems(Startup, (spawn_menu, spawn_main_menu, spawn_pause_menu))
            .add_systems(
                Update,
                (
                    menu_keys,
                    menu_gamepads,
                    main_menu_input,
                    pause_input,
                    show_menu,
                    show_pause_menu,
                    show_main_menu,
                    spin_tile_rays,
                )
                    .chain()
                    .in_set(MenuSystems),
            );
    }
}

const MAX_TOWERS: u32 = crate::gameplay::MAX_TOWERS as u32;

#[derive(Component)]
struct MenuText;

/// The menu screen: the original's title art when staged (`menu_background`
/// filling the screen, `logo` above the options), else a dark panel.
fn spawn_menu(mut commands: Commands, art: Option<Res<Art>>) {
    let image = |name: &str| art.as_ref().and_then(|a| a.image(name));
    let mut root = commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(20.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.06, 0.12, 0.85)),
        GlobalZIndex(10),
        MenuText,
    ));
    if let Some(background) = image("menu_background") {
        root.insert((ImageNode::new(background), BackgroundColor(Color::NONE)));
    }
    root.with_children(|parent| {
        if let Some(logo) = image("logo") {
            parent.spawn((
                ImageNode::new(logo),
                Node {
                    max_width: Val::Percent(50.0),
                    max_height: Val::Percent(30.0),
                    ..default()
                },
            ));
        }
        parent.spawn((
            Text::new(""),
            TextFont {
                font_size: FontSize::Px(26.0),
                ..default()
            },
            TextLayout::justify(Justify::Center),
            MenuLabel,
        ));
    });
}

/// The menu's options text.
#[derive(Component)]
struct MenuLabel;

/// Steps Easy, Normal, Pro, then round again.
fn next_difficulty(config: &mut GameConfig) {
    let now = config.difficulty.unwrap_or_default();
    let at = Difficulty::ALL.iter().position(|&d| d == now).unwrap_or(1);
    config.difficulty = Some(Difficulty::ALL[(at + 1) % Difficulty::ALL.len()]);
}

fn menu_keys(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mut config: ResMut<GameConfig>,
    mut game: ResMut<Match>,
    mut request: ResMut<MatchRequest>,
    mut screen: ResMut<MenuScreen>,
    mut pause: ResMut<PauseMenu>,
) {
    let Some(keys) = keys else { return };
    if keys.just_pressed(KeyCode::Escape) {
        match (game.phase, *screen) {
            // Pause the match.
            (Phase::Playing, _) => {
                pause.resume_to = Some(Phase::Playing);
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
            // Leave a finished match, or step back from the mode menu.
            (Phase::Over, _) | (Phase::Menu, MenuScreen::Modes) => {
                pause.resume_to = None;
                game.phase = Phase::Menu;
                *screen = MenuScreen::Main;
            }
            (Phase::Menu, MenuScreen::Main) => {}
        }
        return;
    }
    if game.phase != Phase::Menu || *screen != MenuScreen::Modes {
        return;
    }
    let index = ModeKind::ALL
        .iter()
        .position(|&m| m == config.mode)
        .unwrap_or(0);
    let count = ModeKind::ALL.len();
    if keys.any_just_pressed([KeyCode::ArrowUp, KeyCode::KeyW, KeyCode::Numpad8]) {
        config.mode = ModeKind::ALL[(index + count - 1) % count];
    }
    if keys.any_just_pressed([KeyCode::ArrowDown, KeyCode::KeyS, KeyCode::Numpad2]) {
        config.mode = ModeKind::ALL[(index + 1) % count];
    }
    if keys.any_just_pressed([KeyCode::ArrowLeft, KeyCode::KeyA, KeyCode::Numpad4]) {
        config.opponents = config.opponents.saturating_sub(1);
    }
    if keys.any_just_pressed([KeyCode::ArrowRight, KeyCode::KeyD, KeyCode::Numpad6]) {
        config.opponents = (config.opponents + 1).min(MAX_TOWERS - config.players);
    }
    // Number keys pick how many people play (players 2 to 4 use gamepads).
    let digits = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
    ];
    if let Some(n) = digits.iter().position(|&k| keys.just_pressed(k)) {
        config.players = n as u32 + 1;
        config.opponents = config.opponents.min(MAX_TOWERS - config.players);
    }
    if keys.just_pressed(KeyCode::Tab) {
        next_difficulty(&mut config);
    }
    if keys.any_just_pressed([KeyCode::Enter, KeyCode::Space]) {
        request.pending = true;
    }
}

/// Any gamepad can pick the mode (d-pad) and start (A or Start).
fn menu_gamepads(
    mut config: ResMut<GameConfig>,
    game: Res<Match>,
    mut request: ResMut<MatchRequest>,
    screen: Res<MenuScreen>,
    gamepads: Query<&Gamepad>,
) {
    if game.phase != Phase::Menu || *screen != MenuScreen::Modes {
        return;
    }
    let count = ModeKind::ALL.len();
    for pad in &gamepads {
        let index = ModeKind::ALL
            .iter()
            .position(|&m| m == config.mode)
            .unwrap_or(0);
        if pad.just_pressed(GamepadButton::DPadUp) {
            config.mode = ModeKind::ALL[(index + count - 1) % count];
        }
        if pad.just_pressed(GamepadButton::DPadDown) {
            config.mode = ModeKind::ALL[(index + 1) % count];
        }
        if pad.just_pressed(GamepadButton::West) {
            next_difficulty(&mut config);
        }
        if pad.any_just_pressed([GamepadButton::South, GamepadButton::Start]) {
            request.pending = true;
        }
    }
}

fn show_menu(
    config: Res<GameConfig>,
    game: Res<Match>,
    screen: Res<MenuScreen>,
    mut menu: Query<(&mut Node, &Children), With<MenuText>>,
    mut texts: Query<&mut Text, With<MenuLabel>>,
) {
    let Ok((mut node, children)) = menu.single_mut() else {
        return;
    };
    let visible = game.phase == Phase::Menu && *screen == MenuScreen::Modes;
    node.display = if visible {
        Display::Flex
    } else {
        Display::None
    };
    if !visible {
        return;
    }
    let modes: Vec<String> = ModeKind::ALL
        .iter()
        .map(|&m| {
            let detail = match m {
                ModeKind::Race => {
                    format!("reach the line {:.0} bricks up", config.race.finish_height)
                }
                ModeKind::Survival => format!(
                    "place {} bricks, {} lives",
                    config.survival.bricks_to_place, config.survival.lives
                ),
                ModeKind::Puzzle => format!("fit {} bricks under the laser", config.puzzle.bricks),
                ModeKind::Endless => "stay above the rising line".into(),
            };
            if m == config.mode {
                format!("> {}  ({detail}) <", m.name())
            } else {
                format!("{}  ({detail})", m.name())
            }
        })
        .collect();
    let resume = "\nEsc: back";
    let text = format!(
        "TRICKY TOWERS\n\n{}\n\nDifficulty: {}\nPlayers: {}   AI wizards: < {} >\n\nUp/Down: mode   Tab: difficulty   1-4: players   Left/Right: AI wizards   Enter: play\nPlayer 1: keyboard   Players 2-4: gamepads\n\nIn game: Left/Right move   Shift+Left/Right nudge   Up/Z rotate   Down drop\nQ light spell   E dark spell   R restart   Esc menu{resume}",
        modes.join("\n"),
        config.difficulty.unwrap_or_default().name(),
        config.players,
        config.opponents
    );
    for &child in children {
        if let Ok(mut t) = texts.get_mut(child) {
            t.0 = text.clone();
        }
    }
}

/// The menu systems. Anything else that changes [`MenuScreen`] on a key
/// press runs after them, so the same press isn't read twice.
#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub struct MenuSystems;

/// Which menu screen is up while the game is in the menu.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuScreen {
    /// The title tiles.
    #[default]
    Main,
    /// Mode, players, AI wizards and difficulty.
    Modes,
    /// "Game Paused" over a match: Resume, Retry, Quit.
    Pause,
}

/// The pause menu's highlighted option and the phase Resume goes back to.
#[derive(Resource, Default)]
pub struct PauseMenu {
    pub selected: usize,
    pub resume_to: Option<Phase>,
}

/// The original's pause options, top to bottom.
pub const PAUSE_OPTIONS: [&str; 3] = ["Resume", "Retry", "Quit"];

#[derive(Component)]
struct PauseRoot;
#[derive(Component)]
struct PauseOption(usize);

/// "Game Paused" and its options over a dimmed match.
fn spawn_pause_menu(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Vh(2.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            GlobalZIndex(12),
            Visibility::Hidden,
            PauseRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("Game Paused"),
                TextFont {
                    font_size: FontSize::Vh(7.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                TextLayout::justify(Justify::Center),
            ));
            for (i, option) in PAUSE_OPTIONS.iter().enumerate() {
                root.spawn((
                    Text::new(*option),
                    TextFont {
                        font_size: FontSize::Vh(4.5),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    TextLayout::justify(Justify::Center),
                    PauseOption(i),
                ));
            }
        });
}

/// Up/Down pick an option, Enter or A takes it. Resume goes back to the
/// match, Retry starts it again, Quit leaves for the title tiles.
fn pause_input(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    gamepads: Query<&Gamepad>,
    mut game: ResMut<Match>,
    mut screen: ResMut<MenuScreen>,
    mut pause: ResMut<PauseMenu>,
    mut request: ResMut<MatchRequest>,
) {
    // Start on any pad pauses, as Esc does.
    if game.phase == Phase::Playing
        && gamepads
            .iter()
            .any(|pad| pad.just_pressed(GamepadButton::Start))
    {
        pause.resume_to = Some(Phase::Playing);
        pause.selected = 0;
        game.phase = Phase::Menu;
        *screen = MenuScreen::Pause;
        return;
    }
    if game.phase != Phase::Menu || *screen != MenuScreen::Pause {
        return;
    }
    let count = PAUSE_OPTIONS.len();
    let (mut up, mut down, mut pick) = (false, false, false);
    if let Some(keys) = keys {
        up |= keys.any_just_pressed([KeyCode::ArrowUp, KeyCode::KeyW, KeyCode::Numpad8]);
        down |= keys.any_just_pressed([KeyCode::ArrowDown, KeyCode::KeyS, KeyCode::Numpad2]);
        pick |= keys.any_just_pressed([KeyCode::Enter, KeyCode::Space, KeyCode::NumpadEnter]);
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
    match PAUSE_OPTIONS[pause.selected] {
        "Resume" => {
            if let Some(phase) = pause.resume_to.take() {
                game.phase = phase;
            }
        }
        "Retry" => {
            pause.resume_to = None;
            request.pending = true;
        }
        _ => {
            pause.resume_to = None;
            *screen = MenuScreen::Main;
        }
    }
}

fn show_pause_menu(
    game: Res<Match>,
    screen: Res<MenuScreen>,
    pause: Res<PauseMenu>,
    mut root: Query<&mut Visibility, With<PauseRoot>>,
    mut options: Query<(&PauseOption, &mut TextColor)>,
) {
    let shown = game.phase == Phase::Menu && *screen == MenuScreen::Pause;
    for mut v in &mut root {
        v.set_if_neq(if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    for (option, mut color) in &mut options {
        let want = if option.0 == pause.selected {
            Color::srgb(1.0, 0.82, 0.25)
        } else {
            Color::WHITE
        };
        if color.0 != want {
            color.0 = want;
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
    },
    Tile {
        key: "single",
        label: "Single Player",
        size: (435.0, 335.0),
        at: (-246.0, -155.5),
        color: (0.25, 0.6, 0.3),
        front: Some(((17.0, -42.0), (488.0, 235.0))),
    },
    Tile {
        key: "online",
        label: "Online Battle",
        size: (441.0, 335.0),
        at: (278.0, 232.5),
        color: (0.25, 0.45, 0.85),
        front: Some(((-21.0, -19.0), (467.0, 283.0))),
    },
    Tile {
        key: "shop",
        label: "Shop",
        size: (450.0, 165.0),
        at: (277.0, -62.5),
        color: (0.85, 0.65, 0.15),
        front: Some(((-20.0, -9.0), (483.0, 128.0))),
    },
    Tile {
        key: "help",
        label: "Help",
        size: (197.0, 150.0),
        at: (146.0, -250.0),
        color: (0.55, 0.35, 0.75),
        front: None,
    },
    Tile {
        key: "settings",
        label: "Settings",
        size: (197.0, 150.0),
        at: (396.0, -251.0),
        color: (0.4, 0.45, 0.5),
        front: Some(((0.0, 0.0), (181.0, 134.0))),
    },
    Tile {
        key: "quit",
        label: "Quit",
        size: (436.0, 68.0),
        at: (0.0, -406.0),
        color: (0.7, 0.2, 0.25),
        front: None,
    },
];

const SCREEN: (f32, f32) = (1920.0, 1080.0);
/// The rotating sun-ray layers behind a selected tile's art.
const RAY_LAYERS: usize = 5;
const RAY_SIZE: f32 = 556.0;

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
struct TileLabel(usize);
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
fn spawn_main_menu(mut commands: Commands, art: Option<Res<Art>>) {
    let image = |name: &str| art.as_ref().and_then(|a| a.image(name));
    let mut root = commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.05, 0.06, 0.12)),
        GlobalZIndex(11),
        Visibility::Hidden,
        MainMenuRoot,
    ));
    if let Some(background) = image("menu_background") {
        root.insert((ImageNode::new(background), BackgroundColor(Color::NONE)));
    }
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
                    on.with_children(|on| {
                        for layer in 0..RAY_LAYERS {
                            let Some(art) = image(&format!("menu_sunrays_{}", layer + 1)) else {
                                continue;
                            };
                            on.spawn((
                                ImageNode::new(art),
                                inside((RAY_SIZE, RAY_SIZE), (0.0, 0.0)),
                                UiTransform::default(),
                                TileRays { layer },
                            ));
                        }
                    });
                    // The art over the tile, grey or in colour, and the frame.
                    let front = || match tile.front {
                        Some((offset, size)) => inside(size, offset),
                        None => fill(),
                    };
                    for (part, picked) in [("_front_deselected", false), ("_front", true)] {
                        if let Some(art) = image(&format!("menu_tile_{key}{part}")) {
                            t.spawn((ImageNode::new(art), front(), shown_when(picked)));
                        }
                    }
                    if let Some(art) = image(&format!("menu_tile_{key}_overlay")) {
                        t.spawn((ImageNode::new(art), fill(), shown_when(true)));
                    }
                    let corner = if small {
                        "menu_corner_small"
                    } else {
                        "menu_corner_large"
                    };
                    if let Some(art) = image(corner) {
                        let (cw, ch) = if small { (38.0, 34.0) } else { (69.0, 68.0) };
                        t.spawn((fill(), shown_when(true)))
                            .with_children(|corners| {
                                for (flip_x, flip_y) in
                                    [(false, false), (true, false), (false, true), (true, true)]
                                {
                                    let mut node = Node {
                                        position_type: PositionType::Absolute,
                                        width: Val::Percent(cw / w * 100.0),
                                        height: Val::Percent(ch / h * 100.0),
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
                    // The label, in the header bar on big tiles.
                    let mut header = t.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Percent(100.0),
                            height: Val::Percent(if small { 100.0 } else { 22.0 }),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        TileLabel(i),
                        Visibility::Hidden,
                    ));
                    if !small && let Some(art) = image("menu_tile_header") {
                        header.insert(ImageNode::new(art));
                    }
                    header.with_children(|h| {
                        h.spawn((
                            Text::new(tile.label),
                            TextFont {
                                font_size: FontSize::Vh(if small { 2.6 } else { 3.2 }),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            TextLayout::justify(Justify::Center),
                        ));
                    });
                });
        }
    });
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
/// Local Battle opens the mode menu, Single Player opens it on Endless Race
/// and Quit closes the game. The other tiles aren't built yet.
#[allow(clippy::too_many_arguments)]
fn main_menu_input(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    gamepads: Query<&Gamepad>,
    game: Res<Match>,
    mut screen: ResMut<MenuScreen>,
    mut menu: ResMut<MainMenu>,
    mut config: ResMut<GameConfig>,
    mut exit: MessageWriter<AppExit>,
) {
    if game.phase != Phase::Menu || *screen != MenuScreen::Main {
        return;
    }
    let mut dir = Vec2::ZERO;
    let mut pick = false;
    if let Some(keys) = keys {
        let pressed = |list: [KeyCode; 3]| keys.any_just_pressed(list);
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
        "local" => *screen = MenuScreen::Modes,
        "single" => {
            config.mode = ModeKind::Endless;
            *screen = MenuScreen::Modes;
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
    art: Option<Res<Art>>,
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
    let staged = |key: &str| {
        art.as_ref()
            .is_some_and(|a| a.image(&format!("menu_tile_{key}_deselected")).is_some())
    };
    for (back, children) in &tiles {
        let on = back.0 == menu.selected;
        for &child in children {
            let Ok((mut v, chosen, label)) = parts.get_mut(child) else {
                continue;
            };
            let want = match (chosen, label) {
                (Some(chosen), _) => chosen.picked == on,
                (None, Some(label)) => on || !staged(TILES[label.0].key),
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
    mut rays: Query<(&TileRays, &mut UiTransform), Without<TileCorner>>,
    mut corners: Query<(&TileCorner, &mut UiTransform), Without<TileRays>>,
) {
    let t = time.elapsed_secs();
    let look = &config.menu;
    for (rays, mut transform) in &mut rays {
        let Some(&(degrees, seconds)) = look.ray_turns.get(rays.layer) else {
            continue;
        };
        let loop_at = (t / seconds.max(0.01)).fract();
        transform.rotation = Rot2::degrees(degrees * loop_at);
    }
    let (reach, seconds) = look.corner_pulse;
    let out = reach * (0.5 - 0.5 * (t / seconds.max(0.01) * std::f32::consts::TAU).cos());
    // Pixels of a 1080-high screen, as a share of the view height.
    let vh = out / SCREEN.1 * 100.0;
    for (corner, mut transform) in &mut corners {
        transform.translation = Val2::new(Val::Vh(corner.0.x * vh), Val::Vh(corner.0.y * vh));
    }
}
