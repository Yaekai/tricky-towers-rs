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
/// A wizard face's bottom-left corner from the button's bottom-left.
const FACE_AT: (f32, f32) = (33.0, 6.0);
/// The leaderboard on the left, its right edge on the centre line, 30 px below
/// the centre; its panel's 9-slice borders and its rows'.
const BOARD: (f32, f32) = (800.0, 800.0);
const BOARD_AT: (f32, f32) = (-400.0, -30.0);
const BOARD_SLICE: [f32; 4] = [0.0, 0.0, 37.0, 51.0];
const ROW_SLICE: [f32; 4] = [6.0, 7.0, 0.0, 0.0];
/// Rows down the leaderboard (LeaderboardEntry prefab, list 50 px in from each
/// side and 70 below the top, one row every 50 px): size, the first row's top
/// and the pitch; the rank box (width, centre from the left), where the name
/// starts with no avatar and the room kept right of it, the score box's width.
const ROW: (f32, f32) = (700.0, 42.0);
const ROW_TOP: f32 = 70.0;
const ROW_STEP: f32 = 50.0;
const RANK: (f32, f32) = (96.5, 15.0 + 96.5 / 2.0);
const NAME_X: f32 = 125.0;
const NAME_RIGHT: f32 = 110.0;
const SCORE_W: f32 = 100.0;
/// Gold, silver and bronze medal sizes for ranks 1 to 3.
const MEDALS: [(f32, f32); 3] = [(44.0, 52.0), (40.0, 49.0), (44.0, 52.0)];
/// Row tints: the list's and YOUR BEST's (darker).
const ROW_TINT: Color = Color::srgb(0.82, 0.85, 0.95);
const BEST_TINT: Color = Color::srgb(0.745, 0.784, 0.937);
/// YOUR BEST block: 50 px in from each side, its bottom 80 px above the panel's.
const BEST_INSET: f32 = 50.0;
const BEST_BOTTOM: f32 = 80.0;
/// EnzoBlackGold / EnzoBlackWhite text faces.
const GOLD: Color = Color::srgb(1.0, 0.82, 0.31);
const PALE: Color = Color::srgb(0.918, 0.933, 1.0);
/// The original shows the Steam name; local runs are all player 1's.
const PLAYER_NAME: &str = "Player 1";
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
    face_size: (f32, f32),
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
        face_size: (110.0, 108.0),
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
        face_size: (97.0, 102.0),
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
        face_size: (110.0, 108.0),
        // Rules from public descriptions; values are guesses until read (?).
        built: true,
        color: Color::srgb(0.55, 0.35, 0.75),
        mode: ModeKind::EndlessPuzzle,
    },
];

/// Where the cursor is on the two screens.
#[derive(Resource, Debug)]
pub struct SingleMenu {
    /// What the player is choosing (AbstractSingleCharacterSelectStateController): 0 the
    /// wizard, 1 the brick pack, 2 the Trials/Endless list. Down or Enter goes on, Up or
    /// Back goes back.
    pub step: u8,
    /// Index into the Trials/Endless list, once on it (Trials first).
    pub selected: usize,
    /// Index into Race, Survival, Puzzle.
    pub endless: usize,
    /// The player's wizard (0 for wizard 1), picked with Left/Right.
    pub wizard: usize,
}

/// A wizard in the Single Player carousel: the picked one (0) or its
/// neighbour on the left (-1) or right (1).
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub(crate) struct Carousel(i32);

/// The red ribbon over a picked Single Player list button.
#[derive(Component)]
pub(crate) struct ListRibbon(usize);

/// The list buttons' words (SinglePlayerSelectButton and ...ButtonEndless): Trials in
/// EnzoBlackWhite32, Endless in EnzoBlackGoldRedOutline26 (the gold face, a wine outline)
/// on the cup band its button always carries; their sizes.
const LIST_LOOKS: [(crate::letters::TextLook, f32); 2] = [
    (
        crate::letters::TextLook {
            body: false,
            face: Some(Color::srgb(0.918, 0.933, 1.0)),
            outline: Some(Color::srgb(0.169, 0.035, 0.137)),
            low: None,
            // Dilate and outline 0.3 (EnzoBlackWhite32).
            outline_edge: 0.5 - 0.6 * 0.8333 / 2.0,
        },
        32.0,
    ),
    (
        crate::letters::TextLook {
            outline: Some(Color::srgb(0.318, 0.043, 0.180)),
            outline_edge: 0.5 - 0.6 * 0.8333 / 2.0,
            ..crate::letters::TextLook::GOLD
        },
        27.4,
    ),
];
/// The list button whose prefab has the cup band (Endless).
const BANDED: usize = 1;

/// Each list button's words in its own look, and the cup band on Endless's.
pub(crate) fn show_list_ribbon(
    menu: Res<SingleMenu>,
    mut ribbons: Query<(&ListRibbon, &mut Visibility)>,
    mut words: Query<(&Item, &mut crate::letters::TextLook, &mut TextColor)>,
) {
    if !menu.is_changed() {
        return;
    }
    for (ribbon, mut visibility) in &mut ribbons {
        visibility.set_if_neq(shown(ribbon.0 == BANDED));
    }
    for (item, mut look, _) in &mut words {
        if let Item::ListText(i) = *item
            && let Some((want, _)) = LIST_LOOKS.get(i)
        {
            look.set_if_neq(*want);
        }
    }
}

/// A sun ray layer on the picked pack panel, turning like the other menus' rays.
#[derive(Component)]
pub(crate) struct SingleRay(usize);

/// Turns the pack panel's rays (GameConfig::menu.ray_turns).
pub(crate) fn turn_single_rays(
    time: Res<Time>,
    config: Res<GameConfig>,
    rays: Query<(&SingleRay, &MaterialNode<crate::ui_material::TurningPicture>)>,
    mut turning: Option<ResMut<Assets<crate::ui_material::TurningPicture>>>,
) {
    let Some(turning) = turning.as_mut() else { return };
    let t = time.elapsed_secs();
    for (ray, picture) in &rays {
        if let Some(&(degrees, seconds)) = config.menu.ray_turns.get(ray.0) {
            let turn = Rot2::degrees(degrees * (t / seconds.max(0.01)).fract());
            crate::ui_material::set_turn(turning, &picture.0, turn.as_radians());
        }
    }
}

/// The brick pack card under the carousel.
#[derive(Component)]
pub(crate) struct PackCard(i32);

/// An Indie guest's game logo over its carousel panel (slot as in `Carousel`).
#[derive(Component)]
pub(crate) struct CarouselLogo(i32);

/// The Indie guests' game logos (wizards 9-14) and their sizes in pixels.
const INDIE_LOGOS: [(&str, (f32, f32)); 6] = [
    ("menu_logo_lethal_league", (637.0, 459.0)),
    ("menu_logo_awesomenauts", (525.0, 112.0)),
    ("menu_logo_towerfall", (397.0, 176.0)),
    ("menu_logo_armello", (164.0, 100.0)),
    ("menu_logo_nuclear_throne", (504.0, 181.0)),
    ("menu_logo_octodad", (518.0, 140.0)),
];
/// The box a logo is fitted into, and its centre above the panel's top edge.
const LOGO_BOX: (f32, f32) = (260.0, 80.0);
const LOGO_ABOVE: f32 = 10.0;

/// Wizards in the carousel, as many as the original has.
const CAROUSEL_WIZARDS: usize = 14;
/// The neighbours' panels and the arrows, from the screen centre (Max's
/// screenshots of the original, ?).
const NEIGHBOUR_X: f32 = 405.0;
/// The scroll panel draws its neighbours at full size (ScrollPanelComponent).
const NEIGHBOUR_SCALE: f32 = 1.0;
/// The choosing arrows either side of the seat and pack panels (the scene's ArrowPrevious
/// and ArrowNext), and how far the choosing corners stand out (as on Select Players).
const ROW_ARROW_X: f32 = 200.0;
/// The scene's ItemHighlight corners: centred 10 px out from the panel's corners.
const FOCUS_OUT: f32 = 10.0;

/// The carousels' screen mask (UI/SinglePlayerSelectMask): an item shows fully between
/// `edge` and 1 - `edge` of the screen's width and fades out over `ramp` past them; the
/// wizards' material has edge 0.22 and ramp 0.1, the packs' 0.35 and 0.05.
const WIZARD_MASK: (f32, f32) = (0.22, 0.1);
const PACK_MASK: (f32, f32) = (0.35, 0.05);
/// The far wizards' panels (two away) only show in the fade's outer part, at about this
/// alpha on average.
const FAR_ALPHA: f32 = 0.3;

/// Spawns `fill` into a full-page box cut `cut` of the screen's width in from each side
/// (ponytail: a hard cut standing in for the shader's fade; a UI material could fade it).
fn masked(page: &mut ChildSpawnerCommands, cut: f32, fill: impl FnOnce(&mut ChildSpawnerCommands)) {
    let share = 1.0 - 2.0 * cut;
    page.spawn(Node {
        position_type: PositionType::Absolute,
        left: Val::Percent(cut * 100.0),
        width: Val::Percent(share * 100.0),
        height: Val::Percent(100.0),
        overflow: Overflow::clip(),
        ..default()
    })
    .with_children(|window| {
        window
            .spawn(Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(-cut / share * 100.0),
                width: Val::Percent(100.0 / share),
                height: Val::Percent(100.0),
                ..default()
            })
            .with_children(fill);
    });
}

impl Default for SingleMenu {
    fn default() -> Self {
        Self {
            step: 0,
            selected: 0,
            endless: 0,
            wizard: 0,
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
    /// The wizard's orange panel, once it is chosen.
    SeatReady,
    /// The brick pack's orange panel and rays, once it is chosen.
    PackReady,
    /// The corners and arrows on the row being chosen: 0 the wizard, 1 the pack.
    RowFocus(u8),
    ListButton(usize),
    ListText(usize),
    /// The four corners around a picked list button.
    ListCorner(usize),
    EndlessButton(usize),
    /// The four corners around the picked Endless button.
    EndlessCorner(usize),
    EndlessText(usize),
    /// A leaderboard row (shown when it holds a score), its name and score.
    BoardRow(usize),
    BoardName(usize),
    BoardScore(usize),
    BestScore,
}

pub(crate) fn spawn_single_screens(
    mut commands: Commands,
    art: Option<Res<Art>>,
    mut turning: Option<ResMut<Assets<crate::ui_material::TurningPicture>>>,
) {
    let art = art.as_deref();
    let image = |name: &str| art.and_then(|a| a.image(name));

    let single = spawn_page(&mut commands, art, "Single Player");
    commands.entity(single).insert(SingleRoot);
    commands.entity(single).with_children(|page| {
        crate::select::shop_hint(page, art);
        // Player 1's seat: grey while the wizard is being chosen, orange once it is.
        let at = (0.0, SEAT_Y);
        let mut grey = page.spawn((place(SEAT_PANEL, at), BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3))));
        if let Some(art) = image("menu_player_select_background_deselected") {
            grey.insert((sliced(art, [PANEL_SLICE; 4]), BackgroundColor(Color::NONE)));
        }
        let mut panel = page.spawn((
            place(SEAT_PANEL, at),
            BackgroundColor(Color::srgba(1.0, 0.8, 0.3, 0.35)),
            Item::SeatReady,
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
        page.spawn((
            ImageNode::default(),
            place(size, (offset.0, SEAT_Y + offset.1)),
            Carousel(0),
        ));
        for slot in -1..=1 {
            let scale = if slot == 0 { 1.0 } else { NEIGHBOUR_SCALE };
            page.spawn((
                ImageNode::default(),
                place(
                    (LOGO_BOX.0 * scale, LOGO_BOX.1 * scale),
                    (
                        NEIGHBOUR_X * slot as f32,
                        SEAT_Y + SEAT_PANEL.1 * scale / 2.0 + LOGO_ABOVE,
                    ),
                ),
                CarouselLogo(slot),
                Visibility::Hidden,
            ));
        }
        // The neighbours on grey panels, two each side, seen through the screen mask.
        // Cut at the fade's outer end, the far ones see-through for the fade they sit in.
        masked(page, WIZARD_MASK.0 - WIZARD_MASK.1, |page| {
            for side in [-2.0f32, -1.0, 1.0, 2.0] {
                let x = NEIGHBOUR_X * side;
                let panel = (
                    SEAT_PANEL.0 * NEIGHBOUR_SCALE,
                    SEAT_PANEL.1 * NEIGHBOUR_SCALE,
                );
                // The far ones sit about a quarter of the way into the fade.
                let alpha = if side.abs() > 1.0 { FAR_ALPHA } else { 1.0 };
                let mut grey = page.spawn((
                    place(panel, (x, SEAT_Y)),
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                ));
                if let Some(art) = image("menu_player_select_background_deselected") {
                    let mut back = sliced(art, [PANEL_SLICE; 4]);
                    back.color = Color::WHITE.with_alpha(alpha);
                    grey.insert((back, BackgroundColor(Color::NONE)));
                }
                let small = (size.0 * NEIGHBOUR_SCALE, size.1 * NEIGHBOUR_SCALE);
                page.spawn((
                    ImageNode {
                        color: Color::WHITE.with_alpha(alpha),
                        ..default()
                    },
                    place(small, (x + offset.0 * NEIGHBOUR_SCALE, SEAT_Y + offset.1)),
                    Carousel(side as i32),
                ));
            }
        });
        // The corners and arrows on the row being chosen (ScrollPanelComponent shows its
        // arrows only while enabled), 200 px either side of the middle.
        let out = |(w, h): (f32, f32)| (w + 2.0 * FOCUS_OUT + CORNER_SMALL.0, h + 2.0 * FOCUS_OUT + CORNER_SMALL.1);
        for (row, size, y) in [(0u8, SEAT_PANEL, SEAT_Y), (1, PACK_PANEL.0, SEAT_Y - PACK_BELOW_SEAT)] {
            if let Some(corner) = image("menu_corner_small") {
                corners(page, &corner, out(size), (0.0, y), CORNER_SMALL, Item::RowFocus(row));
            }
            if let Some(arrow) = image("menu_arrow_small") {
                for flip_x in [true, false] {
                    let side = if flip_x { -1.0 } else { 1.0 };
                    page.spawn((
                        ImageNode {
                            image: arrow.clone(),
                            flip_x,
                            ..default()
                        },
                        place(crate::select::SEAT_ARROW, (side * ROW_ARROW_X, y)),
                        Item::RowFocus(row),
                    ));
                }
            }
        }
        // The pack panel's neighbours (its own scroll panel), grey, with the packs
        // before and after the picked one, cut by their narrower screen mask.
        // Cut half way down the fade.
        masked(page, PACK_MASK.0 - PACK_MASK.1 / 2.0, |page| {
            for side in [-1i32, 1] {
                let mut grey = page.spawn((
                    place(PACK_PANEL.0, (NEIGHBOUR_X * side as f32, SEAT_Y - PACK_BELOW_SEAT)),
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                ));
                if let Some(art) = image("menu_player_select_background_deselected") {
                    grey.insert((sliced(art, [PANEL_SLICE; 4]), BackgroundColor(Color::NONE)));
                }
                grey.with_children(|p| {
                    p.spawn((
                        ImageNode::default(),
                        Node {
                            width: Val::Percent(90.0),
                            height: Val::Percent(80.0),
                            margin: UiRect::AUTO,
                            ..default()
                        },
                        PackCard(side),
                        Visibility::Hidden,
                    ));
                });
            }
        });
        // The brick pack panel under the seat, picked (its item's Selected: the orange
        // panel and turning rays cut to 327 x 95, then the dark frame); no name plate.
        let pack_at = (0.0, SEAT_Y - PACK_BELOW_SEAT);
        let clip = (PACK_PANEL.0.0 - 5.0, PACK_PANEL.0.1 - 5.0);
        let mut grey = page.spawn((place(PACK_PANEL.0, pack_at), BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3))));
        if let Some(art) = image("menu_player_select_background_deselected") {
            grey.insert((sliced(art, [PANEL_SLICE; 4]), BackgroundColor(Color::NONE)));
        }
        page.spawn((
            Node {
                overflow: Overflow::clip(),
                ..place(clip, pack_at)
            },
            BackgroundColor(Color::srgba(1.0, 0.8, 0.3, 0.35)),
            Item::PackReady,
        ))
        .with_children(|c| {
            if let Some(back) = image("menu_player_select_background_selected") {
                c.spawn((
                    sliced(back, [PANEL_SLICE; 4]),
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                ));
            }
            for layer in 0..5 {
                if let Some(rays) = image(&format!("menu_sunrays_{}", layer + 1)) {
                    crate::select::turning_rays(c, &mut turning, rays, clip, SingleRay(layer));
                }
            }
        });
        if let Some(outline) = image("menu_player_select_outline") {
            page.spawn((sliced(outline, [PANEL_SLICE; 4]), place(PACK_PANEL.0, pack_at)));
        }
        let mut pack = page.spawn(place(PACK_PANEL.0, pack_at));
        // The picked brick pack's card on it, as on Select Players.
        pack.with_children(|p| {
            p.spawn((
                ImageNode::default(),
                Node {
                    width: Val::Percent(90.0),
                    height: Val::Percent(80.0),
                    margin: UiRect::AUTO,
                    ..default()
                },
                PackCard(0),
                Visibility::Hidden,
            ));
        });
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
            // The picked button carries the red ribbon across it, a little wider
            // (Max's screenshots of the original).
            if let Some(band) = &band {
                node.with_children(|b| {
                    b.spawn((
                        sliced(band.clone(), [26.0, 26.0, 0.0, 0.0]),
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Percent(-3.0),
                            width: Val::Percent(106.0),
                            top: Val::Percent((1.0 - CUP_BAND / LIST_BUTTON.1) * 50.0),
                            height: Val::Percent(CUP_BAND / LIST_BUTTON.1 * 100.0),
                            ..default()
                        },
                        ListRibbon(index),
                        Visibility::Hidden,
                    ));
                });
            }
            node.with_children(|b| {
                let size = LIST_LOOKS.get(index).map_or(32.0, |l| l.1);
                b.spawn((label(size / 10.8, name), Item::ListText(index)));
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
    });

    let endless = crate::select::spawn_page(&mut commands, art, "Endless");
    commands.entity(endless).insert(EndlessRoot);
    commands.entity(endless).with_children(|page| {
        let corner = image("menu_corner_small");
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
            // "RACE CHALLENGE" right-aligned in a 60 px box hung from 70 px above the
            // button's centre, 30 px in from its right edge (TitleLabel, size 46,
            // pivot top-right as Max's screenshots place it), in EnzoBlackGold60: the gold
            // face, dilate and outline 0.2.
            let mut title = place((ENDLESS_BUTTON.0 - 61.92, 60.0), (at.0 + 0.96, at.1 + 40.0));
            title.justify_content = JustifyContent::FlexEnd;
            page.spawn(title).with_children(|t| {
                t.spawn((
                    crate::select::label_with(
                        46.0 / 10.8,
                        "",
                        crate::letters::TextLook {
                            outline_edge: 0.5 - 0.4 * 0.8333 / 2.0,
                            ..crate::letters::TextLook::GOLD
                        },
                    ),
                    Item::EndlessText(index),
                ));
            });
            // The picked button's corners sit centred on its 690x167 rect's corners.
            if let Some(corner) = &corner {
                corners(
                    page,
                    corner,
                    (ENDLESS_BUTTON.0 + CORNER_SMALL.0, 167.0 + CORNER_SMALL.1),
                    at,
                    CORNER_SMALL,
                    Item::EndlessCorner(index),
                );
            }
            if let Some(face) = image(entry.face) {
                let face_at = (
                    at.0 - ENDLESS_BUTTON.0 / 2.0 + FACE_AT.0 + entry.face_size.0 / 2.0,
                    at.1 - ENDLESS_BUTTON.1 / 2.0 + FACE_AT.1 + entry.face_size.1 / 2.0,
                );
                page.spawn((ImageNode::new(face), place(entry.face_size, face_at)));
            }
        }
        spawn_board(page, &image);
    });
}

/// A point on the leaderboard panel, in pixels from its top-left (y down), as
/// a point from the screen centre (y up).
fn on_board(x: f32, y: f32) -> (f32, f32) {
    (
        BOARD_AT.0 - BOARD.0 / 2.0 + x,
        BOARD_AT.1 + BOARD.1 / 2.0 - y,
    )
}

/// A text box `w` wide centred at board point (`x`, `y`), its words lined up
/// left or centred.
fn board_text(w: f32, h: f32, x: f32, y: f32, left: bool) -> Node {
    let mut node = place((w, h), on_board(x, y));
    if left {
        node.justify_content = JustifyContent::FlexStart;
    }
    node
}

/// The WEEKLY leaderboard (LeaderboardGameModeSelect scene): panel, title and
/// arrow keys on its top edge, ten rows, YOUR BEST at the bottom.
fn spawn_board(page: &mut ChildSpawnerCommands, image: &dyn Fn(&str) -> Option<Handle<Image>>) {
    let mut board = page.spawn((
        place(BOARD, BOARD_AT),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
    ));
    if let Some(art) = image("menu_leaderboard_panel") {
        board.insert((sliced(art, BOARD_SLICE), BackgroundColor(Color::NONE)));
    }
    // Display letters with the dark outline in the given face colour.
    let outlined = |t: &mut ChildSpawnerCommands, size: f32, text: &str, face: Color| {
        t.spawn(crate::select::label_with(
            size,
            text,
            crate::letters::TextLook::OUTLINED,
        ))
        .insert(TextColor(face))
        .id()
    };
    let px = |p: f32| p / 10.8;
    page.spawn(board_text(510.0, 75.0, BOARD.0 / 2.0, -10.68, false))
        .with_children(|t| {
            outlined(t, px(60.0), "Weekly", GOLD);
        });
    for (name, x) in [("menu_key_left", 90.0), ("menu_key_right", BOARD.0 - 90.0)] {
        if let Some(art) = image(name) {
            page.spawn((ImageNode::new(art), place((55.0, 51.0), on_board(x, -5.0))));
        }
    }
    let row_art = image("menu_leaderboard_row");
    let row_look = |node: &mut EntityCommands, tint: Color| {
        node.insert(BackgroundColor(tint.with_alpha(0.5)));
        if let Some(art) = &row_art {
            node.insert((sliced(art.clone(), ROW_SLICE), BackgroundColor(Color::NONE)));
            node.entry::<ImageNode>()
                .and_modify(move |mut i| i.color = tint);
        }
    };
    let medals = [
        "menu_leaderboard_medal_1",
        "menu_leaderboard_medal_2",
        "menu_leaderboard_medal_3",
    ];
    for row in 0..BOARD_LENGTH {
        let y = ROW_TOP + ROW_STEP * row as f32 + ROW.1 / 2.0;
        let mut node = page.spawn((
            place(ROW, on_board(BOARD.0 / 2.0, y)),
            Item::BoardRow(row),
            Visibility::Hidden,
        ));
        row_look(&mut node, ROW_TINT);
        node.with_children(|r| {
            let local = |w: f32, x: f32, left: bool| Node {
                position_type: PositionType::Absolute,
                left: Val::Percent((x - if left { 0.0 } else { w / 2.0 }) / ROW.0 * 100.0),
                width: Val::Percent(w / ROW.0 * 100.0),
                height: Val::Percent(100.0),
                justify_content: if left {
                    JustifyContent::FlexStart
                } else {
                    JustifyContent::Center
                },
                align_items: AlignItems::Center,
                ..default()
            };
            match medals.get(row).and_then(|m| image(m)) {
                Some(art) => {
                    let (w, h) = MEDALS[row];
                    r.spawn((
                        ImageNode::new(art),
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Percent((RANK.1 - w / 2.0) / ROW.0 * 100.0),
                            top: Val::Percent(((ROW.1 - h) / 2.0 + 3.0) / ROW.1 * 100.0),
                            width: Val::Percent(w / ROW.0 * 100.0),
                            height: Val::Percent(h / ROW.1 * 100.0),
                            ..default()
                        },
                    ));
                }
                None => {
                    r.spawn(local(RANK.0, RANK.1, false))
                        .with_child(label(px(26.0), &(row + 1).to_string()));
                }
            }
            r.spawn(local(ROW.0 - NAME_X - NAME_RIGHT, NAME_X, true))
                .with_child((label(px(26.0), ""), Item::BoardName(row)));
            r.spawn(local(SCORE_W, ROW.0 - SCORE_W / 2.0, false))
                .with_child((label(px(26.0), ""), Item::BoardScore(row)));
        });
    }
    // YOUR BEST: its tab and row, the title between two pairs of stars.
    let best_bottom = BOARD.1 - BEST_BOTTOM;
    let mut tab = page.spawn(place(
        (BOARD.0 - 2.0 * BEST_INSET - 410.0, 25.0),
        on_board(BOARD.0 / 2.0, best_bottom - 72.5),
    ));
    row_look(&mut tab, BEST_TINT);
    let row_y = best_bottom - 30.0;
    let mut best = page.spawn(place(
        (BOARD.0 - 2.0 * BEST_INSET, 60.0),
        on_board(BOARD.0 / 2.0, row_y),
    ));
    row_look(&mut best, BEST_TINT);
    let star = image("menu_personal_best_star");
    for side in [-1.0f32, 1.0] {
        let edge = |d: f32| BOARD.0 / 2.0 + side * (BOARD.0 / 2.0 - BEST_INSET - d);
        if let Some(art) = &star {
            page.spawn((
                ImageNode::new(art.clone()),
                place(
                    (43.0 * 0.75, 43.0 * 0.75),
                    on_board(edge(192.0), best_bottom - 73.0),
                ),
                UiTransform::from_rotation(Rot2::degrees(10.0 * side)),
            ));
            page.spawn((
                ImageNode::new(art.clone()),
                place((43.0, 43.0), on_board(edge(215.0), best_bottom - 85.0)),
            ));
        }
    }
    page.spawn(board_text(
        220.0,
        40.0,
        BOARD.0 / 2.0,
        best_bottom - 85.0,
        false,
    ))
    .with_children(|t| {
        outlined(t, px(32.0), "Your best", PALE);
    });
    page.spawn(board_text(RANK.0, 60.0, BEST_INSET + RANK.1, row_y, false))
        .with_children(|t| {
            outlined(t, px(26.0), "-", PALE);
        });
    let name_w = BOARD.0 - 2.0 * BEST_INSET - NAME_X - NAME_RIGHT;
    page.spawn(board_text(
        name_w,
        60.0,
        BEST_INSET + NAME_X + name_w / 2.0,
        row_y,
        true,
    ))
    .with_children(|t| {
        outlined(t, px(26.0), PLAYER_NAME, GOLD);
    });
    page.spawn(board_text(
        SCORE_W,
        60.0,
        BOARD.0 - BEST_INSET - SCORE_W / 2.0,
        row_y,
        false,
    ))
    .with_children(|t| {
        let score = outlined(t, px(26.0), "0", GOLD);
        t.commands().entity(score).insert(Item::BestScore);
    });
}

/// The centre of Endless button `index`'s slot.
/// The Endless buttons top to bottom, as indexes into `ENDLESS`: the original lists
/// Race, Puzzle, Survival (the entries keep their order for the saved boards).
const ENDLESS_ORDER: [usize; 3] = [0, 2, 1];

fn endless_slot(index: usize) -> (f32, f32) {
    let row = ENDLESS_ORDER
        .iter()
        .position(|&i| i == index)
        .unwrap_or(index);
    (
        ENDLESS_TOP_LEFT.0 + ENDLESS_BUTTON.0 / 2.0,
        ENDLESS_TOP_LEFT.1 - ENDLESS_BUTTON.1 / 2.0 - ENDLESS_STEP * row as f32,
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
    mut config: ResMut<GameConfig>,
    art: Option<Res<Art>>,
    mut was_on: Local<bool>,
) {
    let on = game.phase == Phase::Menu && *screen == MenuScreen::Single;
    // The screen opens on the wizard, with nothing chosen yet.
    if on && !*was_on {
        menu.step = 0;
    }
    *was_on = on;
    if !on {
        return;
    }
    let taps = read_taps(keys.as_deref(), &pads);
    if !taps.pad_back.is_empty() {
        if !step_back(&mut menu) {
            *screen = MenuScreen::Main;
        }
        return;
    }
    // Left/Right turn the wizards' carousel, or the brick packs'; the pick plays as tower 1.
    if taps.left || taps.right {
        let (wizard, pack) = config.skins.first().copied().unwrap_or((menu.wizard, Default::default()));
        match menu.step {
            0 => {
                let step = if taps.right { 1 } else { CAROUSEL_WIZARDS - 1 };
                menu.wizard = (menu.wizard + step) % CAROUSEL_WIZARDS;
                config.skins = vec![(menu.wizard, pack)];
            }
            1 => {
                let packs = crate::select::available_packs(art.as_deref());
                let count = packs.len().max(1);
                let at = packs.iter().position(|p| *p == pack).unwrap_or(0);
                let next = if taps.right { (at + 1) % count } else { (at + count - 1) % count };
                if let Some(&pack) = packs.get(next) {
                    config.skins = vec![(wizard, pack)];
                    menu.set_changed();
                }
            }
            _ => {}
        }
    }
    let pick = taps.key_pick || !taps.pad_pick.is_empty();
    if taps.up {
        step_back(&mut menu);
        return;
    }
    if menu.step < 2 {
        if taps.down || pick {
            menu.step += 1;
            menu.selected = 0;
        }
        return;
    }
    if taps.down {
        menu.selected = (menu.selected + 1).min(LIST.len() - 1);
    }
    if pick {
        match LIST[menu.selected].0 {
            "Endless" => {
                menu.endless = 0;
                *screen = MenuScreen::Endless;
            }
            _ => *screen = MenuScreen::Trials,
        }
    }
}

/// Up or Back on the Single Player screen: up the list, else un-choose the pack, else the
/// wizard. False when there is nothing left to undo (Back then leaves the screen).
pub(crate) fn step_back(menu: &mut SingleMenu) -> bool {
    if menu.step == 2 && menu.selected > 0 {
        menu.selected -= 1;
    } else if menu.step > 0 {
        menu.step -= 1;
    } else {
        return false;
    }
    true
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
    let count = ENDLESS_ORDER.len();
    let row = ENDLESS_ORDER
        .iter()
        .position(|&i| i == menu.endless)
        .unwrap_or(0);
    if taps.up {
        menu.endless = ENDLESS_ORDER[(row + count - 1) % count];
    }
    if taps.down {
        menu.endless = ENDLESS_ORDER[(row + 1) % count];
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
    lobby: Res<crate::select::Lobby>,
    board: Res<Leaderboard>,
    mut menu_look: ResMut<crate::visuals::MenuLook>,
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
    // The original draws the picked Endless mode's own background behind the screen, and on
    // Select Mode the picked mode's in the picked difficulty's look (Special has its own).
    let modes_on = menu_up && *screen == MenuScreen::Modes;
    let look = if endless_on {
        Some((ENDLESS[menu.endless].mode, false))
    } else if modes_on {
        let mode = match lobby.modes()[lobby.mode % lobby.modes().len()] {
            crate::select::ModePick::Mode(mode) => mode,
            crate::select::ModePick::Random => ModeKind::Race,
        };
        let special = crate::select::DifficultyPick::ALL[lobby.difficulty % crate::select::DifficultyPick::ALL.len()]
            == crate::select::DifficultyPick::Set(Difficulty::Pro);
        Some((mode, special))
    } else {
        None
    };
    menu_look.set_if_neq(crate::visuals::MenuLook(look));
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
    let scores: &[u32] = if ENDLESS[menu.endless].built {
        &board.scores[menu.endless]
    } else {
        &[]
    };
    for (item, image, colour, text, text_colour, visibility) in &mut items {
        match (*item, visibility) {
            (Item::EndlessCorner(i), Some(mut v)) => {
                v.set_if_neq(shown(menu.endless == i));
                continue;
            }
            (Item::ListCorner(i), Some(mut v)) => {
                v.set_if_neq(shown(menu.step == 2 && menu.selected == i));
                continue;
            }
            (Item::SeatReady, Some(mut v)) => {
                v.set_if_neq(shown(menu.step >= 1));
                continue;
            }
            (Item::PackReady, Some(mut v)) => {
                v.set_if_neq(shown(menu.step >= 2));
                continue;
            }
            (Item::RowFocus(row), Some(mut v)) => {
                v.set_if_neq(shown(menu.step == row));
                continue;
            }
            (Item::BoardRow(row), Some(mut v)) => {
                v.set_if_neq(shown(row < scores.len()));
                continue;
            }
            _ => {}
        }
        let look = match *item {
            // List buttons stay plain; the ribbon marks the picked one.
            Item::ListButton(i) => Some(tint(true, LIST[i].1)),
            // Endless buttons stay bright too (Max's screenshots); corners mark the picked one.
            Item::EndlessButton(i) => Some(tint(true, ENDLESS[i].built)),
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
                    let want = if menu.step == 2 && menu.selected == i {
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
            Item::EndlessText(i) => format!("{} Challenge", ENDLESS[i].name),
            Item::BoardName(_) => PLAYER_NAME.to_string(),
            Item::BoardScore(row) => scores.get(row).map_or(String::new(), u32::to_string),
            Item::BestScore => scores.first().copied().unwrap_or(0).to_string(),
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

/// Shows the picked wizard and its neighbours in the carousel.
#[allow(clippy::type_complexity)]
pub(crate) fn show_carousel(
    menu: Res<SingleMenu>,
    art: Option<Res<Art>>,
    config: Res<GameConfig>,
    mut slots: Query<
        (&Carousel, &mut ImageNode, &mut Visibility),
        (Without<PackCard>, Without<CarouselLogo>),
    >,
    mut cards: Query<(&PackCard, &mut ImageNode, &mut Visibility), Without<Carousel>>,
    mut logos: Query<
        (&CarouselLogo, &mut ImageNode, &mut Visibility, &mut Node),
        (Without<PackCard>, Without<Carousel>),
    >,
) {
    if !menu.is_changed() && !art.as_ref().is_some_and(|a| a.is_changed()) {
        return;
    }
    let packs = crate::select::available_packs(art.as_deref());
    let picked = config.skins.first().map(|s| s.1).unwrap_or_default();
    let at = packs.iter().position(|p| *p == picked).unwrap_or(0) as i32;
    for (slot, mut node, mut visibility) in &mut cards {
        let pack = packs[(at + slot.0).rem_euclid(packs.len().max(1) as i32) as usize];
        // The Spring pack's card is the original's Easter one.
        let key = match pack {
            crate::config::BrickPack::Spring => "easter",
            pack => pack.key(),
        };
        let card = art
            .as_ref()
            .and_then(|a| a.image(&format!("brick_pack_{key}")));
        match &card {
            Some(image) => {
                node.image = image.clone();
                visibility.set_if_neq(Visibility::Inherited);
            }
            None => {
                visibility.set_if_neq(Visibility::Hidden);
            }
        }
    }
    let n = CAROUSEL_WIZARDS as i32;
    for (logo, mut node, mut visibility, mut layout) in &mut logos {
        let wizard = (menu.wizard as i32 + logo.0).rem_euclid(n) as usize;
        let picked = wizard
            .checked_sub(8)
            .and_then(|i| INDIE_LOGOS.get(i))
            .and_then(|&(name, size)| Some((art.as_ref()?.image(name)?, size)));
        match picked {
            Some((image, (w, h))) => {
                node.image = image;
                // Fitted into the box, keeping the logo's shape.
                let scale = if logo.0 == 0 { 1.0 } else { NEIGHBOUR_SCALE };
                let fit = (LOGO_BOX.0 / w).min(LOGO_BOX.1 / h) * scale;
                let x = NEIGHBOUR_X * logo.0 as f32;
                let y = SEAT_Y + SEAT_PANEL.1 * scale / 2.0 + LOGO_ABOVE;
                let want = place((w * fit, h * fit), (x, y));
                layout.left = want.left;
                layout.top = want.top;
                layout.width = want.width;
                layout.height = want.height;
                visibility.set_if_neq(Visibility::Inherited);
            }
            None => {
                visibility.set_if_neq(Visibility::Hidden);
            }
        }
    }
    for (slot, mut node, mut visibility) in &mut slots {
        let wizard = (menu.wizard as i32 + slot.0).rem_euclid(n) as usize;
        let pose = if slot.0 == 0 { "ready" } else { "sleeping" };
        let picture = art
            .as_ref()
            .and_then(|a| a.image(&crate::select::seat_picture(wizard, pose)));
        match picture {
            Some(image) => {
                node.image = image;
                visibility.set_if_neq(Visibility::Inherited);
            }
            None => {
                visibility.set_if_neq(Visibility::Hidden);
            }
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
