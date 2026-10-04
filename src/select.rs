//! The original's local-play screens after the title tiles: "Select Players",
//! where people join a seat (and AI wizards can fill the others), then
//! "Select Mode", two columns (the mode, then the difficulty) that each show
//! one tile at a time with arrows above and below.
//! Layout from docs/research/menus.md, in pixels of a 1920×1080 screen.

use bevy::prelude::*;
use bevy::ui::widget::NodeImageMode;

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

/// A difficulty tile: one of the three, or a random one.
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

    /// The tile's picture: name, centre offset from the tile centre and size.
    fn picture(self) -> (&'static str, (f32, f32), (f32, f32)) {
        match self {
            DifficultyPick::Set(Difficulty::Easy) => {
                ("menu_difficulty_easy", (-3.9, 35.5), (209.0, 157.0))
            }
            DifficultyPick::Set(Difficulty::Normal) => {
                ("menu_difficulty_normal", (-2.0, 38.0), (328.0, 175.0))
            }
            DifficultyPick::Set(Difficulty::Pro) => {
                ("menu_difficulty_pro", (-2.5, 42.0), (370.0, 188.0))
            }
            DifficultyPick::Random => ("menu_difficulty_random", (0.0, 50.0), (365.0, 208.0)),
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

    /// The tile's picture: name, centre offset from the tile centre and size.
    fn picture(self) -> (&'static str, (f32, f32), (f32, f32)) {
        match self {
            ModePick::Mode(ModeKind::Race) => ("menu_mode_race", (-3.0, 9.0), (487.0, 411.0)),
            ModePick::Mode(ModeKind::Survival) => {
                ("menu_mode_survival", (6.0, 0.0), (482.0, 393.0))
            }
            ModePick::Mode(ModeKind::Puzzle) => ("menu_mode_puzzle", (-7.0, -1.0), (505.0, 392.0)),
            ModePick::Mode(ModeKind::Endless) => ("menu_mode_endless", (0.0, 0.0), (495.0, 393.0)),
            ModePick::Random => ("menu_mode_random", (0.0, 0.0), (495.0, 393.0)),
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

/// The seats, and the mode screen's picks.
#[derive(Resource, Debug)]
pub struct Lobby {
    pub seats: [Seat; MAX_TOWERS],
    /// Came from Single Player rather than Local Battle.
    pub single: bool,
    /// The picked mode and difficulty, as indexes into `modes()` and
    /// [`DifficultyPick::ALL`].
    pub mode: usize,
    pub difficulty: usize,
    /// The column Up/Down change: 0 the mode, 1 the difficulty.
    pub column: usize,
}

impl Default for Lobby {
    fn default() -> Self {
        Self {
            seats: [Seat::Empty; MAX_TOWERS],
            single: false,
            mode: 0,
            difficulty: 1,
            column: 0,
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

/// Pixel layout of a 1920×1080 screen, from the middle with y up.
const SCREEN: (f32, f32) = (1920.0, 1080.0);
/// The header bar hanging from the top centre, and its 9-slice borders
/// (left, right, top, bottom).
const HEADER: (f32, f32) = (800.0, 90.0);
const HEADER_SLICE: [f32; 4] = [84.0, 92.0, 75.0, 15.0];
/// Seats sit in 4 columns between 5% and 95% of the width.
fn seat_x(seat: usize) -> f32 {
    (0.05 + 0.225 * (seat as f32 + 0.5)) * SCREEN.0 - SCREEN.0 / 2.0
}
const SEAT_PANEL: (f32, f32) = (332.0, 283.0);
const SEAT_Y: f32 = 115.0;
/// The sun rays behind a ready wizard are cut to this size.
const SEAT_RAYS_CLIP: (f32, f32) = (327.0, 267.0);
const SEAT_ARROW: (f32, f32) = (38.0, 64.0);
const SEAT_ARROW_X: f32 = 165.0;
const PACK_PANEL: ((f32, f32), f32) = ((332.0, 100.0), -110.0);
const NAME_PLATE: ((f32, f32), f32) = ((367.0, 78.0), -230.0);
/// 9-slice borders of the seat panels and outline, and of the name plate.
const PANEL_SLICE: f32 = 60.0;
const PLATE_SLICE: f32 = 13.0;

/// Each seat's wizard, asleep and awake: centre offset from the panel centre
/// and size.
/// A picture's centre offset and size, in pixels.
type Spot = ((f32, f32), (f32, f32));
const WIZARDS: [[Spot; 2]; 4] = [
    [((0.0, 0.0), (213.0, 250.0)), ((0.0, 0.0), (235.0, 314.0))],
    [((0.0, 38.0), (225.0, 323.0)), ((0.0, 21.0), (259.0, 344.0))],
    [
        ((-9.0, 14.0), (237.0, 275.0)),
        ((-3.0, 4.0), (279.0, 300.0)),
    ],
    [((0.0, 11.7), (212.0, 267.0)), ((7.5, 7.0), (253.0, 322.0))],
];

/// Select Mode's columns: centre x of each, the picked tile's frame, the
/// arrows and where the difficulty tile's background sits.
const COLUMN_X: [f32; 2] = [-270.0, 270.0];
const FRAME: (f32, f32) = (513.0, 445.0);
const FRAME_X: f32 = -5.5;
const COLUMN_ARROW: (f32, f32) = (94.0, 55.0);
const ARROW_GAP: f32 = 130.0;
const DIFFICULTY_BACK: ((f32, f32), (f32, f32)) = ((-5.0, 0.0), (461.0, 393.0));
const CORNER_LARGE: (f32, f32) = (69.0, 68.0);
const CORNER_SMALL: (f32, f32) = (38.0, 34.0);

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

/// A 9-sliced picture with these borders (left, right, top, bottom) in its pixels.
fn sliced(image: Handle<Image>, border: [f32; 4]) -> ImageNode {
    ImageNode::new(image).with_mode(NodeImageMode::Sliced(TextureSlicer {
        border: BorderRect {
            min_inset: Vec2::new(border[0], border[2]),
            max_inset: Vec2::new(border[1], border[3]),
        },
        ..default()
    }))
}

/// Four corner pictures on a `size` box at `at`, the art drawn top-left and
/// flipped for the others.
fn corners(
    page: &mut ChildSpawnerCommands,
    art: &Handle<Image>,
    size: (f32, f32),
    at: (f32, f32),
    corner: (f32, f32),
    part: impl Bundle + Clone,
) {
    for (flip_x, flip_y) in [(false, false), (true, false), (false, true), (true, true)] {
        let x = at.0 + (size.0 - corner.0) / 2.0 * if flip_x { 1.0 } else { -1.0 };
        let y = at.1 + (size.1 - corner.1) / 2.0 * if flip_y { -1.0 } else { 1.0 };
        page.spawn((
            ImageNode {
                image: art.clone(),
                flip_x,
                flip_y,
                ..default()
            },
            place(corner, (x, y)),
            part.clone(),
        ));
    }
}

/// What a select-screen node is, so one system can show and update it.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub(crate) enum Part {
    /// A seat's grey panel, shown while nobody sits there.
    SeatIdle(usize),
    /// A taken seat's coloured panel, rays, outline, arrows and corners.
    SeatTaken(usize),
    /// A seat's turning rays, by layer.
    SeatRays(usize, usize),
    /// A seat's wizard, asleep or awake.
    SeatWizard(usize),
    /// A seat's name plate text.
    SeatName(usize),
    /// A column's tile picture (mode or difficulty), and the difficulty background.
    ColumnPicture(usize),
    DifficultyBack,
    /// A column's name under its tile, shown when its picture isn't staged.
    ColumnName(usize),
    /// The frame and corners on the active column.
    Frame,
    /// A column's arrows, bright on the active column.
    Arrow {
        column: usize,
        up: bool,
    },
}

#[derive(Component)]
pub(crate) struct PlayersRoot;
#[derive(Component)]
pub(crate) struct ModesRoot;

/// A full-screen page with the title backdrop and the header bar.
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
            place(HEADER, (0.0, (SCREEN.1 - HEADER.1) / 2.0)),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
        ));
        if let Some(bar) = art.and_then(|a| a.image("menu_header")) {
            header.insert((sliced(bar, HEADER_SLICE), BackgroundColor(Color::NONE)));
        }
        header.with_children(|h| {
            h.spawn(label(4.5, title));
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
        for (seat, wizard) in WIZARDS.iter().enumerate() {
            let x = seat_x(seat);
            let at = (x, SEAT_Y);
            // Asleep: the grey panel.
            let mut idle = page.spawn((
                place(SEAT_PANEL, at),
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.12)),
                Part::SeatIdle(seat),
            ));
            if let Some(grey) = image("menu_player_select_background_deselected") {
                idle.insert((sliced(grey, [PANEL_SLICE; 4]), BackgroundColor(Color::NONE)));
            }
            // Taken: the coloured panel with turning rays, then the outline.
            let mut taken = page.spawn((
                place(SEAT_PANEL, at),
                BackgroundColor(Color::srgba(1.0, 0.8, 0.3, 0.35)),
                Part::SeatTaken(seat),
            ));
            if let Some(colour) = image("menu_player_select_background_selected") {
                taken.insert((sliced(colour, [PANEL_SLICE; 4]), BackgroundColor(Color::NONE)));
            }
            page.spawn((
                Node {
                    overflow: Overflow::clip(),
                    ..place(SEAT_RAYS_CLIP, at)
                },
                Part::SeatTaken(seat),
            ))
            .with_children(|clip| {
                for layer in 0..5 {
                    if let Some(rays) = image(&format!("menu_sunrays_{}", layer + 1)) {
                        clip.spawn((
                            ImageNode::new(rays),
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Percent((SEAT_RAYS_CLIP.0 - 556.0) / 2.0 / SEAT_RAYS_CLIP.0 * 100.0),
                                top: Val::Percent((SEAT_RAYS_CLIP.1 - 556.0) / 2.0 / SEAT_RAYS_CLIP.1 * 100.0),
                                width: Val::Percent(556.0 / SEAT_RAYS_CLIP.0 * 100.0),
                                height: Val::Percent(556.0 / SEAT_RAYS_CLIP.1 * 100.0),
                                ..default()
                            },
                            UiTransform::default(),
                            Part::SeatRays(seat, layer),
                        ));
                    }
                }
            });
            if let Some(outline) = image("menu_player_select_outline") {
                page.spawn((
                    sliced(outline, [PANEL_SLICE; 4]),
                    place(SEAT_PANEL, at),
                    Part::SeatTaken(seat),
                ));
            }
            page.spawn((ImageNode::default(), place(wizard[0].1, at), Part::SeatWizard(seat)));
            if let Some(arrow) = image("menu_arrow_small") {
                for flip_x in [true, false] {
                    let side = if flip_x { -1.0 } else { 1.0 };
                    page.spawn((
                        ImageNode {
                            image: arrow.clone(),
                            flip_x,
                            ..default()
                        },
                        place(SEAT_ARROW, (x + side * SEAT_ARROW_X, SEAT_Y)),
                        Part::SeatTaken(seat),
                    ));
                }
            }
            // The brick pack panel and the name plate.
            let mut pack = page.spawn((
                place(PACK_PANEL.0, (x, PACK_PANEL.1)),
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
            ));
            if let Some(grey) = image("menu_player_select_background_deselected") {
                pack.insert((sliced(grey, [PANEL_SLICE; 4]), BackgroundColor(Color::NONE)));
            }
            if let Some(corner) = image("menu_corner_small") {
                corners(page, &corner, SEAT_PANEL, at, CORNER_SMALL, Part::SeatTaken(seat));
                corners(page, &corner, PACK_PANEL.0, (x, PACK_PANEL.1), CORNER_SMALL, Part::SeatTaken(seat));
            }
            let mut plate = page.spawn((
                place(NAME_PLATE.0, (x, NAME_PLATE.1)),
                BackgroundColor(Color::srgba(0.1, 0.08, 0.2, 0.85)),
            ));
            if let Some(art) = image("menu_player_select_nameplate") {
                plate.insert((sliced(art, [PLATE_SLICE; 4]), BackgroundColor(Color::NONE)));
            }
            plate.with_children(|p| {
                p.spawn((label(2.4, ""), Part::SeatName(seat)));
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
        let (back_offset, back_size) = DIFFICULTY_BACK;
        page.spawn((
            ImageNode::default(),
            place(back_size, (COLUMN_X[1] + back_offset.0, back_offset.1)),
            Part::DifficultyBack,
        ));
        for (column, &x) in COLUMN_X.iter().enumerate() {
            page.spawn((
                ImageNode::default(),
                BackgroundColor(Color::NONE),
                place((480.0, 393.0), (x, 0.0)),
                Part::ColumnPicture(column),
            ));
            page.spawn(place((480.0, 60.0), (x, -FRAME.1 / 2.0 - 40.0)))
                .with_children(|n| {
                    n.spawn((label(4.0, ""), Part::ColumnName(column)));
                });
            for up in [true, false] {
                let y = (FRAME.1 / 2.0 + ARROW_GAP) * if up { 1.0 } else { -1.0 };
                page.spawn((
                    ImageNode {
                        flip_y: !up,
                        ..default()
                    },
                    place(COLUMN_ARROW, (x, y)),
                    Part::Arrow { column, up },
                ));
            }
        }
        // The frame around the active column's tile, with its corners inside.
        let corner = image("menu_corner_large");
        page.spawn((
            Node {
                border: UiRect::all(Val::Vh(0.4)),
                ..place(FRAME, (COLUMN_X[0] + FRAME_X, 0.0))
            },
            BorderColor::all(Color::srgba(1.0, 0.9, 0.5, 0.9)),
            Part::Frame,
        ))
        .with_children(|frame| {
            let Some(corner) = corner else { return };
            for (flip_x, flip_y) in [(false, false), (true, false), (false, true), (true, true)] {
                let mut node = Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(CORNER_LARGE.0 / FRAME.0 * 100.0),
                    height: Val::Percent(CORNER_LARGE.1 / FRAME.1 * 100.0),
                    ..default()
                };
                if flip_x {
                    node.right = Val::Px(0.0);
                }
                if flip_y {
                    node.bottom = Val::Px(0.0);
                }
                frame.spawn((
                    ImageNode {
                        image: corner.clone(),
                        flip_x,
                        flip_y,
                        ..default()
                    },
                    node,
                ));
            }
        });
        page.spawn(place((SCREEN.0, 60.0), (0.0, -SCREEN.1 / 2.0 + 60.0)))
            .with_children(|p| {
                p.spawn(label(
                    2.2,
                    "Up/Down: change   Left/Right or Enter: next column   Enter or A: play   Esc or B: back",
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
        lobby.column = 0;
        *screen = MenuScreen::Modes;
    }
}

/// "Select Mode": Up/Down change the active column's tile, Left/Right move
/// between the columns, Enter or A moves on from the mode to the difficulty
/// and then plays. Random picks Race, Survival or Puzzle, and a Random
/// difficulty one of the three, as the match starts.
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
        step_back(&mut lobby, &mut screen);
        return;
    }
    if taps.left {
        lobby.column = 0;
    }
    if taps.right {
        lobby.column = 1;
    }
    let count = if lobby.column == 0 {
        lobby.modes().len()
    } else {
        DifficultyPick::ALL.len()
    };
    let at = if lobby.column == 0 {
        &mut lobby.mode
    } else {
        &mut lobby.difficulty
    };
    if taps.up {
        *at = (*at + count - 1) % count;
    }
    if taps.down {
        *at = (*at + 1) % count;
    }
    if !(taps.key_pick || !taps.pad_pick.is_empty()) {
        return;
    }
    if lobby.column == 0 {
        lobby.column = 1;
        return;
    }
    // A roll from the clock is plenty for picking a mode.
    let roll = time.elapsed().as_nanos() as usize / 1000;
    config.mode = match lobby.modes()[lobby.mode.min(lobby.modes().len() - 1)] {
        ModePick::Mode(mode) => mode,
        ModePick::Random => [ModeKind::Race, ModeKind::Survival, ModeKind::Puzzle][roll % 3],
    };
    config.difficulty = Some(match DifficultyPick::ALL[lobby.difficulty] {
        DifficultyPick::Set(d) => d,
        DifficultyPick::Random => Difficulty::ALL[(roll / 3) % Difficulty::ALL.len()],
    });
    if lobby.single {
        config.players = 1;
        config.opponents = 0;
    }
    lobby.column = 0;
    request.pending = true;
}

/// Esc or B on Select Mode: from the difficulty back to the mode, then back a
/// screen (player select, or the title for Single Player).
pub(crate) fn step_back(lobby: &mut Lobby, screen: &mut MenuScreen) {
    if lobby.column == 1 {
        lobby.column = 0;
    } else {
        *screen = if lobby.single {
            MenuScreen::Main
        } else {
            MenuScreen::Players
        };
    }
}

fn shown(on: bool) -> Visibility {
    if on {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    }
}

/// Moves a node to `size` pixels centred at `at`, if it isn't there already.
fn move_node(node: &mut Node, size: (f32, f32), at: (f32, f32)) {
    let want = place(size, at);
    if (node.left, node.top, node.width, node.height)
        != (want.left, want.top, want.width, want.height)
    {
        node.left = want.left;
        node.top = want.top;
        node.width = want.width;
        node.height = want.height;
    }
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(crate) fn show_select_screens(
    time: Res<Time>,
    config: Res<GameConfig>,
    game: Res<Match>,
    screen: Res<MenuScreen>,
    lobby: Res<Lobby>,
    art: Option<Res<Art>>,
    mut roots: ParamSet<(
        Query<&mut Visibility, With<PlayersRoot>>,
        Query<&mut Visibility, With<ModesRoot>>,
    )>,
    mut parts: Query<
        (
            &Part,
            &mut Visibility,
            &mut Node,
            Option<&mut ImageNode>,
            Option<&mut BackgroundColor>,
            Option<&mut UiTransform>,
            Option<&mut Text>,
        ),
        (Without<PlayersRoot>, Without<ModesRoot>),
    >,
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
    if !players_on && !modes_on {
        return;
    }
    let image = |name: &str| art.as_ref().and_then(|a| a.image(name));
    let first_empty = lobby.seats.iter().position(|s| *s == Seat::Empty);
    let player_number = |seat: usize| lobby.seats[..=seat].iter().filter(|s| s.human()).count();
    let modes = lobby.modes();
    let mode = modes[lobby.mode.min(modes.len() - 1)];
    let difficulty = DifficultyPick::ALL[lobby.difficulty.min(DifficultyPick::ALL.len() - 1)];
    let t = time.elapsed_secs();
    for (part, mut v, mut node, image_node, colour, transform, text) in &mut parts {
        let want = match *part {
            Part::SeatIdle(seat) => lobby.seats[seat] == Seat::Empty,
            Part::SeatTaken(seat) => lobby.seats[seat] != Seat::Empty,
            Part::SeatRays(seat, layer) => {
                if let (Some(mut transform), Some(&(degrees, seconds))) =
                    (transform, config.menu.ray_turns.get(layer))
                {
                    transform.rotation = Rot2::degrees(degrees * (t / seconds.max(0.01)).fract());
                }
                lobby.seats[seat] != Seat::Empty
            }
            Part::SeatWizard(seat) => {
                let awake = lobby.seats[seat] != Seat::Empty;
                let pose = if awake { "ready" } else { "sleeping" };
                let (offset, size) = WIZARDS[seat][awake as usize];
                move_node(
                    &mut node,
                    size,
                    (seat_x(seat) + offset.0, SEAT_Y + offset.1),
                );
                match (
                    image(&format!("menu_player_select_wizard{}_{pose}", seat + 1)),
                    image_node,
                ) {
                    (Some(handle), Some(mut img)) => {
                        if img.image != handle {
                            img.image = handle;
                        }
                        true
                    }
                    _ => false,
                }
            }
            Part::SeatName(seat) => {
                let name = match lobby.seats[seat] {
                    Seat::Keyboard | Seat::Pad(_) => format!("Player {}", player_number(seat)),
                    Seat::Ai => "AI Wizard".into(),
                    Seat::Empty if Some(seat) == first_empty => {
                        if lobby.seats.contains(&Seat::Keyboard) {
                            "[A]\nTo join".into()
                        } else {
                            "[Enter]\nTo join".into()
                        }
                    }
                    Seat::Empty => String::new(),
                };
                if let Some(mut text) = text
                    && text.0 != name
                {
                    text.0 = name;
                }
                true
            }
            Part::ColumnPicture(column) => {
                let (name, offset, size, fallback) = if column == 0 {
                    let (n, o, s) = mode.picture();
                    (n, o, s, mode.color())
                } else {
                    let (n, o, s) = difficulty.picture();
                    (n, o, s, Color::srgb(0.3, 0.35, 0.5))
                };
                let x = COLUMN_X[column];
                match (image(name), image_node) {
                    (Some(handle), Some(mut img)) => {
                        move_node(&mut node, size, (x + offset.0, offset.1));
                        if img.image != handle {
                            img.image = handle;
                        }
                        if let Some(mut c) = colour {
                            c.0 = Color::NONE;
                        }
                        true
                    }
                    (_, Some(mut img)) => {
                        // No art: a coloured card the size of the frame.
                        move_node(&mut node, (480.0, 393.0), (x, 0.0));
                        img.image = Handle::default();
                        img.color = Color::NONE;
                        if let Some(mut c) = colour
                            && c.0 != fallback
                        {
                            c.0 = fallback;
                        }
                        true
                    }
                    _ => true,
                }
            }
            Part::DifficultyBack => {
                let name = if difficulty == DifficultyPick::Set(Difficulty::Pro) {
                    "menu_difficulty_background_special"
                } else {
                    "menu_difficulty_background"
                };
                let back = image(name).or_else(|| image("menu_difficulty_background"));
                match (back, image_node) {
                    (Some(handle), Some(mut img)) => {
                        if img.image != handle {
                            img.image = handle;
                        }
                        true
                    }
                    _ => false,
                }
            }
            Part::ColumnName(column) => {
                let (name, staged) = if column == 0 {
                    (mode.name(), image(mode.picture().0).is_some())
                } else {
                    (difficulty.name(), image(difficulty.picture().0).is_some())
                };
                if let Some(mut text) = text
                    && text.0 != name
                {
                    text.0 = name.into();
                }
                !staged
            }
            Part::Frame => {
                // The frame slides to the active column.
                move_node(&mut node, FRAME, (COLUMN_X[lobby.column] + FRAME_X, 0.0));
                true
            }
            Part::Arrow { column, up: _ } => {
                let active = lobby.column == column;
                let name = if active {
                    "menu_arrow"
                } else {
                    "menu_arrow_deselected"
                };
                match (image(name).or_else(|| image("menu_arrow")), image_node) {
                    (Some(handle), Some(mut img)) => {
                        if img.image != handle {
                            img.image = handle;
                        }
                        true
                    }
                    _ => false,
                }
            }
        };
        v.set_if_neq(shown(want));
    }
}
