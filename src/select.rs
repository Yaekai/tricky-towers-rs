//! The original's local-play screens after the title tiles: "Select Players",
//! where people join a seat (and AI wizards can fill the others), then
//! "Select Mode", a row of mode tiles each with its own difficulty badge.
//! Layout from docs/research/menus.md, in pixels of a 1920×1080 screen.

use bevy::prelude::*;

use crate::art::Art;
use crate::config::{Difficulty, GameConfig, ModeKind};
use crate::gameplay::{MAX_TOWERS, Match, MatchRequest, Phase};
use crate::menu::MenuScreen;

/// Who sits in a player select seat.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Seat {
    #[default]
    Empty,
    Keyboard,
    Pad(Entity),
    Ai,
}

impl Seat {
    pub fn human(self) -> bool {
        matches!(self, Seat::Keyboard | Seat::Pad(_))
    }
}

/// A difficulty choice on a mode tile: one of the three, or a random one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DifficultyPick {
    Set(Difficulty),
    Random,
}

impl DifficultyPick {
    pub const ALL: [DifficultyPick; 4] = [
        DifficultyPick::Set(Difficulty::Easy),
        DifficultyPick::Set(Difficulty::Normal),
        DifficultyPick::Set(Difficulty::Pro),
        DifficultyPick::Random,
    ];

    pub fn name(self) -> &'static str {
        match self {
            DifficultyPick::Set(d) => d.name(),
            DifficultyPick::Random => "Random",
        }
    }

    /// The original's art names the third difficulty "special".
    fn art_key(self) -> &'static str {
        match self {
            DifficultyPick::Set(Difficulty::Easy) => "easy",
            DifficultyPick::Set(Difficulty::Pro) => "special",
            _ => "normal",
        }
    }

    fn stars(self) -> usize {
        match self {
            DifficultyPick::Set(Difficulty::Easy) => 1,
            DifficultyPick::Set(Difficulty::Normal) => 2,
            DifficultyPick::Set(Difficulty::Pro) => 3,
            DifficultyPick::Random => 0,
        }
    }
}

/// A mode tile: a mode, or Random for one of Race, Survival and Puzzle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModePick {
    Mode(ModeKind),
    Random,
}

impl ModePick {
    pub fn name(self) -> &'static str {
        match self {
            ModePick::Mode(m) => m.name(),
            ModePick::Random => "Random",
        }
    }

    fn color(self) -> Color {
        match self {
            ModePick::Mode(ModeKind::Race) => Color::srgb(0.85, 0.35, 0.2),
            ModePick::Mode(ModeKind::Survival) => Color::srgb(0.25, 0.55, 0.85),
            ModePick::Mode(ModeKind::Puzzle) => Color::srgb(0.55, 0.35, 0.75),
            ModePick::Mode(ModeKind::Endless) => Color::srgb(0.25, 0.6, 0.3),
            ModePick::Random => Color::srgb(0.85, 0.65, 0.15),
        }
    }
}

/// The mode tiles of a local battle, in the original's order.
pub const LOCAL_MODES: [ModePick; 4] = [
    ModePick::Mode(ModeKind::Race),
    ModePick::Mode(ModeKind::Survival),
    ModePick::Mode(ModeKind::Puzzle),
    ModePick::Random,
];
/// Single Player offers Endless Race (Trials aren't built yet).
pub const SINGLE_MODES: [ModePick; 1] = [ModePick::Mode(ModeKind::Endless)];

/// The seats, and the mode screen's highlighted tile and per-tile difficulty.
#[derive(Resource, Debug)]
pub struct Lobby {
    pub seats: [Seat; MAX_TOWERS],
    /// Came from Single Player rather than Local Battle.
    pub single: bool,
    pub mode: usize,
    pub difficulty: [DifficultyPick; 4],
}

impl Default for Lobby {
    fn default() -> Self {
        Self {
            seats: [Seat::Empty; MAX_TOWERS],
            single: false,
            mode: 0,
            difficulty: [DifficultyPick::Set(Difficulty::Normal); 4],
        }
    }
}

impl Lobby {
    pub fn modes(&self) -> &'static [ModePick] {
        if self.single {
            &SINGLE_MODES
        } else {
            &LOCAL_MODES
        }
    }

    fn humans(&self) -> u32 {
        self.seats.iter().filter(|s| s.human()).count() as u32
    }

    fn ais(&self) -> u32 {
        self.seats.iter().filter(|&&s| s == Seat::Ai).count() as u32
    }

    fn seat(&mut self, who: Seat) -> bool {
        match self.seats.iter_mut().find(|s| **s == Seat::Empty) {
            Some(seat) => {
                *seat = who;
                true
            }
            None => false,
        }
    }

    fn unseat_last_ai(&mut self) {
        if let Some(seat) = self.seats.iter_mut().rev().find(|s| **s == Seat::Ai) {
            *seat = Seat::Empty;
        }
    }
}

/// Pixel layout of a 1920×1080 screen.
const SCREEN: (f32, f32) = (1920.0, 1080.0);
const HEADER_HEIGHT: f32 = 100.0;
/// A seat's parts: size and centre height (y up from the screen middle).
const SEAT_BACK: ((f32, f32), f32) = ((332.0, 283.0), 115.0);
const SEAT_WIZARD: ((f32, f32), f32) = ((318.0, 500.0), 60.0);
const SEAT_STRIP: ((f32, f32), f32) = ((332.0, 100.0), -110.0);
const SEAT_PLATE: ((f32, f32), f32) = ((367.0, 78.0), -230.0);
/// Mode tiles: size, centre height and the distance between tile centres.
const MODE_TILE: (f32, f32) = (380.0, 460.0);
const MODE_TILE_Y: f32 = 20.0;
const MODE_TILE_STEP: f32 = 430.0;

/// An absolute node `size` pixels big centred at `at`.
fn place(size: (f32, f32), at: (f32, f32)) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Percent(50.0 + (at.0 - size.0 / 2.0) / SCREEN.0 * 100.0),
        top: Val::Percent(50.0 - (at.1 + size.1 / 2.0) / SCREEN.1 * 100.0),
        width: Val::Percent(size.0 / SCREEN.0 * 100.0),
        height: Val::Percent(size.1 / SCREEN.1 * 100.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

fn label(size: f32, text: &str) -> impl Bundle {
    (
        Text::new(text),
        TextFont {
            font_size: FontSize::Vh(size),
            ..default()
        },
        TextColor(Color::WHITE),
        TextLayout::justify(Justify::Center),
    )
}

#[derive(Component)]
pub(crate) struct PlayersRoot;
#[derive(Component)]
pub(crate) struct ModesRoot;
#[derive(Component)]
pub(crate) struct SeatWizard(usize);
#[derive(Component)]
pub(crate) struct SeatName(usize);
#[derive(Component)]
pub(crate) struct SeatBack(usize);
#[derive(Component)]
pub(crate) struct ModeTile(usize);
#[derive(Component)]
pub(crate) struct ModeName(usize);
#[derive(Component)]
pub(crate) struct ModeDifficulty(usize);
#[derive(Component)]
pub(crate) struct ModeBadge(usize);
#[derive(Component)]
pub(crate) struct ModeBadgeCentre(usize);
#[derive(Component)]
pub(crate) struct ModeStars(usize);

/// A full-screen page with the title backdrop and a header bar.
fn spawn_page(commands: &mut Commands, art: Option<&Art>, title: &str) -> Entity {
    let mut page = commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.05, 0.06, 0.12)),
        GlobalZIndex(11),
        Visibility::Hidden,
    ));
    if let Some(background) = art.and_then(|a| a.image("menu_background")) {
        page.insert((ImageNode::new(background), BackgroundColor(Color::NONE)));
    }
    page.with_children(|page| {
        let mut header = page.spawn((
            place(
                (SCREEN.0, HEADER_HEIGHT),
                (0.0, (SCREEN.1 - HEADER_HEIGHT) / 2.0),
            ),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
        ));
        if let Some(bar) = art.and_then(|a| a.image("menu_header")) {
            header.insert((ImageNode::new(bar), BackgroundColor(Color::NONE)));
        }
        header.with_children(|h| {
            h.spawn(label(5.0, title));
        });
    });
    page.id()
}

pub(crate) fn spawn_select_screens(mut commands: Commands, art: Option<Res<Art>>) {
    let art = art.as_deref();
    let image = |name: &str| art.and_then(|a| a.image(name));

    let players = spawn_page(&mut commands, art, "Select Players");
    commands.entity(players).insert(PlayersRoot);
    commands.entity(players).with_children(|page| {
        let column = SCREEN.0 / MAX_TOWERS as f32;
        for seat in 0..MAX_TOWERS {
            let x = column * (seat as f32 + 0.5) - SCREEN.0 / 2.0;
            let mut back = page.spawn((
                place(SEAT_BACK.0, (x, SEAT_BACK.1)),
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.12)),
                SeatBack(seat),
            ));
            if let Some(art) = image("menu_player_select_background_deselected") {
                back.insert((ImageNode::new(art), BackgroundColor(Color::NONE)));
            }
            page.spawn((
                place(SEAT_WIZARD.0, (x, SEAT_WIZARD.1)),
                ImageNode::default(),
                SeatWizard(seat),
            ));
            page.spawn((
                place(SEAT_STRIP.0, (x, SEAT_STRIP.1)),
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
            ));
            let mut plate = page.spawn((
                place(SEAT_PLATE.0, (x, SEAT_PLATE.1)),
                BackgroundColor(Color::srgba(0.1, 0.08, 0.2, 0.85)),
            ));
            if let Some(art) = image("menu_player_select_nameplate") {
                plate.insert((ImageNode::new(art), BackgroundColor(Color::NONE)));
            }
            plate.with_children(|p| {
                p.spawn((label(3.0, ""), SeatName(seat)));
            });
        }
        page.spawn(place((SCREEN.0, 60.0), (0.0, -SCREEN.1 / 2.0 + 60.0)))
            .with_children(|p| {
                p.spawn(label(
                    2.2,
                    "Enter or A: join, then start   Tab or X: add an AI wizard   Backspace or Y: remove one   Esc or B: back",
                ));
            });
    });

    let modes = spawn_page(&mut commands, art, "Select Mode");
    commands.entity(modes).insert(ModesRoot);
    commands.entity(modes).with_children(|page| {
        for tile in 0..LOCAL_MODES.len() {
            page.spawn((
                place(MODE_TILE, (0.0, MODE_TILE_Y)),
                BackgroundColor(Color::WHITE),
                UiTransform::default(),
                ModeTile(tile),
            ))
            .with_children(|t| {
                // The mode's name across the top.
                t.spawn(Node {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(6.0),
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_children(|n| {
                    n.spawn((label(4.0, ""), ModeName(tile)));
                });
                // The difficulty badge: wings, the centre badge, stars, the name.
                t.spawn(Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Percent(4.0),
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Vh(0.6),
                    ..default()
                })
                .with_children(|b| {
                    b.spawn((
                        Node {
                            width: Val::Vh(191.0 / 10.8),
                            height: Val::Vh(98.0 / 10.8),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ImageNode::default(),
                        ModeBadge(tile),
                    ))
                    .with_child((
                        Node {
                            width: Val::Vh(81.0 / 10.8),
                            height: Val::Vh(80.0 / 10.8),
                            ..default()
                        },
                        ImageNode::default(),
                        ModeBadgeCentre(tile),
                    ));
                    b.spawn((
                        Node {
                            column_gap: Val::Vh(0.4),
                            height: Val::Vh(35.0 / 10.8),
                            ..default()
                        },
                        ModeStars(tile),
                    ));
                    b.spawn((label(3.0, ""), ModeDifficulty(tile)));
                });
            });
        }
        page.spawn(place((SCREEN.0, 60.0), (0.0, -SCREEN.1 / 2.0 + 60.0)))
            .with_children(|p| {
                p.spawn(label(
                    2.2,
                    "Left/Right: mode   Up/Down: difficulty   Enter or A: play   Esc or B: back",
                ));
            });
    });
}

/// Taps on the keyboard and every pad, read once per frame.
#[derive(Default)]
struct Taps {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    /// Enter/Space on the keyboard.
    key_pick: bool,
    key_add: bool,
    key_remove: bool,
    /// Pads that pressed A, B, X and Y.
    pad_pick: Vec<Entity>,
    pad_back: Vec<Entity>,
    pad_add: bool,
    pad_remove: bool,
}

fn read_taps(keys: Option<&ButtonInput<KeyCode>>, pads: &Query<(Entity, &Gamepad)>) -> Taps {
    let mut taps = Taps::default();
    if let Some(keys) = keys {
        taps.up = keys.any_just_pressed([KeyCode::ArrowUp, KeyCode::KeyW, KeyCode::Numpad8]);
        taps.down = keys.any_just_pressed([KeyCode::ArrowDown, KeyCode::KeyS, KeyCode::Numpad2]);
        taps.left = keys.any_just_pressed([KeyCode::ArrowLeft, KeyCode::KeyA, KeyCode::Numpad4]);
        taps.right = keys.any_just_pressed([KeyCode::ArrowRight, KeyCode::KeyD, KeyCode::Numpad6]);
        taps.key_pick =
            keys.any_just_pressed([KeyCode::Enter, KeyCode::Space, KeyCode::NumpadEnter]);
        taps.key_add = keys.just_pressed(KeyCode::Tab);
        taps.key_remove = keys.any_just_pressed([KeyCode::Backspace, KeyCode::Delete]);
    }
    for (entity, pad) in pads {
        taps.up |= pad.just_pressed(GamepadButton::DPadUp);
        taps.down |= pad.just_pressed(GamepadButton::DPadDown);
        taps.left |= pad.just_pressed(GamepadButton::DPadLeft);
        taps.right |= pad.just_pressed(GamepadButton::DPadRight);
        if pad.any_just_pressed([GamepadButton::South, GamepadButton::Start]) {
            taps.pad_pick.push(entity);
        }
        if pad.just_pressed(GamepadButton::East) {
            taps.pad_back.push(entity);
        }
        taps.pad_add |= pad.just_pressed(GamepadButton::West);
        taps.pad_remove |= pad.just_pressed(GamepadButton::North);
    }
    taps
}

/// "Select Players": Enter seats the keyboard player and A seats a pad; once
/// someone is seated, the same press goes on to the modes. Tab or X adds an
/// AI wizard to the next empty seat, Backspace or Y takes the last one away,
/// and B gets a pad up from its seat.
pub(crate) fn players_input(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    pads: Query<(Entity, &Gamepad)>,
    game: Res<Match>,
    mut screen: ResMut<MenuScreen>,
    mut lobby: ResMut<Lobby>,
    mut config: ResMut<GameConfig>,
) {
    if game.phase != Phase::Menu || *screen != MenuScreen::Players {
        return;
    }
    let taps = read_taps(keys.as_deref(), &pads);
    let mut go = false;
    if taps.key_pick {
        if lobby.seats.contains(&Seat::Keyboard) {
            go = true;
        } else {
            lobby.seat(Seat::Keyboard);
        }
    }
    for pad in taps.pad_pick {
        if lobby.seats.contains(&Seat::Pad(pad)) {
            go = true;
        } else {
            lobby.seat(Seat::Pad(pad));
        }
    }
    for pad in taps.pad_back {
        if let Some(seat) = lobby.seats.iter_mut().find(|s| **s == Seat::Pad(pad)) {
            *seat = Seat::Empty;
        } else if !lobby.seats.iter().any(|s| s.human()) {
            *screen = MenuScreen::Main;
            return;
        }
    }
    if taps.key_add || taps.pad_add {
        lobby.seat(Seat::Ai);
    }
    if taps.key_remove || taps.pad_remove {
        lobby.unseat_last_ai();
    }
    if go && lobby.humans() > 0 {
        config.players = lobby.humans();
        config.opponents = lobby.ais();
        lobby.mode = lobby.mode.min(lobby.modes().len() - 1);
        *screen = MenuScreen::Modes;
    }
}

/// "Select Mode": Left/Right pick a tile, Up/Down its difficulty, Enter or A
/// plays it. Random picks Race, Survival or Puzzle, and a Random difficulty
/// one of the three, as the match starts.
#[allow(clippy::too_many_arguments)]
pub(crate) fn modes_input(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    pads: Query<(Entity, &Gamepad)>,
    time: Res<Time>,
    game: Res<Match>,
    mut screen: ResMut<MenuScreen>,
    mut lobby: ResMut<Lobby>,
    mut config: ResMut<GameConfig>,
    mut request: ResMut<MatchRequest>,
) {
    if game.phase != Phase::Menu || *screen != MenuScreen::Modes {
        return;
    }
    let taps = read_taps(keys.as_deref(), &pads);
    if !taps.pad_back.is_empty() {
        *screen = back_from_modes(&lobby);
        return;
    }
    let count = lobby.modes().len();
    if taps.left {
        lobby.mode = (lobby.mode + count - 1) % count;
    }
    if taps.right {
        lobby.mode = (lobby.mode + 1) % count;
    }
    let tile = lobby.mode;
    let picks = DifficultyPick::ALL.len();
    let at = DifficultyPick::ALL
        .iter()
        .position(|&d| d == lobby.difficulty[tile])
        .unwrap_or(1);
    if taps.up {
        lobby.difficulty[tile] = DifficultyPick::ALL[(at + 1) % picks];
    }
    if taps.down {
        lobby.difficulty[tile] = DifficultyPick::ALL[(at + picks - 1) % picks];
    }
    if !(taps.key_pick || !taps.pad_pick.is_empty()) {
        return;
    }
    // A roll from the clock is plenty for picking a mode.
    let roll = time.elapsed().as_nanos() as usize / 1000;
    config.mode = match lobby.modes()[tile] {
        ModePick::Mode(mode) => mode,
        ModePick::Random => [ModeKind::Race, ModeKind::Survival, ModeKind::Puzzle][roll % 3],
    };
    config.difficulty = Some(match lobby.difficulty[tile] {
        DifficultyPick::Set(d) => d,
        DifficultyPick::Random => Difficulty::ALL[(roll / 3) % Difficulty::ALL.len()],
    });
    if lobby.single {
        config.players = 1;
        config.opponents = 0;
    }
    request.pending = true;
}

/// Where Esc or B leads from the mode screen.
pub(crate) fn back_from_modes(lobby: &Lobby) -> MenuScreen {
    if lobby.single {
        MenuScreen::Main
    } else {
        MenuScreen::Players
    }
}

fn shown(on: bool) -> Visibility {
    if on {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    }
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(crate) fn show_select_screens(
    game: Res<Match>,
    screen: Res<MenuScreen>,
    lobby: Res<Lobby>,
    art: Option<Res<Art>>,
    mut roots: ParamSet<(
        Query<&mut Visibility, With<PlayersRoot>>,
        Query<&mut Visibility, With<ModesRoot>>,
    )>,
    mut wizards: Query<
        (&SeatWizard, &mut ImageNode, &mut Visibility),
        (
            Without<ModeBadge>,
            Without<ModeBadgeCentre>,
            Without<PlayersRoot>,
            Without<ModesRoot>,
            Without<ModeTile>,
        ),
    >,
    mut texts: ParamSet<(
        Query<(&SeatName, &mut Text)>,
        Query<(&ModeName, &mut Text)>,
        Query<(&ModeDifficulty, &mut Text)>,
    )>,
    mut backs: Query<(&SeatBack, &mut BackgroundColor), Without<ModeTile>>,
    mut tiles: Query<
        (
            &ModeTile,
            &mut Node,
            &mut BackgroundColor,
            &mut UiTransform,
            &mut Visibility,
        ),
        (
            Without<SeatWizard>,
            Without<ModeBadge>,
            Without<PlayersRoot>,
            Without<ModesRoot>,
        ),
    >,
    mut badges: Query<
        (Option<&ModeBadge>, Option<&ModeBadgeCentre>, &mut ImageNode),
        (
            Without<SeatWizard>,
            Or<(With<ModeBadge>, With<ModeBadgeCentre>)>,
        ),
    >,
    stars: Query<(Entity, &ModeStars, Option<&Children>)>,
    mut commands: Commands,
) {
    let menu = game.phase == Phase::Menu;
    let players_on = menu && *screen == MenuScreen::Players;
    let modes_on = menu && *screen == MenuScreen::Modes;
    for mut v in &mut roots.p0() {
        v.set_if_neq(shown(players_on));
    }
    for mut v in &mut roots.p1() {
        v.set_if_neq(shown(modes_on));
    }
    let image = |name: &str| art.as_ref().and_then(|a| a.image(name));
    if players_on {
        let first_empty = lobby.seats.iter().position(|s| *s == Seat::Empty);
        let mut player = 0;
        let names: Vec<String> = lobby
            .seats
            .iter()
            .enumerate()
            .map(|(i, seat)| match seat {
                Seat::Keyboard | Seat::Pad(_) => {
                    player += 1;
                    format!("Player {player}")
                }
                Seat::Ai => "AI Wizard".into(),
                Seat::Empty if Some(i) == first_empty => {
                    if lobby.seats.contains(&Seat::Keyboard) {
                        "[A] To join".into()
                    } else {
                        "[Enter] To join".into()
                    }
                }
                Seat::Empty => String::new(),
            })
            .collect();
        for (seat, mut text) in &mut texts.p0() {
            if text.0 != names[seat.0] {
                text.0 = names[seat.0].clone();
            }
        }
        for (seat, mut node, mut v) in &mut wizards {
            let taken = lobby.seats[seat.0] != Seat::Empty;
            let pose = if taken { "ready" } else { "sleeping" };
            match image(&format!("menu_player_select_wizard{}_{pose}", seat.0 + 1)) {
                Some(handle) => {
                    if node.image != handle {
                        node.image = handle;
                    }
                    v.set_if_neq(Visibility::Inherited);
                }
                None => {
                    v.set_if_neq(Visibility::Hidden);
                }
            }
        }
        for (seat, mut colour) in &mut backs {
            let taken = lobby.seats[seat.0] != Seat::Empty;
            let alpha = if taken { 0.3 } else { 0.12 };
            if colour.0.alpha() != alpha && colour.0 != Color::NONE {
                colour.0 = colour.0.with_alpha(alpha);
            }
        }
    }
    if !modes_on {
        return;
    }
    let modes = lobby.modes();
    for (tile, mut node, mut colour, mut transform, mut v) in &mut tiles {
        let Some(&pick) = modes.get(tile.0) else {
            v.set_if_neq(Visibility::Hidden);
            continue;
        };
        v.set_if_neq(Visibility::Inherited);
        let x = (tile.0 as f32 - (modes.len() as f32 - 1.0) / 2.0) * MODE_TILE_STEP;
        let left = Val::Percent(50.0 + (x - MODE_TILE.0 / 2.0) / SCREEN.0 * 100.0);
        if node.left != left {
            node.left = left;
        }
        let on = tile.0 == lobby.mode;
        let tint = if on {
            pick.color()
        } else {
            pick.color().mix(&Color::srgb(0.4, 0.4, 0.4), 0.6)
        };
        if colour.0 != tint {
            colour.0 = tint;
        }
        let scale = Vec2::splat(if on { 1.06 } else { 1.0 });
        if transform.scale != scale {
            transform.scale = scale;
        }
    }
    for (name, mut text) in &mut texts.p1() {
        if let Some(pick) = modes.get(name.0) {
            let want = pick.name();
            if text.0 != want {
                text.0 = want.into();
            }
        }
    }
    for (d, mut text) in &mut texts.p2() {
        let want = lobby.difficulty[d.0].name();
        if text.0 != want {
            text.0 = want.into();
        }
    }
    for (wings, centre, mut node) in &mut badges {
        let (tile, part) = match (wings, centre) {
            (Some(w), _) => (w.0, "side"),
            (_, Some(c)) => (c.0, "background"),
            _ => continue,
        };
        let key = lobby.difficulty[tile].art_key();
        if let Some(handle) = image(&format!("menu_mode_title_{part}_{key}"))
            && node.image != handle
        {
            node.image = handle;
        }
    }
    // Stars: rebuilt when the count changes.
    for (entity, row, children) in &stars {
        let pick = lobby.difficulty[row.0];
        let have = children.map_or(0, |c| c.len());
        if have == pick.stars() {
            continue;
        }
        let Some(star) = image(&format!("menu_mode_title_star_{}", pick.art_key())) else {
            continue;
        };
        commands.entity(entity).despawn_children();
        for _ in 0..pick.stars() {
            commands.entity(entity).with_child((
                ImageNode::new(star.clone()),
                Node {
                    width: Val::Vh(37.0 / 10.8),
                    height: Val::Vh(35.0 / 10.8),
                    ..default()
                },
            ));
        }
    }
}
