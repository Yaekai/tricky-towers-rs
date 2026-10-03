//! Spells, as the original awards them. Each tower has a spell line above it;
//! touching it collects a pair, one light spell and one dark curse, and casting
//! either half throws the other away. Light spells help the caster; dark ones
//! curse every other player. Rubber-banding rules in `MagicRules` give players
//! who are behind stronger spells. See `docs/research/spell-awards.md` and
//! `docs/research/spells.md`.

use avian2d::dynamics::solver::islands::{BodyIslandNode, WakeBody};
use avian2d::prelude::*;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;

use crate::config::{AwardRule, Curse, GameConfig, MagicRules, ModeKind, SpellKind};
use crate::gameplay::{
    ActiveBrick, Brick, BrickMod, Match, Phase, PlayerInput, Tower, TowerState, TowerStatus,
    brick_body, brick_collider, piece_def,
};

/// A vine holding two bricks together.
#[derive(Component, Debug)]
pub struct VineJoint {
    pub a: Entity,
    pub b: Entity,
}

/// A brick turned to stone: static, like the ground.
#[derive(Component, Debug)]
pub struct Stone;

/// A brick wrapped in vines.
#[derive(Component, Debug)]
pub struct Vined;

/// A placed Ivy brick waiting to be vined to what it touches.
#[derive(Component, Debug)]
pub struct IvyPending;

/// A brick being zapped away by Undo.
#[derive(Component, Debug)]
pub struct Zapped {
    pub timer: f32,
}

/// A brick lifted by Bubble. It doesn't count toward the tower's height.
#[derive(Component, Debug)]
pub struct Bubbled {
    pub age: f32,
    /// What it touched when the bubble formed; anything new pops it.
    touching: Option<Vec<Entity>>,
}

/// A brick carried off by Ghost; it doesn't count toward the tower's height.
#[derive(Component, Debug)]
pub struct Ghosted {
    pub timer: f32,
    /// Where the skulls carry it to.
    pub to: Vec2,
}

/// A brick covered in grass (looks only).
#[derive(Component, Debug)]
pub struct Grassy;

/// Extra Life's heart orb. Touch it with one of your bricks for a life.
#[derive(Component, Debug)]
pub struct LifeOrb {
    pub tower: Entity,
}

/// One spell cast, for the HUD, visuals, sound or mods.
#[derive(Debug, Clone)]
pub struct SpellCast {
    pub spell: SpellKind,
    pub caster: usize,
    pub target: usize,
}

/// The last few spells cast, newest last, and how many were cast in all.
#[derive(Resource, Debug, Default)]
pub struct SpellLog(pub Vec<SpellCast>, pub u64);

/// Places each tower's spell line, slides it down in a race, and hands out a
/// spell pair when the tower reaches it. Also counts down Mist.
pub(crate) fn update_spell_lines(
    time: Res<Time>,
    config: Res<GameConfig>,
    game: Res<Match>,
    mut towers: Query<&mut TowerState>,
) {
    let dt = time.delta_secs();
    for mut state in &mut towers {
        state.mist_timer = (state.mist_timer - dt).max(0.0);
    }
    let magic = &config.magic;
    if !magic.enabled || game.phase != Phase::Playing || game.mode == ModeKind::Puzzle {
        return;
    }
    let spacing = magic.spacing(game.players);
    let building: Vec<f32> = towers
        .iter()
        .filter(|s| s.status == TowerStatus::Building)
        .map(|s| s.height)
        .collect();
    let opponents = building.len() > 1;
    let leader = building.iter().copied().fold(0.0, f32::max);

    for mut state in &mut towers {
        if state.status != TowerStatus::Building {
            continue;
        }
        let height = state.height;
        let Some(mut line) = state.spell_line else {
            state.spell_line = Some(state.next_line.max(height + spacing));
            state.line_wait = magic.wait(game.players);
            continue;
        };
        if game.mode == ModeKind::Race {
            let above = line - height;
            if above > magic.line_rush_beyond || state.line_wait <= 0.0 {
                let speed = magic.line_slide_speed
                    + (above - magic.line_catch_up_beyond).max(0.0) * magic.line_catch_up_rate;
                line = (line - speed * dt).max(height);
            } else {
                state.line_wait -= dt;
            }
            state.spell_line = Some(line);
        }
        if height < line {
            continue;
        }
        // Collected: the next line is due one spacing higher.
        state.spell_line = None;
        state.next_line = line + spacing;
        state.lines_reached += 1;
        let finish_gap = (game.mode == ModeKind::Race).then(|| game.finish_height - height);
        let behind = leader - height;
        let hurt = state.lives.is_some_and(|l| l < config.survival.lives);
        let Some(rule) = magic
            .award_rules
            .iter()
            .find(|r| rule_holds(r, game.mode, hurt, finish_gap, behind))
        else {
            continue;
        };
        award(&mut state, magic, rule, opponents);
    }
}

fn rule_holds(
    rule: &AwardRule,
    mode: ModeKind,
    hurt: bool,
    finish_gap: Option<f32>,
    behind: f32,
) -> bool {
    rule.mode.is_none_or(|m| m == mode)
        && rule.hurt.is_none_or(|h| h == hurt)
        && rule
            .finish_within
            .is_none_or(|w| finish_gap.is_some_and(|gap| gap <= w))
        && rule.behind_leader.is_none_or(|b| behind >= b)
}

/// Fills both slots from `rule`. A new pair replaces an unused one.
fn award(state: &mut TowerState, magic: &MagicRules, rule: &AwardRule, opponents: bool) {
    let has_lives = state.lives.is_some();
    let mut light: Vec<(SpellKind, u32)> = rule
        .light
        .iter()
        .copied()
        .filter(|(s, w)| *w > 0 && (has_lives || *s != SpellKind::ExtraLife))
        .collect();
    // The same light spell never comes twice in a row.
    if light.len() > 1 {
        light.retain(|(s, _)| Some(*s) != state.last_light);
    }
    let Some(spell) = pick_weighted(state, &light, |(_, w)| *w).map(|(s, _)| s) else {
        return;
    };
    state.light_spell = Some(spell);
    state.last_light = Some(spell);

    let set = rule.dark.clone().or_else(|| {
        rule.dark_after
            .iter()
            .find(|(s, _)| *s == spell)
            .map(|(_, name)| name.clone())
    });
    state.dark_spell = None;
    if !opponents {
        return;
    }
    if let Some(set) = set.and_then(|name| magic.dark_set(&name)) {
        state.dark_spell = pick_weighted(state, &set.curses, |c| c.weight);
    }
}

fn pick_weighted<T: Clone>(
    state: &mut TowerState,
    items: &[T],
    weight: impl Fn(&T) -> u32,
) -> Option<T> {
    let total: u64 = items.iter().map(|i| weight(i) as u64).sum();
    if total == 0 {
        return None;
    }
    let mut roll = state.random() % total;
    for item in items {
        let w = weight(item) as u64;
        if roll < w {
            return Some(item.clone());
        }
        roll -= w;
    }
    None
}

enum Cast {
    Light(SpellKind),
    Dark(Curse),
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(crate) fn cast_spells(
    mut commands: Commands,
    config: Res<GameConfig>,
    game: Res<Match>,
    mut log: ResMut<SpellLog>,
    mut towers: Query<(Entity, &Tower, &mut TowerState, &mut PlayerInput)>,
    mut active: Query<(&mut ActiveBrick, &mut Brick)>,
    placed: Query<
        (
            Entity,
            &Brick,
            &Position,
            &RigidBody,
            Has<Stone>,
            Has<Vined>,
        ),
        Without<ActiveBrick>,
    >,
) {
    let mut casts = Vec::new();
    for (entity, tower, mut state, mut input) in &mut towers {
        let light = std::mem::take(&mut input.cast_light_pressed);
        let dark = std::mem::take(&mut input.cast_dark_pressed);
        if game.phase != Phase::Playing || state.status != TowerStatus::Building {
            continue;
        }
        // Using one half of the pair throws the other away.
        if light && let Some(spell) = state.light_spell.take() {
            state.dark_spell = None;
            casts.push((entity, tower.index, Cast::Light(spell)));
        } else if dark && let Some(curse) = state.dark_spell.take() {
            state.light_spell = None;
            casts.push((entity, tower.index, Cast::Dark(curse)));
        }
    }

    let magic = &config.magic;
    for (caster, caster_index, cast) in casts {
        let (spells, targets, bricks): (Vec<SpellKind>, Vec<(Entity, usize)>, Option<u32>) =
            match cast {
                Cast::Light(spell) => (vec![spell], vec![(caster, caster_index)], None),
                Cast::Dark(curse) => (
                    curse.spells,
                    towers
                        .iter()
                        .filter(|(e, _, s, _)| *e != caster && s.status == TowerStatus::Building)
                        .map(|(e, t, _, _)| (e, t.index))
                        .collect(),
                    curse.bricks,
                ),
            };
        for (target, target_index) in targets {
            let Ok((_, tower, mut state, _)) = towers.get_mut(target) else {
                continue;
            };
            for &spell in &spells {
                let count = bricks.unwrap_or(match spell {
                    SpellKind::Fast => magic.fast_bricks,
                    SpellKind::Ice => magic.ice_bricks,
                    SpellKind::AutoBuild => magic.auto_build_bricks,
                    _ => 1,
                });
                if let Some(effect) = brick_effect(spell, magic) {
                    if spell == SpellKind::AutoBuild {
                        // The coming bricks, not the one in hand.
                        for i in 0..count as usize {
                            state.queue_mod(i, &effect);
                        }
                    } else {
                        curse_bricks(
                            &mut commands,
                            &config,
                            &mut state,
                            &mut active,
                            &effect,
                            count,
                        );
                    }
                } else {
                    cast_on_tower(
                        &mut commands,
                        &config,
                        spell,
                        target,
                        tower,
                        &mut state,
                        &placed,
                    );
                }
                log.0.push(SpellCast {
                    spell,
                    caster: caster_index,
                    target: target_index,
                });
                log.1 += 1;
            }
        }
    }
    let excess = log.0.len().saturating_sub(5);
    log.0.drain(..excess);
}

/// What a spell does to a brick, for spells that change bricks.
fn brick_effect(spell: SpellKind, magic: &MagicRules) -> Option<BrickMod> {
    let mut m = BrickMod::default();
    match spell {
        SpellKind::Ivy => m.ivy = true,
        SpellKind::Petrify => m.petrify = true,
        SpellKind::AutoBuild => m.auto_build = true,
        SpellKind::Large => m.large = 1,
        SpellKind::Fast => m.fast = magic.fast_extra_speed,
        SpellKind::NoRotate => m.no_rotate = true,
        SpellKind::AutoRotate => m.auto_rotate = Some(magic.auto_rotate_interval),
        SpellKind::Ice => {
            m.friction = Some(magic.ice_friction);
            m.tint = Some(Color::srgb(0.75, 0.92, 1.0));
        }
        SpellKind::Slow => m.slow = true,
        SpellKind::Mystery => m.mystery = true,
        _ => return None,
    }
    Some(m)
}

/// Puts `effect` on the brick in hand (if any) and the bricks after it, `count` in all.
fn curse_bricks(
    commands: &mut Commands,
    config: &GameConfig,
    state: &mut TowerState,
    active: &mut Query<(&mut ActiveBrick, &mut Brick)>,
    effect: &BrickMod,
    count: u32,
) {
    let mut remaining = count as usize;
    if remaining == 0 {
        return;
    }
    if let Some(entity) = state.active
        && let Ok((mut control, mut brick)) = active.get_mut(entity)
    {
        control.mods.merge(effect);
        if effect.large > 0 {
            brick.scale = control.mods.scale(config);
            control.reshape = true;
        }
        if effect.mystery
            && let Some(shape) = state.draw_mystery(config.magic.mystery_pieces.len())
        {
            brick.piece = config.pieces.len() + shape;
            let def = piece_def(config, brick.piece);
            brick.cells = def.centered_cells();
            brick.color = Color::srgb(def.color[0], def.color[1], def.color[2]);
            brick.mass = def.mass;
            control.reshape = true;
        }
        if let Some(tint) = effect.tint {
            brick.color = tint;
        }
        if let Some(friction) = effect.friction {
            commands.entity(entity).insert(Friction::new(friction));
        }
        remaining -= 1;
    }
    for i in 0..remaining {
        state.queue_mod(i, effect);
    }
}

/// Spells that act on the tower rather than on a brick.
#[allow(clippy::type_complexity)]
fn cast_on_tower(
    commands: &mut Commands,
    config: &GameConfig,
    spell: SpellKind,
    target: Entity,
    tower: &Tower,
    state: &mut TowerState,
    placed: &Query<
        (
            Entity,
            &Brick,
            &Position,
            &RigidBody,
            Has<Stone>,
            Has<Vined>,
        ),
        Without<ActiveBrick>,
    >,
) {
    let magic = &config.magic;
    let c = config.cell_size;
    let last = state
        .last_placed
        .and_then(|e| placed.get(e).ok())
        .filter(|b| b.1.tower == target);
    match spell {
        SpellKind::Undo => {
            if let Some((entity, ..)) = last {
                commands.entity(entity).insert(Zapped {
                    timer: magic.undo_seconds,
                });
                state.last_placed = None;
                state.bricks_placed = state.bricks_placed.saturating_sub(1);
            }
        }
        SpellKind::ExtraLife => {
            let y = state.height.max(magic.life_orb_floor) + magic.life_orb_lift;
            commands.spawn((
                LifeOrb { tower: target },
                Transform::from_xyz(
                    tower.origin_x + config.cells(magic.life_orb_x),
                    config.cells(y),
                    0.0,
                ),
            ));
        }
        SpellKind::Minibase => {
            let (lo, hi) = magic.minibase_offset;
            let t = (state.random() % 1000) as f32 / 1000.0;
            let side = if state.random().is_multiple_of(2) {
                1.0
            } else {
                -1.0
            };
            let offset = ((lo + (hi - lo) * t) * 2.0).round() * 0.5 * side;
            let y = state.height.max(magic.minibase_floor) + magic.minibase_lift;
            let width = magic.minibase_width.max(1.0).round() as i32;
            let cells: Vec<Vec2> = (0..width)
                .map(|i| Vec2::new(i as f32 - (width - 1) as f32 * 0.5, 0.0))
                .collect();
            let ledge = Brick {
                tower: target,
                piece: 0,
                cells,
                color: Color::srgb(0.5, 0.5, 0.52),
                scale: 1.0,
                mass: 1.0,
            };
            let (collider, _) = brick_body(config, &ledge);
            let position = Vec2::new(tower.origin_x + config.cells(offset), config.cells(y));
            commands.spawn((
                ledge,
                Stone,
                RigidBody::Static,
                collider,
                Friction::new(config.brick_friction),
                Transform::from_translation(position.extend(0.0)),
                Position(position),
                Rotation::IDENTITY,
            ));
        }
        SpellKind::Mist => state.mist_timer = magic.mist_seconds,
        SpellKind::Bubble => {
            // Only a loose brick can float off.
            if let Some((entity, _, _, RigidBody::Dynamic, false, false)) = last {
                let tilt = (3.0 + (state.random() % 900) as f32 / 100.0).to_radians();
                let sign = if state.random().is_multiple_of(2) {
                    1.0
                } else {
                    -1.0
                };
                commands.entity(entity).insert((
                    Bubbled {
                        age: 0.0,
                        touching: None,
                    },
                    GravityScale(1.0),
                    SleepingDisabled,
                    CollidingEntities::default(),
                    AngularVelocity(tilt * sign),
                ));
                commands.queue_silenced(WakeBody(entity));
            }
        }
        SpellKind::Ghost => {
            // Only a loose brick can be carried off.
            if let Some((entity, _, at, RigidBody::Dynamic, false, false)) = last {
                let outward = if at.0.x >= tower.origin_x { 1.0 } else { -1.0 };
                let to = at.0
                    + Vec2::new(
                        outward * config.cells(magic.ghost_carry),
                        config.cells(magic.ghost_lift),
                    );
                commands.entity(entity).insert((
                    Ghosted {
                        timer: magic.ghost_seconds,
                        to,
                    },
                    RigidBody::Kinematic,
                    LinearVelocity::ZERO,
                    AngularVelocity::ZERO,
                ));
                state.last_placed = None;
            }
        }
        SpellKind::Grassy => {
            if let Some((_, _, centre, ..)) = last {
                let reach = config.cells(magic.grassy_radius);
                for (entity, brick, position, ..) in placed {
                    if brick.tower == target && position.0.distance(centre.0) <= reach {
                        commands.entity(entity).insert(Grassy);
                    }
                }
            }
        }
        _ => {}
    }
    let _ = c;
}

/// Rebuilds the shape of steered bricks a spell made larger or turned into a
/// Mystery shape, lifting them clear if they grew into something.
pub(crate) fn reshape_active_bricks(
    mut commands: Commands,
    config: Res<GameConfig>,
    spatial: SpatialQuery,
    mut bricks: Query<(Entity, &mut ActiveBrick, &Brick, &Position, &Rotation)>,
) {
    for (entity, mut control, brick, position, rotation) in &mut bricks {
        if !std::mem::take(&mut control.reshape) {
            continue;
        }
        let shape = crate::gameplay::steer_shape(&config, brick);
        let (_, density) = brick_body(&config, brick);
        let filter = SpatialQueryFilter::from_excluded_entities([entity]);
        let mut at = position.0;
        for _ in 0..64 {
            if spatial
                .shape_intersections(&shape, at, rotation.as_radians(), &filter)
                .is_empty()
            {
                break;
            }
            at.y += config.cell_size * 0.5;
        }
        control.query_shape = shape.clone();
        commands
            .entity(entity)
            .insert((shape, density, Position(at)));
    }
}

/// Vines a freshly placed Ivy brick to every brick of its tower it touches.
/// Touching a stone brick petrifies the whole cluster instead.
#[allow(clippy::type_complexity)]
pub(crate) fn grow_ivy(
    mut commands: Commands,
    config: Res<GameConfig>,
    spatial: SpatialQuery,
    bricks: Query<
        (
            Entity,
            &Brick,
            &Position,
            &Rotation,
            &RigidBody,
            Has<Stone>,
            Has<BodyIslandNode>,
            Has<IvyPending>,
        ),
        Without<ActiveBrick>,
    >,
    joints: Query<(Entity, &VineJoint)>,
) {
    let mut joined: HashSet<(Entity, Entity)> = joints
        .iter()
        .map(|(_, v)| (v.a.min(v.b), v.a.max(v.b)))
        .collect();
    let mut cut: HashSet<Entity> = HashSet::default();
    for (entity, brick, position, rotation, body, stone, in_island, pending) in &bricks {
        if !pending {
            continue;
        }
        if *body == RigidBody::Static || stone {
            commands.entity(entity).remove::<IvyPending>();
            continue;
        }
        // Avian can only join bodies already in its island graph.
        if !in_island {
            continue;
        }
        commands.entity(entity).remove::<IvyPending>();
        let cell = config.cell_size * brick.scale;
        let reach = brick_collider(
            &brick.cells,
            cell,
            config.collider_scale + 2.0 * config.magic.ivy_reach / brick.scale,
        );
        let filter = SpatialQueryFilter::from_excluded_entities([entity]);
        let touching: Vec<_> = spatial
            .shape_intersections(&reach, position.0, rotation.as_radians(), &filter)
            .into_iter()
            .filter_map(|e| bricks.get(e).ok())
            .filter(|b| b.1.tower == brick.tower)
            .collect();
        if touching.iter().any(|b| b.5) {
            // Part of the cluster is stone, so all of it turns to stone.
            let mut cluster = vec![entity];
            let mut seen: HashSet<Entity> = cluster.iter().copied().collect();
            while let Some(e) = cluster.pop() {
                for (joint, vine) in &joints {
                    let other = if vine.a == e {
                        vine.b
                    } else if vine.b == e {
                        vine.a
                    } else {
                        continue;
                    };
                    if cut.insert(joint) {
                        commands.entity(joint).despawn();
                    }
                    if seen.insert(other) {
                        cluster.push(other);
                    }
                }
            }
            for e in seen {
                commands.entity(e).insert((
                    RigidBody::Static,
                    Stone,
                    LinearVelocity::ZERO,
                    AngularVelocity::ZERO,
                ));
            }
            continue;
        }
        for (other, _, other_position, _, _, _, other_in_island, _) in touching {
            if !other_in_island {
                continue;
            }
            let pair = (entity.min(other), entity.max(other));
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

/// Despawns every vine on `brick`, once each.
pub(crate) fn cut_vines(
    commands: &mut Commands,
    joints: &Query<(Entity, &VineJoint)>,
    cut: &mut HashSet<Entity>,
    brick: Entity,
) {
    for (joint, vine) in joints {
        if (vine.a == brick || vine.b == brick) && cut.insert(joint) {
            commands.entity(joint).despawn();
        }
    }
}

/// Removes bricks once Undo's zap is over.
pub(crate) fn zap_bricks(
    mut commands: Commands,
    time: Res<Time>,
    mut zapped: Query<(Entity, &mut Zapped)>,
    joints: Query<(Entity, &VineJoint)>,
) {
    let mut cut = HashSet::default();
    for (entity, mut zap) in &mut zapped {
        zap.timer -= time.delta_secs();
        if zap.timer <= 0.0 {
            // Joints go before the bodies they hold.
            cut_vines(&mut commands, &joints, &mut cut, entity);
            commands.entity(entity).despawn();
        }
    }
}

/// Lifts bubbled bricks, easing the pull, and pops the bubble when time is up
/// or another brick hits it.
pub(crate) fn float_bubbles(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    mut bubbles: Query<(
        Entity,
        &mut Bubbled,
        &mut GravityScale,
        Option<&CollidingEntities>,
    )>,
) {
    let magic = &config.magic;
    let (start, eased, rest) = magic.bubble_lift;
    for (entity, mut bubble, mut gravity, colliding) in &mut bubbles {
        bubble.age += time.delta_secs();
        let now: Vec<Entity> = colliding
            .map(|c| c.iter().copied().collect())
            .unwrap_or_default();
        let touching = bubble.touching.get_or_insert_with(|| now.clone());
        let hit = now.iter().any(|e| !touching.contains(e));
        if bubble.age >= magic.bubble_seconds || hit {
            commands
                .entity(entity)
                .remove::<(Bubbled, SleepingDisabled)>()
                .insert(GravityScale(1.0));
            continue;
        }
        let lift = if bubble.age < magic.bubble_ease {
            start + (eased - start) * bubble.age / magic.bubble_ease.max(f32::EPSILON)
        } else {
            rest
        };
        gravity.0 = 1.0 - lift / config.gravity.max(f32::EPSILON);
    }
}

/// Carries Ghost's brick to its spot, then lets it fall.
pub(crate) fn carry_ghosts(
    mut commands: Commands,
    time: Res<Time>,
    mut ghosts: Query<(Entity, &mut Ghosted, &Position, &mut LinearVelocity)>,
) {
    let dt = time.delta_secs();
    for (entity, mut ghost, position, mut velocity) in &mut ghosts {
        ghost.timer -= dt;
        if ghost.timer <= 0.0 {
            commands
                .entity(entity)
                .remove::<Ghosted>()
                .insert(RigidBody::Dynamic);
            velocity.0 = Vec2::ZERO;
            continue;
        }
        // Ease toward the spot over the first second or so.
        velocity.0 = (ghost.to - position.0) * 2.0;
    }
}

/// Gives a life to the tower whose brick touches its heart orb.
pub(crate) fn collect_life_orbs(
    mut commands: Commands,
    config: Res<GameConfig>,
    spatial: SpatialQuery,
    orbs: Query<(Entity, &LifeOrb, &Transform)>,
    bricks: Query<&Brick>,
    mut towers: Query<&mut TowerState>,
) {
    let orb = Collider::circle(config.cell_size * 0.5);
    for (entity, life, transform) in &orbs {
        let hits = spatial.shape_intersections(
            &orb,
            transform.translation.truncate(),
            0.0,
            &SpatialQueryFilter::default(),
        );
        let touched = hits
            .iter()
            .any(|e| bricks.get(*e).is_ok_and(|b| b.tower == life.tower));
        if touched {
            if let Ok(mut state) = towers.get_mut(life.tower)
                && let Some(lives) = state.lives.as_mut()
            {
                *lives += 1;
            }
            commands.entity(entity).despawn();
        }
    }
}
