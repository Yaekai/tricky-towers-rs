//! The original's local-play screens after the title tiles: "Select Players",
//! where people join a seat (and AI wizards can fill the others), then
//! "Select Mode", two columns (the mode, then the difficulty) that each show
//! one tile at a time with arrows above and below.
//! Layout from docs/research/menus.md, in pixels of a 1920×1080 screen.

use bevy::prelude::*;
use bevy::ui::widget::NodeImageMode;
use bevy::window::PrimaryWindow;

use crate::art::Art;
use crate::config::{BrickPack, Difficulty, GameConfig, ModeKind, WIZARD_COUNT};
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


    /// Its name's place across the tile and face colour (ModeItem01-04 and their
    /// EnzoBlack<Mode> materials; Random is milky white).
    fn label(self) -> (f32, Option<Color>) {
        match self {
            ModePick::Mode(ModeKind::Race) => (64.0, Some(Color::srgb(0.729, 0.784, 0.996))),
            ModePick::Mode(ModeKind::Survival) => (-42.0, Some(Color::srgb(0.745, 0.616, 0.937))),
            ModePick::Mode(ModeKind::Puzzle) => (67.0, Some(Color::srgb(0.969, 0.741, 0.384))),
            _ => (-6.0, None),
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
            // Played from the Endless screen's list, not from a tile.
            ModePick::Mode(_) => ("menu_mode_endless", (0.0, 0.0), (495.0, 393.0)),
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
    /// Each seat's wizard (0 for wizard 1) and brick pack, and which of the two
    /// pickers (0 character, 1 brick pack) its player is on.
    pub skins: [(usize, BrickPack); MAX_TOWERS],
    pub skin_row: [usize; MAX_TOWERS],
    /// Down confirms a seat's wizard (row 0) and then its brick pack (row 1); a
    /// confirmed wizard shines.
    pub confirmed: [[bool; 2]; MAX_TOWERS],
}

impl Default for Lobby {
    fn default() -> Self {
        Self {
            seats: [Seat::Empty; MAX_TOWERS],
            single: false,
            mode: 0,
            difficulty: 1,
            column: 0,
            skins: std::array::from_fn(|seat| (seat, BrickPack::Default)),
            skin_row: [0; MAX_TOWERS],
            confirmed: [[false; 2]; MAX_TOWERS],
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

    /// A local battle needs two players at least (PlayerSelectComponent's canStartGame).
    pub fn can_start(&self) -> bool {
        self.seats.iter().filter(|s| **s != Seat::Empty).count() >= 2
    }

    fn ais(&self) -> u32 {
        self.seats.iter().filter(|&&s| s == Seat::Ai).count() as u32
    }

    /// Each tower's (wizard, brick pack), in tower order: people first, then the AI
    /// wizards, each in seat order.
    pub fn tower_skins(&self) -> Vec<(usize, BrickPack)> {
        let humans = (0..MAX_TOWERS).filter(|&s| self.seats[s].human());
        let ais = (0..MAX_TOWERS).filter(|&s| self.seats[s] == Seat::Ai);
        humans.chain(ais).map(|s| self.skins[s]).collect()
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
pub(crate) const SCREEN: (f32, f32) = (1920.0, 1080.0);
/// The header bar hanging from the top centre, and its 9-slice borders
/// (left, right, top, bottom).
const HEADER: (f32, f32) = (800.0, 90.0);
/// gph_menuHeaderBackground's 9-slice borders (left, right, top, bottom): Unity's
/// (84, 75, 92, 15) are left, bottom, right, top.
const HEADER_SLICE: [f32; 4] = [84.0, 92.0, 15.0, 75.0];
/// The header's picture is 100 px taller than the header and reaches up off the screen
/// (the scenes' Header/Background: stretched, 50 up, 100 taller).
const HEADER_BACK_EXTRA: f32 = 100.0;
/// Seats sit in 4 columns between 5% and 95% of the width.
fn seat_x(seat: usize) -> f32 {
    (0.05 + 0.225 * (seat as f32 + 0.5)) * SCREEN.0 - SCREEN.0 / 2.0
}
pub(crate) const SEAT_PANEL: (f32, f32) = (332.0, 283.0);
const SEAT_Y: f32 = 115.0;
/// The sun rays behind a ready wizard are cut to this size.
pub(crate) const SEAT_RAYS_CLIP: (f32, f32) = (327.0, 267.0);
pub(crate) const SEAT_ARROW: (f32, f32) = (38.0, 64.0);
const SEAT_ARROW_X: f32 = 165.0;
pub(crate) const PACK_PANEL: ((f32, f32), f32) = ((332.0, 100.0), -110.0);
/// The brick pack cards are about 252 x 100 px.
const PACK_CARD: (f32, f32) = (252.0, 100.0);
/// The card sits 6 px over the panel's middle (the pack item's Graphic).
const PACK_CARD_UP: f32 = 6.0;
/// A confirmed pack's orange and rays are cut to this (MenuSunraysMasked).
const PACK_RAYS_CLIP: (f32, f32) = (327.0, 95.0);
pub(crate) const NAME_PLATE: ((f32, f32), f32) = ((367.0, 78.0), -230.0);
/// The Shop hint's row, like the hints on the right (400 x 50).
const SHOP_HINT: (f32, f32) = (400.0, 50.0);
/// How far the choosing corners stand out from their panel.
const FOCUS_OUT: f32 = 20.0;
/// The join prompt's key or button picture: size, and how far above the panel's middle
/// (measured on Max's Select Players clip).
const JOIN_GLYPH: (f32, f32) = (60.0, 40.0);
const JOIN_GLYPH_Y: f32 = 28.0;
/// JoinLabel auto-sizes down to 18 px inside the panel less 30 px a side (in view heights,
/// as the label's 3.6).
const JOIN_FIT: crate::letters::AutoSize = crate::letters::AutoSize {
    min: 18.0 / 10.8,
    width: (SEAT_PANEL.0 - 60.0) / 10.8,
};
/// 9-slice borders of the seat panels and outline, and of the name plate.
pub(crate) const PANEL_SLICE: f32 = 60.0;
pub(crate) const PLATE_SLICE: f32 = 13.0;

/// Each wizard's picture on Select Players, asleep and awake: centre offset from
/// the seat panel's centre and size (the original's Character01..14).
/// A picture's centre offset and size, in pixels.
pub(crate) type Spot = ((f32, f32), (f32, f32));
pub(crate) const WIZARDS: [[Spot; 2]; WIZARD_COUNT] = [
    [((0.0, 0.0), (213.0, 250.0)), ((0.0, 0.0), (235.0, 314.0))],
    [((0.0, 38.0), (225.0, 323.0)), ((0.0, 21.0), (259.0, 344.0))],
    [
        ((-9.0, 14.0), (237.0, 275.0)),
        ((-3.0, 4.0), (279.0, 300.0)),
    ],
    [((0.0, 11.7), (212.0, 267.0)), ((7.5, 7.0), (253.0, 322.0))],
    [
        ((-12.0, 1.0), (259.0, 246.0)),
        ((-3.0, -13.0), (307.0, 281.0)),
    ],
    [
        ((16.0, 0.0), (271.0, 244.0)),
        ((-8.0, 18.0), (258.0, 347.0)),
    ],
    [((-1.0, 1.0), (288.0, 245.0)), ((9.0, 11.0), (282.0, 330.0))],
    [((9.0, 9.0), (269.0, 264.0)), ((14.0, 2.0), (282.0, 290.0))],
    [((-3.0, 0.0), (234.0, 258.0)), ((-5.0, 1.0), (231.0, 291.0))],
    [((3.5, -1.6), (299.0, 263.0)), ((5.0, 34.0), (305.0, 385.0))],
    [
        ((-7.1, -2.5), (241.0, 251.0)),
        ((11.4, 25.5), (257.0, 341.0)),
    ],
    [((1.2, 1.2), (290.0, 268.0)), ((-1.6, 6.1), (256.0, 302.0))],
    [
        ((-2.3, -1.4), (215.0, 263.0)),
        ((-0.7, 4.4), (233.0, 302.0)),
    ],
    [((2.0, 3.0), (215.0, 253.0)), ((-6.0, -0.1), (233.0, 280.0))],
];

/// The staged Select Players picture of `wizard` (0 for wizard 1) in `pose`:
/// wizards 1-4, the spirit animals, then the indie guests.
pub(crate) fn seat_picture(wizard: usize, pose: &str) -> String {
    match wizard {
        0..=3 => format!("menu_player_select_wizard{}_{pose}", wizard + 1),
        4..=7 => format!("menu_player_select_wizard_animal0{}_{pose}", wizard - 3),
        _ => format!("menu_player_select_wizard_indie0{}_{pose}", wizard - 7),
    }
}

/// Select Mode's columns: centre x of each, the picked tile's frame, the
/// arrows and where the difficulty tile's background sits.
/// The original lays its panels 540 px apart around the centre; a local battle has
/// three: the mode, the difficulty, and single match or a cup (GameModeSelectStateController).
const COLUMN_X: [f32; 3] = [-540.0, 0.0, 540.0];
/// The cup column's items: Single Match, then each cup, as (picture, offset, size)
/// (CupItem01-04), and their name and "First to N points" centres below the tile.
const CUP_PICTURES: [(&str, (f32, f32), (f32, f32)); 4] = [
    ("menu_quick_match", (0.0, 21.0), (139.0, 167.0)),
    ("trophy", (0.0, 24.0), (111.0, 132.0)),
    ("trophy_big", (-2.0, 13.2), (203.0, 203.0)),
    ("trophy_super", (0.0, 15.3), (307.0, 247.0)),
];
/// The cup column's name: font 60 on a 400 px line (CupItem's Text, 50 px inside the 450).
const CUP_NAME_SIZE: f32 = 60.0;
const CUP_NAME_LINE: f32 = 400.0;
const CUP_NAME_Y: f32 = -95.0;
/// The scroll lists' item pitch and the window they show through (ScrollPanelComponent puts
/// items 430 px apart; on Max's Select Mode clip the window runs from 350 px over the middle to
/// 290 px under it).
const ITEM_PITCH: f32 = 430.0;
const SCROLL_SECONDS: f32 = 0.3;
const WINDOW: ((f32, f32), f32) = ((520.0, 640.0), 30.0);
/// An item's card (the tile pictures are drawn on a 480 x 393 card).
const ITEM_CARD: (f32, f32) = (480.0, 393.0);
/// A mode's or difficulty's name on its tile (ModeItem/DifficultyItem: 280 px down from the
/// item's top, which is 185 px over its middle).
const TILE_NAME_Y: f32 = -95.0;
const CUP_TARGET_Y: f32 = -143.9;
const FRAME: (f32, f32) = (513.0, 445.0);
const FRAME_X: f32 = -5.5;
const COLUMN_ARROW: (f32, f32) = (94.0, 55.0);
/// The arrows sit this far inside the top and bottom of the column's 480×800 panel.
const COLUMN_PANEL_HEIGHT: f32 = 800.0;
const ARROW_INSET: f32 = 130.0;
const DIFFICULTY_BACK: ((f32, f32), (f32, f32)) = ((-5.0, 0.0), (461.0, 393.0));
const CORNER_LARGE: (f32, f32) = (69.0, 68.0);
pub(crate) const CORNER_SMALL: (f32, f32) = (38.0, 34.0);

/// An absolute node `size` pixels big centred at `at`.
pub(crate) fn place(size: (f32, f32), at: (f32, f32)) -> Node {
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

/// A menu text: the original's body letters, near-black, in capitals.
pub(crate) fn label(size: f32, text: &str) -> impl Bundle {
    label_with(size, text, crate::letters::TextLook::BODY)
}

/// A menu text in capitals with the given look.
pub(crate) fn label_with(size: f32, text: &str, look: crate::letters::TextLook) -> impl Bundle {
    (
        Text::new(text.to_uppercase()),
        look,
        TextFont {
            font_size: FontSize::Vh(size),
            ..default()
        },
        TextColor(Color::WHITE),
        TextLayout::justify(Justify::Center),
    )
}

/// A 9-sliced picture with these borders (left, right, top, bottom) in its pixels.
pub(crate) fn sliced(image: Handle<Image>, border: [f32; 4]) -> ImageNode {
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
pub(crate) fn corners(
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

/// "[Tab] SHOP" in the bottom-left corner, mirroring the hints on the right (Max's Select
/// Players and Single Player clips; the screens' OPEN_STORE navigation item).
pub(crate) fn shop_hint(page: &mut ChildSpawnerCommands, art: Option<&Art>) {
    let image = |name: &str| art.and_then(|a| a.image(name));
    page.spawn(Node {
        justify_content: JustifyContent::FlexStart,
        align_items: AlignItems::Center,
        column_gap: Val::Vh(12.0 / SCREEN.1 * 100.0),
        ..place(
            SHOP_HINT,
            (-SCREEN.0 / 2.0 + 30.0 + SHOP_HINT.0 / 2.0, -SCREEN.1 / 2.0 + 50.0 + SHOP_HINT.1 / 2.0),
        )
    })
    .with_children(|row| {
        if let Some(tab) = image("menu_key_tab") {
            row.spawn((
                ImageNode::new(tab),
                Node {
                    height: Val::Vh(SHOP_HINT.1 / SCREEN.1 * 100.0),
                    ..default()
                },
            ));
        }
        row.spawn(label_with(
            46.0 / SCREEN.1 * 100.0,
            "SHOP",
            crate::letters::TextLook::GOLD,
        ));
    });
}

/// What a select-screen node is, so one system can show and update it.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub enum Part {
    /// A seat's grey panel, shown while nobody sits there.
    SeatIdle(usize),
    /// A taken seat's coloured panel, rays, outline, arrows and corners.
    SeatTaken(usize),
    /// A seat's turning rays, by layer.
    SeatRays(usize, usize),
    /// A seat's wizard, asleep or awake.
    SeatWizard(usize),
    /// A seat's brick pack card, and the light panel behind it (while someone sits there).
    SeatPack(usize),
    SeatPackPanel(usize),
    /// A confirmed pack's dark frame, over its orange.
    PackFrame(usize),
    /// A confirmed brick pack's orange sunburst panel, shown behind the card.
    PackRays(usize),
    /// One layer of that sunburst, by layer.
    PackRay(usize, usize),
    /// A seat's name plate text.
    SeatName(usize),
    /// "To join" over an empty seat's panel.
    SeatJoin(usize),
    /// The box holding the join text; the letter copy hides with its parent.
    SeatJoinBox(usize),
    /// The corners on the panel a seated player is choosing on: 0 the character, 1 the pack.
    SeatFocus(usize, usize),
    /// The arrows beside that panel, by row and side.
    SeatArrow(usize, usize, bool),
    /// The join prompt's key or button picture.
    SeatJoinGlyph(usize),
    /// The cup column's name ("Tiny Cup") and target ("First to 6 points").
    CupName,
    CupTarget,
    /// The difficulty's name on its tile.
    ColumnName(usize),
    /// The difficulty tile's light and dark magic slots (0, 1), and what the difficulty takes
    /// away drawn in them.
    MagicSlot(usize),
    MagicIcon(usize),
    /// The window the active column's neighbouring items peek into.
    ColumnWindow(usize),
    /// The current item (0) and the ones before (-1, above) and after (1, below) it in a
    /// column, sliding as the column scrolls: each item's background card and its picture.
    NeighbourBack(usize, i8),
    NeighbourPicture(usize, i8),
    /// A mode's name on its tile, by its place in the mode list (each has its own colour).
    ModeName(usize),
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

/// A full-screen page with the header bar, over the live scene (visuals::MenuScene).
pub(crate) fn spawn_page(commands: &mut Commands, art: Option<&Art>, title: &str) -> Entity {
    let mut page = commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        // The live scene shows through (visuals::MenuScene).
        GlobalZIndex(11),
        Visibility::Hidden,
    ));
    page.with_children(|page| {
        let back = (HEADER.0, HEADER.1 + HEADER_BACK_EXTRA);
        let mut bar = page.spawn((
            place(back, (0.0, (SCREEN.1 - HEADER.1 + HEADER_BACK_EXTRA) / 2.0)),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
        ));
        if let Some(art) = art.and_then(|a| a.image("menu_header")) {
            bar.insert((sliced(art, HEADER_SLICE), BackgroundColor(Color::NONE)));
        }
        page.spawn(place(HEADER, (0.0, (SCREEN.1 - HEADER.1) / 2.0))).with_children(|h| {
            // 60 px, as the scenes' header titles (ModeSelect, PlayerSelect).
            // The scenes' header titles space -1.
            h.spawn((label_with(60.0 / 10.8, title, crate::letters::TextLook::TITLE), crate::letters::LetterSpacing(-1.0)));
        });
    });
    page.id()
}

pub(crate) fn spawn_select_screens(
    mut commands: Commands,
    art: Option<Res<Art>>,
    mut turning: Option<ResMut<Assets<crate::ui_material::TurningPicture>>>,
) {
    let art = art.as_deref();
    let image = |name: &str| art.and_then(|a| a.image(name));

    let players = spawn_page(&mut commands, art, "Select Players");
    commands.entity(players).insert(PlayersRoot);
    commands.entity(players).with_children(|page| {
        shop_hint(page, art);
        for (seat, wizard) in WIZARDS.iter().take(MAX_TOWERS).enumerate() {
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
                taken.insert((
                    sliced(colour, [PANEL_SLICE; 4]),
                    BackgroundColor(Color::NONE),
                ));
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
                        turning_rays(clip, &mut turning, rays, SEAT_RAYS_CLIP, Part::SeatRays(seat, layer));
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
            page.spawn((
                ImageNode::default(),
                place(wizard[0].1, at),
                crate::ui_material::KeepShape,
                Part::SeatWizard(seat),
            ));
            // "To join" fills the empty panel.
            page.spawn((
                place(SEAT_PANEL, at),
                Visibility::Hidden,
                Part::SeatJoinBox(seat),
            ))
            .with_children(|p| {
                p.spawn((
                    ImageNode::default(),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(50.0 - JOIN_GLYPH.0 / 2.0 / SEAT_PANEL.0 * 100.0),
                        top: Val::Percent(
                            50.0 - (JOIN_GLYPH_Y + JOIN_GLYPH.1 / 2.0) / SEAT_PANEL.1 * 100.0,
                        ),
                        width: Val::Percent(JOIN_GLYPH.0 / SEAT_PANEL.0 * 100.0),
                        height: Val::Percent(JOIN_GLYPH.1 / SEAT_PANEL.1 * 100.0),
                        ..default()
                    },
                    Part::SeatJoinGlyph(seat),
                ));
                p.spawn((label_with(3.6, "", crate::letters::TextLook::TITLE), JOIN_FIT, Part::SeatJoin(seat)));
            });
            if let Some(arrow) = image("menu_arrow_small") {
                for (row, y) in [(0, SEAT_Y), (1, PACK_PANEL.1)] {
                    for flip_x in [true, false] {
                        let side = if flip_x { -1.0 } else { 1.0 };
                        page.spawn((
                            ImageNode {
                                image: arrow.clone(),
                                flip_x,
                                ..default()
                            },
                            place(SEAT_ARROW, (x + side * SEAT_ARROW_X, y)),
                            Part::SeatArrow(seat, row, flip_x),
                        ));
                    }
                }
            }
            // The brick pack panel and the name plate.
            // Only a taken seat has its pack panel.
            let mut pack = page.spawn((
                place(PACK_PANEL.0, (x, PACK_PANEL.1)),
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                Part::SeatPackPanel(seat),
            ));
            if let Some(grey) = image("menu_player_select_background_deselected") {
                pack.insert((sliced(grey, [PANEL_SLICE; 4]), BackgroundColor(Color::NONE)));
            }
            if let Some(corner) = image("menu_corner_small") {
                // Only the panel a player is choosing on has corners.
                // They stand 20 px out from the panel's edges (Max's Select Players clip).
                let out = |(w, h): (f32, f32)| (w + 2.0 * FOCUS_OUT, h + 2.0 * FOCUS_OUT);
                corners(page, &corner, out(SEAT_PANEL), at, CORNER_SMALL, Part::SeatFocus(seat, 0));
                corners(
                    page,
                    &corner,
                    out(PACK_PANEL.0),
                    (x, PACK_PANEL.1),
                    CORNER_SMALL,
                    Part::SeatFocus(seat, 1),
                );
            }
            // A confirmed pack (the pack item's Selected): the seat's orange panel and the
            // turning rays cut to 327 x 95, then the dark frame over them, under the card.
            page.spawn((
                Node {
                    overflow: Overflow::clip(),
                    ..place(PACK_RAYS_CLIP, (x, PACK_PANEL.1))
                },
                BackgroundColor(Color::NONE),
                Part::PackRays(seat),
            ))
            .with_children(|clip| {
                if let Some(back) = image("menu_player_select_background_selected") {
                    // The 332 x 283 seat panel, centred, so only its middle shows.
                    let share = (SEAT_PANEL.0 / PACK_RAYS_CLIP.0, SEAT_PANEL.1 / PACK_RAYS_CLIP.1);
                    clip.spawn((
                        sliced(back, [PANEL_SLICE; 4]),
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Percent((1.0 - share.0) * 50.0),
                            top: Val::Percent((1.0 - share.1) * 50.0),
                            width: Val::Percent(share.0 * 100.0),
                            height: Val::Percent(share.1 * 100.0),
                            ..default()
                        },
                    ));
                }
                for layer in 0..5 {
                    if let Some(rays) = image(&format!("menu_sunrays_{}", layer + 1)) {
                        turning_rays(clip, &mut turning, rays, PACK_RAYS_CLIP, Part::PackRay(seat, layer));
                    }
                }
            });
            if let Some(outline) = image("menu_player_select_outline") {
                page.spawn((
                    sliced(outline, [PANEL_SLICE; 4]),
                    place(PACK_PANEL.0, (x, PACK_PANEL.1)),
                    Part::PackFrame(seat),
                ));
            }
            let mut plate = page.spawn((
                Visibility::Hidden,
                place(NAME_PLATE.0, (x, NAME_PLATE.1)),
                BackgroundColor(Color::srgba(0.1, 0.08, 0.2, 0.85)),
            ));
            if let Some(art) = image("menu_player_select_nameplate") {
                plate.insert((sliced(art, [PLATE_SLICE; 4]), BackgroundColor(Color::NONE)));
            }
            plate.with_children(|p| {
                p.spawn((label(2.4, ""), Part::SeatName(seat)));
            });
            // The picked brick pack's card in the pack panel, once someone sits here.
            page.spawn((
                ImageNode::default(),
                place(PACK_CARD, (x, PACK_PANEL.1 + PACK_CARD_UP)),
                Part::SeatPack(seat),
            ));
        }
    });

    let modes = spawn_page(&mut commands, art, "Select Mode");
    commands.entity(modes).insert(ModesRoot);
    commands.entity(modes).with_children(|page| {
        for (part, size, y) in [(Part::CupName, CUP_NAME_SIZE, CUP_NAME_Y), (Part::CupTarget, 28.0, CUP_TARGET_Y)] {
            // Over the column windows spawned after it, or the cards hide it.
            page.spawn((place((CUP_NAME_LINE, 100.0), (COLUMN_X[2], y)), ZIndex(1)))
                .with_children(|n| {
                    // EnzoBlackMilkyWhiteOutline60 and EnzoBlackMilkyWhite26 (CupItem).
                    n.spawn((label_with(size / 10.8, "", crate::letters::TextLook::TITLE), part));
                });
        }
        for (column, &x) in COLUMN_X.iter().enumerate() {
            let (window, wy) = WINDOW;
            page.spawn((
                Node {
                    overflow: Overflow::clip(),
                    ..place(window, (x, wy))
                },
                Part::ColumnWindow(column),
            ))
            .with_children(|w| {
                for side in [-1i8, 0, 1] {
                    // In the window's own pixels, from its middle with y up.
                    let at = (0.0, -(side as f32) * ITEM_PITCH - wy);
                    let in_window = |size: (f32, f32), at: (f32, f32)| Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent((window.0 / 2.0 + at.0 - size.0 / 2.0) / window.0 * 100.0),
                        top: Val::Percent((window.1 / 2.0 - at.1 - size.1 / 2.0) / window.1 * 100.0),
                        width: Val::Percent(size.0 / window.0 * 100.0),
                        height: Val::Percent(size.1 / window.1 * 100.0),
                        ..default()
                    };
                    w.spawn((ImageNode::default(), in_window(ITEM_CARD, at), Part::NeighbourBack(column, side)));
                    w.spawn((ImageNode::default(), in_window(ITEM_CARD, at), Part::NeighbourPicture(column, side)));
                }
            });
            // The name on the tile, 95 px under its middle, 60 px (DifficultyItem: milky
            // white with an outline).
            if column == 1 {
                // The magic slots (DifficultyItem: 56 px at 85 and 145 px right of the middle,
                // 190 px down), with the explanation's icon in them.
                for slot in 0..2 {
                    page.spawn((
                        ImageNode::default(),
                        place((56.0, 56.0), (x + 85.0 + 60.0 * slot as f32, -190.0)),
                        Part::MagicSlot(slot),
                    ))
                    .with_child((
                        ImageNode::default(),
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Percent(3.0 / 56.0 * 100.0),
                            top: Val::Percent(3.0 / 56.0 * 100.0),
                            width: Val::Percent(50.0 / 56.0 * 100.0),
                            height: Val::Percent(50.0 / 56.0 * 100.0),
                            ..default()
                        },
                        Part::MagicIcon(slot),
                    ));
                }
                page.spawn(place((400.0, 100.0), (x, TILE_NAME_Y)))
                    .with_children(|n| {
                        n.spawn((
                            label_with(60.0 / 10.8, "", crate::letters::TextLook::TITLE),
                            Part::ColumnName(column),
                            // DifficultyItem/CupItem names space -1 (SPECIAL and SUPER CUP -2,
                            // set with their words).
                            crate::letters::LetterSpacing(-1.0),
                        ));
                    });
            }
            if column == 0 {
                for (i, pick) in LOCAL_MODES.iter().chain(SINGLE_MODES.iter()).enumerate() {
                    let (dx, face) = pick.label();
                    let look = match face {
                        Some(face) => crate::letters::TextLook {
                            face: Some(face),
                            ..crate::letters::TextLook::OUTLINED
                        },
                        None => crate::letters::TextLook::TITLE,
                    };
                    page.spawn((
                        place((400.0, 100.0), (x + dx, TILE_NAME_Y)),
                        Visibility::Hidden,
                        Part::ModeName(i),
                    ))
                    .with_children(|n| {
                        // ModeItem names space -1, SURVIVAL -2.
                        let spacing = if pick.name().eq_ignore_ascii_case("survival") { -2.0 } else { -1.0 };
                        n.spawn((label_with(60.0 / 10.8, pick.name(), look), crate::letters::LetterSpacing(spacing)));
                    });
                }
            }
            for up in [true, false] {
                let y = (COLUMN_PANEL_HEIGHT / 2.0 - ARROW_INSET) * if up { 1.0 } else { -1.0 };
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
        // The frame around the active column's tile: the four large corners, or
        // a thin outline when they aren't staged.
        let corner = image("menu_corner_large");
        let outline = if corner.is_some() { 0.0 } else { 0.4 };
        page.spawn((
            Node {
                border: UiRect::all(Val::Vh(outline)),
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
                // Pinned to their own edges: the frame centres its children.
                if flip_x {
                    node.right = Val::Px(0.0);
                } else {
                    node.left = Val::Px(0.0);
                }
                if flip_y {
                    node.bottom = Val::Px(0.0);
                } else {
                    node.top = Val::Px(0.0);
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
    });
}

/// Taps on the keyboard and every pad, read once per frame.
#[derive(Default)]
pub(crate) struct Taps {
    pub(crate) up: bool,
    pub(crate) down: bool,
    pub(crate) left: bool,
    pub(crate) right: bool,
    /// Enter/Space on the keyboard.
    pub(crate) key_pick: bool,
    pub(crate) key_add: bool,
    pub(crate) key_remove: bool,
    /// Pads that pressed A, B, X and Y.
    pub(crate) pad_pick: Vec<Entity>,
    pub(crate) pad_back: Vec<Entity>,
    pub(crate) pad_add: bool,
    pub(crate) pad_remove: bool,
}

pub(crate) fn read_taps(
    keys: Option<&ButtonInput<KeyCode>>,
    pads: &Query<(Entity, &Gamepad)>,
) -> Taps {
    let mut taps = Taps::default();
    if let Some(keys) = keys {
        // DEBUG-TOOL: not in the original; remove before the final build (docs/release-checklist.md). (WASD and numpad keys in menus)
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

/// Down on a seat's row confirms it (it shines); the wizard row also moves the
/// cursor to the brick pack row.
fn confirm_row(lobby: &mut Lobby, seat: usize) {
    let row = lobby.skin_row[seat];
    lobby.confirmed[seat][row] = true;
    lobby.skin_row[seat] = (row + 1).min(1);
}

/// Up or Back on a seat undoes its last step (PlayerSlotComponent): a confirmed pack goes
/// back to choosing, then the wizard (it stops shining), then the seat is left. False when
/// there was nothing to undo, so Back leaves the screen.
pub(crate) fn step_back_seat(lobby: &mut Lobby, seat: usize) -> bool {
    match lobby.confirmed[seat] {
        [_, true] => {
            lobby.confirmed[seat][1] = false;
            lobby.skin_row[seat] = 1;
        }
        [true, false] => {
            lobby.confirmed[seat][0] = false;
            lobby.skin_row[seat] = 0;
        }
        _ if lobby.seats[seat] != Seat::Empty => {
            lobby.seats[seat] = Seat::Empty;
            lobby.skin_row[seat] = 0;
        }
        _ => return false,
    }
    true
}

/// Every seated human has confirmed both the wizard and the brick pack, so the next
/// input goes on to the cup and mode choice.
fn everyone_confirmed(lobby: &Lobby) -> bool {
    lobby.humans() > 0
        && lobby
            .seats
            .iter()
            .enumerate()
            .filter(|(_, s)| s.human())
            .all(|(seat, _)| lobby.confirmed[seat] == [true, true])
}

/// "Select Players": Enter seats the keyboard player and A seats a pad; once
/// someone is seated, the same press goes on to the modes. Tab or X adds an
/// AI wizard to the next empty seat, Backspace or Y takes the last one away,
/// and B gets a pad up from its seat.
#[allow(clippy::too_many_arguments)]
pub(crate) fn players_input(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    pads: Query<(Entity, &Gamepad)>,
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    game: Res<Match>,
    mut screen: ResMut<MenuScreen>,
    mut lobby: ResMut<Lobby>,
    mut config: ResMut<GameConfig>,
    art: Option<Res<Art>>,
) {
    if game.phase != Phase::Menu || *screen != MenuScreen::Players {
        return;
    }
    let taps = read_taps(keys.as_deref(), &pads);
    // A click on an empty seat seats the keyboard there; on the keyboard's seat it
    // steps Down (confirm), and on its side arrows Left or Right.
    let mut clicked: Option<(usize, f32)> = None;
    let mut mouse_move = (0, 0);
    if let Ok(window) = windows.single()
        && mouse
            .as_deref()
            .is_some_and(|m| m.just_pressed(MouseButton::Left))
        && let Some(c) = window.cursor_position()
    {
        let p = Vec2::new(
            c.x / window.size().x * SCREEN.0 - SCREEN.0 / 2.0,
            SCREEN.1 / 2.0 - c.y / window.size().y * SCREEN.1,
        );
        for seat in 0..MAX_TOWERS {
            let x = seat_x(seat);
            if (p.x - x).abs() <= SEAT_PANEL.0 / 2.0 && (p.y - SEAT_Y).abs() <= SEAT_PANEL.1 / 2.0 {
                let arrow = if (p.x - (x - SEAT_ARROW_X)).abs() <= SEAT_ARROW.0 / 2.0 {
                    -1.0
                } else if (p.x - (x + SEAT_ARROW_X)).abs() <= SEAT_ARROW.0 / 2.0 {
                    1.0
                } else {
                    0.0
                };
                clicked = Some((seat, arrow));
            }
        }
    }
    if let Some((seat, arrow)) = clicked {
        if lobby.seats[seat] == Seat::Empty {
            if !lobby.seats.contains(&Seat::Keyboard) {
                lobby.seat(Seat::Keyboard);
            }
        } else if lobby.seats[seat] == Seat::Keyboard {
            mouse_move = (arrow as i32, if arrow == 0.0 { 1 } else { 0 });
        }
    }
    let mut go = false;
    if taps.key_pick {
        if let Some(seat) = lobby.seats.iter().position(|s| *s == Seat::Keyboard) {
            if everyone_confirmed(&lobby) {
                go = lobby.can_start();
            } else {
                confirm_row(&mut lobby, seat);
            }
        } else {
            lobby.seat(Seat::Keyboard);
        }
    }
    for pad in taps.pad_pick {
        if let Some(seat) = lobby.seats.iter().position(|s| *s == Seat::Pad(pad)) {
            if everyone_confirmed(&lobby) {
                go = lobby.can_start();
            } else {
                confirm_row(&mut lobby, seat);
            }
        } else {
            lobby.seat(Seat::Pad(pad));
        }
    }
    for pad in taps.pad_back {
        if let Some(seat) = lobby.seats.iter().position(|s| *s == Seat::Pad(pad)) {
            step_back_seat(&mut lobby, seat);
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
    // Each seated person steers their own pickers: Up/Down between Character and
    // Brick pack, Left/Right through what's there (docs/research/skins.md).
    let wizards = available_wizards(art.as_deref());
    let packs = available_packs(art.as_deref());
    let mut moves: Vec<(Seat, i32, i32)> = Vec::new();
    moves.push((Seat::Keyboard, mouse_move.0, mouse_move.1));
    if let Some(keys) = keys.as_deref() {
        let dx = keys.just_pressed(KeyCode::ArrowRight) as i32
            - keys.just_pressed(KeyCode::ArrowLeft) as i32;
        let dy = keys.just_pressed(KeyCode::ArrowDown) as i32
            - keys.just_pressed(KeyCode::ArrowUp) as i32;
        moves.push((Seat::Keyboard, dx, dy));
    }
    for (entity, pad) in &pads {
        let dx = pad.just_pressed(GamepadButton::DPadRight) as i32
            - pad.just_pressed(GamepadButton::DPadLeft) as i32;
        let dy = pad.just_pressed(GamepadButton::DPadDown) as i32
            - pad.just_pressed(GamepadButton::DPadUp) as i32;
        moves.push((Seat::Pad(entity), dx, dy));
    }
    for (who, dx, dy) in moves {
        let Some(seat) = lobby.seats.iter().position(|s| *s == who) else {
            continue;
        };
        // Down on a row confirms it, and on the wizard row also moves to the packs.
        if dy > 0 {
            confirm_row(&mut lobby, seat);
        } else if dy < 0 {
            step_back_seat(&mut lobby, seat);
        }
        // Only the row being chosen turns (the other is confirmed or not reached yet).
        if lobby.confirmed[seat][lobby.skin_row[seat]] {
            continue;
        }
        if dx != 0 {
            let (wizard, pack) = lobby.skins[seat];
            if lobby.skin_row[seat] == 0 {
                lobby.skins[seat].0 = step_in(&wizards, wizard, dx);
            } else {
                lobby.skins[seat].1 = step_in(&packs, pack, dx);
            }
        }
    }
    if go && lobby.humans() > 0 {
        config.players = lobby.humans();
        config.opponents = lobby.ais();
        config.skins = lobby.tower_skins();
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
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
    game: Res<Match>,
    mut screen: ResMut<MenuScreen>,
    mut lobby: ResMut<Lobby>,
    mut config: ResMut<GameConfig>,
    mut request: ResMut<MatchRequest>,
    mut cup: ResMut<crate::cups::Cup>,
    mut last_cursor: Local<Option<Vec2>>,
) {
    if game.phase != Phase::Menu || *screen != MenuScreen::Modes {
        return;
    }
    let mut taps = read_taps(keys.as_deref(), &pads);
    // The mouse: a column under the cursor is selected as it moves; a click on its
    // arrows steps it, and a click on its tile is Enter.
    if let Ok(window) = windows.single() {
        let cursor = window.cursor_position();
        if let Some(c) = cursor {
            let p = Vec2::new(
                c.x / window.size().x * SCREEN.0 - SCREEN.0 / 2.0,
                SCREEN.1 / 2.0 - c.y / window.size().y * SCREEN.1,
            );
            let column = COLUMN_X
                .iter()
                .position(|&x| (p.x - x).abs() <= 240.0 && p.y.abs() <= 400.0);
            if let Some(col) = column
                && cursor != *last_cursor
            {
                lobby.column = col;
            }
            if mouse
                .as_deref()
                .is_some_and(|m| m.just_pressed(MouseButton::Left))
            {
                if let Some(col) = column {
                    lobby.column = col;
                    let arrow = COLUMN_PANEL_HEIGHT / 2.0 - ARROW_INSET;
                    let x = COLUMN_X[col];
                    if (p.x - x).abs() <= 80.0 && (p.y - arrow).abs() <= 40.0 {
                        taps.up = true;
                    } else if (p.x - x).abs() <= 80.0 && (p.y + arrow).abs() <= 40.0 {
                        taps.down = true;
                    } else {
                        taps.key_pick = true;
                    }
                }
            }
        }
        *last_cursor = cursor;
    }
    if !taps.pad_back.is_empty() {
        step_back(&mut lobby, &mut screen);
        return;
    }
    if taps.left {
        lobby.column = lobby.column.saturating_sub(1);
    }
    if taps.right {
        lobby.column = (lobby.column + 1).min(COLUMN_X.len() - 1);
    }
    let count = match lobby.column {
        0 => lobby.modes().len(),
        1 => DifficultyPick::ALL.len(),
        _ => config.cups.sizes.len() + 1,
    };
    let mut item = cup.pick.map_or(0, |i| i + 1);
    let at = match lobby.column {
        0 => &mut lobby.mode,
        1 => &mut lobby.difficulty,
        _ => &mut item,
    };
    if taps.up {
        *at = (*at + count - 1) % count;
    }
    if taps.down {
        *at = (*at + 1) % count;
    }
    cup.pick = item.checked_sub(1);
    if !(taps.key_pick || !taps.pad_pick.is_empty()) {
        return;
    }
    if lobby.column < COLUMN_X.len() - 1 {
        lobby.column += 1;
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
    config.trial = None;
    // The picked cup (or single matches) starts afresh. A cup with a Random mode or
    // difficulty picks a new pair for each match.
    let modes: Vec<ModeKind> = match lobby.modes()[lobby.mode.min(lobby.modes().len() - 1)] {
        ModePick::Mode(mode) => vec![mode],
        ModePick::Random => vec![ModeKind::Race, ModeKind::Survival, ModeKind::Puzzle],
    };
    let difficulties: Vec<Difficulty> = match DifficultyPick::ALL[lobby.difficulty] {
        DifficultyPick::Set(d) => vec![d],
        DifficultyPick::Random => Difficulty::ALL.to_vec(),
    };
    let options = modes
        .iter()
        .flat_map(|&m| difficulties.iter().map(move |&d| (m, d)))
        .collect();
    cup.current = config.difficulty.map(|d| (config.mode, d));
    cup.options = options;
    cup.begin();
    request.pending = true;
}

/// Esc or B on Select Mode: from the difficulty back to the mode, then back a
/// screen (player select, or the title for Single Player).
pub(crate) fn step_back(lobby: &mut Lobby, screen: &mut MenuScreen) {
    if lobby.column > 0 {
        lobby.column -= 1;
    } else {
        *screen = if lobby.single {
            MenuScreen::Main
        } else {
            MenuScreen::Players
        };
    }
}

pub(crate) fn shown(on: bool) -> Visibility {
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
    cup: Res<crate::cups::Cup>,
    art: Option<Res<Art>>,
    pads: Query<(Entity, &Gamepad)>,
    mut turning: Option<ResMut<Assets<crate::ui_material::TurningPicture>>>,
    mut scroll: Local<[(usize, f32, f32); 3]>,
    mut was_modes: Local<bool>,
    mut pops: Local<[Option<f32>; MAX_TOWERS]>,
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
            Option<&mut UiTransform>,
            Option<&mut Text>,
            Option<&MaterialNode<crate::ui_material::TurningPicture>>,
            Option<&mut TextFont>,
            Option<&mut crate::letters::LetterSpacing>,
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
        *was_modes = false;
        return;
    }
    let image = |name: &str| art.as_ref().and_then(|a| a.image(name));
    // The cup column: 0 is a single match, then each cup.
    let cup_item = cup.pick.map_or(0, |i| i + 1);
    let player_number = |seat: usize| lobby.seats[..=seat].iter().filter(|s| s.human()).count();
    let modes = lobby.modes();
    let mode = modes[lobby.mode.min(modes.len() - 1)];
    let difficulty = DifficultyPick::ALL[lobby.difficulty.min(DifficultyPick::ALL.len() - 1)];
    let t = time.elapsed_secs();
    // A turning layer's turn now.
    let mut turn_layer = |layer: usize, transform: Option<Mut<UiTransform>>, picture: Option<&MaterialNode<crate::ui_material::TurningPicture>>| {
        if let Some(&(degrees, seconds)) = config.menu.ray_turns.get(layer) {
            let turn = Rot2::degrees(degrees * (t / seconds.max(0.01)).fract());
            match (picture, turning.as_mut()) {
                (Some(picture), Some(turning)) => {
                    crate::ui_material::set_turn(turning, &picture.0, turn.as_radians())
                }
                _ => {
                    if let Some(mut transform) = transform {
                        transform.rotation = turn;
                    }
                }
            }
        }
    };
    // Each device not yet seated offers a join prompt on an empty seat, pads first, then the
    // keyboard; the other empty seats ask for a controller (PlayerSlotComponent).
    let mut prompts: Vec<&'static str> = Vec::new();
    for (entity, _) in &pads {
        if !lobby.seats.contains(&Seat::Pad(entity)) {
            prompts.push(pad_join_glyph(&pads, entity));
        }
    }
    if !lobby.seats.contains(&Seat::Keyboard) {
        prompts.push("menu_key_space");
    }
    let join_glyph = |seat: usize| {
        let empty_before = lobby.seats[..seat].iter().filter(|s| **s == Seat::Empty).count();
        prompts.get(empty_before).copied()
    };
    let seated = |seat: usize| lobby.seats[seat] != Seat::Empty;
    // The character is ready (orange, awake) once confirmed (CharacterScrollPanelItemRenderer).
    let ready = |seat: usize| seated(seat) && (!lobby.seats[seat].human() || lobby.confirmed[seat][0]);
    // The row a seated person is choosing on, if any.
    let choosing = |seat: usize| {
        let row = lobby.skin_row[seat].min(1);
        (lobby.seats[seat].human() && !lobby.confirmed[seat][row]).then_some(row)
    };
    // Each column slides to its new item over 0.3 s, out-quad (ScrollPanelComponent): `slide`
    // is how far the items still are from their places, in items, signed by the direction.
    let counts = [modes.len(), DifficultyPick::ALL.len(), CUP_PICTURES.len().min(config.cups.sizes.len() + 1)];
    let indexes = [lobby.mode, lobby.difficulty, cup_item];
    let mut slide = [0.0f32; 3];
    // The screen opens with every column at rest.
    let opening = !*was_modes;
    *was_modes = modes_on;
    for c in 0..3 {
        let (last, since, dir) = &mut scroll[c];
        if opening || !modes_on {
            *last = indexes[c];
            *since = SCROLL_SECONDS;
        } else if *last != indexes[c] {
            let n = counts[c].max(1);
            *dir = if indexes[c] == (*last + 1) % n { 1.0 } else { -1.0 };
            *last = indexes[c];
            *since = 0.0;
        } else {
            *since += time.delta_secs();
        }
        let x = (*since / SCROLL_SECONDS).min(1.0);
        slide[c] = *dir * (1.0 - x) * (1.0 - x);
    }
    // A name shows once its item is less than half an item from its place (its alpha is
    // 1 - 2 x the distance in the original).
    let named = |c: usize| slide[c].abs() < 0.5;
    // A pack card pops when confirmed: 1.1 wide back to 1 over 0.6 s, out-elastic
    // (BrickPackScrollPanelItemRenderer.SetReady).
    for seat in 0..MAX_TOWERS {
        pops[seat] = lobby.confirmed[seat][1].then(|| pops[seat].map_or(0.0, |t| t + time.delta_secs()));
    }
    for (part, mut v, mut node, image_node, transform, text, picture, font, spacing) in &mut parts {
        let want = match *part {
            Part::SeatIdle(seat) => !ready(seat),
            Part::SeatTaken(seat) => ready(seat),
            Part::SeatPackPanel(seat) => seated(seat),
            Part::SeatRays(seat, layer) => {
                turn_layer(layer, transform, picture);
                ready(seat)
            }
            Part::SeatArrow(seat, row, _) | Part::SeatFocus(seat, row) => choosing(seat) == Some(row),
            Part::SeatJoinGlyph(seat) => {
                match (join_glyph(seat).and_then(|g| image(g)), image_node) {
                    (Some(handle), Some(mut img)) if lobby.seats[seat] == Seat::Empty => {
                        if img.image != handle {
                            img.image = handle;
                        }
                        // The picture sits on the label's first (empty) line, which rises as
                        // the translated label takes more lines (the block stays centred).
                        let (lines, line) = art
                            .as_deref()
                            .map_or((2, 0.0), |art| crate::letters::translated_lines(art, "\nTO JOIN", 3.6, &JOIN_FIT));
                        let y = JOIN_GLYPH_Y + lines.saturating_sub(2) as f32 * line * 10.8 / 2.0;
                        let top = Val::Percent(50.0 - (y + JOIN_GLYPH.1 / 2.0) / SEAT_PANEL.1 * 100.0);
                        if node.top != top {
                            node.top = top;
                        }
                        true
                    }
                    _ => false,
                }
            }
            Part::SeatWizard(seat) if !seated(seat) => false,
            Part::SeatWizard(seat) => {
                let awake = ready(seat);
                let pose = if awake { "ready" } else { "sleeping" };
                let picked = lobby.skins[seat].0.min(WIZARD_COUNT - 1);
                let (offset, size) = WIZARDS[picked][awake as usize];
                move_node(
                    &mut node,
                    size,
                    (seat_x(seat) + offset.0, SEAT_Y + offset.1),
                );
                // The picked wizard's picture, else the seat's own.
                match (
                    image(&seat_picture(picked, pose)).or_else(|| image(&seat_picture(seat, pose))),
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
            Part::SeatPack(seat) => {
                // The Spring pack's card is the original's Easter one.
                let key = match lobby.skins[seat].1 {
                    BrickPack::Spring => "easter",
                    pack => pack.key(),
                };

                if let Some(mut t) = transform {
                    let pop = pops[seat].map_or(0.0, |t| 1.0 - crate::packs::out_elastic((t / 0.6).min(1.0)));
                    let want = Vec2::new(1.0 + 0.1 * pop, 1.0);
                    if t.scale != want {
                        t.scale = want;
                    }
                }
                match (
                    image(&format!("brick_pack_{key}"))
                        .filter(|_| lobby.seats[seat] != Seat::Empty),
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
                    Seat::Empty => String::new(),
                };
                if let Some(mut text) = text
                    && text.0 != name
                {
                    text.0 = name;
                }
                true
            }
            Part::SeatJoin(seat) => {
                // Under the key or button picture when this seat offers a join.
                let join = if join_glyph(seat).is_some() {
                    "\nTO JOIN"
                } else {
                    "CONNECT\nCONTROLLER"
                };
                if let Some(mut text) = text
                    && text.0 != join
                {
                    text.0 = join.into();
                }
                true
            }
            Part::SeatJoinBox(seat) => lobby.seats[seat] == Seat::Empty,
            Part::PackRays(seat) | Part::PackFrame(seat) => lobby.confirmed[seat][1],
            Part::PackRay(seat, layer) => {
                turn_layer(layer, transform, picture);
                lobby.confirmed[seat][1]
            }
            Part::CupName | Part::CupTarget if !named(2) => false,
            Part::CupName | Part::CupTarget => {
                let size = cup_item
                    .checked_sub(1)
                    .and_then(|i| config.cups.sizes.get(i));
                let words = match (*part, size) {
                    (Part::CupName, Some(size)) => size.name.to_uppercase(),
                    (Part::CupName, None) => "SINGLE MATCH".to_string(),
                    (_, Some(size)) => format!("First to {} points", size.target),
                    (_, None) => String::new(),
                };
                // The name auto-sizes (TMP, 60 down to 18) to fit its 400 px line.
                if *part == Part::CupName
                    && let (Some(art), Some(mut font)) = (art.as_ref(), font)
                {
                    let wide = crate::letters::translated_width(art, &words, CUP_NAME_SIZE);
                    let size = if wide > CUP_NAME_LINE {
                        (CUP_NAME_SIZE * CUP_NAME_LINE / wide).max(18.0)
                    } else {
                        CUP_NAME_SIZE
                    };
                    let size = FontSize::Vh(size / 10.8);
                    if font.font_size != size {
                        font.font_size = size;
                    }
                }
                if let Some(mut text) = text
                    && text.0 != words
                {
                    text.0 = words;
                }
                true
            }
            Part::ColumnName(2) => false,
            Part::ColumnName(_) if !named(1) => false,
            Part::ColumnName(_) => {
                let name = difficulty.name().to_uppercase();
                // DifficultyItem03 (SPECIAL) spaces -2, the others -1.
                if let Some(mut spacing) = spacing {
                    spacing.set_if_neq(crate::letters::LetterSpacing(if name == "SPECIAL" { -2.0 } else { -1.0 }));
                }
                if let Some(mut text) = text
                    && text.0 != name
                {
                    text.0 = name;
                }
                true
            }
            Part::MagicSlot(_) | Part::MagicIcon(_) if !named(1) => false,
            Part::MagicSlot(slot) | Part::MagicIcon(slot) => {
                // Random shows no slots; Endless and Random modes read as Race.
                let kind = match mode {
                    ModePick::Mode(kind) => kind.rules(),
                    ModePick::Random => ModeKind::Race,
                };
                let name = match (difficulty, *part) {
                    (DifficultyPick::Random, _) => None,
                    (_, Part::MagicSlot(_)) => Some(if slot == 0 { "spell_slot_light" } else { "spell_slot_dark" }),
                    (DifficultyPick::Set(d), _) => crate::explain::slots(kind, d)[slot],
                };
                match (name.and_then(|n| image(n)), image_node) {
                    (Some(handle), Some(mut img)) => {
                        if img.image != handle {
                            img.image = handle;
                        }
                        true
                    }
                    _ => false,
                }
            }
            Part::ColumnWindow(_) => true,
            // Only the active column shows the items around its current one.
            Part::NeighbourBack(column, side) | Part::NeighbourPicture(column, side)
                if side != 0 && column != lobby.column && slide[column] == 0.0 =>
            {
                false
            }
            Part::NeighbourBack(column, side) | Part::NeighbourPicture(column, side) => {
                let count = match column {
                    0 => modes.len(),
                    1 => DifficultyPick::ALL.len(),
                    _ => CUP_PICTURES.len().min(config.cups.sizes.len() + 1),
                };
                let at = match column {
                    0 => lobby.mode,
                    1 => lobby.difficulty,
                    _ => cup_item,
                };
                let k = (at as i32 + side as i32).rem_euclid(count.max(1) as i32) as usize;
                let back = matches!(*part, Part::NeighbourBack(..));
                // The card: the difficulty background (Special's for Pro) behind the
                // difficulty and cup pictures; mode pictures are whole cards.
                let (name, offset, size) = match (column, back) {
                    (0, true) => ("", (0.0, 0.0), ITEM_CARD),
                    (0, false) => modes[k].picture(),
                    (1, true) => (
                        if DifficultyPick::ALL[k] == DifficultyPick::Set(Difficulty::Pro) {
                            "menu_difficulty_background_special"
                        } else {
                            "menu_difficulty_background"
                        },
                        DIFFICULTY_BACK.0,
                        DIFFICULTY_BACK.1,
                    ),
                    (1, false) => DifficultyPick::ALL[k].picture(),
                    (_, true) => ("menu_difficulty_background", DIFFICULTY_BACK.0, DIFFICULTY_BACK.1),
                    (_, false) => CUP_PICTURES[k],
                };
                let (window, wy) = WINDOW;
                let at = (offset.0, offset.1 - (side as f32 + slide[column]) * ITEM_PITCH - wy);
                node.left = Val::Percent((window.0 / 2.0 + at.0 - size.0 / 2.0) / window.0 * 100.0);
                node.top = Val::Percent((window.1 / 2.0 - at.1 - size.1 / 2.0) / window.1 * 100.0);
                node.width = Val::Percent(size.0 / window.0 * 100.0);
                node.height = Val::Percent(size.1 / window.1 * 100.0);
                match (image(name).or_else(|| (column == 1 && back).then(|| image("menu_difficulty_background")).flatten()), image_node) {
                    (Some(handle), Some(mut img)) => {
                        if img.image != handle {
                            img.image = handle;
                        }
                        true
                    }
                    _ => false,
                }
            }
            Part::ModeName(_) if !named(0) => false,
            Part::ModeName(i) => LOCAL_MODES.iter().chain(SINGLE_MODES.iter()).nth(i) == Some(&mode),
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

/// The wizards to pick from: those whose art is staged (all of them without art).
pub(crate) fn available_wizards(art: Option<&Art>) -> Vec<usize> {
    let staged: Vec<usize> = (0..WIZARD_COUNT)
        .filter(|w| art.is_some_and(|a| a.entries.contains_key(&format!("wizard{}", w + 1))))
        .collect();
    if staged.is_empty() {
        (0..WIZARD_COUNT).collect()
    } else {
        staged
    }
}

/// The brick packs to pick from: the default, and every pack whose art is staged (all
/// of them without art). Hologram draws the default bricks, so it is always there.
pub(crate) fn available_packs(art: Option<&Art>) -> Vec<BrickPack> {
    let Some(art) = art.filter(|a| a.image("brick_I").is_some()) else {
        return BrickPack::ALL.to_vec();
    };
    BrickPack::ALL
        .into_iter()
        .filter(|pack| match pack {
            BrickPack::Default | BrickPack::Hologram => true,
            BrickPack::Galaxy => art.image("brick_galaxy_background").is_some(),
            pack => art.image(&pack.brick_art("I")).is_some(),
        })
        .collect()
}

/// The entry `dx` steps from `current` in `list`, wrapping around.
fn step_in<T: Copy + PartialEq>(list: &[T], current: T, dx: i32) -> T {
    if list.is_empty() {
        return current;
    }
    let at = list.iter().position(|&x| x == current).unwrap_or(0) as i32;
    let n = list.len() as i32;
    list[((at + dx) % n + n) as usize % list.len()]
}

/// A wizard's name on the character picker (docs/research/skins.md).
pub fn wizard_name(wizard: usize) -> String {
    match wizard {
        0..=3 => format!("Wizard {}", wizard + 1),
        4..=7 => format!("Spirit Animal {}", wizard - 3),
        8 => "Candyman".into(),
        9 => "Clunk".into(),
        10 => "Last of the Order".into(),
        11 => "Thane".into(),
        12 => "Fish".into(),
        13 => "Octodad".into(),
        _ => format!("Wizard {}", wizard + 1),
    }
}

/// One turning sunray layer filling a panel of `size`, cut to it: drawn by the turning
/// material when there is one, else a turned picture (which isn't cut).
pub(crate) fn turning_rays(
    clip: &mut ChildSpawnerCommands,
    turning: &mut Option<ResMut<Assets<crate::ui_material::TurningPicture>>>,
    rays: Handle<Image>,
    size: (f32, f32),
    part: impl Bundle,
) {
    // The ray pictures are 556 px square.
    let share = Vec2::new(556.0 / size.0, 556.0 / size.1);
    match turning.as_mut() {
        Some(turning) => {
            clip.spawn((
                MaterialNode(turning.add(crate::ui_material::TurningPicture::new(rays, share))),
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                part,
            ));
        }
        None => {
            clip.spawn((
                ImageNode::new(rays),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent((1.0 - share.x) * 50.0),
                    top: Val::Percent((1.0 - share.y) * 50.0),
                    width: Val::Percent(share.x * 100.0),
                    height: Val::Percent(share.y * 100.0),
                    ..default()
                },
                UiTransform::default(),
                part,
            ));
        }
    }
}

/// The picture of the button that joins on `pad`: Cross on a PlayStation pad, else A.
fn pad_join_glyph(pads: &Query<(Entity, &Gamepad)>, pad: Entity) -> &'static str {
    let playstation = pads
        .get(pad)
        .ok()
        .and_then(|(_, p)| p.vendor_id())
        .is_some_and(|v| v == 0x054C);
    if playstation { "menu_pad_cross" } else { "menu_pad_a" }
}
