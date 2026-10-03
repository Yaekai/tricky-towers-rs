//! Spells. A tower earns one every few bricks and holds it until cast. Light
//! spells help the caster; dark spells hit the opponent with the tallest tower.
//!
//! The behaviour of each spell is rebuilt from what the original visibly does;
//! numbers live in `MagicRules` so they can be tuned against the real game.

use avian2d::dynamics::solver::islands::BodyIslandNode;
use avian2d::prelude::*;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;

use crate::config::{GameConfig, SpellKind};
use crate::gameplay::{
    ActiveBrick, Brick, Match, Phase, PlayerInput, Tower, TowerState, TowerStatus, brick_collider,
};

/// A vine holding two bricks together.
#[derive(Component, Debug)]
pub struct VineJoint {
    pub a: Entity,
    pub b: Entity,
}

/// A brick turned to stone by the Stone spell.
#[derive(Component, Debug)]
pub struct Stone;

/// A brick wrapped in vines.
#[derive(Component, Debug)]
pub struct Vined;

/// One spell cast, for the HUD, visuals, sound or mods.
#[derive(Debug, Clone)]
pub struct SpellCast {
    pub spell: SpellKind,
    pub caster: usize,
    pub target: usize,
}

/// The last few spells cast, newest last.
#[derive(Resource, Debug, Default)]
pub struct SpellLog(pub Vec<SpellCast>);

pub(crate) fn grant_spells(
    config: Res<GameConfig>,
    game: Res<Match>,
    mut towers: Query<&mut TowerState>,
) {
    let magic = &config.magic;
    if !magic.enabled || game.phase != Phase::Playing {
        return;
    }
    let opponents = towers
        .iter()
        .filter(|s| s.status == TowerStatus::Building)
        .count()
        > 1;
    for mut state in &mut towers {
        if state.spell.is_some() || state.spell_progress < magic.bricks_per_spell {
            continue;
        }
        state.spell_progress = 0;
        let mut pool: Vec<SpellKind> = magic.light.clone();
        if opponents {
            pool.extend(magic.dark.iter().copied());
        }
        if pool.is_empty() {
            continue;
        }
        let pick = (state.random() % pool.len() as u64) as usize;
        state.spell = Some(pool[pick]);
    }
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(crate) fn cast_spells(
    mut commands: Commands,
    config: Res<GameConfig>,
    game: Res<Match>,
    spatial: SpatialQuery,
    mut log: ResMut<SpellLog>,
    mut towers: Query<(Entity, &Tower, &mut TowerState, &mut PlayerInput)>,
    joints: Query<(Entity, &VineJoint)>,
    bricks: Query<
        (
            Entity,
            &Brick,
            &Position,
            &Rotation,
            &LinearVelocity,
            Has<Stone>,
            Has<BodyIslandNode>,
        ),
        Without<ActiveBrick>,
    >,
) {
    let mut casts = Vec::new();
    for (entity, tower, mut state, mut input) in &mut towers {
        if !std::mem::take(&mut input.cast_pressed) {
            continue;
        }
        if game.phase != Phase::Playing || state.status != TowerStatus::Building {
            continue;
        }
        if let Some(spell) = state.spell.take() {
            casts.push((entity, tower.index, spell));
        }
    }

    let magic = &config.magic;
    for (caster, caster_index, spell) in casts {
        let target = if spell.is_dark() {
            // The opponent with the tallest tower.
            towers
                .iter()
                .filter(|(e, _, s, _)| *e != caster && s.status == TowerStatus::Building)
                .max_by(|a, b| a.2.height.total_cmp(&b.2.height))
                .map(|(e, t, _, _)| (e, t.index))
        } else {
            Some((caster, caster_index))
        };
        let Some((target, target_index)) = target else {
            continue;
        };
        let Ok((_, _, mut state, _)) = towers.get_mut(target) else {
            continue;
        };
        match spell {
            SpellKind::Shrink => state.next_mod.scale = magic.shrink_scale,
            SpellKind::Enlarge => state.next_mod.scale = magic.enlarge_scale,
            SpellKind::Slippery => {
                state.next_mod.friction = Some(magic.slippery_friction);
                state.next_mod.tint = Some(Color::srgb(0.75, 0.92, 1.0));
            }
            SpellKind::Stone => {
                let settled = config.cells(0.5);
                let mut cut: HashSet<Entity> = HashSet::default();
                for (entity, brick, _, _, velocity, stone, _) in &bricks {
                    if brick.tower == target && !stone && velocity.0.length() < settled {
                        // Stone replaces vines: avian's islands break if a
                        // jointed body turns static.
                        for (joint, vine) in &joints {
                            if (vine.a == entity || vine.b == entity) && cut.insert(joint) {
                                commands.entity(joint).despawn();
                            }
                        }
                        commands.entity(entity).insert((
                            RigidBody::Static,
                            Stone,
                            LinearVelocity::ZERO,
                            AngularVelocity::ZERO,
                        ));
                    }
                }
            }
            SpellKind::Vines => grow_vines(&mut commands, &config, &spatial, &bricks, target),
        }
        log.0.push(SpellCast {
            spell,
            caster: caster_index,
            target: target_index,
        });
    }
    let excess = log.0.len().saturating_sub(5);
    log.0.drain(..excess);
}

/// Joins every brick of a tower to each brick it touches.
#[allow(clippy::type_complexity)]
fn grow_vines(
    commands: &mut Commands,
    config: &GameConfig,
    spatial: &SpatialQuery,
    bricks: &Query<
        (
            Entity,
            &Brick,
            &Position,
            &Rotation,
            &LinearVelocity,
            Has<Stone>,
            Has<BodyIslandNode>,
        ),
        Without<ActiveBrick>,
    >,
    tower: Entity,
) {
    let mut joined: HashSet<(Entity, Entity)> = HashSet::default();
    for (entity, brick, position, rotation, _, _, in_island) in
        bricks.iter().filter(|b| b.1.tower == tower)
    {
        // Avian can only join bodies that are already in its island graph: not
        // stone (static) bricks, nor one released this step.
        if !in_island {
            continue;
        }
        // Slightly oversized, so bricks resting against each other count as touching.
        let reach = brick_collider(&brick.cells, config.cell_size * brick.scale, 1.12);
        let filter = SpatialQueryFilter::from_excluded_entities([entity]);
        for other in spatial.shape_intersections(&reach, position.0, rotation.as_radians(), &filter)
        {
            let Ok((_, other_brick, other_position, _, _, _, other_in_island)) = bricks.get(other)
            else {
                continue;
            };
            if other_brick.tower != tower || !other_in_island {
                continue;
            }
            let pair = if entity < other {
                (entity, other)
            } else {
                (other, entity)
            };
            if !joined.insert(pair) {
                continue;
            }
            let anchor = (position.0 + other_position.0) * 0.5;
            commands.spawn((
                // Global anchor and basis keep the bricks' current relative pose;
                // a local identity basis would snap them to the same angle.
                FixedJoint::new(pair.0, pair.1)
                    .with_anchor(anchor)
                    .with_basis(Rotation::IDENTITY),
                VineJoint {
                    a: pair.0,
                    b: pair.1,
                },
            ));
            commands.entity(pair.0).insert(Vined);
            commands.entity(pair.1).insert(Vined);
        }
    }
}
