//! What the player sees of local cups (picked in Select Mode's third column,
//! src/select.rs): the standings panel between cup matches (laid out from the
//! original's Tiny Cup screen, `GameConfig::cup_panel`) and the cup winner popup after a
//! decided cup (TournamentWinnerPopup).

use bevy::prelude::*;

use crate::art::Art;
use crate::config::{GameConfig, keyed};
use crate::cups::Cup;
use crate::gameplay::{MAX_TOWERS, Tower, TowerSkin};
use crate::letters::TextLook;
use crate::popup::{Popup, px};
use crate::select::{SCREEN, label_with, shown};

pub struct CupsMenuPlugin;

impl Plugin for CupsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_cup_panel, spawn_cup_winner))
            .add_systems(Update, (show_cup_panel, show_cup_winner, celebrate, fly_winner).chain())
            .add_systems(Update, medal_sounds);
    }
}

/// Most medal slots a row holds: the biggest cup's target.
const SLOTS: usize = 18;

/// One piece of the standings panel. Rows are in standings order, not seat order.
#[derive(Component, Clone, Copy, PartialEq)]
enum Part {
    Root,
    Panel,
    /// The panel's picture (PlayerInfo: gph_popupBackgroundPanel, 9-sliced), sized
    /// (target - 1) * 47 + 420 by rows * 98 + 180 and centred 102 px above the middle.
    Frame,
    /// A cup decoration: right end or not, leaves or cup.
    Deco(bool, bool),
    Title,
    Row(usize),
    Name(usize),
    Face(usize),
    Slot(usize, usize),
    Trophy(usize),
    /// "+N": medals past the target (TournamentResultsItem's OvertimeLabel).
    Extra(usize),
    Continue,
}

fn rgb(c: (u8, u8, u8)) -> Color {
    Color::srgb_u8(c.0, c.1, c.2)
}

/// An absolute box in pixels from its parent's top-left corner.
fn at(x: f32, y: f32, w: f32, h: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(x),
        top: px(y),
        width: px(w),
        height: px(h),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

fn blank() -> ImageNode {
    ImageNode {
        color: Color::NONE,
        ..default()
    }
}

fn spawn_cup_panel(mut commands: Commands, config: Res<GameConfig>, art: Option<Res<Art>>) {
    let p = &config.cup_panel;
    let bar_y = p.bar_top - p.panel_top;
    let frame = art.as_ref().and_then(|a| a.image("menu_leaderboard_panel"));
    // With the picture, it draws the panel and its title bar.
    let fill = |c| if frame.is_some() { Color::NONE } else { rgb(c) };
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(rgb(p.dim_colour).with_alpha(p.dim)),
            GlobalZIndex(22),
            Visibility::Hidden,
            Part::Root,
        ))
        .with_children(|root| {
            if let Some(frame) = frame.clone() {
                root.spawn((
                    at(0.0, 0.0, 0.0, 0.0),
                    crate::popup::sliced(frame, [0.0, 0.0, 37.0, 51.0]),
                    Part::Frame,
                ));
            }
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    margin: UiRect::left(px(p.panel_left - SCREEN.0 / 2.0)),
                    top: px(p.panel_top),
                    width: px(p.panel_width),
                    height: px(p.panel_height),
                    ..default()
                },
                BackgroundColor(fill(p.panel_colour)),
                Part::Panel,
            ))
            .with_children(|panel| {
                panel.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        ..at(0.0, bar_y, p.panel_width, p.bar_height)
                    },
                    BackgroundColor(fill(p.bar_colour)),
                    children![(
                        label_with(
                            p.title_size,
                            "",
                            TextLook {
                                face: Some(rgb(p.title_colour)),
                                ..TextLook::OUTLINED
                            }
                        ),
                        UiTransform::from_translation(Val2::px(0.0, -p.title_lift)),
                        Part::Title
                    )],
                ));
                // TournamentResultsItem's PlayerName: 18 px Enzo.
                let name_look = TextLook {
                    body: false,
                    face: Some(rgb(p.name_colour)),
                    outline: None,
                    low: None,
                    outline_edge: 0.0,
                };
                for i in 0..MAX_TOWERS {
                    panel
                        .spawn((
                            at(
                                p.row_left - p.panel_left,
                                p.row_top - p.panel_top + i as f32 * p.row_pitch,
                                p.row_width,
                                p.row_height,
                            ),
                            BackgroundColor(rgb(p.row_colour)),
                            Visibility::Hidden,
                            Part::Row(i),
                        ))
                        .with_children(|row| {
                            row.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(p.name_left - p.row_left),
                                    top: px(p.name_top),
                                    ..default()
                                },
                                // As written, not in capitals (the row's PlayerName is EnzoBlack small
                                // letters).
                                children![(
                                    Text::new(""),
                                    name_look,
                                    TextFont { font_size: FontSize::Vh(p.name_size), ..default() },
                                    TextColor(Color::WHITE),
                                    TextLayout::justify(Justify::Center),
                                    Part::Name(i)
                                )],
                            ));
                            row.spawn((
                                // The wizard's avatar (PLAYER_OVERVIEW_AVATAR: the round game-info icon,
                                // 49 x 45).
                                at(
                                    p.portrait_left - p.row_left + (p.portrait_size - 49.0) / 2.0,
                                    (p.row_height - 45.0) / 2.0,
                                    49.0,
                                    45.0,
                                ),
                                blank(),
                                Part::Face(i),
                            ));
                            for k in 0..SLOTS {
                                row.spawn((
                                    Node {
                                        position_type: PositionType::Absolute,
                                        border_radius: BorderRadius::MAX,
                                        ..default()
                                    },
                                    blank(),
                                    BackgroundColor(Color::NONE),
                                    Visibility::Hidden,
                                    Part::Slot(i, k),
                                ));
                            }
                            row.spawn((
                                Node {
                                    left: Val::Auto,
                                    right: px(p.row_left + p.row_width - p.trophy_x - p.trophy_size / 2.0),
                                    ..at(0.0, (p.row_height - p.trophy_size) / 2.0, p.trophy_size, p.trophy_size)
                                },
                                blank(),
                                Visibility::Hidden,
                                Part::Trophy(i),
                            ));
                            // From the row's right end, so a wider cup's row carries it.
                            row.spawn((
                                Node {
                                    left: Val::Auto,
                                    right: px(p.row_left + p.row_width - p.trophy_x - p.trophy_size / 2.0),
                                    ..at(0.0, 0.0, p.trophy_size, p.row_height)
                                },
                                children![(
                                    label_with(p.name_size * 1.6, "", name_look),
                                    Part::Extra(i)
                                )],
                            ));
                        });
                }
            });
            // Leaves first, so the cups sit over them.
            for leaves in [true, false] {
                for right in [false, true] {
                    root.spawn((at(0.0, 0.0, 0.0, 0.0), blank(), Part::Deco(right, leaves)));
                }
            }
            // CONTINUE on its picked-item bar, centred 35 px under the panel
            // (TournamentResultsPopup's ConfirmButton: EnzoBlackGold46 on the 28 px high
            // gph_menuItemSelectedBackground).
            let bar = art.as_ref().and_then(|a| a.image("menu_item_selected"));
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    margin: UiRect::left(px(-CONTINUE_BOX.0 / 2.0)),
                    width: px(CONTINUE_BOX.0),
                    height: px(CONTINUE_BOX.1),
                    top: px(p.tab_bottom + CONTINUE_BELOW - CONTINUE_BOX.1 / 2.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                Visibility::Hidden,
                Part::Continue,
            ))
            .with_children(|c| {
                if let Some(bar) = bar {
                    c.spawn((
                        ImageNode::new(bar).with_mode(NodeImageMode::Sliced(TextureSlicer {
                            border: BorderRect {
                                min_inset: Vec2::new(16.0, 0.0),
                                max_inset: Vec2::new(5.0, 0.0),
                            },
                            ..default()
                        })),
                        Node {
                            position_type: PositionType::Absolute,
                            width: px(CONTINUE_BAR.0),
                            height: px(CONTINUE_BAR.1),
                            ..default()
                        },
                    ));
                }
                c.spawn(label_with(
                    p.continue_size,
                    "Continue",
                    TextLook::GOLD,
                ));
            });
        });
}

/// The CONTINUE box, how far under the panel's picture (its tab included) its middle sits,
/// and its bar.
const CONTINUE_BOX: (f32, f32) = (400.0, 60.0);
const CONTINUE_BELOW: f32 = 35.0;
const CONTINUE_BAR: (f32, f32) = (280.0, 28.0);

fn set_box(node: &mut Node, x: f32, y: f32, w: f32, h: f32) {
    let want = (px(x), px(y), px(w), px(h));
    if (node.left, node.top, node.width, node.height) != want {
        (node.left, node.top, node.width, node.height) = want;
    }
}

fn set_image(node: &mut ImageNode, image: Option<Handle<Image>>) {
    let (image, color) = match image {
        Some(image) => (image, Color::WHITE),
        None => (node.image.clone(), Color::NONE),
    };
    if node.image != image {
        node.image = image;
    }
    if node.color != color {
        node.color = color;
    }
}

/// The standings panel: slides up when the popup is due, fills each row's medal
/// slots one by one (one per point, the medal of the place that earned it), puts
/// the trophy on a decided winner's row, then shows CONTINUE.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn show_cup_panel(
    config: Res<GameConfig>,
    cup: Res<Cup>,
    popup: Res<Popup>,
    art: Option<Res<Art>>,
    towers: Query<(&Tower, Option<&TowerSkin>)>,
    mut nodes: Query<(
        &Part,
        &mut Node,
        &mut Visibility,
        Option<&mut ImageNode>,
        Option<&mut BackgroundColor>,
    )>,
    mut texts: Query<(&Part, &mut Text)>,
    grey: Option<ResMut<crate::post::GreyTarget>>,
    mut was_up: Local<bool>,
) {
    let p = &config.cup_panel;
    let up = popup.panel_up(&cup, p);
    // Behind the standings and the winner popup the world turns grey (the popups' Black &
    // White background). Only a change writes it, so the pause can grey the world too.
    let grey_up = up || popup.winner_since().is_some();
    if let Some(mut grey) = grey
        && grey_up != *was_up
    {
        grey.0 = if grey_up { 1.0 } else { 0.0 };
    }
    *was_up = grey_up;
    let image = |name: &str| art.as_ref().and_then(|a| a.image(name));
    let order: Vec<usize> = cup.panel_order();
    let tower = |row: usize| {
        order
            .get(row)
            .and_then(|&t| towers.iter().find(|(tower, _)| tower.index == t))
    };
    let size = cup.rules(&config.cups);
    let target = size.map_or(0, |s| s.target as usize).min(SLOTS);
    let since = popup.since_shown();
    let grow = order.len().saturating_sub(2) as f32 * p.row_pitch;
    // A bigger cup is wider by a slot per point past the Tiny Cup's 6, on both sides.
    let wide = (target as f32 - 6.0) * p.slot_pitch;
    let pitch = p.slot_pitch;
    let (cell, dot) = (p.slot_size, p.dot_size);
    // BasePopup: the popup fades in, then its panel moves up a whole screen with out-back.
    let slide = if p.slide.1 > 0.0 {
        let t = ((since - p.slide.0) / p.slide.1).clamp(0.0, 1.0);
        let (c1, c3) = (1.70158, 2.70158);
        1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
    } else {
        1.0
    };
    let drop = (1.0 - slide) * SCREEN.1;
    let winner = cup.standings.as_ref().and_then(|s| s.first().copied());
    let (fill, fill_end) = cup.fill_times(p.fill_start, p.fill_step, p.fill_row_gap);
    let filled = size.is_some() && since >= fill_end;
    // Medals from earlier matches are there at once; the latest match's fill in.
    let shown_medals = |t: usize| shown_medals(&cup, &fill, since, t);

    for (part, mut node, mut visibility, image_node, colour) in &mut nodes {
        if *part == Part::Root {
            visibility.set_if_neq(shown(up));
        }
        if !up {
            continue;
        }
        match *part {
            Part::Panel => {
                let (top, height) = (px(p.panel_top + drop), px(p.panel_height + grow));
                let (margin, width) = (
                    px(p.panel_left - SCREEN.0 / 2.0 - wide / 2.0),
                    px(p.panel_width + wide),
                );
                if node.top != top || node.height != height {
                    (node.top, node.height) = (top, height);
                }
                if node.margin.left != margin || node.width != width {
                    (node.margin.left, node.width) = (margin, width);
                }
            }
            Part::Frame => {
                let width = (target as f32 - 1.0).max(0.0) * p.slot_pitch + 420.0;
                // It follows the white panel (31 px over it, 75 px taller: Max's two-row clip),
                // so a third and fourth row grow it downwards with the rows.
                let height = p.panel_height + grow + 75.0;
                let want = (
                    Val::Percent(50.0),
                    px(-width / 2.0),
                    px(p.panel_top - 31.0 + drop),
                    px(width),
                    px(height),
                );
                if (node.left, node.margin.left, node.top, node.width, node.height) != want {
                    (node.left, node.margin.left, node.top, node.width, node.height) = want;
                }
            }
            Part::Deco(right, leaves) => {
                let deco = cup.size.and_then(|s| p.decos.get(s));
                let (name, size, at) = match deco {
                    Some(d) if leaves => (d.leaves.as_str(), d.leaves_size, d.leaves_at),
                    Some(d) => (d.cup.as_str(), d.cup_size, d.cup_at),
                    None => ("", (0.0, 0.0), (0.0, 0.0)),
                };
                // The left end is the right one mirrored about its corner.
                let corner = if right {
                    p.bar_ends.1 + wide / 2.0
                } else {
                    p.bar_ends.0 - wide / 2.0
                };
                let x = if right { corner + at.0 } else { corner - at.0 };
                let y = p.bar_top + drop - at.1;
                // Centred like the panel, so wide screens keep it in place.
                let want = (
                    Val::Percent(50.0),
                    px(x - size.0 / 2.0 - SCREEN.0 / 2.0),
                    px(y - size.1 / 2.0),
                    px(size.0),
                    px(size.1),
                );
                if (node.left, node.margin.left, node.top, node.width, node.height) != want {
                    (node.left, node.margin.left, node.top, node.width, node.height) = want;
                }
                if let Some(mut deco) = image_node {
                    set_image(&mut deco, image(name));
                    // The leaves carry a -1 scale on the right; the left end flips the cup.
                    let flip = right == leaves;
                    if deco.flip_x != flip {
                        deco.flip_x = flip;
                    }
                }
            }
            Part::Row(i) => {
                visibility.set_if_neq(shown(i < order.len()));
                let width = px(p.row_width + wide);
                if node.width != width {
                    node.width = width;
                }
            }
            Part::Face(i) => {
                if let Some(mut face) = image_node {
                    let number = tower(i).map(|(t, skin)| skin.map_or(t.index, |s| s.wizard));
                    set_image(
                        &mut face,
                        number.and_then(|w| match w {
                            0..=3 => image(&format!("wizard_{}", w + 1)),
                            4..=7 => image(&format!("menu_game_info_wizard_animal0{}", w - 3)),
                            _ => image(&format!("menu_game_info_wizard_indie0{}", w - 7)),
                        }),
                    );
                }
            }
            Part::Slot(i, k) => {
                visibility.set_if_neq(shown(k < target));
                if k >= target {
                    continue;
                }
                let medal = order
                    .get(i)
                    .filter(|&&t| k < shown_medals(t))
                    .and_then(|&t| cup.medal(t, k))
                    .map(|place| place.min(3));
                let x = p.slot_left - p.row_left + k as f32 * pitch;
                let y = (p.row_height - cell) / 2.0;
                let (picture, fill) = match medal {
                    Some(place) => {
                        let (w, h) = p.medal_size;
                        set_box(&mut node, x + (cell - w) / 2.0, y + (cell - h) / 2.0, w, h);
                        let picture = p.medals.get(place).and_then(|n| image(n));
                        let fill = match picture {
                            Some(_) => Color::NONE,
                            None => p
                                .medal_colours
                                .get(place)
                                .copied()
                                .map_or(Color::WHITE, rgb),
                        };
                        (picture, fill)
                    }
                    None => {
                        let inset = (cell - dot) / 2.0;
                        set_box(&mut node, x + inset, y + inset, dot, dot);
                        (None, rgb(p.dot_colour))
                    }
                };
                if let Some(mut slot) = image_node {
                    set_image(&mut slot, picture);
                }
                if let Some(mut colour) = colour
                    && colour.0 != fill
                {
                    colour.0 = fill;
                }
            }
            Part::Trophy(i) => {
                let on = filled && winner.is_some() && order.get(i).copied() == winner;
                visibility.set_if_neq(shown(on));
                let small = cup.size.and_then(|s| p.row_trophies.get(s));
                if let Some((_, w, h)) = small {
                    let want = (px(*w), px(*h), px((p.row_height - h) / 2.0));
                    if (node.width, node.height, node.top) != want {
                        (node.width, node.height, node.top) = want;
                    }
                    let right = px(p.row_left + p.row_width - p.trophy_x - w / 2.0);
                    if node.right != right {
                        node.right = right;
                    }
                }
                if let Some(mut trophy) = image_node {
                    let picture = small
                        .and_then(|(name, ..)| image(name))
                        .or_else(|| size.and_then(|s| image(&s.trophy)));
                    set_image(&mut trophy, picture);
                }
            }
            Part::Continue => {
                visibility.set_if_neq(shown(popup.panel_continue(&cup, &config)));
                let top = px(p.tab_bottom + grow + CONTINUE_BELOW - CONTINUE_BOX.1 / 2.0);
                if node.top != top {
                    node.top = top;
                }
            }
            _ => {}
        }
    }
    if !up {
        return;
    }
    for (part, mut text) in &mut texts {
        let words = match *part {
            Part::Title => size.map(|s| s.name.to_uppercase()),
            // Cleared once the trophy comes (ShowWinner).
            Part::Extra(i) => order.get(i).map(|&t| {
                let extra = shown_medals(t).saturating_sub(target);
                let trophy = filled && winner == Some(t);
                if extra > 0 && !trophy { format!("+{extra}") } else { String::new() }
            }),
            Part::Name(i) => tower(i).map(|(t, _)| match t.player {
                Some(n) => format!("Player {}", n + 1),
                None => format!("Wizard {}", t.index + 1),
            }),
            _ => None,
        };
        if let Some(words) = words
            && text.0 != words
        {
            text.0 = words;
        }
    }
}
/// One piece of the cup winner popup (TournamentWinnerPopup), laid out on the 1920 x 1080
/// screen from its prefab: PlayerInfo 950 x 580 centred 75 px over the middle, the title on
/// its top edge, the name panel 100 px down with four medal counts on it, and the trophy at
/// the bottom on its starburst.
#[derive(Component, Clone, Copy, PartialEq)]
enum Win {
    Root,
    Fade,
    Title,
    Name,
    /// A medal count: 0 fourth, 1 bronze, 2 silver, 3 gold (the order they count in).
    Medal(usize),
    Count(usize),
    Trophy,
    /// The trophy's two sunray layers (TrophySunraysAnimatorController's RotateSunrays).
    Rays(usize),
    Cup,
    Deco(bool),
}

/// PlayerInfo's top-left corner and size.
const WIN_PANEL: (f32, f32, f32, f32) = (485.0, 175.0, 950.0, 580.0);
/// PlayerPanel: 800 x 125, its top 100 px under PlayerInfo's.
const WIN_NAME_PANEL: (f32, f32) = (800.0, 125.0);
const WIN_NAME_DOWN: f32 = 100.0;
/// Each medal box's middle from the name panel's left (Fourth 215 from the left, Bronze and
/// Silver 60 either side of the middle, Gold 215 from the right; all 35 px up from its
/// bottom), its medal picture's size and lift, and the picture.
const WIN_MEDALS: [(f32, (f32, f32), f32, &str); 4] = [
    (215.0, (48.0, 48.0), 6.0, "menu_leaderboard_medal_4"),
    (340.0, (44.0, 54.0), -3.0, "menu_leaderboard_medal_3"),
    (460.0, (40.0, 49.0), 2.0, "menu_leaderboard_medal_2"),
    (585.0, (44.0, 52.0), 0.0, "menu_leaderboard_medal_1"),
];
/// TrophyContainer: 475 x 195, its bottom 10 px over PlayerInfo's, 7 px left of the middle.
const WIN_TROPHY_BOX: (f32, f32) = (475.0, 195.0);
/// Each cup's trophy picture size and how far under its box's middle it sits, and its
/// sunrays' size (TrophyTinyCup, TrophyBigCup, TrophySuperCup).
const WIN_TROPHIES: [((f32, f32), f32, f32); 3] = [
    ((111.0, 132.0), 17.33, 200.0),
    ((203.0, 203.0), 0.0, 300.0),
    ((307.0, 247.0), 0.0, 350.0),
];

/// A box centred like the 1920-wide screen, from its top-left corner in screen pixels.
fn screen_box(x: f32, y: f32, w: f32, h: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Percent(50.0),
        margin: UiRect::left(px(x - SCREEN.0 / 2.0)),
        top: px(y),
        width: px(w),
        height: px(h),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

fn spawn_cup_winner(mut commands: Commands, config: Res<GameConfig>, art: Option<Res<Art>>) {
    let p = &config.cup_panel;
    let image = |name: &str| art.as_ref().and_then(|a| a.image(name));
    let (px0, py0, pw, ph) = WIN_PANEL;
    let name_top = py0 + WIN_NAME_DOWN;
    let name_left = SCREEN.0 / 2.0 - WIN_NAME_PANEL.0 / 2.0;
    let trophy_top = py0 + ph - 10.0 - WIN_TROPHY_BOX.1;
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            // As dark as the standings behind it (Max's Tiny Cup clip measures the same).
            BackgroundColor(rgb(p.dim_colour).with_alpha(p.dim)),
            GlobalZIndex(23),
            Visibility::Hidden,
            Win::Root,
        ))
        .with_children(|root| {
            let mut frame = root.spawn(screen_box(px0, py0, pw, ph));
            match image("menu_leaderboard_panel") {
                Some(panel) => frame.insert(crate::popup::sliced(panel, [0.0, 0.0, 37.0, 51.0])),
                None => frame.insert(BackgroundColor(rgb(p.panel_colour))),
            };
            // The cup decorations by the panel's top corners (CUP_DECO_<cup>), the left one
            // mirrored.
            for right in [false, true] {
                root.spawn((screen_box(0.0, 0.0, 0.0, 0.0), blank(), Win::Deco(right)));
            }
            // Title: EnzoBlackGold46 at 65, its middle 14.81 px over the panel's top.
            root.spawn(screen_box(
                px0 + 100.0,
                py0 - 14.81 - 50.0,
                pw - 200.0,
                100.0,
            ))
            .with_children(|t| {
                t.spawn((label_with(65.0 / 10.8, "", TextLook::GOLD), Win::Title));
            });
            let mut back = root.spawn(screen_box(
                name_left,
                name_top,
                WIN_NAME_PANEL.0,
                WIN_NAME_PANEL.1,
            ));
            if let Some(name_back) = image("cup_winner_name_back") {
                back.insert(ImageNode::new(name_back));
            }
            // PlayerName: EnzoBlackGold46 at 68.85 on a 25 px line from 10 px over the panel.
            root.spawn(screen_box(
                name_left + 50.0,
                name_top - 10.0 - 37.5,
                WIN_NAME_PANEL.0 - 100.0,
                100.0,
            ))
            .with_children(|t| {
                t.spawn((label_with(68.85 / 10.8, "", TextLook::GOLD), Win::Name));
            });
            for (i, &(x, size, lift, picture)) in WIN_MEDALS.iter().enumerate() {
                let (cx, cy) = (name_left + x, name_top + WIN_NAME_PANEL.1 - 35.0);
                root.spawn((
                    screen_box(cx - 50.0, cy - 25.0, 100.0, 50.0),
                    UiTransform::default(),
                    Win::Medal(i),
                ))
                .with_children(|m| {
                    // The medal sits in the box's bottom-right corner; its count on a
                    // 50 px line from the box's left, 30 px up (EnzoBlackBlack 26).
                    let mut medal =
                        m.spawn(at(100.0 - size.0, 50.0 - size.1 - lift, size.0, size.1));
                    if let Some(picture) = image(picture) {
                        medal.insert(ImageNode::new(picture));
                    }
                    m.spawn(at(0.0, -30.0, 50.0, 100.0)).with_children(|c| {
                        let black = TextLook {
                            face: Some(Color::BLACK),
                            ..default()
                        };
                        c.spawn((label_with(26.0 / 10.8, "0", black), Win::Count(i)));
                    });
                });
            }
            let mut starburst = root.spawn(screen_box(
                SCREEN.0 / 2.0 - 7.0 - WIN_TROPHY_BOX.0 / 2.0,
                trophy_top,
                WIN_TROPHY_BOX.0,
                WIN_TROPHY_BOX.1,
            ));
            if let Some(picture) = image("cup_winner_trophy_back") {
                starburst.insert(ImageNode::new(picture));
            }
            // The trophy: a 100 px box standing on the container's bottom edge, its rays and
            // cup centred in it, growing in as one.
            root.spawn((
                screen_box(
                    SCREEN.0 / 2.0 - 7.0 - 50.0,
                    trophy_top + WIN_TROPHY_BOX.1 - 100.0,
                    100.0,
                    100.0,
                ),
                UiTransform::from_scale(Vec2::ZERO),
                Win::Trophy,
            ))
            .with_children(|t| {
                if let Some(rays) = image("winner_sunrays") {
                    for k in 0..2 {
                        t.spawn((
                            ImageNode {
                                color: Color::srgba(0.929, 0.969, 1.0, 0.235),
                                ..ImageNode::new(rays.clone())
                            },
                            at(0.0, 0.0, 0.0, 0.0),
                            UiTransform::default(),
                            Win::Rays(k),
                        ));
                    }
                }
                t.spawn((at(0.0, 0.0, 0.0, 0.0), blank(), Win::Cup));
            });
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                Win::Fade,
            ));
        });
}

/// The cup winner popup's medal counts `t` seconds after it opened: each count shown and its
/// medal's scale (fourth, bronze, silver, gold), and when the trophy starts growing. A count
/// with nothing to count is skipped (AnimateMedalAmountEffect).
pub fn winner_counts(
    t: f32,
    amounts: [u32; 4],
    p: &crate::config::CupPanel,
) -> ([u32; 4], [f32; 4], f32) {
    let (big, pop) = p.winner_pop;
    let mut at = p.winner_in + p.winner_medals_after;
    let (mut shown, mut scale) = ([0; 4], [1.0; 4]);
    for i in 0..4 {
        let amount = amounts[i];
        if amount == 0 {
            continue;
        }
        let step = p.winner_count / amount as f32;
        // Up, one number a step (and a step more on the last), hold, back down.
        let counting = at + pop;
        let held = counting + step * (amount + 1) as f32 + p.winner_count_hold;
        if t >= counting {
            shown[i] = (((t - counting) / step) as u32).min(amount);
        }
        let k = |from: f32| ((t - from) / pop.max(0.001)).clamp(0.0, 1.0);
        scale[i] = if t < held {
            1.0 + (big - 1.0) * k(at)
        } else {
            big - (big - 1.0) * k(held)
        };
        at = held + pop;
    }
    (shown, scale, at + p.winner_trophy_after)
}

/// When the trophy is fully in, in seconds from the popup's opening: the drum roll stops and
/// the win music starts then.
pub fn winner_trophy_in(cup: &Cup, config: &GameConfig) -> f32 {
    let (_, _, at) = winner_counts(0.0, winner_amounts(cup), &config.cup_panel);
    at + config.cup_panel.winner_trophy_grow.1
}

/// The cup winner's medal counts: fourths, bronze, silver and gold (one per point earned,
/// so a win counts 3, as the original's label does).
fn winner_amounts(cup: &Cup) -> [u32; 4] {
    let Some(w) = cup.standings.as_ref().and_then(|s| s.first().copied()) else {
        return [0; 4];
    };
    let count = |place: usize| {
        cup.medals
            .get(w)
            .map_or(0, |m| m.iter().filter(|&&q| q == place).count() as u32)
    };
    [
        cup.fourths.get(w).copied().unwrap_or(0),
        count(2),
        count(1),
        count(0),
    ]
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn show_cup_winner(
    mut commands: Commands,
    config: Res<GameConfig>,
    cup: Res<Cup>,
    popup: Res<Popup>,
    art: Option<Res<Art>>,
    towers: Query<&Tower>,
    mut nodes: Query<(
        &Win,
        &mut Node,
        &mut Visibility,
        Option<&mut ImageNode>,
        Option<&mut BackgroundColor>,
        Option<&mut UiTransform>,
    )>,
    mut texts: Query<(&Win, &mut Text)>,
    mut sounded: Local<bool>,
) {
    let p = &config.cup_panel;
    let since = popup.winner_since();
    let t = since.unwrap_or(0.0);
    let (shown, scale, trophy_at) = winner_counts(t, winner_amounts(&cup), p);
    let (from, grow) = p.winner_trophy_grow;
    let grown = ((t - trophy_at) / grow.max(0.001)).clamp(0.0, 1.0);
    let trophy_scale = if t < trophy_at {
        0.0
    } else {
        from + (1.0 - from) * crate::packs::out_elastic(grown)
    };
    // The trophy's sound (TROPHY) as it starts growing.
    if since.is_some() && t >= trophy_at && !*sounded {
        *sounded = true;
        if let Some(handle) = art.as_ref().and_then(|a| a.sounds.get("sfx_trophy")) {
            commands.spawn((AudioPlayer(handle.clone()), PlaybackSettings::DESPAWN));
        }
    }
    if since.is_none() {
        *sounded = false;
    }
    let image = |name: &str| art.as_ref().and_then(|a| a.image(name));
    let size = cup.size.unwrap_or(0).min(WIN_TROPHIES.len() - 1);
    let (trophy_size, trophy_drop, rays_size) = WIN_TROPHIES[size];
    for (part, mut node, mut visibility, image_node, background, transform) in &mut nodes {
        match *part {
            Win::Root => {
                visibility.set_if_neq(shown_if(since.is_some()));
            }
            Win::Fade => {
                // Closing fades the screen to black (TOURNAMENT_FADE_OUT_OVERLAY).
                let k = ((t - p.winner_close) / p.winner_fade.max(0.001)).clamp(0.0, 1.0);
                if let Some(mut background) = background {
                    let want = Color::BLACK.with_alpha(k);
                    if background.0 != want {
                        background.0 = want;
                    }
                }
            }
            Win::Medal(i) => {
                if let Some(mut transform) = transform {
                    transform.scale = Vec2::splat(scale[i]);
                }
            }
            Win::Trophy => {
                if let Some(mut transform) = transform {
                    transform.scale = Vec2::splat(trophy_scale);
                }
            }
            Win::Rays(k) => {
                // Layer 1 turns 180 degrees in 2 s; layer 2 -90, at 1.2 for a second, then
                // easing to 1.1 by the loop's end.
                let lap = t % 2.0;
                let (turn, size_k) = if k == 0 {
                    (90.0 * lap, 1.0)
                } else {
                    (
                        -45.0 * lap,
                        if lap < 1.0 {
                            1.2
                        } else {
                            1.2 - 0.1 * (lap - 1.0)
                        },
                    )
                };
                set_box(
                    &mut node,
                    50.0 - rays_size / 2.0,
                    50.0 - rays_size / 2.0,
                    rays_size,
                    rays_size,
                );
                if let Some(mut transform) = transform {
                    // Unity turns anticlockwise for a positive z; Bevy's UI clockwise.
                    transform.rotation = Rot2::degrees(-turn);
                    transform.scale = Vec2::splat(size_k);
                }
            }
            Win::Cup => {
                set_box(
                    &mut node,
                    50.0 - trophy_size.0 / 2.0,
                    50.0 - trophy_size.1 / 2.0 + trophy_drop,
                    trophy_size.0,
                    trophy_size.1,
                );
                if let Some(mut picture) = image_node {
                    let name = cup.rules(&config.cups).map(|s| s.trophy.clone());
                    set_image(&mut picture, name.and_then(|n| image(&n)));
                }
            }
            Win::Deco(right) => {
                let deco = cup.size.and_then(|s| p.decos.get(s));
                let (name, dsize, d_at) = match deco {
                    Some(d) => (d.cup.as_str(), d.cup_size, d.cup_at),
                    None => ("", (0.0, 0.0), (0.0, 0.0)),
                };
                // DecoContainerRight/Left: 100 px boxes on the panel's top corners, 50 px
                // out; the cup sits in them as on the standings panel.
                let (px0, py0, pw, _) = WIN_PANEL;
                let corner = if right { px0 + pw } else { px0 };
                let x = if right {
                    corner + d_at.0
                } else {
                    corner - d_at.0
                };
                let y = py0 - d_at.1;
                let want = (
                    Val::Percent(50.0),
                    px(x - dsize.0 / 2.0 - SCREEN.0 / 2.0),
                    px(y - dsize.1 / 2.0),
                    px(dsize.0),
                    px(dsize.1),
                );
                if (
                    node.left,
                    node.margin.left,
                    node.top,
                    node.width,
                    node.height,
                ) != want
                {
                    (
                        node.left,
                        node.margin.left,
                        node.top,
                        node.width,
                        node.height,
                    ) = want;
                }
                if let Some(mut deco) = image_node {
                    set_image(&mut deco, image(name));
                    if deco.flip_x != !right {
                        deco.flip_x = !right;
                    }
                }
            }
            _ => {}
        }
    }
    let winner = cup.standings.as_ref().and_then(|s| s.first().copied());
    let name = winner
        .and_then(|w| towers.iter().find(|t| t.index == w))
        .map(|t| match t.player {
            Some(n) => format!("PLAYER {}", n + 1),
            None => format!("WIZARD {}", t.index + 1),
        })
        .unwrap_or_default();
    // POPUP_TOURNAMENT_WINNER_TITLE: "{cupType} WINNER!".
    let title = cup
        .rules(&config.cups)
        .map(|s| format!("{} WINNER!", s.name.to_uppercase()))
        .unwrap_or_default();
    for (part, mut text) in &mut texts {
        let words = match *part {
            Win::Title => title.clone(),
            Win::Name => name.clone(),
            Win::Count(i) => shown[i].to_string(),
            _ => continue,
        };
        if text.0 != words {
            text.0 = words;
        }
    }
}

fn shown_if(on: bool) -> Visibility {
    if on {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    }
}

/// The cup winner popup's camera (TournamentWizardCamera): orthographic size 10, so a unit
/// is 1080 / 20 pixels; its particles are placed in these units from the screen's middle.
const WIN_UNIT: f32 = 54.0;

/// One particle of the cup winner's confetti and fireworks, in units from the screen's
/// middle (y up), drawn over the popup.
#[derive(Component)]
struct Spark {
    at: Vec2,
    speed: Vec2,
    age: f32,
    life: f32,
    /// Size in units, and its curve over the life (time, share), held past the ends.
    size: f32,
    /// Its SizeModule curve (seconds as a share of its life, value, in and out slope).
    size_curve: &'static [(f32, f32, f32, f32)],
    /// Alpha over the life.
    alpha_curve: &'static [(f32, f32)],
    /// Colours spread evenly over the life (one holds still).
    colours: &'static [[u8; 3]],
    tint: Color,
    spin: f32,
    angle: f32,
    /// A sprite sheet of square frames side by side: how many, and how many times they run
    /// over the life.
    frames: Option<(u32, f32)>,
    /// ClampVelocityModule: the speed it eases down to, and the share of the excess taken
    /// off each frame (at 60 frames a second).
    limit: Option<(f32, f32)>,
}

/// A firework's rocket (Firework): from (+-6, -6) to its burst point over 1 s (DOMove's
/// default out-quad), trailing its colour; then the burst.
#[derive(Component)]
struct Rocket {
    from: Vec2,
    to: Vec2,
    age: f32,
    colour: &'static [[u8; 3]],
    trail: f32,
    whistle: Option<Entity>,
}

const WHITE: &[[u8; 3]] = &[[255, 255, 255]];
/// FireWorksRed / FireWorksBlue start colours.
const ROCKET_RED: &[[u8; 3]] = &[[251, 41, 80]];
const ROCKET_BLUE: &[[u8; 3]] = &[[99, 255, 242]];
/// The trail's size and alpha over its 0.8 s (SizeModule x 0.5, ColorModule).
const TRAIL_SIZE: &[(f32, f32, f32, f32)] =
    &[(0.0, 0.6313, 0.0, 0.0), (0.6195, 0.1162, 0.0927, 0.0927), (0.7752, 0.2677, 0.0, 0.0), (1.0, 0.0, 0.0, 0.0)];
const TRAIL_ALPHA: &[(f32, f32)] = &[(0.0, 1.0), (0.78, 1.0), (1.0, 0.0)];
/// The bursting stars (FireWorksEndParticle01/02): size, alpha and the colours they run
/// through (their ColorModule's keys, taken as evenly spaced).
const STAR_SIZE: &[(f32, f32, f32, f32)] =
    &[(0.0, 1.0, 0.0, 0.0), (0.6138, 0.6943, 0.0927, 0.0927), (0.7752, 1.0, 0.0, 0.0), (1.0, 0.0, 0.0, 0.0)];
const STAR_ALPHA: &[(f32, f32)] = &[(0.0, 1.0), (0.856, 1.0), (1.0, 0.0)];
const STAR_COLOURS: &[[u8; 3]] = &[
    [255, 38, 188],
    [255, 255, 255],
    [28, 255, 48],
    [255, 255, 255],
    [253, 255, 96],
];
/// The flash (Explosion, 0.07 s) and the ring (Shockwave: grows to 40 x its 0.1, fading by
/// a fifth of its second).
const FLASH_ALPHA: &[(f32, f32)] = &[(0.0, 1.0), (0.5, 0.55)];
const RING_SIZE: &[(f32, f32, f32, f32)] =
    &[(0.0, 0.0, 2.78, 2.78), (0.2503, 0.8517, 0.4494, 0.4494), (0.9971, 1.0, 0.035, 0.035)];
const RING_ALPHA: &[(f32, f32)] = &[(0.0, 1.0), (0.118, 0.03), (0.2, 0.0)];
const STEADY: &[(f32, f32)] = &[(0.0, 1.0)];
const STEADY_SIZE: &[(f32, f32, f32, f32)] = &[(0.0, 1.0, 0.0, 0.0)];
/// ConfettiParticle01/02: their two pairs of colour gradients (a random point between them),
/// speed and life; both 10 a second along a 24-unit edge, 0.25 units, spinning up to 18.85
/// radians a second either way, their 3 frames running 52 times over the life.
const CONFETTI: [([[u8; 3]; 2], [[u8; 3]; 2], f32, f32, &str); 2] = [
    (
        [[255, 58, 160], [255, 0, 58]],
        [[77, 85, 255], [174, 163, 255]],
        4.0,
        7.0,
        "confetti_1",
    ),
    (
        [[203, 58, 255], [255, 0, 58]],
        [[255, 77, 77], [255, 149, 68]],
        3.0,
        8.0,
        "confetti_2",
    ),
];

fn mix(a: [u8; 3], b: [u8; 3], k: f32) -> [f32; 3] {
    [0, 1, 2].map(|i| (a[i] as f32 + (b[i] as f32 - a[i] as f32) * k) / 255.0)
}

/// Runs the cup winner's confetti and fireworks once its wizard is due (trophy in, then
/// 0.92 s): confetti from the top edge (ponytail: the original's edge sits at y +5 and
/// emits along its normal; Max's clip shows it falling from the top, which is what this
/// does), a new firework every 0.3 to 0.6 s until the popup closes.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn celebrate(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    cup: Res<Cup>,
    popup: Res<Popup>,
    art: Option<Res<Art>>,
    roots: Query<(Entity, &Win)>,
    mut sparks: Query<(
        Entity,
        &mut Spark,
        &mut Node,
        &mut ImageNode,
        &mut UiTransform,
        &mut Visibility,
    )>,
    mut rockets: Query<(Entity, &mut Rocket)>,
    images: Option<Res<Assets<Image>>>,
    mut clock: Local<(f32, [f32; 2], u32)>,
) {
    let (next_rocket, confetti_due, seed) = &mut *clock;
    let Some(since) = popup.winner_since() else {
        for (entity, ..) in &sparks {
            commands.entity(entity).despawn();
        }
        for (entity, rocket) in &rockets {
            if let Some(whistle) = rocket.whistle {
                commands.entity(whistle).try_despawn();
            }
            commands.entity(entity).despawn();
        }
        *next_rocket = 0.0;
        *confetti_due = [0.0; 2];
        return;
    };
    let dt = time.delta_secs();
    if *seed == 0 {
        *seed = 0x9e37_79b9 ^ (time.elapsed_secs() * 1000.0) as u32 | 1;
    }
    let mut roll = || crate::particles::roll(seed);
    let Some(root) = roots.iter().find(|(_, w)| **w == Win::Root).map(|(e, _)| e) else {
        return;
    };
    let image = |name: &str| art.as_ref().and_then(|a| a.image(name));
    let start = winner_trophy_in(&cup, &config) + WIN_WIZARD_AFTER;
    let closing = since >= config.cup_panel.winner_close;
    let spawn = |commands: &mut Commands, picture: &str, spark: Spark| {
        let Some(picture) = image(picture) else {
            return;
        };
        let node = Node {
            position_type: PositionType::Absolute,
            ..default()
        };
        let child = commands
            .spawn((
                ImageNode::new(picture),
                node,
                UiTransform::default(),
                ZIndex(5),
                // Hidden until placed, or it shows a frame in the corner at its own size.
                Visibility::Hidden,
                spark,
            ))
            .id();
        commands.entity(root).add_child(child);
    };
    if since >= start && !closing {
        // Confetti: 10 a second from each emitter.
        for (k, &(warm, cool, speed, life, picture)) in CONFETTI.iter().enumerate() {
            if confetti_due[k] == 0.0 {
                confetti_due[k] = since;
            }
            while confetti_due[k] <= since {
                confetti_due[k] += 0.1;
                let t = roll();
                let a = mix(warm[0], warm[1], t);
                let b = mix(cool[0], cool[1], t);
                let k2 = roll();
                let colour = Color::srgb(
                    a[0] + (b[0] - a[0]) * k2,
                    a[1] + (b[1] - a[1]) * k2,
                    a[2] + (b[2] - a[2]) * k2,
                );
                spawn(
                    &mut commands,
                    picture,
                    Spark {
                        at: Vec2::new(-12.0 + 24.0 * roll(), 10.5),
                        speed: Vec2::new(0.0, -speed),
                        age: 0.0,
                        life,
                        size: 0.25,
                        size_curve: STEADY_SIZE,
                        alpha_curve: STEADY,
                        colours: WHITE,
                        tint: colour,
                        spin: 18.85 * (roll() * 2.0 - 1.0),
                        angle: roll() * std::f32::consts::TAU,
                        frames: Some((3, 52.0)),
                        limit: None,
                    },
                );
            }
        }
        // A new firework every 0.3 to 0.6 s (the first one at once), red or blue, from
        // either side.
        if *next_rocket == 0.0 {
            *next_rocket = since;
        }
        if since >= *next_rocket {
            *next_rocket = since + 0.3 + 0.3 * roll();
            let right = roll() < 0.5;
            let (x0, (lo, hi)) = if right {
                (6.0, (4.0, 8.0))
            } else {
                (-6.0, (-8.0, -4.0))
            };
            let colour = if roll() < 0.5 {
                ROCKET_RED
            } else {
                ROCKET_BLUE
            };
            // The whistle: one of the four falling-brick sounds, at 0.3, pitch 0.9 to 1.2.
            let whistle = art
                .as_ref()
                .and_then(|a| {
                    a.sounds.get(&format!(
                        "sfx_brick_fall_{}",
                        1 + (roll() * 4.0) as usize % 4
                    ))
                })
                .map(|h| {
                    commands
                        .spawn((
                            AudioPlayer(h.clone()),
                            PlaybackSettings::DESPAWN
                                .with_volume(bevy::audio::Volume::Linear(0.3))
                                .with_speed(0.9 + 0.3 * roll()),
                        ))
                        .id()
                });
            commands.spawn(Rocket {
                from: Vec2::new(x0, -6.0),
                to: Vec2::new(lo + (hi - lo) * roll(), -3.0 + 7.0 * roll()),
                age: 0.0,
                colour,
                trail: 0.0,
                whistle,
            });
        }
    }
    for (entity, mut rocket) in &mut rockets {
        rocket.age += dt;
        let k = rocket.age.min(1.0);
        let at = rocket.from + (rocket.to - rocket.from) * (1.0 - (1.0 - k) * (1.0 - k));
        // The trail: 100 a second, still where they are left.
        rocket.trail += dt * 100.0;
        while rocket.trail >= 1.0 && rocket.age < 1.0 {
            rocket.trail -= 1.0;
            spawn(
                &mut commands,
                "explosion_flash",
                Spark {
                    at,
                    speed: Vec2::ZERO,
                    age: 0.0,
                    life: 0.8,
                    size: 0.8,
                    size_curve: TRAIL_SIZE,
                    alpha_curve: TRAIL_ALPHA,
                    colours: rocket.colour,
                    tint: Color::WHITE,
                    spin: 0.0,
                    angle: 0.0,
                    frames: None,
                    limit: None,
                },
            );
        }
        if rocket.age < 1.0 {
            continue;
        }
        // The burst: stars flying out and slowing, a flash, a ring, and a bang.
        if let Some(whistle) = rocket.whistle {
            commands.entity(whistle).try_despawn();
        }
        if let Some(bang) = art.as_ref().and_then(|a| {
            a.sounds
                .get(&format!("sfx_firework_{}", 1 + (roll() * 3.0) as usize % 3))
        }) {
            commands.spawn((AudioPlayer(bang.clone()), PlaybackSettings::DESPAWN));
        }
        for (count, size, picture, limit, frames) in [
            (
                3 + (roll() * 3.0) as usize,
                0.4,
                "firework_blob",
                0.6,
                Some((2, 1.0)),
            ),
            (
                10 + (roll() * 6.0) as usize,
                0.5,
                "survival_star",
                0.5,
                None,
            ),
        ] {
            for _ in 0..count {
                let turn = roll() * std::f32::consts::TAU;
                spawn(
                    &mut commands,
                    picture,
                    Spark {
                        at: rocket.to,
                        speed: Vec2::from_angle(turn) * 60.0,
                        age: 0.0,
                        life: 0.8,
                        size,
                        size_curve: STAR_SIZE,
                        alpha_curve: STAR_ALPHA,
                        colours: STAR_COLOURS,
                        tint: Color::WHITE,
                        spin: roll() * 6.0 - 3.0,
                        angle: turn,
                        frames,
                        limit: Some((limit, 0.6)),
                    },
                );
            }
        }
        spawn(
            &mut commands,
            "firework_blob",
            Spark {
                at: rocket.to,
                speed: Vec2::ZERO,
                age: 0.0,
                life: 0.07,
                size: 3.0,
                size_curve: STEADY_SIZE,
                alpha_curve: FLASH_ALPHA,
                colours: WHITE,
                tint: Color::WHITE,
                spin: 0.0,
                angle: std::f32::consts::FRAC_PI_2,
                frames: Some((2, 1.0)),
                limit: None,
            },
        );
        spawn(
            &mut commands,
            "cast_circle_thin",
            Spark {
                at: rocket.to,
                speed: Vec2::ZERO,
                age: 0.0,
                life: 1.0,
                size: 4.0,
                size_curve: RING_SIZE,
                alpha_curve: RING_ALPHA,
                colours: WHITE,
                tint: Color::WHITE,
                spin: 0.0,
                angle: 0.0,
                frames: None,
                limit: None,
            },
        );
        commands.entity(entity).despawn();
    }
    for (entity, mut spark, mut node, mut picture, mut transform, mut visibility) in &mut sparks {
        visibility.set_if_neq(Visibility::Inherited);
        spark.age += dt;
        let t = spark.age / spark.life.max(0.001);
        if t >= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }
        if let Some((cap, dampen)) = spark.limit {
            let speed = spark.speed.length();
            if speed > cap {
                let keep = (1.0 - dampen).powf(dt * 60.0);
                spark.speed *= (cap + (speed - cap) * keep) / speed;
            }
        }
        let step = spark.speed * dt;
        spark.at += step;
        spark.angle += spark.spin * dt;
        let size = spark.size * crate::config::curve(spark.size_curve, t) * WIN_UNIT;
        let x = SCREEN.0 / 2.0 + spark.at.x * WIN_UNIT - size / 2.0;
        let y = SCREEN.1 / 2.0 - spark.at.y * WIN_UNIT - size / 2.0;
        // Centred on the screen like the panel.
        node.left = Val::Percent(50.0);
        node.margin.left = px(x - SCREEN.0 / 2.0);
        node.top = px(y);
        node.width = px(size);
        node.height = px(size);
        transform.rotation = Rot2::radians(-spark.angle);
        let n = spark.colours.len();
        let at = (t * (n - 1) as f32).min((n - 1) as f32 - 0.0001).max(0.0);
        let (i, k) = (at as usize, at.fract());
        let c = mix(spark.colours[i], spark.colours[(i + 1).min(n - 1)], k);
        let tint = spark.tint.to_srgba();
        picture.color = Color::srgba(
            c[0] * tint.red,
            c[1] * tint.green,
            c[2] * tint.blue,
            keyed(spark.alpha_curve, t),
        );
        // Square frames side by side: show this one's slice of the sheet.
        if let Some((count, cycles)) = spark.frames
            && let Some(size) = images.as_ref().and_then(|i| i.get(&picture.image)).map(|i| i.size_f32())
        {
            let frame = ((t * cycles).fract() * count as f32) as u32 % count.max(1);
            let w = size.x / count as f32;
            let want = Some(Rect::new(w * frame as f32, 0.0, w * (frame + 1) as f32, size.y));
            if picture.rect != want {
                picture.rect = want;
            }
        }
    }
}

/// The winner's wizard and the confetti come this long after the trophy is in.
const WIN_WIZARD_AFTER: f32 = 0.92;

/// The cup winner's wizard (ControllableWizard): it pops in at WizardSpawnPosition, 650 px
/// right of and 380 px under the screen's middle, and the winner flies it about: its speed
/// eases towards 10 units a second in the held direction (3 x the frame time each frame),
/// and it faces the way it goes. It pops away as the popup closes.
#[derive(Component)]
struct Flyer {
    at: Vec2,
    speed: Vec2,
    facing_left: bool,
    /// The winner's wizard (0 for wizard 1), for its voice lines.
    wizard: usize,
    /// A nudge under way: seconds since, and its way (+1 right, -1 left).
    nudge: Option<(f32, f32)>,
}

/// One of the 8 ghosts a nudge leaves behind the wizard (ControllableWizard._NudgeWizard):
/// ghost `i` sits (1 - i / 8) of 2.5 units back along the nudge, at (1 - i / 8) x 0.6 alpha,
/// fading out over 0.3 s (TrailEffect).
#[derive(Component)]
struct Ghost(usize);

/// The nudge: a 3-unit jump, the wizard squashed to half height growing back over 0.3 s
/// (out-expo), and its ghost trail.
const NUDGE_JUMP: f32 = 3.0;
const NUDGE_TRAIL: f32 = 2.5;
const NUDGE_TIME: f32 = 0.3;
const GHOSTS: usize = 8;

/// Medals on the standings: those from earlier matches at once, the latest match's as their
/// pop starts (`Cup::fill_times`).
fn shown_medals(cup: &crate::cups::Cup, fill: &[Vec<f32>], since: f32, t: usize) -> usize {
    let before = cup.before.get(t).copied().unwrap_or(0);
    let started = fill.get(t).map_or(0, |times| times.iter().filter(|&&at| since >= at).count());
    cup.shown(t).min(before + started)
}

/// Each medal landing on the standings plays its sound (AnimateMedalEffect): gold
/// LEVEL_UP, silver and bronze their own, the empty one one of six MEDAL_EARNED_EMPTY_
/// (staged as sfx_4th_place_01..06; the name match is ours). Only medals inside the target
/// land as pictures; past it they count up on "+N" with the same sound.
fn medal_sounds(
    mut commands: Commands,
    config: Res<GameConfig>,
    cup: Res<crate::cups::Cup>,
    popup: Res<Popup>,
    art: Option<Res<Art>>,
    mut heard: Local<Vec<usize>>,
) {
    let p = &config.cup_panel;
    if !popup.panel_up(&cup, p) {
        heard.clear();
        return;
    }
    // Each medal's sound comes as its pop peaks (MedalIn's IN_COMPLETE, one step in).
    let since = popup.since_shown() - p.fill_step;
    let (fill, _) = cup.fill_times(p.fill_start, p.fill_step, p.fill_row_gap);
    let towers = cup.medals.len();
    if heard.len() != towers {
        *heard = (0..towers).map(|t| cup.before.get(t).copied().unwrap_or(0)).collect();
    }
    for t in 0..towers {
        let now = shown_medals(&cup, &fill, since, t);
        while heard[t] < now {
            let name = match cup.medal(t, heard[t]) {
                Some(0) => "sfx_medal_earned_gold".to_string(),
                Some(1) => "sfx_medal_earned_silver".to_string(),
                Some(2) => "sfx_medal_earned_bronze".to_string(),
                _ => format!("sfx_4th_place_0{}", (since * 1000.0) as u32 % 6 + 1),
            };
            heard[t] += 1;
            if let Some(sound) = art.as_ref().and_then(|a| a.sounds.get(&name)) {
                commands.spawn((AudioPlayer(sound.clone()), PlaybackSettings::DESPAWN));
            }
        }
    }
}

/// One of the wizard's voice lines of `kind` at random (its WizardSpeech table's volume and
/// pitch), for the winner's tricks.
fn say(commands: &mut Commands, art: &Art, wizard: usize, kind: &str, roll: f32) {
    let key = format!("wizard{}", wizard + 1);
    let count = (1..)
        .take_while(|i| art.sounds.contains_key(&format!("{key}_say_{kind}_{i}")))
        .count();
    if count == 0 {
        return;
    }
    let i = 1 + (roll * count as f32) as usize % count;
    if let Some(sound) = art.sounds.get(&format!("{key}_say_{kind}_{i}")) {
        let (volume, pitch) = crate::wizards::voice_mix(wizard as u32 + 1, kind);
        commands.spawn((
            AudioPlayer(sound.clone()),
            PlaybackSettings::DESPAWN
                .with_volume(bevy::audio::Volume::Linear(volume))
                .with_speed(pitch),
        ));
    }
}

/// The winner's wizard is the in-match wizard rig, drawn by its own camera over the popup
/// (the original's ControllableWizard is the world wizard in front of the popup), on this
/// render layer, `WIN_UNIT` pixels a unit like the popup's sparks.
const FLYER_LAYER: usize = 7;
const FLYER_SPAWN: Vec2 = Vec2::new(650.0 / WIN_UNIT, -380.0 / WIN_UNIT);
/// Where the wand is on the rig, in units from its middle facing left (the lightning starts
/// there; SpellCastContainer's spot isn't read, a guess from the cast pose).
const FLYER_WAND: Vec2 = Vec2::new(-0.9, 1.0);

/// The camera drawing the winner's wizard over the popup.
#[derive(Component)]
struct FlyerCamera;

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn fly_winner(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    cup: Res<Cup>,
    popup: Res<Popup>,
    art: Option<Res<Art>>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    pads: Query<&Gamepad>,
    towers: Query<(&Tower, Option<&TowerSkin>)>,
    mut flyers: Query<
        (
            Entity,
            &mut Flyer,
            &mut Transform,
            &mut crate::wizards::Wizard,
        ),
        Without<Ghost>,
    >,
    mut ghosts: Query<(Entity, &Ghost, &mut Transform, &Children), Without<Flyer>>,
    mut sprites: Query<&mut Sprite>,
    cameras: Query<Entity, With<FlyerCamera>>,
    main: Query<
        (&Camera, &bevy::camera::RenderTarget, Option<&Msaa>),
        With<crate::visuals::MainCamera>,
    >,
    bindings: Option<Res<crate::keybinding::KeyBindings>>,
    mut seed: Local<u32>,
) {
    let since = popup.winner_since();
    let due = winner_trophy_in(&cup, &config) + WIN_WIZARD_AFTER;
    let up = since.is_some_and(|t| t >= due && t < config.cup_panel.winner_close);
    let pop = |commands: &mut Commands| {
        if let Some(sound) = art.as_ref().and_then(|a| a.sounds.get("sfx_wizard_lose")) {
            commands.spawn((AudioPlayer(sound.clone()), PlaybackSettings::DESPAWN));
        }
    };
    let layer = bevy::camera::visibility::RenderLayers::layer(FLYER_LAYER);
    if !up {
        for (entity, ..) in &flyers {
            // Closing pops it away; leaving the screen otherwise just clears it.
            if since.is_some() {
                pop(&mut commands);
            }
            commands.entity(entity).despawn();
        }
        for (entity, ..) in &ghosts {
            commands.entity(entity).despawn();
        }
        for camera in &cameras {
            commands.entity(camera).despawn();
        }
        return;
    }
    let Some(art) = art.as_deref() else { return };
    if flyers.is_empty() {
        let winner = cup.standings.as_ref().and_then(|s| s.first().copied());
        let wizard = towers
            .iter()
            .find(|(t, _)| Some(t.index) == winner)
            .map_or(0, |(t, skin)| skin.map_or(t.index, |s| s.wizard));
        let Some(rig) =
            crate::wizards::spawn_free_wizard(&mut commands, art, wizard, WIN_UNIT, layer.clone())
                .or_else(|| {
                    crate::wizards::spawn_free_wizard(
                        &mut commands,
                        art,
                        0,
                        WIN_UNIT,
                        layer.clone(),
                    )
                })
        else {
            return;
        };
        pop(&mut commands);
        // It cheers as it comes in (ControllableWizard.Init).
        commands.entity(rig).queue(|mut e: EntityWorldMut| {
            if let Some(mut w) = e.get_mut::<crate::wizards::Wizard>() {
                w.act(crate::wizards::Clip::Cheer);
            }
        });
        commands.entity(rig).insert(Flyer {
            at: FLYER_SPAWN,
            speed: Vec2::ZERO,
            facing_left: true,
            wizard,
            nudge: None,
        });
        if cameras.is_empty() {
            // Over everything the main camera drew (its UI too), into the same picture.
            let (order, target, msaa) = main
                .single()
                .map(|(c, t, m)| (c.order + 1, t.clone(), m.copied()))
                .unwrap_or((1, bevy::camera::RenderTarget::default(), None));
            commands.spawn((
                Camera2d,
                Camera {
                    order,
                    clear_color: ClearColorConfig::None,
                    ..default()
                },
                target,
                msaa.unwrap_or_default(),
                Projection::Orthographic(OrthographicProjection {
                    scaling_mode: bevy::camera::ScalingMode::FixedVertical {
                        viewport_height: SCREEN.1,
                    },
                    ..OrthographicProjection::default_2d()
                }),
                Transform::from_xyz(0.0, 0.0, 999.0),
                layer.clone(),
                FlyerCamera,
            ));
        }
        return;
    }
    // The held direction: the arrows, or any pad's d-pad or left stick.
    let mut dir = Vec2::ZERO;
    if let Some(keys) = &keys {
        dir.x += keys.pressed(KeyCode::ArrowRight) as i32 as f32
            - keys.pressed(KeyCode::ArrowLeft) as i32 as f32;
        dir.y += keys.pressed(KeyCode::ArrowUp) as i32 as f32
            - keys.pressed(KeyCode::ArrowDown) as i32 as f32;
    }
    for pad in &pads {
        dir.x += pad.pressed(GamepadButton::DPadRight) as i32 as f32
            - pad.pressed(GamepadButton::DPadLeft) as i32 as f32;
        dir.y += pad.pressed(GamepadButton::DPadUp) as i32 as f32
            - pad.pressed(GamepadButton::DPadDown) as i32 as f32;
        dir += pad.left_stick();
    }
    let dir = dir.clamp_length_max(1.0);
    let dt = time.delta_secs();
    // The tricks, on the player's own keys and pad buttons (ControllableWizard._DoAction):
    // light magic casts on itself (with its lightning), dark magic laughs, game info laughs
    // evilly, turning cheers, the nudges nudge.
    use crate::keybinding::Action;
    use crate::wizards::Clip;
    let key = |action: Action| {
        let code = bindings
            .as_ref()
            .map_or(action.default_key(), |b| b.key(action));
        keys.as_ref().is_some_and(|k| k.just_pressed(code))
    };
    let pad = |button: GamepadButton| pads.iter().any(|p| p.just_pressed(button));
    let light = key(Action::LightMagic) || pad(GamepadButton::West);
    let dark = key(Action::DarkMagic) || pad(GamepadButton::East);
    let overview = key(Action::GameInfo) || pad(GamepadButton::North);
    let cheer =
        key(Action::TurnRight) || pad(GamepadButton::South) || pad(GamepadButton::RightTrigger2);
    let nudge = if key(Action::NudgeRight) || pad(GamepadButton::RightTrigger) {
        Some(1.0)
    } else if key(Action::NudgeLeft) || pad(GamepadButton::LeftTrigger) {
        Some(-1.0)
    } else {
        None
    };
    if *seed == 0 {
        *seed = 0x2545_f491;
    }
    let mut roll = || crate::particles::roll(&mut seed);
    for (_, mut flyer, mut transform, mut rig) in &mut flyers {
        for (on, kind, clip) in [
            (light, "cast_self", Clip::CastSelf),
            (dark, "laugh", Clip::LaughHard),
            (overview, "laugh_evil", Clip::LaughEvil),
            (cheer, "cheer", Clip::Cheer),
        ] {
            if on {
                rig.act(clip);
                say(&mut commands, art, flyer.wizard, kind, roll());
            }
        }
        if light {
            // _HandleSpellCastSelf: a bolt from the wand to 18 units out the way it faces, at
            // a random height within 15, and one of the two light cast sounds.
            let face = if flyer.facing_left { 1.0 } else { -1.0 };
            let wand = flyer.at + Vec2::new(FLYER_WAND.x * face, FLYER_WAND.y);
            let to = Vec2::new(-18.0 * face, roll() * 30.0 - 15.0);
            crate::casts::spawn_bolt(
                &mut commands,
                wand * WIN_UNIT,
                to * WIN_UNIT,
                WIN_UNIT,
                Some(layer.clone()),
            );
            let name = if roll() < 0.5 {
                "sfx_cast_light_magic_1"
            } else {
                "sfx_cast_light_magic_2"
            };
            if let Some(sound) = art
                .sounds
                .get(name)
                .or_else(|| art.sounds.get("sfx_cast_light"))
            {
                commands.spawn((AudioPlayer(sound.clone()), PlaybackSettings::DESPAWN));
            }
        }
        if let Some(way) = nudge
            && flyer.nudge.is_none()
        {
            flyer.at.x += way * NUDGE_JUMP;
            flyer.nudge = Some((0.0, way));
            if let Some(sound) = art.sounds.get("sfx_nudge") {
                commands.spawn((AudioPlayer(sound.clone()), PlaybackSettings::DESPAWN));
            }
            for i in 0..GHOSTS {
                if let Some(ghost) = crate::wizards::spawn_free_wizard(
                    &mut commands,
                    art,
                    flyer.wizard,
                    WIN_UNIT,
                    layer.clone(),
                ) {
                    commands
                        .entity(ghost)
                        .insert((Ghost(i), Visibility::Hidden));
                }
            }
        }
        let target = dir * 10.0;
        let k = (dt * 3.0).min(1.0);
        flyer.speed = flyer.speed + (target - flyer.speed) * k;
        let speed = flyer.speed;
        flyer.at += speed * dt;
        if flyer.speed.x > 0.5 {
            flyer.facing_left = false;
        } else if flyer.speed.x < -0.5 {
            flyer.facing_left = true;
        }
        // A nudge squashes it to half height, growing back out-expo; its ghosts trail it.
        let mut squash = 1.0;
        if let Some((age, way)) = flyer.nudge {
            let age = age + dt;
            let k = (age / NUDGE_TIME).min(1.0);
            squash = 0.5 + 0.5 * (1.0 - 2f32.powf(-10.0 * k));
            flyer.nudge = (age < NUDGE_TIME).then_some((age, way));
            for (entity, ghost, mut ghost_at, children) in &mut ghosts {
                if age >= NUDGE_TIME {
                    commands.entity(entity).despawn();
                    continue;
                }
                let share = 1.0 - ghost.0 as f32 / GHOSTS as f32;
                let at = flyer.at - Vec2::new(way * NUDGE_TRAIL * share, 0.0);
                *ghost_at = rig_transform(at, flyer.facing_left, squash, 10.0 - share);
                for child in children.iter() {
                    if let Ok(mut sprite) = sprites.get_mut(child) {
                        sprite.color = Color::WHITE.with_alpha(share * 0.6 * (1.0 - k));
                    }
                }
                commands.entity(entity).insert(Visibility::Inherited);
            }
        }
        *transform = rig_transform(flyer.at, flyer.facing_left, squash, 20.0);
    }
}

/// The rig's place (units from the screen's middle, in pixels), facing (it looks left at
/// scale 1, as ControllableWizard's) and squash.
fn rig_transform(at: Vec2, facing_left: bool, squash: f32, z: f32) -> Transform {
    Transform::from_translation((at * WIN_UNIT).extend(z)).with_scale(Vec3::new(
        if facing_left { 1.0 } else { -1.0 },
        squash,
        1.0,
    ))
}
#[cfg(test)]
mod tests {
    use super::winner_counts;

    #[test]
    fn the_winner_counts_run_in_turn_and_skip_empty_ones() {
        let p = crate::config::CupPanel::default();
        // Two wins (6 gold points) only: fourth, bronze and silver are skipped.
        let start = p.winner_in + p.winner_medals_after;
        let (shown, scale, _) = winner_counts(start + 0.01, [0, 0, 0, 6], &p);
        assert_eq!(shown, [0, 0, 0, 0]);
        assert!(scale[3] > 1.0 && scale[0] == 1.0, "gold pops first");
        let (shown, _, trophy) = winner_counts(start + 0.33, [0, 0, 0, 6], &p);
        assert_eq!(shown[3], 5, "a number every 0.05 s");
        let (shown, scale, _) = winner_counts(trophy, [0, 0, 0, 6], &p);
        assert_eq!((shown[3], scale[3]), (6, 1.0), "done before the trophy");
        let (_, _, later) = winner_counts(0.0, [1, 1, 2, 3], &p);
        assert!(later > trophy, "more counts take longer");
    }
}
