//! A trial's enemy lane (`gameplay::RivalLane`): in race trials the race house with
//! the rival tower popping in brick by brick at the start, as in the original
//! (docs/research/trials-obstacles-roofs.md, "Rival towers"). The towers' layouts are
//! staged from the install as `rival_towers.ron` (tools/extract/rival_towers.ps1);
//! without it the lane shows the house only. Puzzle trials and battles show each
//! tower's bricks still to come floating in a beam (`puzzle_column`). Looks only.

use std::collections::HashMap;

use bevy::prelude::*;
use serde::Deserialize;

use crate::art::Art;
use crate::config::GameConfig;
use crate::gameplay::RivalLane;
use crate::visuals::art_sprite;

pub struct RivalPlugin;

impl Plugin for RivalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                build_rival_lane,
                pop_in,
                (puzzle_column, bob_column)
                    .chain()
                    .after(crate::visuals::FramedCamera),
            ),
        );
    }
}

/// One stone piece of a rival tower, in cells from the lane centre at the base top.
#[derive(Deserialize)]
struct Piece {
    art: String,
    x: f32,
    y: f32,
    turn: f32,
    scale: f32,
    /// Pieces of a group pop in together, groups in order.
    group: u32,
}

/// Seconds the whole tower takes to pop in, and the least time between pop sounds.
const BUILD_SECONDS: f32 = 1.5;
const POP_SOUND_GAP: f32 = 0.075;

/// The pop-in under way: time so far, groups shown, time since the last pop sound.
#[derive(Component)]
struct PopIn {
    time: f32,
    groups: u32,
    shown: u32,
    since_sound: f32,
}

#[derive(Component)]
struct RivalPiece(u32);

fn read_layouts() -> HashMap<u32, Vec<Piece>> {
    let path = crate::art::extracted_dir().join("rival_towers.ron");
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| ron::from_str(&text).ok())
        .unwrap_or_default()
}

fn build_rival_lane(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    game: Res<crate::gameplay::Match>,
    lanes: Query<(Entity, &RivalLane), Added<RivalLane>>,
    mut layouts: Local<Option<HashMap<u32, Vec<Piece>>>>,
) {
    let c = config.cell_size;
    for (entity, lane) in &lanes {
        let mut node = commands.entity(entity);
        node.insert((
            Transform::from_xyz(lane.origin_x, 0.0, 0.0),
            Visibility::default(),
        ));
        // The lane's own background, placed like a platform's (centre half a cell down).
        node.with_children(|parent| {
            parent
                .spawn((
                    Transform::from_xyz(0.0, -c * 0.5, 0.0),
                    Visibility::default(),
                ))
                .with_children(|layers| {
                    crate::visuals::spawn_lane_layers(layers, &art, game.mode, c)
                });
        });
        let Some(rival) = lane.rival else { continue };
        // The race house, its top at the lane's y = 0.
        if let Some((sprite, anchor, transform)) = art_sprite(&art, "platform_race", c) {
            node.with_child((sprite, anchor, transform));
        }
        let layouts = layouts.get_or_insert_with(read_layouts);
        let Some(pieces) = layouts.get(&rival) else {
            continue;
        };
        for piece in pieces {
            let key = match piece.art.strip_prefix("ivy_") {
                Some(shape) => format!("vine_overlay_{shape}"),
                None => piece.art.clone(),
            };
            let Some((mut sprite, anchor, mut transform)) = art_sprite(&art, &key, c) else {
                continue;
            };
            // Ivy on a rival brick is fully grown: the last frame.
            if let Some(frame) = art.frame(&key, 1.0e6) {
                sprite.image = frame;
            }
            transform.translation = Vec3::new(piece.x * c, piece.y * c, 0.5);
            transform.rotation = Quat::from_rotation_z(piece.turn.to_radians());
            transform.scale *= Vec3::new(piece.scale, piece.scale, 1.0);
            node.with_child((
                sprite,
                anchor,
                transform,
                Visibility::Hidden,
                RivalPiece(piece.group),
            ));
        }
        let groups = pieces.iter().map(|p| p.group).max().unwrap_or(0);
        node.insert(PopIn {
            time: 0.0,
            groups,
            shown: 0,
            since_sound: POP_SOUND_GAP,
        });
    }
}

/// Shows round(groups × t / 1.5) groups at time t, with a bubble pop at most every
/// 0.075 s, its pitch rising 1 → 2.5 over the build.
fn pop_in(
    mut commands: Commands,
    time: Res<Time>,
    art: Res<Art>,
    mut lanes: Query<(&mut PopIn, &Children)>,
    mut pieces: Query<(&RivalPiece, &mut Visibility)>,
) {
    for (mut pop, children) in &mut lanes {
        if pop.shown >= pop.groups {
            continue;
        }
        pop.time += time.delta_secs();
        pop.since_sound += time.delta_secs();
        let progress = (pop.time / BUILD_SECONDS).min(1.0);
        let due = ((pop.groups as f32 * progress).round() as u32).min(pop.groups);
        if due > pop.shown {
            for child in children.iter() {
                if let Ok((piece, mut visible)) = pieces.get_mut(child)
                    && piece.0 <= due
                {
                    *visible = Visibility::Inherited;
                }
            }
            pop.shown = due;
            if pop.since_sound > POP_SOUND_GAP
                && let Some(sound) = art.sounds.get("sfx_bubble_pop")
            {
                pop.since_sound = 0.0;
                commands.spawn((
                    AudioPlayer(sound.clone()),
                    PlaybackSettings::DESPAWN.with_speed(1.0 + progress * 1.5),
                ));
            }
        }
    }
}

/// A Puzzle tower's column (WizardPuzzleBricksView, PuzzleWizardHUD): its bricks
/// still to come, top to bottom from 36 cells above the camera's lowest bottom down
/// 28 cells in all, each popping in over 0.5 s (out-back) in turn over the first
/// 0.5 s, then bobbing ±0.5 cell and swaying ±10°; the top one goes as each brick is
/// dealt. Behind it a beam grows to half width over 1 s and spans the view. In a
/// battle every tower has one, 11 cells to its right in its own world.
const COLUMN_TOP: f32 = 36.0;
const COLUMN_SPAN: f32 = 28.0;
const COLUMN_SIDE: f32 = 11.0;
const BEAM_WIDTH: f32 = 0.5;
const BEAM_ALPHA: f32 = 0.392;
/// Where a trial's column stands across the view, from Max's screenshot of the
/// original's Puzzle trial: our trial lanes aren't framed like the original's yet, so
/// it keeps to the screen instead of 11 cells into the enemy's world (?).
const COLUMN_SCREEN_X: f32 = 0.875;

#[derive(Component)]
struct PuzzleColumn {
    tower: Entity,
    /// Centre x in a battle; `None` keeps a trial's to the screen.
    x: Option<f32>,
    time: f32,
    used: usize,
    /// The lowest the view's bottom has been (the original's camera `minBottom`).
    floor: f32,
}

#[derive(Component)]
struct ColumnBrick {
    index: usize,
    count: usize,
    start: Vec2,
    offset: f32,
}

/// The beam behind a column, until its art is swapped for a glow copy.
#[derive(Component)]
struct ColumnBeam {
    glowing: bool,
}

/// The bricks a running Puzzle trial deals, in order.
fn trial_order(config: &GameConfig) -> Vec<String> {
    config
        .trial
        .and_then(|i| config.trials.get(i))
        .filter(|t| t.mode == crate::config::ModeKind::Puzzle)
        .map(|t| t.order.clone())
        .unwrap_or_default()
}

#[allow(clippy::type_complexity)]
fn puzzle_column(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    game: Res<crate::gameplay::Match>,
    towers: Query<(
        Entity,
        &crate::gameplay::Tower,
        &crate::gameplay::TowerState,
        Option<&crate::gameplay::TowerSkin>,
        Option<&crate::gameplay::BrickBag>,
    )>,
    mut columns: Query<(Entity, &mut PuzzleColumn)>,
    bricks: Query<(Entity, &ColumnBrick, &ChildOf)>,
    in_hand: Query<&crate::gameplay::Brick, With<crate::gameplay::ActiveBrick>>,
) {
    let trial = trial_order(&config);
    let battle = config.trial.is_none() && game.mode == crate::config::ModeKind::Puzzle;
    let wanted = (!trial.is_empty() || battle) && game.phase != crate::gameplay::Phase::Menu;
    for (entity, column) in &columns {
        if !wanted || !towers.contains(column.tower) {
            commands.entity(entity).despawn();
        }
    }
    if !wanted {
        return;
    }
    let c = config.cell_size;
    for (id, tower, state, skin, bag) in &towers {
        // A trial has one column, the player's.
        if !battle && tower.index != 0 {
            continue;
        }
        if let Some((entity, mut column)) = columns.iter_mut().find(|(_, col)| col.tower == id) {
            // Each brick leaves the top as it's dealt (the original's BRICK_LEFT count).
            let dealt = in_hand.iter().any(|b| b.tower == id) as u32;
            let used = (state.bricks_used() + dealt) as usize;
            if used > column.used {
                for (brick, info, parent) in &bricks {
                    if parent.parent() == entity && info.index < used {
                        commands.entity(brick).despawn();
                    }
                }
                column.used = used;
            }
            continue;
        }
        // A battle's bricks are the tower's bag ahead (everyone gets the same).
        let order: Vec<String> = match (battle, bag) {
            (true, Some(bag)) => {
                let mut ahead = bag.clone();
                (0..config.puzzle.bricks)
                    .filter_map(|_| config.pieces.get(ahead.draw()).map(|p| p.name.clone()))
                    .collect()
            }
            (true, None) => continue,
            _ => trial.clone(),
        };
        let pack = skin.map(|s| s.pack).unwrap_or_default();
        let count = order.len();
        let mut node = commands.spawn((
            PuzzleColumn {
                tower: id,
                x: battle.then(|| tower.origin_x + COLUMN_SIDE * c),
                time: 0.0,
                used: 0,
                floor: f32::MAX,
            },
            Transform::from_xyz(tower.origin_x, 0.0, -1.0),
            Visibility::default(),
        ));
        if let Some(image) = art
            .image("cast_lane_beam_survival")
            .or_else(|| art.image("cast_lane_beam"))
        {
            node.with_child((
                ColumnBeam { glowing: false },
                Sprite::from_image(image),
                Transform::from_scale(Vec3::new(0.0, 1.0, 1.0)),
            ));
        }
        for (index, name) in order.iter().enumerate() {
            let Some(def) = config.pieces.iter().find(|p| &p.name == name) else {
                continue;
            };
            let start = Vec2::new(
                0.0,
                (COLUMN_TOP - index as f32 * COLUMN_SPAN / count as f32) * c,
            );
            let cells = def.centered_cells();
            let image = [pack.brick_art(&def.name), format!("brick_{}", def.name)]
                .iter()
                .find_map(|k| art.image(k));
            // A fixed offset per brick so they don't bob in step (the original rolls 0-5).
            let offset = ((index as f32 + 1.0) * 0.618_034).fract() * 5.0;
            node.with_children(|column| {
                let mut brick = column.spawn((
                    ColumnBrick {
                        index,
                        count,
                        start,
                        offset,
                    },
                    Transform::from_translation(start.extend(index as f32 * 0.01))
                        .with_scale(Vec3::ZERO),
                    Visibility::default(),
                ));
                match image {
                    Some(image) => {
                        let min = cells.iter().fold(Vec2::MAX, |a, &b| a.min(b));
                        let max = cells.iter().fold(Vec2::MIN, |a, &b| a.max(b));
                        brick.with_child((
                            Sprite {
                                image,
                                custom_size: Some((max - min + Vec2::ONE) * c),
                                ..default()
                            },
                            Transform::from_translation(((min + max) * 0.5 * c).extend(0.0)),
                        ));
                    }
                    None => {
                        let color = Color::srgb(def.color[0], def.color[1], def.color[2]);
                        brick.with_children(|b| crate::visuals::cell_sprites(b, &cells, c, color));
                    }
                }
            });
        }
    }
}

/// Pops the columns' bricks in, bobs them, grows the beams and fits them to the view.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn bob_column(
    mut commands: Commands,
    time: Res<Time>,
    art: Res<Art>,
    config: Res<GameConfig>,
    view: Option<Res<crate::visuals::ViewSize>>,
    mut images: ResMut<Assets<Image>>,
    camera: Query<(&Transform, &Projection), (With<Camera2d>, Without<PuzzleColumn>)>,
    mut columns: Query<
        (&mut PuzzleColumn, &mut Transform),
        (Without<ColumnBrick>, Without<ColumnBeam>, Without<Camera2d>),
    >,
    mut bricks: Query<
        (&ColumnBrick, &mut Transform),
        (
            Without<PuzzleColumn>,
            Without<ColumnBeam>,
            Without<Camera2d>,
        ),
    >,
    mut beams: Query<
        (&mut ColumnBeam, &mut Transform, &mut Sprite),
        (
            Without<PuzzleColumn>,
            Without<ColumnBrick>,
            Without<Camera2d>,
        ),
    >,
) {
    let Ok((camera, projection)) = camera.single() else {
        return;
    };
    let zoom = match projection {
        Projection::Orthographic(o) => o.scale,
        _ => 1.0,
    };
    let view = view.map_or(Vec2::new(1280.0, 720.0), |v| v.0) * zoom;
    let dt = time.delta_secs();
    // Every column hangs from the lowest view bottom and they share one clock.
    let mut clock = None;
    for (mut column, mut at) in &mut columns {
        column.floor = column.floor.min(camera.translation.y - view.y / 2.0);
        at.translation.x = column
            .x
            .unwrap_or(camera.translation.x + (COLUMN_SCREEN_X - 0.5) * view.x);
        at.translation.y = column.floor;
        let before = column.time;
        column.time += dt;
        clock = Some((before, column.time, at.translation.y));
    }
    let Some((before, t, column_y)) = clock else {
        return;
    };
    let c = config.cell_size;
    // The pop sounds play once for all columns: they come in together.
    let mut popped = Vec::new();
    for (brick, mut transform) in &mut bricks {
        let count = brick.count.max(1) as f32;
        let delay = brick.index as f32 / count * 0.5;
        if before <= delay && t > delay && !popped.contains(&brick.index) {
            popped.push(brick.index);
            if let Some(sound) = art.sounds.get("sfx_bubble_pop") {
                commands.spawn((
                    AudioPlayer(sound.clone()),
                    PlaybackSettings::DESPAWN.with_speed(1.0 + brick.index as f32 / count * 1.5),
                ));
            }
        }
        let o = brick.offset;
        transform.translation.x = brick.start.x + (o + t * 0.4852).sin() * 0.5 * c;
        transform.translation.y = brick.start.y + (o + t * 0.762).cos() * 0.5 * c;
        transform.rotation = Quat::from_rotation_z(((o + t * 0.5).sin() * 10.0).to_radians());
        transform.scale = Vec3::splat(out_back(((t - delay) / 0.5).clamp(0.0, 1.0)));
    }
    for (mut beam, mut transform, mut sprite) in &mut beams {
        if !beam.glowing
            && let Some(glow) = images
                .get(&sprite.image)
                .and_then(|i| crate::visuals::glow_copy(i, BEAM_ALPHA))
        {
            sprite.image = images.add(glow);
            beam.glowing = true;
        }
        let size = images
            .get(&sprite.image)
            .map_or(Vec2::ONE, |i| i.size_f32());
        // Spans the view and follows the camera up and down.
        transform.translation.y = camera.translation.y - column_y;
        transform.scale.y = view.y / size.y.max(1.0);
        // Grows to half its width over 1 s; the art is 26 px per cell.
        transform.scale.x = BEAM_WIDTH * t.min(1.0) * c / 26.0;
    }
}
/// DOTween's OutBack ease (overshoot 1.70158).
fn out_back(t: f32) -> f32 {
    let s = 1.70158;
    let t = t - 1.0;
    t * t * ((s + 1.0) * t + s) + 1.0
}
