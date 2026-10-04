//! Single Player: the player's seat over a short list (Trials, Endless), then
//! the Endless screen, three wide buttons (Race, Survival, Puzzle) beside a
//! leaderboard. Layout from docs/research/menus.md, in pixels of a 1920×1080
//! screen; only Endless Race is built, so the rest show dimmed.

use bevy::prelude::*;

use crate::art::Art;
use crate::config::{Difficulty, GameConfig, ModeKind};
use crate::gameplay::{Match, MatchRequest, Phase, Tower, TowerState};
use crate::menu::MenuScreen;
use crate::select::{
    NAME_PLATE, PANEL_SLICE, PLATE_SLICE, SEAT_PANEL, WIZARDS, label, place, read_taps, shown,
    sliced, spawn_page,
};

/// The seat sits this far above the centre; the list starts below it.
const SEAT_Y: f32 = 200.0;
const LIST_BUTTON: (f32, f32) = (550.0, 78.0);
const LIST_TOP: f32 = -175.0;
const LIST_STEP: f32 = 100.0;
/// The Endless buttons down the left, the leaderboard on the right (guessed
/// spots until the PC reads them).
const ENDLESS_BUTTON: (f32, f32) = (690.0, 173.0);
const ENDLESS_X: f32 = -430.0;
const ENDLESS_YS: [f32; 3] = [230.0, 0.0, -230.0];
const FACE: (f32, f32) = (150.0, 150.0);
const BOARD: (f32, f32) = (800.0, 800.0);
const BOARD_AT: (f32, f32) = (430.0, -20.0);
/// Scores kept per Endless mode.
const BOARD_LENGTH: usize = 10;

/// A Single Player list entry: its name and whether it's built yet.
const LIST: [(&str, bool); 2] = [("Trials", false), ("Endless", true)];

/// An Endless button: its name, picture, wizard face and whether it's built.
struct EndlessEntry {
    name: &'static str,
    picture: &'static str,
    face: &'static str,
    built: bool,
    color: Color,
}

const ENDLESS: [EndlessEntry; 3] = [
    EndlessEntry {
        name: "Race",
        picture: "menu_mode_endless",
        face: "menu_challenge_wizard2",
        built: true,
        color: Color::srgb(0.85, 0.35, 0.2),
    },
    EndlessEntry {
        name: "Survival",
        picture: "menu_mode_endless_survival",
        face: "menu_challenge_wizard3",
        built: false,
        color: Color::srgb(0.25, 0.55, 0.85),
    },
    EndlessEntry {
        name: "Puzzle",
        picture: "menu_mode_endless_puzzle",
        face: "menu_challenge_wizard4",
        built: false,
        color: Color::srgb(0.55, 0.35, 0.75),
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

/// Best Endless scores this run, best first, per Endless mode.
#[derive(Resource, Debug, Default)]
pub struct Leaderboard {
    pub scores: [Vec<u32>; 3],
}

impl Leaderboard {
    pub fn add(&mut self, mode: usize, score: u32) {
        let list = &mut self.scores[mode];
        list.push(score);
        list.sort_unstable_by(|a, b| b.cmp(a));
        list.truncate(BOARD_LENGTH);
    }
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
    EndlessButton(usize),
    EndlessText(usize),
    BoardTitle,
    BoardText,
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
        // The name plate tucked under the panel, above the list.
        let plate_y = SEAT_Y - SEAT_PANEL.1 / 2.0 - NAME_PLATE.0.1 / 2.0 - 10.0;
        let mut plate = page.spawn((
            place(NAME_PLATE.0, (0.0, plate_y)),
            BackgroundColor(Color::srgba(0.1, 0.08, 0.2, 0.85)),
        ));
        if let Some(art) = image("menu_player_select_nameplate") {
            plate.insert((sliced(art, [PLATE_SLICE; 4]), BackgroundColor(Color::NONE)));
        }
        plate.with_children(|p| {
            p.spawn(label(2.4, "Player 1"));
        });
        let button = image("menu_button_background");
        for (index, (name, _)) in LIST.iter().enumerate() {
            let y = LIST_TOP - LIST_STEP * index as f32;
            let mut node = page.spawn((
                place(LIST_BUTTON, (0.0, y)),
                BackgroundColor(LIST_COLOR),
                Item::ListButton(index),
            ));
            if let Some(art) = &button {
                node.insert((
                    sliced(art.clone(), [PLATE_SLICE; 4]),
                    BackgroundColor(Color::NONE),
                ));
            }
            node.with_children(|b| {
                b.spawn((label(3.2, name), Item::ListText(index)));
            });
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
            let at = (ENDLESS_X, ENDLESS_YS[index]);
            let mut node = page.spawn((
                place(ENDLESS_BUTTON, at),
                BackgroundColor(entry.color),
                Item::EndlessButton(index),
            ));
            let picture = image(entry.picture);
            if let Some(art) = &picture {
                node.insert((ImageNode::new(art.clone()), BackgroundColor(Color::NONE)));
            }
            if picture.is_none() {
                node.with_children(|b| {
                    b.spawn((label(4.0, entry.name), Item::EndlessText(index)));
                });
            }
            if let Some(face) = image(entry.face) {
                let face_at = (at.0 - ENDLESS_BUTTON.0 / 2.0 + FACE.0 / 2.0 + 10.0, at.1);
                page.spawn((ImageNode::new(face), place(FACE, face_at)));
            }
        }
        let mut board = page.spawn((
            place(BOARD, BOARD_AT),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
        ));
        if let Some(art) = image("menu_button_cup_background") {
            board.insert((sliced(art, [PANEL_SLICE; 4]), BackgroundColor(Color::NONE)));
        }
        board.insert(Node {
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexStart,
            padding: UiRect::top(Val::Percent(6.0)),
            row_gap: Val::Percent(4.0),
            ..place(BOARD, BOARD_AT)
        });
        board.with_children(|b| {
            b.spawn((label(4.0, ""), Item::BoardTitle));
            b.spawn((label(3.0, ""), Item::BoardText));
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
/// A list button's colour when its art isn't staged.
const LIST_COLOR: Color = Color::srgb(0.3, 0.24, 0.55);

/// Up/Down move along the list, Enter or A opens Endless (Trials aren't built
/// yet), B goes back to the title.
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
    if (taps.key_pick || !taps.pad_pick.is_empty()) && LIST[menu.selected].0 == "Endless" {
        menu.endless = 0;
        *screen = MenuScreen::Endless;
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
    config.mode = ModeKind::Endless;
    config.difficulty = Some(Difficulty::Normal);
    config.players = 1;
    config.opponents = 0;
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
    for (item, image, colour, text, text_colour) in &mut items {
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
            Item::BoardText => {
                let scores = &board.scores[menu.endless];
                if !entry.built {
                    "Coming soon".into()
                } else if scores.is_empty() {
                    "No scores yet".into()
                } else {
                    scores
                        .iter()
                        .enumerate()
                        .map(|(place, score)| format!("{}. {score}", place + 1))
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            }
            _ => continue,
        };
        if let Some(mut text) = text
            && text.0 != words
        {
            text.0 = words;
        }
    }
}

/// When an Endless Race ends, each player's score goes on the leaderboard.
pub(crate) fn record_endless_scores(
    game: Res<Match>,
    config: Res<GameConfig>,
    towers: Query<(&Tower, &TowerState)>,
    mut board: ResMut<Leaderboard>,
    mut recorded: Local<bool>,
) {
    let over = game.phase == Phase::Over && game.mode == ModeKind::Endless;
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
        if tower.human {
            board.add(0, config.endless.score(state.best_height));
        }
    }
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
}
