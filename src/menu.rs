//! The menus: the original's title screen of tiles (Local Battle, Single
//! Player, ...) and behind it the mode menu, where you pick the mode, how many
//! people play and how many AI wizards join.

use bevy::prelude::*;

use crate::art::Art;
use crate::config::{Difficulty, GameConfig, ModeKind};
use crate::gameplay::{Match, MatchRequest, Phase, TowerState};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuScreen>()
            .init_resource::<MainMenu>()
            .add_systems(Startup, (spawn_menu, spawn_main_menu))
            .add_systems(
                Update,
                (
                    menu_keys,
                    menu_gamepads,
                    main_menu_input,
                    show_menu,
                    show_main_menu,
                    spin_tile_rays,
                )
                    .chain(),
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
    towers: Query<(), With<TowerState>>,
    mut request: ResMut<MatchRequest>,
    mut screen: ResMut<MenuScreen>,
    mut left_from: Local<Option<Phase>>,
) {
    let Some(keys) = keys else { return };
    if keys.just_pressed(KeyCode::Escape) {
        match game.phase {
            Phase::Playing | Phase::Over => {
                *left_from = Some(game.phase);
                game.phase = Phase::Menu;
            }
            // Back to the match we left.
            Phase::Menu if !towers.is_empty() => {
                if let Some(phase) = left_from.take() {
                    game.phase = phase;
                }
            }
            // Back to the title tiles.
            Phase::Menu => *screen = MenuScreen::Main,
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
        *left_from = None;
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
    towers: Query<(), With<TowerState>>,
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
    let resume = if towers.is_empty() {
        ""
    } else {
        "\nEsc: back to the game"
    };
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

/// Which menu screen is up while the game is in the menu.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuScreen {
    /// The title tiles.
    #[default]
    Main,
    /// Mode, players, AI wizards and difficulty.
    Modes,
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
}

/// The original's title tiles in its order. Sizes are the original's; the
/// centres are laid out from those sizes as two columns of equal height over
/// the Quit bar (the original's anchors aren't read yet).
pub const TILES: [Tile; 7] = [
    Tile {
        key: "local",
        label: "Local Battle",
        size: (437.0, 335.0),
        at: (-249.0, 222.0),
        color: (0.85, 0.35, 0.2),
    },
    Tile {
        key: "single",
        label: "Single Player",
        size: (435.0, 335.0),
        at: (-246.0, -133.0),
        color: (0.25, 0.6, 0.3),
    },
    Tile {
        key: "online",
        label: "Online Battle",
        size: (441.0, 335.0),
        at: (278.0, 222.0),
        color: (0.25, 0.45, 0.85),
    },
    Tile {
        key: "shop",
        label: "Shop",
        size: (450.0, 165.0),
        at: (277.0, -48.0),
        color: (0.85, 0.65, 0.15),
    },
    Tile {
        key: "help",
        label: "Help",
        size: (197.0, 150.0),
        at: (146.0, -226.0),
        color: (0.55, 0.35, 0.75),
    },
    Tile {
        key: "settings",
        label: "Settings",
        size: (197.0, 150.0),
        at: (396.0, -226.0),
        color: (0.4, 0.45, 0.5),
    },
    Tile {
        key: "quit",
        label: "Quit",
        size: (436.0, 68.0),
        at: (0.0, -355.0),
        color: (0.7, 0.2, 0.25),
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

/// The title screen. Each tile shows `menu_tile_<key>_deselected` (grey) until
/// it is picked; then `menu_tile_<key>_selected` with the spinning
/// `menu_sunrays_1..5`, its `menu_tile_<key>_front` art, the
/// `menu_tile_header` bar with the label and `menu_corner_large|small` in its
/// corners. Missing art falls back to plain colours.
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
            let mut node = percent_node(tile.size, tile.at);
            node.overflow = Overflow::clip();
            parent
                .spawn((node, UiTransform::default(), TileBack(i)))
                .with_children(|t| {
                    // Grey when not picked.
                    let grey = image(&format!("menu_tile_{}_deselected", tile.key));
                    let mut back =
                        t.spawn((fill(), TileChosen { picked: false }, Visibility::Inherited));
                    match grey {
                        Some(h) => back.insert(ImageNode::new(h)),
                        None => back.insert(BackgroundColor(Color::srgb(
                            0.35 + r * 0.15,
                            0.35 + g * 0.15,
                            0.35 + b * 0.15,
                        ))),
                    };
                    // Everything shown when picked.
                    t.spawn((fill(), TileChosen { picked: true }, Visibility::Hidden))
                        .with_children(|on| {
                            let mut colour = on.spawn(fill());
                            match image(&format!("menu_tile_{}_selected", tile.key)) {
                                Some(h) => colour.insert(ImageNode::new(h)),
                                None => colour.insert(BackgroundColor(Color::srgb(r, g, b))),
                            };
                            for layer in 0..RAY_LAYERS {
                                let Some(h) = image(&format!("menu_sunrays_{}", layer + 1)) else {
                                    continue;
                                };
                                let (w, hgt) = tile.size;
                                on.spawn((
                                    ImageNode::new(h),
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: Val::Percent((w - RAY_SIZE) / 2.0 / w * 100.0),
                                        top: Val::Percent((hgt - RAY_SIZE) / 2.0 / hgt * 100.0),
                                        width: Val::Percent(RAY_SIZE / w * 100.0),
                                        height: Val::Percent(RAY_SIZE / hgt * 100.0),
                                        ..default()
                                    },
                                    UiTransform::default(),
                                    TileRays { layer },
                                ));
                            }
                            if let Some(h) = image(&format!("menu_tile_{}_front", tile.key)) {
                                on.spawn((ImageNode::new(h), fill()));
                            }
                            let corner = if small {
                                "menu_corner_small"
                            } else {
                                "menu_corner_large"
                            };
                            if let Some(h) = image(corner) {
                                let (cw, ch) = if small { (38.0, 34.0) } else { (69.0, 68.0) };
                                for (flip_x, flip_y) in
                                    [(false, false), (true, false), (false, true), (true, true)]
                                {
                                    let mut node = Node {
                                        position_type: PositionType::Absolute,
                                        width: Val::Percent(cw / tile.size.0 * 100.0),
                                        height: Val::Percent(ch / tile.size.1 * 100.0),
                                        ..default()
                                    };
                                    if flip_x {
                                        node.right = Val::Px(0.0);
                                    }
                                    if flip_y {
                                        node.bottom = Val::Px(0.0);
                                    }
                                    on.spawn((
                                        ImageNode {
                                            image: h.clone(),
                                            flip_x,
                                            flip_y,
                                            ..default()
                                        },
                                        node,
                                    ));
                                }
                            }
                        });
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
                    if !small && let Some(h) = image("menu_tile_header") {
                        header.insert(ImageNode::new(h));
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
/// sideways distance counting double.
fn tile_towards(from: usize, dir: Vec2) -> usize {
    let at = |i: usize| Vec2::new(TILES[i].at.0, TILES[i].at.1);
    let here = at(from);
    (0..TILES.len())
        .filter(|&i| i != from && (at(i) - here).dot(dir) > 1.0)
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
    mut tiles: Query<(&TileBack, &Children, &mut UiTransform)>,
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
    for (back, children, mut transform) in &mut tiles {
        let on = back.0 == menu.selected;
        let scale = if on { Vec2::splat(1.04) } else { Vec2::ONE };
        if transform.scale != scale {
            transform.scale = scale;
        }
        for &child in children {
            let Ok((mut v, chosen, label)) = parts.get_mut(child) else {
                continue;
            };
            let want = if let Some(chosen) = chosen {
                chosen.picked == on
            } else if let Some(label) = label {
                on || !staged(TILES[label.0].key)
            } else {
                continue;
            };
            v.set_if_neq(if want {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            });
        }
    }
}

fn spin_tile_rays(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut rays: Query<(&TileRays, &mut UiTransform, &InheritedVisibility)>,
) {
    for (rays, mut transform, visible) in &mut rays {
        if !visible.get() {
            continue;
        }
        // Each layer turns at its own speed, alternating direction.
        let sign = if rays.layer % 2 == 0 { 1.0 } else { -1.0 };
        let speed = config.moments.sunray_spin * (1.0 + rays.layer as f32 * 0.25) * sign;
        let angle = transform.rotation.as_radians() + speed.to_radians() * time.delta_secs();
        transform.rotation = Rot2::radians(angle);
    }
}
