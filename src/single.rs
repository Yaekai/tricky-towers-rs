//! Single Player: the player's seat over a short list (Trials, Endless), then
//! the Endless screen, three wide buttons (Race, Survival, Puzzle) beside a
//! leaderboard. Layout from docs/research/menus.md, in pixels of a 1920×1080
//! screen. Endless Survival and
//! Endless Puzzle follow public descriptions until their values are read (?).

use std::path::PathBuf;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::art::Art;
use crate::config::{Difficulty, GameConfig, ModeKind};
use crate::gameplay::{Match, MatchRequest, Phase, Tower, TowerState};
use crate::menu::MenuScreen;
use crate::select::{
    CORNER_SMALL, PACK_PANEL, PANEL_SLICE, SEAT_PANEL, WIZARDS, corners, label, place, read_taps,
    shown, sliced, spawn_page,
};

/// The seat sits this far above the centre, its brick pack panel this far
/// below the seat's centre; the list starts below them.
const SEAT_Y: f32 = 200.0;
const PACK_BELOW_SEAT: f32 = 225.0;
const LIST_BUTTON: (f32, f32) = (550.0, 78.0);
const LIST_TOP: f32 = -175.0;
const LIST_STEP: f32 = 100.0;
/// `menu_button_background`'s 9-slice borders (left, right, top, bottom), the
/// gap between a picked button and its corners, and the cup band's height.
const BUTTON_SLICE: [f32; 4] = [18.0, 16.0, 71.0, 15.0];
const CORNER_GAP: f32 = 10.0;
const CUP_BAND: f32 = 54.0;
/// The Endless buttons down the right: the column's top-left corner, the slot
/// each button fills and the pitch between them.
const ENDLESS_TOP_LEFT: (f32, f32) = (135.0, 375.0);
const ENDLESS_BUTTON: (f32, f32) = (690.0, 173.0);
const ENDLESS_STEP: f32 = 200.0;
/// A wizard face's size and its bottom-left corner from the button's bottom-left.
const FACE: (f32, f32) = (110.0, 108.0);
const FACE_AT: (f32, f32) = (33.0, 6.0);
/// The leaderboard on the left, its right edge on the centre line, 30 px below
/// the centre; its panel's 9-slice borders and its rows'.
const BOARD: (f32, f32) = (800.0, 800.0);
const BOARD_AT: (f32, f32) = (-400.0, -30.0);
const BOARD_SLICE: [f32; 4] = [0.0, 0.0, 51.0, 37.0];
const ROW_SLICE: [f32; 4] = [6.0, 7.0, 0.0, 0.0];
/// Rows down the leaderboard (guessed until read): size, the first row's
/// centre from the board's top and the pitch.
const ROW: (f32, f32) = (700.0, 56.0);
const ROW_TOP: f32 = 130.0;
const ROW_STEP: f32 = 62.0;
/// Scores kept per Endless mode.
const BOARD_LENGTH: usize = 10;

/// A Single Player list entry: its name and whether it's built yet.
const LIST: [(&str, bool); 2] = [("Trials", true), ("Endless", true)];

/// An Endless button: its name, picture (its size and how far up it sits in
/// the slot), wizard face and whether it's built.
struct EndlessEntry {
    name: &'static str,
    picture: &'static str,
    size: (f32, f32),
    lift: f32,
    face: &'static str,
    built: bool,
    color: Color,
    mode: ModeKind,
}

const ENDLESS: [EndlessEntry; 3] = [
    EndlessEntry {
        name: "Race",
        picture: "menu_mode_endless",
        size: (690.0, 173.0),
        lift: 0.0,
        face: "menu_challenge_wizard3",
        built: true,
        color: Color::srgb(0.85, 0.35, 0.2),
        mode: ModeKind::Endless,
    },
    EndlessEntry {
        name: "Survival",
        picture: "menu_mode_endless_survival",
        size: (689.0, 167.0),
        lift: 0.0,
        face: "menu_challenge_wizard2",
        // Rules from public descriptions; values are guesses until read (?).
        built: true,
        color: Color::srgb(0.25, 0.55, 0.85),
        mode: ModeKind::EndlessSurvival,
    },
    EndlessEntry {
        name: "Puzzle",
        picture: "menu_mode_endless_puzzle",
        size: (690.0, 224.0),
        lift: 28.0,
        face: "menu_challenge_wizard4",
        // Rules from public descriptions; values are guesses until read (?).
        built: true,
        color: Color::srgb(0.55, 0.35, 0.75),
        mode: ModeKind::EndlessPuzzle,
    },
];

/// Where the cursor is on the two screens.
#[derive(Resource, Debug)]
pub struct SingleMenu {
    /// Index into the Trials/Endless list. Starts on Endless, the one that plays.
    pub selected: usize,
    /// Index into Race, Survival, Puzzle.
    pub endless: usize,
}

impl Default for SingleMenu {
    fn default() -> Self {
        Self {
            selected: 1,
            endless: 0,
        }
    }
}

/// Best Endless scores, best first, per Endless mode. Kept between runs in
/// `file` when one is set (the game sets it; tests and offscreen pictures don't).
#[derive(Resource, Debug, Default, Serialize, Deserialize)]
pub struct Leaderboard {
    pub scores: [Vec<u32>; 3],
    #[serde(skip)]
    pub file: Option<PathBuf>,
}

impl Leaderboard {
    pub fn add(&mut self, mode: usize, score: u32) {
        let list = &mut self.scores[mode];
        list.push(score);
        list.sort_unstable_by(|a, b| b.cmp(a));
        list.truncate(BOARD_LENGTH);
    }

    /// The board saved in `file`, or an empty one that will save there.
    pub fn load(file: PathBuf) -> Self {
        let mut board: Self = std::fs::read_to_string(&file)
            .ok()
            .and_then(|text| ron::from_str(&text).ok())
            .unwrap_or_default();
        for list in &mut board.scores {
            list.sort_unstable_by(|a, b| b.cmp(a));
            list.truncate(BOARD_LENGTH);
        }
        board.file = Some(file);
        board
    }

    /// Writes the board to its file, if it has one. A failed save is logged, not fatal.
    pub fn save(&self) {
        let Some(file) = &self.file else {
            return;
        };
        let written = file
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|_| {
                let text = ron::to_string(self).map_err(std::io::Error::other)?;
                std::fs::write(file, text)
            });
        if let Err(err) = written {
            warn!(
                "could not save the leaderboard to {}: {err}",
                file.display()
            );
        }
    }
}

/// Folder for saved data: `TT_SAVE_DIR` if set, else `%APPDATA%\tricky-towers-rs` on
/// Windows and `$XDG_DATA_HOME/tricky-towers-rs` (or `~/.local/share/...`) elsewhere.
pub fn save_dir() -> PathBuf {
    let env = |name: &str| {
        std::env::var_os(name)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    if let Some(dir) = env("TT_SAVE_DIR") {
        return dir;
    }
    let base = env("APPDATA")
        .or_else(|| env("XDG_DATA_HOME"))
        .or_else(|| env("HOME").map(|home| home.join(".local").join("share")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("tricky-towers-rs")
}

#[derive(Component)]
pub(crate) struct SingleRoot;
#[derive(Component)]
pub(crate) struct EndlessRoot;

/// A node the show system updates.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub(crate) enum Item {
    ListButton(usize),
    ListText(usize),
    /// The four corners around a picked list button.
    ListCorner(usize),
    EndlessButton(usize),
    EndlessText(usize),
    BoardTitle,
    BoardRow(usize),
}

pub(crate) fn spawn_single_screens(mut commands: Commands, art: Option<Res<Art>>) {
    let art = art.as_deref();
    let image = |name: &str| art.and_then(|a| a.image(name));

    let single = spawn_page(&mut commands, art, "Single Player");
    commands.entity(single).insert(SingleRoot);
    commands.entity(single).with_children(|page| {
        // Player 1's seat, awake.
        let at = (0.0, SEAT_Y);
        let mut panel = page.spawn((
            place(SEAT_PANEL, at),
            BackgroundColor(Color::srgba(1.0, 0.8, 0.3, 0.35)),
        ));
        if let Some(colour) = image("menu_player_select_background_selected") {
            panel.insert((
                sliced(colour, [PANEL_SLICE; 4]),
                BackgroundColor(Color::NONE),
            ));
        }
        if let Some(outline) = image("menu_player_select_outline") {
            page.spawn((sliced(outline, [PANEL_SLICE; 4]), place(SEAT_PANEL, at)));
        }
        let (offset, size) = WIZARDS[0][1];
        if let Some(wizard) = image("menu_player_select_wizard1_ready") {
            page.spawn((
                ImageNode::new(wizard),
                place(size, (offset.0, SEAT_Y + offset.1)),
            ));
        }
        // The brick pack panel under the seat; no name plate here.
        let mut pack = page.spawn((
            place(PACK_PANEL.0, (0.0, SEAT_Y - PACK_BELOW_SEAT)),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
        ));
        if let Some(grey) = image("menu_player_select_background_deselected") {
            pack.insert((sliced(grey, [PANEL_SLICE; 4]), BackgroundColor(Color::NONE)));
        }
        let button = image("menu_button_background");
        let band = image("menu_button_cup_background");
        let corner = image("menu_corner_small");
        for (index, (name, _)) in LIST.iter().enumerate() {
            let y = LIST_TOP - LIST_STEP * index as f32;
            let mut node = page.spawn((
                place(LIST_BUTTON, (0.0, y)),
                BackgroundColor(LIST_COLOR),
                Item::ListButton(index),
            ));
            if let Some(art) = &button {
                node.insert((
                    sliced(art.clone(), BUTTON_SLICE),
                    BackgroundColor(Color::NONE),
                ));
            }
            // Endless carries the cup band across it, as wide as the button.
            if *name == "Endless"
                && let Some(band) = &band
            {
                node.with_children(|b| {
                    b.spawn((
                        ImageNode::new(band.clone()),
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Percent(100.0),
                            height: Val::Percent(CUP_BAND / LIST_BUTTON.1 * 100.0),
                            ..default()
                        },
                    ));
                });
            }
            node.with_children(|b| {
                b.spawn((label(3.2, name), Item::ListText(index)));
            });
            if let Some(corner) = &corner {
                let frame = (
                    LIST_BUTTON.0 + 2.0 * CORNER_GAP,
                    LIST_BUTTON.1 + 2.0 * CORNER_GAP,
                );
                corners(
                    page,
                    corner,
                    frame,
                    (0.0, y),
                    CORNER_SMALL,
                    Item::ListCorner(index),
                );
            }
        }
        page.spawn(place((SCREEN_WIDTH, 60.0), (0.0, -540.0 + 60.0)))
            .with_children(|p| {
                p.spawn(label(
                    2.2,
                    "Up/Down: choose   Enter or A: pick   Esc or B: back",
                ));
            });
    });

    let endless = spawn_page(&mut commands, art, "Endless");
    commands.entity(endless).insert(EndlessRoot);
    commands.entity(endless).with_children(|page| {
        for (index, entry) in ENDLESS.iter().enumerate() {
            let at = endless_slot(index);
            let mut node = page.spawn((
                place(ENDLESS_BUTTON, at),
                BackgroundColor(entry.color),
                Item::EndlessButton(index),
            ));
            let picture = image(entry.picture);
            if let Some(art) = &picture {
                // The picture keeps its own size, centred in the slot.
                node.insert((
                    ImageNode::new(art.clone()),
                    BackgroundColor(Color::NONE),
                    place(entry.size, (at.0, at.1 + entry.lift)),
                ));
            }
            if picture.is_none() {
                node.with_children(|b| {
                    b.spawn((label(4.0, entry.name), Item::EndlessText(index)));
                });
            }
            if let Some(face) = image(entry.face) {
                let face_at = (
                    at.0 - ENDLESS_BUTTON.0 / 2.0 + FACE_AT.0 + FACE.0 / 2.0,
                    at.1 - ENDLESS_BUTTON.1 / 2.0 + FACE_AT.1 + FACE.1 / 2.0,
                );
                page.spawn((ImageNode::new(face), place(FACE, face_at)));
            }
        }
        let mut board = page.spawn((
            place(BOARD, BOARD_AT),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
        ));
        if let Some(art) = image("menu_leaderboard_panel") {
            board.insert((sliced(art, BOARD_SLICE), BackgroundColor(Color::NONE)));
        }
        let row_art = image("menu_leaderboard_row");
        board.with_children(|b| {
            b.spawn((Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(4.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },))
                .with_children(|t| {
                    t.spawn((label(4.0, ""), Item::BoardTitle));
                });
            for row in 0..BOARD_LENGTH {
                let top = ROW_TOP + ROW_STEP * row as f32 - ROW.1 / 2.0;
                let mut node = b.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent((BOARD.0 - ROW.0) / 2.0 / BOARD.0 * 100.0),
                        top: Val::Percent(top / BOARD.1 * 100.0),
                        width: Val::Percent(ROW.0 / BOARD.0 * 100.0),
                        height: Val::Percent(ROW.1 / BOARD.1 * 100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.08)),
                ));
                if let Some(art) = &row_art {
                    node.insert((sliced(art.clone(), ROW_SLICE), BackgroundColor(Color::NONE)));
                }
                node.with_children(|r| {
                    r.spawn((label(2.8, ""), Item::BoardRow(row)));
                });
            }
        });
        page.spawn(place((SCREEN_WIDTH, 60.0), (0.0, -540.0 + 60.0)))
            .with_children(|p| {
                p.spawn(label(
                    2.2,
                    "Up/Down: choose   Enter or A: play   Esc or B: back",
                ));
            });
    });
}

const SCREEN_WIDTH: f32 = 1920.0;

/// The centre of Endless button `index`'s slot.
fn endless_slot(index: usize) -> (f32, f32) {
    (
        ENDLESS_TOP_LEFT.0 + ENDLESS_BUTTON.0 / 2.0,
        ENDLESS_TOP_LEFT.1 - ENDLESS_BUTTON.1 / 2.0 - ENDLESS_STEP * index as f32,
    )
}
/// A list button's colour when its art isn't staged.
const LIST_COLOR: Color = Color::srgb(0.3, 0.24, 0.55);

/// Up/Down move along the list, Enter or A opens Trials or Endless, B goes back
/// to the title.
pub(crate) fn single_input(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    pads: Query<(Entity, &Gamepad)>,
    game: Res<Match>,
    mut screen: ResMut<MenuScreen>,
    mut menu: ResMut<SingleMenu>,
) {
    if game.phase != Phase::Menu || *screen != MenuScreen::Single {
        return;
    }
    let taps = read_taps(keys.as_deref(), &pads);
    if !taps.pad_back.is_empty() {
        *screen = MenuScreen::Main;
        return;
    }
    if taps.up || taps.down {
        menu.selected = (menu.selected + 1) % LIST.len();
    }
    if taps.key_pick || !taps.pad_pick.is_empty() {
        match LIST[menu.selected].0 {
            "Endless" => {
                menu.endless = 0;
                *screen = MenuScreen::Endless;
            }
            _ => *screen = MenuScreen::Trials,
        }
    }
}

/// Up/Down pick Race, Survival or Puzzle; Enter or A plays a built one alone;
/// B goes back to the list.
pub(crate) fn endless_input(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    pads: Query<(Entity, &Gamepad)>,
    game: Res<Match>,
    mut screen: ResMut<MenuScreen>,
    mut menu: ResMut<SingleMenu>,
    mut config: ResMut<GameConfig>,
    mut request: ResMut<MatchRequest>,
) {
    if game.phase != Phase::Menu || *screen != MenuScreen::Endless {
        return;
    }
    let taps = read_taps(keys.as_deref(), &pads);
    if !taps.pad_back.is_empty() {
        *screen = MenuScreen::Single;
        return;
    }
    let count = ENDLESS.len();
    if taps.up {
        menu.endless = (menu.endless + count - 1) % count;
    }
    if taps.down {
        menu.endless = (menu.endless + 1) % count;
    }
    if !(taps.key_pick || !taps.pad_pick.is_empty()) || !ENDLESS[menu.endless].built {
        return;
    }
    config.mode = ENDLESS[menu.endless].mode;
    // Endless runs use Normal's values (?).
    config.difficulty = Some(Difficulty::Normal);
    config.players = 1;
    config.opponents = 0;
    config.trial = None;
    request.pending = true;
}

#[allow(clippy::type_complexity)]
pub(crate) fn show_single_screens(
    game: Res<Match>,
    screen: Res<MenuScreen>,
    menu: Res<SingleMenu>,
    board: Res<Leaderboard>,
    mut roots: ParamSet<(
        Query<&mut Visibility, With<SingleRoot>>,
        Query<&mut Visibility, With<EndlessRoot>>,
    )>,
    mut items: Query<
        (
            &Item,
            Option<&mut ImageNode>,
            Option<&mut BackgroundColor>,
            Option<&mut Text>,
            Option<&mut TextColor>,
            Option<&mut Visibility>,
        ),
        (Without<SingleRoot>, Without<EndlessRoot>),
    >,
) {
    let menu_up = game.phase == Phase::Menu;
    let single_on = menu_up && *screen == MenuScreen::Single;
    let endless_on = menu_up && *screen == MenuScreen::Endless;
    for mut v in &mut roots.p0() {
        v.set_if_neq(shown(single_on));
    }
    for mut v in &mut roots.p1() {
        v.set_if_neq(shown(endless_on));
    }
    if !single_on && !endless_on {
        return;
    }
    // The picked button bright, the others dimmed; unbuilt ones greyed out.
    let tint = |picked: bool, built: bool| match (picked, built) {
        (true, true) => Color::WHITE,
        (true, false) => Color::srgb(0.7, 0.7, 0.7),
        (false, true) => Color::srgb(0.55, 0.55, 0.6),
        (false, false) => Color::srgb(0.35, 0.35, 0.4),
    };
    let entry = &ENDLESS[menu.endless];
    for (item, image, colour, text, text_colour, visibility) in &mut items {
        if let (Item::ListCorner(i), Some(mut v)) = (*item, visibility) {
            v.set_if_neq(shown(menu.selected == i));
            continue;
        }
        let look = match *item {
            Item::ListButton(i) => Some(tint(menu.selected == i, LIST[i].1)),
            Item::EndlessButton(i) => Some(tint(menu.endless == i, ENDLESS[i].built)),
            _ => None,
        };
        if let Some(look) = look {
            if let Some(mut image) = image
                && image.color != look
            {
                image.color = look;
            }
            if let Some(mut c) = colour
                && c.0 != Color::NONE
            {
                let base = match *item {
                    Item::EndlessButton(i) => ENDLESS[i].color,
                    _ => LIST_COLOR,
                }
                .to_srgba();
                let l = look.to_srgba();
                let want = Color::srgb(base.red * l.red, base.green * l.green, base.blue * l.blue);
                if c.0 != want {
                    c.0 = want;
                }
            }
            continue;
        }
        let words = match *item {
            Item::ListText(i) => {
                let (name, built) = LIST[i];
                if let Some(mut c) = text_colour {
                    let want = if menu.selected == i {
                        Color::srgb(1.0, 0.85, 0.3)
                    } else {
                        Color::WHITE
                    };
                    if c.0 != want {
                        c.0 = want;
                    }
                }
                if built {
                    name.to_string()
                } else {
                    format!("{name} (soon)")
                }
            }
            Item::EndlessText(i) => {
                if ENDLESS[i].built {
                    ENDLESS[i].name.to_string()
                } else {
                    format!("{} (soon)", ENDLESS[i].name)
                }
            }
            Item::BoardTitle => format!("Endless {}", entry.name),
            Item::BoardRow(row) => match board.scores[menu.endless].get(row) {
                Some(score) if entry.built => format!("{}.  {score}", row + 1),
                _ => format!("{}.  -", row + 1),
            },
            _ => continue,
        };
        // Menu text is in capitals, as in the original.
        let words = words.to_uppercase();
        if let Some(mut text) = text
            && text.0 != words
        {
            text.0 = words;
        }
    }
}

/// When an Endless run ends, each player's score goes on that mode's leaderboard.
pub(crate) fn record_endless_scores(
    game: Res<Match>,
    config: Res<GameConfig>,
    towers: Query<(&Tower, &TowerState)>,
    mut board: ResMut<Leaderboard>,
    mut recorded: Local<bool>,
) {
    let board_index = game.mode.endless_index();
    let over = game.phase == Phase::Over && board_index.is_some();
    if !over {
        if game.phase == Phase::Playing {
            *recorded = false;
        }
        return;
    }
    if *recorded {
        return;
    }
    *recorded = true;
    for (tower, state) in &towers {
        if tower.human
            && let Some(index) = board_index
        {
            let score = match game.mode {
                ModeKind::Endless => config.endless.score(state.best_height),
                _ => state.endless_score(),
            };
            board.add(index, score);
        }
    }
    board.save();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_leaderboard_keeps_the_best_ten_best_first() {
        let mut board = Leaderboard::default();
        for score in 1..=12 {
            board.add(0, score * 10);
        }
        assert_eq!(board.scores[0].len(), BOARD_LENGTH);
        assert_eq!(board.scores[0][0], 120);
        assert_eq!(*board.scores[0].last().unwrap(), 30);
    }

    #[test]
    fn the_leaderboard_survives_a_restart() {
        let file = std::env::temp_dir()
            .join(format!("tt-board-{}", std::process::id()))
            .join("leaderboard.ron");
        let mut board = Leaderboard::load(file.clone());
        assert!(board.scores[0].is_empty());
        board.add(0, 250);
        board.add(2, 40);
        board.save();
        let again = Leaderboard::load(file.clone());
        assert_eq!(again.scores, [vec![250], vec![], vec![40]]);
        let _ = std::fs::remove_dir_all(file.parent().unwrap());
    }
}
