//! What the player sees of local cups (picked in Select Mode's third column,
//! src/select.rs): the standings panel between cup matches (laid out from the
//! original's Tiny Cup screen, `GameConfig::cup_panel`) and the cup winner's trophy
//! popping in at the end.

use bevy::prelude::*;

use crate::art::Art;
use crate::config::GameConfig;
use crate::cups::Cup;
use crate::gameplay::{MAX_TOWERS, Match, Phase, Tower, TowerSkin};
use crate::letters::TextLook;
use crate::popup::{Popup, px};
use crate::select::{SCREEN, label, label_with, shown};

pub struct CupsMenuPlugin;

impl Plugin for CupsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_cup_ui, spawn_cup_panel))
            .add_systems(Update, (show_cup_ui, show_cup_panel).chain());
    }
}

#[derive(Component)]
struct CupTrophy;
#[derive(Component)]
struct CupTrophyText;

fn spawn_cup_ui(mut commands: Commands, art: Option<Res<Art>>) {
    // The trophy for the cup winner, centred under WINNER!.
    let trophy_image = art.as_ref().and_then(|a| a.image("trophy"));
    let mut trophy = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(45.0),
            width: Val::Vh(20.0),
            height: Val::Vh(20.0),
            margin: UiRect::left(Val::Vh(-10.0)),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexEnd,
            align_items: AlignItems::Center,
            ..default()
        },
        UiTransform::default(),
        GlobalZIndex(20),
        Visibility::Hidden,
        CupTrophy,
    ));
    match trophy_image {
        Some(image) => trophy.insert(ImageNode::new(image)),
        None => trophy.insert(BackgroundColor(Color::srgb(0.95, 0.75, 0.2))),
    };
    trophy.with_children(|t| {
        t.spawn((
            label(2.4, ""),
            CupTrophyText,
            Node {
                margin: UiRect::bottom(Val::Vh(-4.0)),
                ..default()
            },
        ));
    });
}

fn player_name(towers: &Query<&Tower>, index: usize) -> String {
    match towers
        .iter()
        .find(|t| t.index == index)
        .and_then(|t| t.player)
    {
        Some(p) => format!("Player {}", p + 1),
        None => format!("Wizard {}", index + 1),
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn show_cup_ui(
    time: Res<Time>,
    game: Res<Match>,
    config: Res<GameConfig>,
    cup: Res<Cup>,
    popup: Res<Popup>,
    towers: Query<&Tower>,
    mut trophies: Query<(&mut Visibility, &mut UiTransform, Option<&mut ImageNode>), With<CupTrophy>>,
    mut texts: Query<&mut Text>,
    trophy_text: Query<Entity, With<CupTrophyText>>,
    art: Option<Res<Art>>,
) {
    let set = |texts: &mut Query<&mut Text>, entity: Entity, words: String| {
        if let Ok(mut text) = texts.get_mut(entity)
            && text.0 != words
        {
            text.0 = words;
        }
    };
    let cup_rules = cup.rules(&config.cups);
    // The trophy once the cup is over.
    let winner = cup.standings.as_ref().and_then(|s| s.first().copied());
    let show_trophy = game.phase == Phase::Over
        && winner.is_some()
        && popup.shown
        && !popup.panel_up(&cup, &config.cup_panel);
    // Each cup size has its own trophy picture.
    let picture = cup_rules.and_then(|size| art.as_ref().and_then(|a| a.image(&size.trophy)));
    for (mut visibility, mut transform, image) in &mut trophies {
        if let (Some(mut node), Some(picture)) = (image, picture.clone())
            && node.image != picture
        {
            node.image = picture;
        }
        if !show_trophy {
            visibility.set_if_neq(Visibility::Hidden);
            transform.scale = Vec2::splat(config.moments.trophy_pop.0);
            continue;
        }
        visibility.set_if_neq(Visibility::Inherited);
        let (from, over) = config.moments.trophy_pop;
        let next = (transform.scale.x + (1.0 - from) * time.delta_secs() / over.max(0.01)).min(1.0);
        transform.scale = Vec2::splat(next);
    }
    if let (Some(winner), Some(size)) = (winner, cup_rules) {
        let words = format!("{} wins the {}!", player_name(&towers, winner), size.name);
        for entity in &trophy_text {
            set(&mut texts, entity, words.clone());
        }
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
                    TextLook {
                        face: Some(Color::srgb(1.0, 0.84, 0.2)),
                        ..TextLook::OUTLINED
                    },
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
) {
    let p = &config.cup_panel;
    let up = popup.panel_up(&cup, p);
    // Behind the standings the world turns grey (the popup's Black & White background).
    if let Some(mut grey) = grey {
        let want = if up { 1.0 } else { 0.0 };
        if grey.0 != want {
            grey.0 = want;
        }
    }
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
