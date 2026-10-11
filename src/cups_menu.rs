//! What the player sees of local cups (picked in Select Mode's third column,
//! src/select.rs): the standings panel between cup matches (laid out from the
//! original's Tiny Cup screen, `GameConfig::cup_panel`) and the cup winner popup after a
//! decided cup (TournamentWinnerPopup).

use bevy::prelude::*;

use crate::art::Art;
use crate::config::GameConfig;
use crate::cups::Cup;
use crate::gameplay::{MAX_TOWERS, Tower, TowerSkin};
use crate::letters::TextLook;
use crate::popup::{Popup, px};
use crate::select::{SCREEN, label_with, shown};

pub struct CupsMenuPlugin;

impl Plugin for CupsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_cup_panel, spawn_cup_winner))
            .add_systems(Update, (show_cup_panel, show_cup_winner).chain());
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
                                children![(label_with(p.name_size, "", name_look), Part::Name(i))],
                            ));
                            row.spawn((
                                at(
                                    p.portrait_left - p.row_left,
                                    (p.row_height - p.portrait_size) / 2.0,
                                    p.portrait_size,
                                    p.portrait_size,
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
    let order: Vec<usize> = cup.standings.clone().unwrap_or_else(|| {
        let mut order: Vec<usize> = (0..cup.points.len()).collect();
        order.sort_by(|&a, &b| cup.points[b].cmp(&cup.points[a]).then(a.cmp(&b)));
        order
    });
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
    let slide = if p.slide > 0.0 {
        let t = (since / p.slide).clamp(0.0, 1.0);
        1.0 - (1.0 - t) * (1.0 - t)
    } else {
        1.0
    };
    let drop = (1.0 - slide) * (SCREEN.1 - p.bar_top);
    let winner = cup.standings.as_ref().and_then(|s| s.first().copied());
    let filled = size.is_some() && since >= p.continue_at(cup.fresh());
    // Medals from earlier matches are there at once; the latest match's fill in.
    let shown_medals = |t: usize| {
        let (all, before) = (
            cup.medals.get(t).map_or(0, Vec::len),
            cup.before.get(t).copied().unwrap_or(0),
        );
        let fresh = ((since - p.fill_start) / p.fill_step.max(0.01)).floor().max(-1.0) + 1.0;
        all.min(before + fresh.max(0.0) as usize)
    };

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
                let height = order.len() as f32 * 98.0 + 180.0;
                let want = (
                    Val::Percent(50.0),
                    px(-width / 2.0),
                    px(SCREEN.1 / 2.0 - 102.0 - height / 2.0 + drop),
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
                    let number = tower(i).map(|(t, skin)| skin.map_or(t.index, |s| s.wizard) + 1);
                    set_image(
                        &mut face,
                        number.and_then(|n| image(&format!("wizard{n}_face"))),
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
                    .and_then(|&t| cup.medals.get(t))
                    .and_then(|m| m.get(k).copied())
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
                if let Some(mut trophy) = image_node {
                    set_image(&mut trophy, size.and_then(|s| image(&s.trophy)));
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
                Some(n) => format!("PLAYER {}", n + 1),
                None => format!("WIZARD {}", t.index + 1),
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
