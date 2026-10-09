//! The "Game Completed" popup after a match. Like the original it slides up
//! into the bottom-right corner, beside WINNER! rather than over it, with a
//! right-aligned list: Rematch, Select Mode, Main menu. While a cup runs, its standings
//! panel (src/cups_menu.rs) takes the popup's place, and CONTINUE plays the next match.
//!
//! The original also counts up XP, but only online and in cups; local matches
//! give none (docs/research/match-moments.md).

use bevy::prelude::*;
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
}

impl Popup {
    pub fn since_shown(&self) -> f32 {
        self.clock - self.shown_at
    }

    /// The standings panel takes the popup's place while a cup runs, and for a
    /// while after it is decided.
    pub fn panel_up(&self, cup: &Cup, panel: &CupPanel) -> bool {
        self.shown
            && cup.size.is_some()
            && (cup.in_progress() || self.since_shown() < panel.decided_hold)
    }

    /// CONTINUE shows on the panel once its medals have filled.
    pub fn panel_continue(&self, cup: &Cup, config: &GameConfig) -> bool {
        self.panel_up(cup, &config.cup_panel)
            && cup
                .rules(&config.cups)
                .is_some_and(|size| self.since_shown() >= config.cup_panel.continue_at(size.target))
    }
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

fn height() -> f32 {
    ROW * POPUP_ITEMS.len() as f32 + HEADER
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

fn spawn_popup(mut commands: Commands, art: Option<Res<Art>>) {
    let image = |name: &str| art.as_ref().and_then(|a| a.image(name));
    let mut root = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: px(0.0),
            bottom: px(0.0),
            width: px(WIDTH),
            height: px(height()),
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
                width: px(WIDTH + SPILL.0),
                height: px(height() + SPILL.1),
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.06, 0.14, 0.92)),
        ));
        if let Some(back) = image("popup_background") {
            panel.insert((sliced(back, PANEL_SLICE), BackgroundColor(Color::NONE)));
        }
        let bar = image("menu_item_selected");
        for (i, item) in POPUP_ITEMS.iter().enumerate() {
            let top = HEADER + ROW * i as f32 + (ROW - LABEL.1) / 2.0;
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
                    TextFont {
                        font_size: FontSize::Vh(LABEL.1 / 10.8),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    TextLayout::justify(Justify::Right),
                    crate::letters::TextLook::OUTLINED,
                    PopupWords(i),
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
) {
    if game.phase != Phase::Over || popup.round != game.round {
        popup.round = game.round;
        popup.clock = 0.0;
        popup.shown = false;
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
    if !popup.shown
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
    config: Res<GameConfig>,
    mut popup: ResMut<Popup>,
    mut game: ResMut<Match>,
    mut screen: ResMut<MenuScreen>,
    mut request: ResMut<MatchRequest>,
    mut cup: ResMut<crate::cups::Cup>,
) {
    if !popup.shown || game.phase != Phase::Over {
        return;
    }
    let count = if config.trial.is_some() {
        TRIAL_ITEMS.len()
    } else {
        POPUP_ITEMS.len()
    };
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
    }
    if popup.panel_up(&cup, &config.cup_panel) {
        // CONTINUE takes an unfinished cup on; a decided cup's panel gives way by itself.
        up = false;
        down = false;
        pick &= cup.in_progress() && popup.panel_continue(&cup, &config);
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
    )>,
    cup: Res<crate::cups::Cup>,
    mut texts: Query<(&PopupWords, &mut Text, &mut TextColor)>,
) {
    let trial = config.trial.is_some();
    for (item, mut text, mut color) in &mut texts {
        let words = match (trial, item.0) {
            (true, i) => TRIAL_ITEMS.get(i).copied().unwrap_or(""),
            (false, i) => POPUP_ITEMS[i],
        }
        .to_uppercase();
        if text.0 != words {
            text.0 = words;
        }
        let want = if item.0 == popup.selected {
            PICKED
        } else {
            OTHER
        };
        if color.0 != want {
            color.0 = want;
        }
    }
    // Slides up from below the bottom edge, easing out.
    let t = if config.moments.popup_slide > 0.0 {
        ((popup.clock - config.moments.popup_after) / config.moments.popup_slide).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let eased = 1.0 - (1.0 - t) * (1.0 - t);
    let bottom = px(-(height() + SPILL.1) * (1.0 - eased));
    let visible = popup.shown && !popup.panel_up(&cup, &config.cup_panel);
    for (mut v, mut node) in &mut root {
        v.set_if_neq(if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        if node.bottom != bottom {
            node.bottom = bottom;
        }
    }
    if !visible {
        return;
    }
    for (bar, image, mut colour, mut shown) in &mut bars {
        shown.set_if_neq(if trial && bar.0 >= TRIAL_ITEMS.len() {
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
