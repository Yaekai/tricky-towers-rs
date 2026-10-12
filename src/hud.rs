//! The in-match HUD as the original draws it: nothing along the bottom of the
//! screen, everything in each player's lane. The next-brick card hangs from the
//! top edge (with the Survival brick count beside it), the place label (1ST, 2ND)
//! or the hearts sit under the wizard, and the held spells float as bubbles beside
//! it. Once a match is won, the winner gets its roof and the losers turn pale and
//! their wizards vanish (docs/research/video-notes.md, `GameConfig::hud`).

use std::collections::HashSet;

use bevy::prelude::*;

use crate::art::Art;
use crate::config::{GameConfig, HudLook, ModeKind};
use crate::gameplay::{
    Brick, BrickBag, Match, Phase, RoofPiece, RoofStage, Tower, TowerState, TowerStatus, placings,
};
use crate::visuals::{FramedCamera, ViewSize};
use crate::wizards::Wizard;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_scores).add_systems(Update, show_scores);
        app.add_systems(
            Update,
            (
                spawn_items,
                fill_items,
                place_items,
                drop_roofs,
                fade_losers,
            )
                .chain()
                .after(FramedCamera),
        );
    }
}

fn rgb([r, g, b]: [u8; 3]) -> Color {
    Color::srgb_u8(r, g, b)
}

/// What a HUD item shows for its tower.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum Kind {
    Card,
    Count,
    Place,
    Hearts,
}

// The held spells show in the spell clouds beside each wizard (casts.rs), as in
// the original; there are no separate HUD bubbles.
const KINDS: [Kind; 4] = [Kind::Card, Kind::Count, Kind::Place, Kind::Hearts];

/// What an item showed last, so it is only rebuilt when that changes.
#[derive(Clone, PartialEq, Debug)]
enum Shows {
    Nothing,
    /// The next brick, drawn with the tower's brick pack.
    Card(usize, crate::config::BrickPack),
    Text(String, Color),
    /// Lives left and the most there have been: filled hearts, then crosses.
    Hearts(u32, u32),
}

#[derive(Component)]
struct HudItem {
    tower: Entity,
    kind: Kind,
    shows: Option<Shows>,
    /// The view height the item was built at; sizes follow it.
    built_at: f32,
}

/// A brick sprite already turned pale for a lost match.
#[derive(Component)]
struct Faded;

/// True once a match is decided and this tower lost it.
pub fn lost(game: &Match, tower: &Tower, state: &TowerState) -> bool {
    game.phase == Phase::Over
        && game.winner.is_some_and(|w| w != tower.index)
        && state.status != TowerStatus::Won
}

fn spawn_items(
    mut commands: Commands,
    game: Res<Match>,
    towers: Query<Entity, With<Tower>>,
    items: Query<(Entity, &HudItem)>,
) {
    let mut have = HashSet::new();
    for (entity, item) in &items {
        if game.phase == Phase::Menu || !towers.contains(item.tower) {
            commands.entity(entity).despawn();
        } else {
            have.insert((item.tower, item.kind));
        }
    }
    if game.phase == Phase::Menu {
        return;
    }
    for tower in &towers {
        for kind in KINDS {
            if have.contains(&(tower, kind)) {
                continue;
            }
            commands.spawn((
                HudItem {
                    tower,
                    kind,
                    shows: None,
                    built_at: 0.0,
                },
                Node {
                    position_type: PositionType::Absolute,
                    ..default()
                },
                // The Survival count sits over the card's corner.
                GlobalZIndex(if kind == Kind::Count { 6 } else { 5 }),
                Visibility::Hidden,
            ));
        }
    }
}

/// What each item should show now.
fn wanted(
    kind: Kind,
    config: &GameConfig,
    game: &Match,
    place: Option<usize>,
    state: &TowerState,
    bag: Option<&mut BrickBag>,
    pack: crate::config::BrickPack,
    count_up: Option<u32>,
    tower: usize,
) -> Shows {
    let look = &config.hud;
    let playing = game.phase == Phase::Playing && state.status == TowerStatus::Building;
    match kind {
        Kind::Card => {
            let finished_puzzle =
                game.mode == ModeKind::Puzzle && state.bricks_used() + 1 >= config.puzzle.bricks;
            let intro = game.phase == Phase::Intro && game.mode == ModeKind::Survival;
            // While Survival's count counts up the card flips through random bricks
            // from its own bag, once per two counted (AnimateBrickCountEffect), then
            // lands on the real next brick.
            let flip = count_up.filter(|_| intro).map(|counted| {
                let mut flips = BrickBag::new(config.seed ^ (tower as u64 + 1) * 0x9E37, config.pieces.len());
                (0..=counted / 2).map(|_| flips.draw()).last().unwrap_or(0)
            });
            match bag {
                Some(bag) if (playing || intro) && !finished_puzzle => {
                    Shows::Card(flip.unwrap_or_else(|| bag.peek()), pack)
                }
                _ => Shows::Nothing,
            }
        }
        Kind::Count if game.mode == ModeKind::Survival && game.phase != Phase::Menu => {
            let left = count_up.unwrap_or(
                config
                    .survival
                    .bricks_to_place
                    .saturating_sub(state.bricks_used()),
            );
            let over = game.phase == Phase::Over;
            // An out tower's HUD hides with its next card (GameFinishController).
            if state.status == TowerStatus::Out
                || over && state.status != TowerStatus::Won && game.winner.is_some()
            {
                Shows::Nothing
            } else {
                Shows::Text(left.to_string(), rgb(look.place_first))
            }
        }
        // Places only in battles: trials show none.
        Kind::Place if game.mode == ModeKind::Race && config.trial.is_none() && playing => {
            match place {
                Some(p) => {
                    let text = match p {
                        0 => "1ST".to_string(),
                        1 => "2ND".to_string(),
                        2 => "3RD".to_string(),
                        n => format!("{}TH", n + 1),
                    };
                    let color = if p == 0 {
                        look.place_first
                    } else {
                        look.place_other
                    };
                    Shows::Text(text, rgb(color))
                }
                None => Shows::Nothing,
            }
        }
        Kind::Hearts if state.status != TowerStatus::Out && game.phase != Phase::Over => {
            match state.lives {
                Some(lives) if lives > 0 => Shows::Hearts(lives, lives.max(config.survival.lives)),
                _ => Shows::Nothing,
            }
        }
        _ => Shows::Nothing,
    }
}

/// Rebuilds an item's contents when what it shows changes.
#[allow(clippy::too_many_arguments)]
fn fill_items(
    mut commands: Commands,
    config: Res<GameConfig>,
    game: Res<Match>,
    art: Res<Art>,
    view: Res<ViewSize>,
    mut towers: Query<(
        &Tower,
        &TowerState,
        Option<&mut BrickBag>,
        Option<&crate::gameplay::TowerSkin>,
    )>,
    mut items: Query<(Entity, &mut HudItem, &mut Visibility)>,
    count_up: Option<Res<crate::intro::IntroCount>>,
) {
    let order: Vec<(usize, TowerState)> = towers
        .iter()
        .filter(|(_, s, _, _)| s.status == TowerStatus::Building)
        .map(|(t, s, _, _)| (t.index, s.clone()))
        .collect();
    let refs: Vec<(usize, &TowerState)> = order.iter().map(|(i, s)| (*i, s)).collect();
    let places = placings(&game, &refs);
    let height = view.0.y.max(1.0);
    for (entity, mut item, mut visibility) in &mut items {
        let Ok((tower, state, bag, skin)) = towers.get_mut(item.tower) else {
            continue;
        };
        let place = places.iter().position(|&i| i == tower.index);
        let shows = wanted(
            item.kind,
            &config,
            &game,
            place,
            state,
            bag.map(|b| b.into_inner()),
            skin.map(|s| s.pack).unwrap_or_default(),
            count_up.as_ref().and_then(|c| c.0),
            tower.index,
        );
        if item.shows.as_ref() == Some(&shows) && item.built_at == height && !art.is_changed() {
            continue;
        }
        let mut node = commands.entity(entity);
        node.despawn_children();
        // Only the parts that change are rebuilt; the item keeps its place.
        node.remove::<(
            Text,
            TextFont,
            TextColor,
            TextShadow,
            ImageNode,
            BackgroundColor,
        )>();
        let layout = build(&mut node, &config.hud, &config, &art, &shows, height);
        node.insert(layout);
        visibility.set_if_neq(if shows == Shows::Nothing {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
        item.shows = Some(shows);
        item.built_at = height;
    }
}

/// Fills an item's node; returns its new size and layout.
fn build(
    node: &mut EntityCommands,
    look: &HudLook,
    config: &GameConfig,
    art: &Art,
    shows: &Shows,
    height: f32,
) -> Node {
    let base = Node {
        position_type: PositionType::Absolute,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    };
    let text = |node: &mut EntityCommands, text: &str, color: Color, size: f32| {
        node.insert((
            Text::new(text),
            TextFont {
                font_size: FontSize::Px(size),
                ..default()
            },
            TextColor(color),
            TextShadow {
                offset: Vec2::splat(size * 0.06),
                color: Color::srgb_u8(40, 20, 60),
            },
            // The original's display letters with their dark outline.
            crate::letters::TextLook::OUTLINED,
        ));
    };
    match shows {
        Shows::Nothing => base,
        Shows::Text(words, color) => {
            let count = words.parse::<u32>().is_ok();
            let size = if count { look.count_size } else { look.place_size } * height;
            text(node, words, *color, size);
            // The Survival count is BricksView's EnzoBlackGold60: the gold face, dilate and
            // outline 0.2.
            if count {
                node.insert(crate::letters::TextLook {
                    outline_edge: 0.5 - 0.4 * 0.8333 / 2.0,
                    ..crate::letters::TextLook::GOLD
                })
                .remove::<TextShadow>();
            }
            base
        }
        Shows::Hearts(lives, most) => {
            let size = look.heart_size * height;
            match art.image("heart") {
                Some(image) => {
                    // A lost life shows a cross (`heart_lost`, gph_cross), else a dim heart.
                    let lost = art.image("heart_lost");
                    node.with_children(|row| {
                        for i in 0..*most {
                            let (picture, color) = if i < *lives {
                                (image.clone(), Color::WHITE)
                            } else {
                                match &lost {
                                    Some(cross) => (cross.clone(), Color::WHITE),
                                    None => (image.clone(), Color::srgba(0.2, 0.2, 0.25, 0.6)),
                                }
                            };
                            row.spawn((
                                ImageNode::new(picture).with_color(color),
                                Node {
                                    width: Val::Px(size),
                                    height: Val::Px(size),
                                    ..default()
                                },
                            ));
                        }
                    });
                }
                None => text(
                    node,
                    "♥".repeat(*lives as usize).as_str(),
                    Color::srgb(1.0, 0.35, 0.45),
                    size,
                ),
            }
            Node {
                // Hearts 1.3 cells apart, each 1.12 wide (the original's health view).
                column_gap: Val::Px(size * 0.16),
                ..base
            }
        }
        Shows::Card(piece, pack) => {
            let card_h = look.card_height * height;
            let card_w = card_h * look.card_aspect;
            let card_art = art.image("next_card");
            let bar = if card_art.is_some() { 0.0 } else { card_h * look.card_bar };
            match card_art.clone() {
                Some(image) => {
                    node.insert(ImageNode::new(image));
                }
                None => {
                    node.insert(BackgroundColor(rgb(look.card_color)));
                    node.with_child((
                        Node {
                            position_type: PositionType::Absolute,
                            top: Val::Px(0.0),
                            left: Val::Px(0.0),
                            width: Val::Percent(100.0),
                            height: Val::Px(bar),
                            ..default()
                        },
                        BackgroundColor(rgb(look.card_bar_color)),
                    ));
                }
            }
            // The brick, fitted inside the white part.
            let def = &config.pieces[*piece];
            let cells = def.centered_cells();
            let min = cells.iter().fold(Vec2::MAX, |a, &b| a.min(b));
            let max = cells.iter().fold(Vec2::MIN, |a, &b| a.max(b));
            let span = max - min + Vec2::ONE;
            let room = Vec2::new(card_w, card_h - bar) * 0.75;
            let [cell, icon_x, icon_y] = look.card_icon;
            let unit = if card_art.is_some() {
                card_h * cell
            } else {
                (room.x / span.x).min(room.y / span.y)
            };
            let size = span * unit;
            node.with_children(|card| {
                let mut holder = card.spawn(match card_art {
                    // Placed as on the original's card.
                    Some(_) => Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(card_w * (0.5 + icon_x) - size.x / 2.0),
                        top: Val::Px(card_h * icon_y - size.y / 2.0),
                        width: Val::Px(size.x),
                        height: Val::Px(size.y),
                        ..default()
                    },
                    None => Node {
                        width: Val::Px(size.x),
                        height: Val::Px(size.y),
                        margin: UiRect::top(Val::Px(bar)),
                        ..default()
                    },
                });
                // The pack's own card icon, else its brick art, else the default.
                let picture = art
                    .image(&format!("next_brick_{}_{}", pack.key(), def.name))
                    .or_else(|| art.image(&pack.brick_art(&def.name)))
                    .or_else(|| art.image(&format!("brick_{}", def.name)));
                match picture {
                    Some(image) => {
                        holder.insert(ImageNode::new(image));
                    }
                    None => {
                        let color = Color::srgb(def.color[0], def.color[1], def.color[2]);
                        holder.with_children(|holder| {
                            for c in &cells {
                                let at = (*c - min) * unit;
                                holder.spawn((
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: Val::Px(at.x),
                                        // UI y runs down; cells run up.
                                        top: Val::Px((span.y - 1.0) * unit - at.y),
                                        width: Val::Px(unit),
                                        height: Val::Px(unit),
                                        border: UiRect::all(Val::Px(1.0)),
                                        ..default()
                                    },
                                    BackgroundColor(color),
                                    BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                                ));
                            }
                        });
                    }
                }
            });
            Node {
                width: Val::Px(card_w),
                height: Val::Px(card_h),
                top: Val::Px(0.0),
                ..base
            }
        }
    }
}

/// Where the wizard of a tower floats (world pixels): the wizard itself when it
/// is drawn, else where it would be.
fn wizard_point(config: &GameConfig, solo: bool, tower: &Tower, state: &TowerState) -> Vec2 {
    let look = &config.wizards;
    let offset = if solo {
        look.solo_offset_x
    } else {
        look.offset_x
    };
    Vec2::new(
        tower.origin_x + config.cells(offset),
        config.cells(state.height + look.lift),
    )
}

#[allow(clippy::too_many_arguments)]
fn place_items(
    config: Res<GameConfig>,
    view: Res<ViewSize>,
    camera: Query<(&Camera, &GlobalTransform), With<crate::visuals::MainCamera>>,
    towers: Query<(&Tower, &TowerState)>,
    wizards: Query<(&Wizard, &GlobalTransform)>,
    mut items: Query<(&HudItem, &mut Node, &ComputedNode)>,
) {
    let Ok((camera, eye)) = camera.single() else {
        return;
    };
    let look = &config.hud;
    let solo = towers.iter().count() == 1;
    let screen = |p: Vec2| camera.world_to_viewport(eye, p.extend(0.0)).ok();
    for (item, mut node, computed) in &mut items {
        let Ok((tower, state)) = towers.get(item.tower) else {
            continue;
        };
        let size = computed.size() * computed.inverse_scale_factor();
        let wizard = wizards
            .iter()
            .find(|(w, _)| !w.enemy() && w.tower() == item.tower)
            .map(|(_, t)| t.translation().truncate())
            .unwrap_or_else(|| wizard_point(&config, solo, tower, state));
        let card_h = look.card_height * view.0.y;
        let card_w = card_h * look.card_aspect;
        let column = screen(Vec2::new(tower.origin_x, 0.0)).map(|p| p.x);
        let card_x = column.map(|x| x - look.card_left * view.0.x);
        // Centre of the item on screen.
        let at = match item.kind {
            Kind::Card => card_x.map(|x| Vec2::new(x, card_h * 0.5)),
            // Over the card's lower-left corner: BricksView's label sits 50.8 px left of
            // and 91.2 px under the 126 x 101 card's top middle (NextBrickView), as Max's
            // Survival clip shows.
            Kind::Count => card_x.map(|x| Vec2::new(x - card_w * 0.403, card_h * 0.903)),
            Kind::Place => screen(wizard - Vec2::Y * config.cells(look.place_below)),
            Kind::Hearts => screen(wizard - Vec2::Y * config.cells(look.heart_below)),
        };
        let Some(at) = at else {
            continue;
        };
        node.left = Val::Px(at.x - size.x * 0.5);
        node.top = Val::Px(at.y - size.y * 0.5);
    }
}

/// Draws the roof the winner steers (art `roof_race` / `roof_survival`, else its
/// boxes in red) and plays its drum roll until it lands, then the landing sound.
#[allow(clippy::type_complexity)]
fn drop_roofs(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    new_roofs: Query<(Entity, &RoofPiece), Added<RoofPiece>>,
    towers: Query<(Entity, &TowerState)>,
    mut drums: Local<Vec<(Entity, Entity)>>,
) {
    let c = config.cell_size;
    for (entity, roof) in &new_roofs {
        let (key, art_y, boxes) = if roof.survival {
            (
                "roof_survival",
                config.roofs.survival_art_y,
                &config.roofs.survival,
            )
        } else {
            ("roof_race", config.roofs.race_art_y, &config.roofs.race)
        };
        let mut node = commands.entity(entity);
        node.insert(Visibility::default());
        if let Some((sprite, anchor, mut transform)) = crate::visuals::art_sprite(&art, key, c) {
            transform.translation = Vec3::new(0.0, c * art_y, 6.0);
            node.with_child((sprite, anchor, transform));
        } else {
            for b in boxes {
                let size = Vec2::new(b.x.1 - b.x.0, b.y.1 - b.y.0) * c;
                let centre = Vec2::new(b.x.0 + b.x.1, b.y.0 + b.y.1) * 0.5 * c;
                node.with_child((
                    Sprite::from_color(Color::srgb(0.85, 0.22, 0.2), size),
                    Transform::from_translation(centre.extend(6.0)),
                ));
            }
        }
        if let Some(sound) = art.sounds.get("sfx_drum_roll") {
            let drum = commands
                .spawn((AudioPlayer(sound.clone()), PlaybackSettings::LOOP))
                .id();
            drums.push((roof.tower, drum));
        }
    }
    drums.retain(|&(tower, drum)| {
        let steering = towers
            .get(tower)
            .is_ok_and(|(_, s)| s.roof == RoofStage::Steering);
        if !steering {
            commands.entity(drum).despawn();
            if towers.get(tower).is_ok_and(|(_, s)| s.roof_pending())
                && let Some(sound) = art.sounds.get("sfx_roof_placed")
            {
                commands.spawn((AudioPlayer(sound.clone()), PlaybackSettings::DESPAWN));
            }
        }
        steering
    });
}

/// Losing towers turn pale blue-grey at once when the match is decided.
fn fade_losers(
    mut commands: Commands,
    config: Res<GameConfig>,
    game: Res<Match>,
    towers: Query<(&Tower, &TowerState)>,
    bricks: Query<(&Brick, &Children)>,
    mut sprites: Query<(Entity, &mut Sprite), Without<Faded>>,
) {
    if game.phase != Phase::Over {
        return;
    }
    let look = &config.hud;
    let [r, g, b] = look.loser_tint;
    let pale = Color::srgb(r, g, b);
    for (brick, children) in &bricks {
        let Ok((tower, state)) = towers.get(brick.tower) else {
            continue;
        };
        if !lost(&game, tower, state) {
            continue;
        }
        for child in children {
            if let Ok((entity, mut sprite)) = sprites.get_mut(*child) {
                let alpha = sprite.color.alpha();
                sprite.color = sprite.color.mix(&pale, look.loser_mix).with_alpha(alpha);
                commands.entity(entity).insert(Faded);
            }
        }
    }
}
/// A piece of the Endless score panels (RaceLeaderboardHUD, EndlessPuzzleHUD): the score
/// on the left of the next-brick card (BricksViewLeaderboardBackground/Foreground) and the
/// personal best in the top-right corner (HighscoreView).
#[derive(Component, Clone, Copy, PartialEq)]
enum ScorePart {
    Root,
    Score,
    Best,
}

/// A box on the 1920 x 1080 screen from its top-left corner: `centred` boxes keep their
/// place from the screen's middle, the others from its right edge.
fn hud_box(x: f32, y: f32, w: f32, h: f32, centred: bool) -> Node {
    let k = |v: f32| Val::Vh(v / 10.8);
    let mut node = Node {
        position_type: PositionType::Absolute,
        top: k(y),
        width: k(w),
        height: k(h),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    if centred {
        node.left = Val::Percent(50.0);
        node.margin = UiRect::left(k(x - 960.0));
    } else {
        node.right = k(1920.0 - x - w);
    }
    node
}

fn spawn_scores(mut commands: Commands, art: Option<Res<Art>>) {
    use crate::letters::TextLook;
    use crate::select::label_with;
    let back = art.as_ref().and_then(|a| a.image("menu_info_background"));
    // gph_menuInfoBackground's 9-slice: left 9, right 5, bottom 14.
    let panel = |image: &Handle<Image>| crate::popup::sliced(image.clone(), [9.0, 5.0, 0.0, 14.0]);
    // The labels' words end at their box's right edge.
    let to_right = |node: Node| Node {
        justify_content: JustifyContent::FlexEnd,
        ..node
    };
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            GlobalZIndex(5),
            Visibility::Hidden,
            ScorePart::Root,
        ))
        .with_children(|root| {
            if let Some(back) = &back {
                // 492 x 188, its top-right corner 71 px left of the middle and 119.5 px
                // over the screen's top.
                root.spawn((
                    hud_box(960.0 - 71.045 - 492.0, -119.5, 492.0, 188.0, true),
                    panel(back),
                ));
                // 327 x 191, 125 px past the right edge and 122 px over the top.
                root.spawn((
                    hud_box(1920.0 + 125.0 - 327.44, -122.0, 327.44, 191.3, false),
                    panel(back),
                ));
            }
            // SCORE: (EnzoBlackMilkyWhiteOutline46 at 64) ends 342 px left of the middle.
            root.spawn(to_right(hud_box(960.0 - 342.0 - 268.4, 0.3, 268.4, 80.0, true)))
                .with_children(|b| {
                    b.spawn(label_with(64.0 / 10.8, "SCORE:", TextLook::TITLE));
                });
            // The score (EnzoBlackGold75) centred 253.5 px left of the middle.
            root.spawn(hud_box(
                960.0 - 253.5 - 100.0,
                45.4 - 50.0,
                200.0,
                100.0,
                true,
            ))
            .with_children(|b| {
                b.spawn((
                    label_with(75.0 / 10.8, "0", TextLook::GOLD),
                    ScorePart::Score,
                ));
            });
            // PERSONAL BEST: (26) ends 32 px from the right edge; the best (EnzoBlackGold60)
            // centred 91 px from it.
            root.spawn(to_right(hud_box(1920.0 - 32.0 - 268.3, -5.0, 268.3, 50.0, false)))
                .with_children(|b| {
                    b.spawn(label_with(26.0 / 10.8, "PERSONAL BEST:", TextLook::TITLE));
                });
            root.spawn(hud_box(
                1920.0 - 91.0 - 100.0,
                64.5 - 50.0,
                200.0,
                100.0,
                false,
            ))
            .with_children(|b| {
                b.spawn((
                    label_with(60.0 / 10.8, "0", TextLook::GOLD),
                    ScorePart::Best,
                ));
            });
        });
}

/// Shows the Endless score panels while an Endless run plays: the score (Endless Race's
/// height score, the bricks in the tower otherwise) and the best of it and the mode's
/// leaderboard (HighscoreView).
fn show_scores(
    config: Res<GameConfig>,
    game: Res<Match>,
    board: Option<Res<crate::single::Leaderboard>>,
    towers: Query<(&Tower, &TowerState)>,
    mut parts: Query<(&ScorePart, &mut Visibility)>,
    mut texts: Query<(&ScorePart, &mut Text)>,
) {
    let index = game.mode.endless_index();
    let on = index.is_some() && game.phase != Phase::Menu;
    for (part, mut visibility) in &mut parts {
        if *part == ScorePart::Root {
            visibility.set_if_neq(if on {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            });
        }
    }
    let Some(index) = index.filter(|_| on) else {
        return;
    };
    let score = towers
        .iter()
        .find(|(t, _)| t.human)
        .map_or(0, |(_, state)| match game.mode {
            ModeKind::Endless => config.endless.score(state.best_height),
            _ => state.endless_score(),
        });
    let best = board
        .as_ref()
        .and_then(|b| b.scores[index].first().copied())
        .unwrap_or(0)
        .max(score);
    for (part, mut text) in &mut texts {
        let words = match part {
            ScorePart::Score => score.to_string(),
            ScorePart::Best => best.to_string(),
            ScorePart::Root => continue,
        };
        if text.0 != words {
            text.0 = words;
        }
    }
}
