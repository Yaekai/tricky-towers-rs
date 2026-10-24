//! The "Game Completed" popup after a match. Like the original it slides up
//! into the bottom-right corner, beside WINNER! rather than over it, with a
//! right-aligned list: Rematch, Select Mode, Main menu. While a cup runs, its standings
//! panel (src/cups_menu.rs) takes the popup's place, and CONTINUE plays the next match.
//!
//! The original also counts up XP, but only online and in cups; local matches
//! give none (docs/research/match-moments.md).

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::ui::widget::NodeImageMode;

use crate::art::Art;
use crate::config::{CupPanel, GameConfig};
use crate::cups::Cup;
use crate::gameplay::{Match, MatchRequest, Phase};
use crate::menu::{MenuScreen, MenuSystems};

pub struct PopupPlugin;

impl Plugin for PopupPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Popup>()
            .add_systems(Startup, spawn_popup)
            .add_systems(
                Update,
                (time_popup, popup_input, show_popup)
                    .chain()
                    .after(MenuSystems),
            );
    }
}

/// The popup's items, top to bottom.
pub const POPUP_ITEMS: [&str; 3] = ["Rematch", "Select Mode", "Main menu"];

/// Whether the popup is up, and its highlighted item.
#[derive(Resource, Default)]
pub struct Popup {
    pub shown: bool,
    pub selected: usize,
    /// Seconds since the match ended.
    clock: f32,
    /// The clock when the popup came up.
    shown_at: f32,
    round: u64,
    /// The clock when a decided cup's CONTINUE opened the cup winner popup.
    winner_at: Option<f32>,
}

impl Popup {
    pub fn since_shown(&self) -> f32 {
        self.clock - self.shown_at
    }

    /// The standings panel takes the popup's place while a cup runs, and for a
    /// while after it is decided.
    pub fn panel_up(&self, cup: &Cup, _panel: &CupPanel) -> bool {
        self.shown && cup.size.is_some() && self.winner_at.is_none()
    }

    /// Seconds since the cup winner popup opened, while it is up.
    pub fn winner_since(&self) -> Option<f32> {
        self.winner_at.map(|at| self.clock - at)
    }

    /// CONTINUE shows on the panel once its medals have filled.
    pub fn panel_continue(&self, cup: &Cup, config: &GameConfig) -> bool {
        self.panel_up(cup, &config.cup_panel)
            && cup.rules(&config.cups).is_some()
            && self.since_shown() >= {
                let p = &config.cup_panel;
                cup.fill_times(p.fill_start, p.fill_step, p.fill_row_gap).1
            }
    }
}

/// The popup item under a cursor (top-left origin) in a window `window` big: its labels
/// sit 40 px in from the right edge, `ROW` apart below the header.
fn popup_row_at(cursor: Vec2, window: Vec2, count: usize) -> Option<usize> {
    let x = cursor.x / window.x * 1920.0;
    let y = cursor.y / window.y * 1080.0;
    let right = 1920.0 - x;
    if !(LABEL_RIGHT..=LABEL_RIGHT + LABEL.0).contains(&right) {
        return None;
    }
    let top = 1080.0 - height(count);
    (0..count).find(|&i| {
        let centre = top + ITEMS_TOP + ROW * i as f32 + ROW / 2.0;
        (y - centre).abs() <= ROW / 2.0
    })
}

/// "1st Place" to "4th Place".
pub fn place_name(place: usize) -> String {
    let suffix = match place {
        1 => "st",
        2 => "nd",
        3 => "rd",
        _ => "th",
    };
    format!("{place}{suffix} Place")
}

/// Layout in pixels of a 1080-high screen, from the bottom-right corner: the
/// content box, how far the panel art spills past it, the item rows, and the
/// bar behind the picked item.
const WIDTH: f32 = 610.0;
const HEADER: f32 = 70.0;
/// Where the items start under the panel's top: the first item's words sit 62 px under it on
/// Max's Endless clip (ListSelectComponent's own offset isn't read).
const ITEMS_TOP: f32 = 49.0;
const ROW: f32 = 60.0;
const SPILL: (f32, f32) = (120.0, 100.0);
const LABEL: (f32, f32) = (465.0, 30.0);
const LABEL_RIGHT: f32 = 40.0;
const BAR_HEIGHT: f32 = 24.0;
const BAR_RIGHT: f32 = 30.0;
/// 9-slice borders (left, right, top, bottom) of the panel and the bar.
const PANEL_SLICE: [f32; 4] = [0.0, 0.0, 198.0, 6.0];
pub(crate) const BAR_SLICE: [f32; 4] = [16.0, 5.0, 0.0, 0.0];

/// Pixels of a 1080-high screen as a UI length.
pub(crate) fn px(pixels: f32) -> Val {
    Val::Vh(pixels / 10.8)
}

pub(crate) fn sliced(image: Handle<Image>, border: [f32; 4]) -> ImageNode {
    ImageNode::new(image).with_mode(NodeImageMode::Sliced(TextureSlicer {
        border: BorderRect {
            min_inset: Vec2::new(border[0], border[2]),
            max_inset: Vec2::new(border[1], border[3]),
        },
        ..default()
    }))
}

/// EndGamePopup's Content: 60 px an item and 70 more.
fn height(items: usize) -> f32 {
    ROW * items as f32 + HEADER
}

#[derive(Component)]
struct PopupRoot;
/// The bar behind an item, shown on the picked one.
#[derive(Component)]
struct PopupBar(usize);
/// An item's words: the trial items replace the list after a trial.
#[derive(Component)]
struct PopupWords(usize);

/// The picked item is yellow, the others pink, as on the original's trial panel.
const PICKED: Color = Color::srgb(1.0, 0.85, 0.25);
const OTHER: Color = Color::srgb(1.0, 0.31, 0.55);

/// The items after a trial: Retry and Select Trial (the original's slanted panel).
const TRIAL_ITEMS: [&str; 2] = ["Retry", "Select Trial"];
/// After an Endless run: Retry and Back to menu (EndGamePopup with no Next, its quit label
/// END_GAME_POPUP_QUIT_LEADERBOARD), which go where Rematch and Select Mode go.
const ENDLESS_ITEMS: [&str; 2] = ["Retry", "Back to menu"];

/// The two-item lists that replace Rematch / Select Mode / Main menu.
fn short_items(config: &GameConfig) -> Option<[&'static str; 2]> {
    if config.trial.is_some() {
        Some(TRIAL_ITEMS)
    } else if config.mode.is_endless() {
        Some(ENDLESS_ITEMS)
    } else {
        None
    }
}

fn spawn_popup(mut commands: Commands, art: Option<Res<Art>>) {
    let image = |name: &str| art.as_ref().and_then(|a| a.image(name));
    let mut root = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: px(0.0),
            bottom: px(0.0),
            width: px(WIDTH),
            height: px(height(POPUP_ITEMS.len())),
            ..default()
        },
        GlobalZIndex(21),
        Visibility::Hidden,
        PopupRoot,
    ));
    root.with_children(|root| {
        // The panel reaches past the right and bottom edges.
        let mut panel = root.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0.0),
                top: px(0.0),
                bottom: px(-SPILL.1),
                width: px(WIDTH + SPILL.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.06, 0.14, 0.92)),
        ));
        if let Some(back) = image("popup_background") {
            panel.insert((sliced(back, PANEL_SLICE), BackgroundColor(Color::NONE)));
        }
        let bar = image("menu_item_selected");
        for (i, item) in POPUP_ITEMS.iter().enumerate() {
            let top = ITEMS_TOP + ROW * i as f32 + (ROW - LABEL.1) / 2.0;
            // The bar sits behind the label, as wide as the label's words.
            let mut row = root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: px(BAR_RIGHT),
                    top: px(top + (LABEL.1 - BAR_HEIGHT) / 2.0),
                    height: px(BAR_HEIGHT),
                    max_width: px(LABEL.0 + (LABEL_RIGHT - BAR_RIGHT)),
                    padding: UiRect::new(px(24.0), px(LABEL_RIGHT - BAR_RIGHT), px(0.0), px(0.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::FlexEnd,
                    overflow: Overflow::visible(),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                PopupBar(i),
            ));
            if let Some(bar) = &bar {
                let mut node = sliced(bar.clone(), BAR_SLICE);
                node.color = Color::NONE;
                row.insert(node);
            }
            row.with_children(|r| {
                r.spawn((
                    Text::new(item.to_uppercase()),
                    // WinPopupMenuItem: 40 px, EnzoBlackGold46 when picked, EnzoBlackRed46 not.
                    TextFont {
                        font_size: FontSize::Vh(40.0 / 10.8),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    TextLayout::justify(Justify::Right),
                    crate::letters::TextLook::PINK,
                    PopupWords(i),
                    // WinPopupMenuItem spaces -1.
                    crate::letters::LetterSpacing(-1.0),
                ));
            });
        }
    });
}

/// Puts the popup up a while after the match ends, and takes it down when a
/// new match starts or the menu opens.
fn time_popup(
    time: Res<Time>,
    config: Res<GameConfig>,
    game: Res<Match>,
    mut popup: ResMut<Popup>,
    towers: Query<(
        &crate::gameplay::Tower,
        &crate::gameplay::TowerState,
        Option<&crate::collapse::Collapse>,
    )>,
    puzzle: Option<Res<crate::results::PuzzleCountClock>>,
) {
    if game.phase != Phase::Over || popup.round != game.round {
        popup.round = game.round;
        popup.clock = 0.0;
        popup.shown = false;
        popup.winner_at = None;
        return;
    }
    // The winner's roof goes on first.
    if towers
        .iter()
        .any(|(tower, state, _)| crate::gameplay::roof_waiting(&game, tower, state))
    {
        return;
    }
    popup.clock += time.delta_secs();
    // A Puzzle battle's popup waits for its count and the end timer after it.
    let counted = puzzle.as_ref().is_none_or(|p| {
        p.end.is_none_or(|end| p.since >= end + config.puzzle.count_end_wait)
    });
    if !popup.shown
        && counted
        && popup.clock >= config.moments.popup_after
        && crate::collapse::all_done(&game, towers)
    {
        popup.shown = true;
        popup.shown_at = popup.clock;
        popup.selected = 0;
    }
}

/// Up/Down (or the d-pad) pick an item, Enter or A takes it. Rematch plays the
/// same mode again, Select Mode goes back to the mode tiles (or the Endless
/// screen after Endless Race), Main menu to the title tiles.
#[allow(clippy::too_many_arguments)]
fn popup_input(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    gamepads: Query<&Gamepad>,
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
    mut config: ResMut<GameConfig>,
    mut popup: ResMut<Popup>,
    mut game: ResMut<Match>,
    mut screen: ResMut<MenuScreen>,
    mut request: ResMut<MatchRequest>,
    mut cup: ResMut<crate::cups::Cup>,
    mut last_cursor: Local<Option<Vec2>>,
) {
    if !popup.shown || game.phase != Phase::Over {
        return;
    }
    // The cup winner popup closes by itself, fades to black and goes to Select Mode
    // (CupMatchFlowController._HandleFinishTournament).
    if let Some(since) = popup.winner_since() {
        let p = &config.cup_panel;
        if since >= p.winner_close + p.winner_fade {
            game.phase = Phase::Menu;
            *screen = MenuScreen::Modes;
            popup.shown = false;
            popup.winner_at = None;
        }
        return;
    }
    let count = short_items(&config).map_or(POPUP_ITEMS.len(), |items| items.len());
    let (mut up, mut down, mut pick) = (false, false, false);
    if let Some(keys) = keys {
        // DEBUG-TOOL: not in the original; remove before the final build (docs/release-checklist.md).
        // WASD and the numpad arrows: scripted presses on Windows and habit (see menu.rs).
        up |= keys.any_just_pressed([KeyCode::ArrowUp, KeyCode::KeyW, KeyCode::Numpad8]);
        down |= keys.any_just_pressed([KeyCode::ArrowDown, KeyCode::KeyS, KeyCode::Numpad2]);
        pick |= keys.any_just_pressed([KeyCode::Enter, KeyCode::Space, KeyCode::NumpadEnter]);
    }
    for pad in &gamepads {
        up |= pad.just_pressed(GamepadButton::DPadUp);
        down |= pad.just_pressed(GamepadButton::DPadDown);
        pick |= pad.just_pressed(crate::input::continue_back(pad).0);
    }
    // The mouse: an item under the cursor is selected as it moves, and a click picks it.
    if let Ok(window) = windows.single() {
        let cursor = window.cursor_position();
        let over = cursor.and_then(|c| popup_row_at(c, window.size(), count));
        if let Some(i) = over
            && cursor != *last_cursor
        {
            popup.selected = i;
        }
        *last_cursor = cursor;
        if let Some(i) = over
            && mouse.as_deref().is_some_and(|m| m.just_pressed(MouseButton::Left))
        {
            popup.selected = i;
            pick = true;
        }
    }
    if popup.panel_up(&cup, &config.cup_panel) {
        // CONTINUE takes an unfinished cup on, or opens a decided cup's winner popup.
        up = false;
        down = false;
        pick &= popup.panel_continue(&cup, &config);
        if pick && !cup.in_progress() {
            popup.winner_at = Some(popup.clock);
            return;
        }
    }
    if up {
        popup.selected = (popup.selected + count - 1) % count;
    }
    if down {
        popup.selected = (popup.selected + 1) % count;
    }
    if !pick {
        return;
    }
    match POPUP_ITEMS[popup.selected.min(POPUP_ITEMS.len() - 1)] {
        "Main menu" => {
            game.phase = Phase::Menu;
            *screen = MenuScreen::Main;
        }
        "Select Mode" => {
            game.phase = Phase::Menu;
            *screen = if config.trial.is_some() {
                MenuScreen::Trials
            } else if config.mode.is_endless() {
                MenuScreen::Endless
            } else {
                MenuScreen::Modes
            };
        }
        _ => {
            // Next match carries on a running cup; after its last match (or with
            // no cup) Rematch starts the same again from scratch.
            if !cup.in_progress() {
                cup.begin();
                request.retry = true;
            } else if let Some((mode, difficulty)) =
                cup.next_match(time.elapsed().as_nanos() as usize / 1000)
            {
                config.mode = mode;
                config.difficulty = Some(difficulty);
            }
            request.pending = true;
        }
    }
    popup.shown = false;
}

#[allow(clippy::type_complexity)]
fn show_popup(
    config: Res<GameConfig>,
    popup: Res<Popup>,
    mut root: Query<(&mut Visibility, &mut Node), (With<PopupRoot>, Without<PopupBar>)>,
    mut bars: Query<(
        &PopupBar,
        Option<&mut ImageNode>,
        &mut BackgroundColor,
        &mut Visibility,
        &mut Node,
    )>,
    cup: Res<crate::cups::Cup>,
    art: Option<Res<Art>>,
    mut texts: Query<(&PopupWords, &mut Text, &mut TextColor, &mut crate::letters::TextLook)>,
) {
    let short = short_items(&config);
    for (item, mut text, mut color, mut look) in &mut texts {
        let words = match short {
            Some(items) => items.get(item.0).copied().unwrap_or(""),
            None => POPUP_ITEMS[item.0],
        }
        .to_uppercase();
        if text.0 != words {
            text.0 = words;
        }
        let picked = item.0 == popup.selected;
        let want = if picked { PICKED } else { OTHER };
        if color.0 != want {
            color.0 = want;
        }
        look.set_if_neq(if picked {
            crate::letters::TextLook::GOLD
        } else {
            crate::letters::TextLook::PINK
        });
    }
    // Slides up from below the bottom edge, easing out.
    let t = if config.moments.popup_slide > 0.0 {
        ((popup.clock - config.moments.popup_after) / config.moments.popup_slide).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let eased = 1.0 - (1.0 - t) * (1.0 - t);
    let items = short.map_or(POPUP_ITEMS.len(), |items| items.len());
    let bottom = px(-(height(items) + SPILL.1) * (1.0 - eased));
    let visible = popup.shown
        && !popup.panel_up(&cup, &config.cup_panel)
        && popup.winner_since().is_none();
    for (mut v, mut node) in &mut root {
        v.set_if_neq(if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        if node.bottom != bottom {
            node.bottom = bottom;
        }
        let tall = px(height(items));
        if node.height != tall {
            node.height = tall;
        }
    }
    if !visible {
        return;
    }
    for (bar, image, mut colour, mut shown, mut node) in &mut bars {
        // The bar is as wide as its words (the label's content-size fit) and the padding;
        // the words are drawn by their letter twin, which gives the row no width of its own.
        let words = match short {
            Some(items) => items.get(bar.0).copied().unwrap_or(""),
            None => POPUP_ITEMS[bar.0],
        };
        if let Some(art) = art.as_deref() {
            let wide = crate::letters::line_width(art, &words.to_uppercase(), 40.0);
            if wide > 0.0 {
                let width = px(wide + 24.0 + (LABEL_RIGHT - BAR_RIGHT));
                if node.width != width {
                    node.width = width;
                }
            }
        }
        shown.set_if_neq(if short.is_some_and(|items| bar.0 >= items.len()) {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
        let picked = bar.0 == popup.selected;
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
