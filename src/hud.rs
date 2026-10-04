//! The in-match HUD as the original draws it: nothing along the bottom of the
//! screen, everything in each player's lane. The next-brick card hangs from the
//! top edge (with the Survival brick count beside it), the place label (1ST, 2ND)
//! or the hearts sit under the wizard, and the held spells float as bubbles beside
//! it. Once a match is won, the winner gets its roof and the losers turn pale and
//! their wizards vanish (docs/research/video-notes.md, `GameConfig::hud`).

use std::collections::HashSet;

use bevy::prelude::*;

use crate::art::{Animated, Art};
use crate::config::{GameConfig, HudLook, ModeKind, SpellKind};
use crate::gameplay::{Brick, BrickBag, Match, Phase, Tower, TowerState, TowerStatus, placings};
use crate::visuals::{FramedCamera, ViewSize};
use crate::wizards::Wizard;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
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
    /// The light (0) or dark (1) spell bubble.
    Bubble(u8),
}

const KINDS: [Kind; 6] = [
    Kind::Card,
    Kind::Count,
    Kind::Place,
    Kind::Hearts,
    Kind::Bubble(0),
    Kind::Bubble(1),
];

/// What an item showed last, so it is only rebuilt when that changes.
#[derive(Clone, PartialEq, Debug)]
enum Shows {
    Nothing,
    /// The next brick, drawn with the tower's brick pack.
    Card(usize, crate::config::BrickPack),
    Text(String, Color),
    Hearts(u32),
    Light(SpellKind),
    Dark,
}

#[derive(Component)]
struct HudItem {
    tower: Entity,
    kind: Kind,
    shows: Option<Shows>,
    /// The view height the item was built at; sizes follow it.
    built_at: f32,
}

/// The roof dropped onto a finished tower, and how long it has fallen.
#[derive(Component)]
struct Roof {
    tower: Entity,
    time: f32,
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
                GlobalZIndex(5),
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
) -> Shows {
    let look = &config.hud;
    let playing = game.phase == Phase::Playing && state.status == TowerStatus::Building;
    match kind {
        Kind::Card => {
            let finished_puzzle =
                game.mode == ModeKind::Puzzle && state.bricks_used() + 1 >= config.puzzle.bricks;
            match bag {
                Some(bag) if playing && !finished_puzzle => Shows::Card(bag.peek(), pack),
                _ => Shows::Nothing,
            }
        }
        Kind::Count if game.mode == ModeKind::Survival && game.phase != Phase::Menu => {
            let left = config
                .survival
                .bricks_to_place
                .saturating_sub(state.bricks_used());
            let over = game.phase == Phase::Over;
            if over && state.status != TowerStatus::Won && game.winner.is_some() {
                Shows::Nothing
            } else {
                Shows::Text(left.to_string(), rgb(look.place_first))
            }
        }
        Kind::Place if game.mode == ModeKind::Race && playing => match place {
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
        },
        Kind::Hearts if state.status != TowerStatus::Out && game.phase != Phase::Over => {
            match state.lives {
                Some(lives) if lives > 0 => Shows::Hearts(lives),
                _ => Shows::Nothing,
            }
        }
        Kind::Bubble(0) if config.magic.enabled && playing => {
            state.light_spell.map_or(Shows::Nothing, Shows::Light)
        }
        Kind::Bubble(_) if config.magic.enabled && playing => {
            if state.dark_spell.is_some() {
                Shows::Dark
            } else {
                Shows::Nothing
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
        ));
    };
    match shows {
        Shows::Nothing => base,
        Shows::Text(words, color) => {
            let size = match words.parse::<u32>() {
                Ok(_) => look.count_size,
                Err(_) => look.place_size,
            } * height;
            text(node, words, *color, size);
            base
        }
        Shows::Hearts(lives) => {
            let size = look.heart_size * height;
            match art.image("heart") {
                Some(image) => {
                    node.with_children(|row| {
                        for _ in 0..*lives {
                            row.spawn((
                                ImageNode::new(image.clone()),
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
                column_gap: Val::Px(size * 0.1),
                ..base
            }
        }
        Shows::Light(_) | Shows::Dark => {
            let size = look.bubble_size * height;
            let (frame, color) = match shows {
                Shows::Light(_) => ("spell_bubble_light", rgb(look.bubble_light)),
                _ => ("spell_bubble_dark", rgb(look.bubble_dark)),
            };
            match art.image(frame) {
                Some(image) => {
                    node.insert(ImageNode::new(image));
                }
                None => {
                    node.insert(BackgroundColor(color.with_alpha(0.85)));
                }
            }
            node.with_children(|bubble| match shows {
                Shows::Light(spell) => match art.image(&format!("spell_icon_{spell:?}")) {
                    Some(image) => {
                        let mut icon = bubble.spawn((
                            ImageNode::new(image),
                            Node {
                                width: Val::Px(size * 0.7),
                                height: Val::Px(size * 0.7),
                                ..default()
                            },
                        ));
                        let name = format!("spell_icon_{spell:?}");
                        if art.is_animation(&name) {
                            icon.insert(Animated::new(name));
                        }
                    }
                    None => {
                        bubble.spawn((
                            Text::new(spell.name()),
                            TextFont {
                                font_size: FontSize::Px(size * 0.22),
                                ..default()
                            },
                        ));
                    }
                },
                _ => {
                    bubble.spawn((
                        Text::new("?"),
                        TextFont {
                            font_size: FontSize::Px(size * 0.6),
                            ..default()
                        },
                    ));
                }
            });
            Node {
                width: Val::Px(size),
                height: Val::Px(size),
                border_radius: BorderRadius::MAX,
                ..base
            }
        }
        Shows::Card(piece, pack) => {
            let card_h = look.card_height * height;
            let card_w = card_h * look.card_aspect;
            let bar = card_h * look.card_bar;
            match art.image("next_card") {
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
            let unit = (room.x / span.x).min(room.y / span.y);
            node.with_children(|card| {
                let mut holder = card.spawn(Node {
                    width: Val::Px(span.x * unit),
                    height: Val::Px(span.y * unit),
                    margin: UiRect::top(Val::Px(bar)),
                    ..default()
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
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
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
            .find(|(w, _)| w.tower() == item.tower)
            .map(|(_, t)| t.translation().truncate())
            .unwrap_or_else(|| wizard_point(&config, solo, tower, state));
        // Toward the tower: left of a wizard floating right of it.
        let toward = if wizard.x > tower.origin_x { -1.0 } else { 1.0 };
        let card_h = look.card_height * view.0.y;
        let card_w = card_h * look.card_aspect;
        let column = screen(Vec2::new(tower.origin_x, 0.0)).map(|p| p.x);
        let card_x = column.map(|x| x - look.card_left * view.0.x);
        // Centre of the item on screen.
        let at = match item.kind {
            Kind::Card => card_x.map(|x| Vec2::new(x, card_h * 0.5)),
            Kind::Count => card_x.map(|x| Vec2::new(x - card_w * 0.5, card_h * 0.85)),
            Kind::Place => screen(wizard - Vec2::Y * config.cells(look.place_below)),
            Kind::Hearts => screen(wizard - Vec2::Y * config.cells(look.heart_below)),
            Kind::Bubble(i) => {
                let (x, y) = look.bubble_at;
                let (sx, sy) = look.bubble_step;
                let i = i as f32;
                let offset = Vec2::new(toward * (x + sx * i), y + sy * i);
                screen(wizard + config.cells(1.0) * offset)
            }
        };
        let Some(at) = at else {
            continue;
        };
        node.left = Val::Px(at.x - size.x * 0.5);
        node.top = Val::Px(at.y - size.y * 0.5);
    }
}

/// Drops a roof onto each finished tower: the race winner, or a Survival tower
/// that placed all its bricks. Art `roof_<mode>` or `roof`, else a stepped red spire.
fn drop_roofs(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    game: Res<Match>,
    art: Res<Art>,
    towers: Query<(Entity, &Tower, &TowerState)>,
    mut roofs: Query<(Entity, &mut Roof, &mut Transform)>,
) {
    let look = &config.hud;
    let c = config.cell_size;
    for (entity, mut roof, mut transform) in &mut roofs {
        let Ok((_, tower, state)) = towers.get(roof.tower) else {
            commands.entity(entity).despawn();
            continue;
        };
        if game.phase == Phase::Menu {
            commands.entity(entity).despawn();
            continue;
        }
        roof.time += time.delta_secs();
        // Falls like a dropped weight: slow at first, fast at the end.
        let t = (roof.time / look.roof_drop.max(0.01)).min(1.0);
        let top = config.cells(state.height) + c * 0.5;
        transform.translation.x = tower.origin_x;
        transform.translation.y = top + config.cells(look.roof_from) * (1.0 - t * t);
    }
    if !matches!(game.mode.rules(), ModeKind::Race | ModeKind::Survival) {
        return;
    }
    for (entity, tower, state) in &towers {
        let finished = state.status == TowerStatus::Won
            || (game.phase == Phase::Over && game.winner == Some(tower.index));
        if !finished || roofs.iter().any(|(_, r, _)| r.tower == entity) {
            continue;
        }
        let key = format!("roof_{}", crate::visuals::mode_key(game.mode));
        let image = art.image(&key).or_else(|| art.image("roof"));
        let (w, h) = (
            config.cells(look.roof_size.0),
            config.cells(look.roof_size.1),
        );
        let mut roof = commands.spawn((
            Roof {
                tower: entity,
                time: 0.0,
            },
            Transform::from_xyz(
                tower.origin_x,
                config.cells(state.height + look.roof_from),
                6.0,
            ),
            Visibility::default(),
        ));
        match image {
            Some(image) => {
                roof.with_child((
                    Sprite {
                        image,
                        custom_size: Some(Vec2::new(w, h)),
                        ..default()
                    },
                    bevy::sprite::Anchor::BOTTOM_CENTER,
                ));
            }
            None => {
                // A spire of narrowing red steps with a gold tip.
                let steps = 6;
                roof.with_children(|roof| {
                    for i in 0..steps {
                        let share = 1.0 - i as f32 / steps as f32;
                        roof.spawn((
                            Sprite::from_color(
                                Color::srgb(0.85, 0.2 + 0.03 * i as f32, 0.2),
                                Vec2::new(w * share, h / steps as f32),
                            ),
                            Transform::from_xyz(0.0, h * (i as f32 + 0.5) / steps as f32, 0.0),
                        ));
                    }
                    roof.spawn((
                        Sprite::from_color(Color::srgb(1.0, 0.8, 0.3), Vec2::splat(w * 0.12)),
                        Transform::from_xyz(0.0, h + w * 0.06, 0.1),
                    ));
                });
            }
        }
    }
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
