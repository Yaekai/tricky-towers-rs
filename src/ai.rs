//! AI wizards. Each opponent picks a spot for its brick by test-dropping it in
//! every rotation and position, then steers there through the same
//! [`PlayerInput`] a human uses, at a human-like pace.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::config::GameConfig;
use crate::gameplay::{
    ActiveBrick, AiControlled, Brick, GameplaySystems, Match, Phase, PlayerInput, Tower,
    TowerState, TowerStatus, brick_collider, xorshift,
};

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (add_brains, drive_ai).chain().before(GameplaySystems),
        );
    }
}

#[derive(Component, Debug)]
pub struct AiBrain {
    brick: Option<Entity>,
    rotations_left: u32,
    target_x: f32,
    timer: f32,
    rng: u64,
}

fn add_brains(
    mut commands: Commands,
    config: Res<GameConfig>,
    new: Query<(Entity, &Tower), Added<AiControlled>>,
) {
    for (entity, tower) in &new {
        commands.entity(entity).insert(AiBrain {
            brick: None,
            rotations_left: 0,
            target_x: 0.0,
            timer: 0.0,
            rng: config.seed ^ (0x9E37_79B9 * (tower.index as u64 + 1)),
        });
    }
}

#[allow(clippy::type_complexity)]
fn drive_ai(
    time: Res<Time>,
    config: Res<GameConfig>,
    game: Res<Match>,
    spatial: SpatialQuery,
    mut towers: Query<(&Tower, &TowerState, &mut PlayerInput, &mut AiBrain)>,
    bricks: Query<(&Brick, &Position, &Rotation), With<ActiveBrick>>,
) {
    if game.phase != Phase::Playing {
        return;
    }
    let dt = time.delta_secs();
    let c = config.cell_size;
    for (tower, state, mut input, mut brain) in &mut towers {
        input.left_held = false;
        input.right_held = false;
        input.fast_fall_held = false;
        if state.status != TowerStatus::Building {
            continue;
        }
        // Curse the others when there's a curse to cast, now and then help yourself.
        if state.dark_spell.is_some() && !xorshift(&mut brain.rng).is_multiple_of(3) {
            input.cast_dark_pressed = true;
        } else if state.light_spell.is_some() {
            input.cast_light_pressed = true;
        }
        let Some(entity) = state.active else { continue };
        let Ok((brick, position, rotation)) = bricks.get(entity) else {
            continue;
        };

        if brain.brick != Some(entity) {
            let (rotations, target_x) = plan(
                &config, &spatial, tower, entity, brick, position.0, *rotation,
            );
            brain.brick = Some(entity);
            brain.rotations_left = rotations;
            brain.target_x = target_x;
            if (xorshift(&mut brain.rng) % 1000) as f32 / 1000.0 < config.ai.mistake_chance {
                let off = if xorshift(&mut brain.rng).is_multiple_of(2) {
                    1.0
                } else {
                    -1.0
                };
                brain.target_x += off * c;
            }
            brain.timer = config.ai.think_delay;
        }

        brain.timer -= dt;
        if brain.timer > 0.0 {
            continue;
        }
        if brain.rotations_left > 0 {
            input.rotate_pressed = true;
            brain.rotations_left -= 1;
            brain.timer = config.ai.step_interval;
            continue;
        }
        let dx = brain.target_x - position.0.x;
        if dx.abs() > config.cells(config.move_step) * 0.5 {
            if dx > 0.0 {
                input.right_pressed = true;
            } else {
                input.left_pressed = true;
            }
            brain.timer = config.ai.step_interval;
        } else {
            input.fast_fall_held = true;
        }
    }
}

/// Test-drops the brick in every rotation and half-cell position over the
/// platform and returns the best (rotations, x). Lower, centred, flat
/// placements score best.
#[allow(clippy::too_many_arguments)]
fn plan(
    config: &GameConfig,
    spatial: &SpatialQuery,
    tower: &Tower,
    entity: Entity,
    brick: &Brick,
    start: Vec2,
    rotation: Rotation,
) -> (u32, f32) {
    let c = config.cell_size;
    let cell = c * brick.scale;
    let step = config.cells(config.move_step);
    let half = config.cells(config.base_half_width(config.mode));
    let filter = crate::gameplay::tower_filter(tower.index).with_excluded_entities([entity]);
    // Plan with the brick's real size: the shrunk steering shape would happily
    // squeeze it into spots where it overlaps its neighbours once placed.
    let probe = brick_collider(&brick.cells, cell, config.collider_scale);
    let mut best = (f32::MAX, 0, tower.origin_x);

    for turns in 0..4u32 {
        let rot = rotation * Rotation::degrees(-90.0 * turns as f32);
        let rotated: Vec<Vec2> = brick.cells.iter().map(|&p| rot * (p * cell)).collect();
        let min = rotated.iter().fold(Vec2::MAX, |a, &b| a.min(b)) - Vec2::splat(cell * 0.5);
        let max = rotated.iter().fold(Vec2::MIN, |a, &b| a.max(b)) + Vec2::splat(cell * 0.5);
        let steps = (half / step).ceil() as i32;
        for k in -steps..=steps {
            let x = tower.origin_x + k as f32 * step;
            // Keep the whole brick over the platform.
            if x + min.x < tower.origin_x - half || x + max.x > tower.origin_x + half {
                continue;
            }
            let from = Vec2::new(x, start.y);
            if !spatial
                .shape_intersections(&probe, from, rot.as_radians(), &filter)
                .is_empty()
            {
                continue;
            }
            let Some(hit) = spatial.cast_shape(
                &probe,
                from,
                rot.as_radians(),
                Dir2::NEG_Y,
                &ShapeCastConfig::from_max_distance(config.cells(200.0)),
                &filter,
            ) else {
                continue;
            };
            let center_y = from.y - hit.distance;
            let top = center_y + max.y;
            let bottom = center_y + min.y;
            // Gaps under the brick make towers wobble and topple: count how far
            // each cell with nothing of its own below it would hang in the air.
            let mut gap = 0.0;
            for &p in &rotated {
                let own_below = rotated.iter().any(|&q| {
                    (q.x - p.x).abs() < cell * 0.25 && (p.y - q.y - cell).abs() < cell * 0.25
                });
                if own_below {
                    continue;
                }
                let foot = Vec2::new(x + p.x, center_y + p.y - cell * 0.5);
                let air = spatial
                    .cast_ray(foot, Dir2::NEG_Y, config.cells(3.0), true, &filter)
                    .map_or(3.0, |h| h.distance / c);
                gap += air.min(3.0);
            }
            let score = top / c
                + 0.5 * bottom / c
                + 1.5 * gap
                + 0.3 * (x - tower.origin_x).abs() / c
                + 0.05 * turns as f32;
            if score < best.0 {
                best = (score, turns, x);
            }
        }
    }
    (best.1, best.2)
}
