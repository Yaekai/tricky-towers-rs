//! The core loop: spawn a brick, let the player steer it down, hand it over to
//! the physics engine when it touches the tower, and judge the round.
//!
//! This module has no rendering or keyboard code, so it runs headless in tests
//! and mods can drive it with their own input or AI.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::config::{GameConfig, ModeConfig};

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerInput>()
            .add_systems(Startup, setup_world)
            .add_systems(
                FixedUpdate,
                (
                    restart_round,
                    spawn_next_brick,
                    control_active_brick,
                    track_tower,
                    judge_round,
                )
                    .chain()
                    .in_set(GameplaySystems),
            );
    }
}

/// System set for the fixed-step gameplay systems, so mods can order around it.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameplaySystems;

/// Player intent for the current fixed step. Written by the input layer (or a
/// test, bot, or replay) and consumed by [`control_active_brick`].
#[derive(Resource, Default, Debug, Clone)]
pub struct PlayerInput {
    pub left_held: bool,
    pub right_held: bool,
    pub fast_fall_held: bool,
    /// One-shot presses, cleared once the gameplay step has used them.
    pub left_pressed: bool,
    pub right_pressed: bool,
    pub rotate_pressed: bool,
    pub restart_pressed: bool,
}

/// Any brick in play, controlled or not.
#[derive(Component, Debug, Clone)]
pub struct Brick {
    pub piece: usize,
    /// Cell centres in cell units, relative to the brick's origin.
    pub cells: Vec<Vec2>,
    pub color: Color,
}

/// The brick the player is currently steering.
#[derive(Component, Debug, Default)]
pub struct ActiveBrick {
    repeat_timer: f32,
    repeating: bool,
}

#[derive(Component)]
pub struct Platform;

/// Per-piece colliders, built once from the config.
#[derive(Resource)]
pub struct PieceColliders {
    /// Used by the physics simulation once a brick is released.
    pub physics: Vec<Collider>,
    /// Slightly smaller, used for the "can I move here?" checks while steering,
    /// so a brick can slide into a gap exactly its own width.
    pub query: Vec<Collider>,
}

/// Seeded 7-bag randomizer: every piece appears once per bag.
#[derive(Resource, Debug, Clone)]
pub struct BrickBag {
    state: u64,
    bag: Vec<usize>,
    piece_count: usize,
}

impl BrickBag {
    pub fn new(seed: u64, piece_count: usize) -> Self {
        Self {
            state: seed.max(1),
            bag: Vec::new(),
            piece_count,
        }
    }

    fn next_u64(&mut self) -> u64 {
        // xorshift64*
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        self.state.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn refill(&mut self) {
        let mut bag: Vec<usize> = (0..self.piece_count).collect();
        for i in (1..bag.len()).rev() {
            let j = (self.next_u64() % (i as u64 + 1)) as usize;
            bag.swap(i, j);
        }
        // Drawn from the back, so reverse to keep draw order = shuffled order.
        bag.reverse();
        self.bag.splice(0..0, bag);
    }

    pub fn peek(&mut self) -> usize {
        if self.bag.is_empty() {
            self.refill();
        }
        *self.bag.last().unwrap()
    }

    pub fn draw(&mut self) -> usize {
        let piece = self.peek();
        self.bag.pop();
        piece
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Playing,
    Won,
    Lost,
}

/// Score and state of the current round.
#[derive(Resource, Debug, Clone)]
pub struct Round {
    pub phase: Phase,
    pub bricks_placed: u32,
    pub bricks_dropped: u32,
    /// Remaining lives, for modes that have them.
    pub lives: Option<u32>,
    /// Height of the settled tower above the platform, in cells.
    pub tower_height: f32,
    /// Top of any brick above the platform, settled or not, in cells. New
    /// bricks spawn above this so they never appear inside a falling one.
    pub stack_top: f32,
    /// Seconds the tower has stayed above the race finish line.
    pub hold_timer: f32,
}

impl Round {
    pub fn new(config: &GameConfig) -> Self {
        let lives = match config.mode {
            ModeConfig::Survival { lives, .. } => Some(lives),
            ModeConfig::Race { .. } => None,
        };
        Self {
            phase: Phase::Playing,
            bricks_placed: 0,
            bricks_dropped: 0,
            lives,
            tower_height: 0.0,
            stack_top: 0.0,
            hold_timer: 0.0,
        }
    }
}

fn setup_world(mut commands: Commands, config: Res<GameConfig>) {
    let c = config.cell_size;
    commands.insert_resource(Gravity(Vec2::NEG_Y * config.gravity * c));

    let shrink = config.collider_scale;
    let build = |cells: &[Vec2], scale: f32| {
        Collider::compound(
            cells
                .iter()
                .map(|&p| (p * c, Rotation::IDENTITY, Collider::rectangle(c * scale, c * scale)))
                .collect(),
        )
    };
    let shapes: Vec<Vec<Vec2>> = config.pieces.iter().map(|p| p.centered_cells()).collect();
    commands.insert_resource(PieceColliders {
        physics: shapes.iter().map(|s| build(s, shrink)).collect(),
        query: shapes.iter().map(|s| build(s, shrink - 0.04)).collect(),
    });
    commands.insert_resource(BrickBag::new(config.seed, config.pieces.len()));
    commands.insert_resource(Round::new(&config));

    // The platform's top surface sits at y = 0.
    let width = config.cells(config.platform_width);
    commands.spawn((
        Platform,
        RigidBody::Static,
        Collider::rectangle(width, c),
        Friction::new(config.brick_friction),
        Transform::from_xyz(0.0, -c * 0.5, 0.0),
    ));
}

fn restart_round(
    mut commands: Commands,
    mut input: ResMut<PlayerInput>,
    config: Res<GameConfig>,
    bricks: Query<Entity, With<Brick>>,
) {
    if !std::mem::take(&mut input.restart_pressed) {
        return;
    }
    for entity in &bricks {
        commands.entity(entity).despawn();
    }
    commands.insert_resource(Round::new(&config));
    commands.insert_resource(BrickBag::new(config.seed, config.pieces.len()));
}

fn spawn_next_brick(
    mut commands: Commands,
    config: Res<GameConfig>,
    colliders: Res<PieceColliders>,
    round: Res<Round>,
    mut bag: ResMut<BrickBag>,
    spatial: SpatialQuery,
    active: Query<(), With<ActiveBrick>>,
) {
    if round.phase != Phase::Playing || !active.is_empty() {
        return;
    }
    let piece = bag.draw();
    let def = &config.pieces[piece];
    let mut position = Vec2::new(0.0, config.cells(round.stack_top + config.spawn_gap));
    // Bricks flying around can still be in the way; spawn above them.
    let filter = SpatialQueryFilter::default();
    for _ in 0..32 {
        if spatial.shape_intersections(&colliders.query[piece], position, 0.0, &filter).is_empty() {
            break;
        }
        position.y += config.cell_size;
    }
    commands.spawn((
        Brick {
            piece,
            cells: def.centered_cells(),
            color: Color::srgb(def.color[0], def.color[1], def.color[2]),
        },
        ActiveBrick::default(),
        RigidBody::Kinematic,
        colliders.physics[piece].clone(),
        Friction::new(config.brick_friction),
        Restitution::new(config.brick_restitution),
        ColliderDensity(config.brick_density),
        Transform::from_translation(position.extend(0.0)),
        Position(position),
        Rotation::IDENTITY,
    ));
}

/// Moves the controlled brick: nudges, rotation and falling. Movement is
/// checked against the world with shape queries rather than simulated, which
/// gives the crisp grid-like control of the original while the brick is held.
fn control_active_brick(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    colliders: Res<PieceColliders>,
    mut input: ResMut<PlayerInput>,
    spatial: SpatialQuery,
    mut round: ResMut<Round>,
    // Position and Rotation are read here and written back through commands,
    // because SpatialQuery already reads them for every collider.
    mut active: Query<(Entity, &Brick, &mut ActiveBrick, &Position, &Rotation)>,
) {
    let left_pressed = std::mem::take(&mut input.left_pressed);
    let right_pressed = std::mem::take(&mut input.right_pressed);
    let rotate_pressed = std::mem::take(&mut input.rotate_pressed);

    let Ok((entity, brick, mut control, position, rotation)) = active.single_mut() else {
        return;
    };
    let (mut position, mut rotation) = (*position, *rotation);
    if round.phase != Phase::Playing {
        return;
    }
    let dt = time.delta_secs();
    let c = config.cell_size;
    let shape = &colliders.query[brick.piece];
    let filter = SpatialQueryFilter::from_excluded_entities([entity]);
    let fits = |pos: Vec2, rot: Rotation| {
        spatial
            .shape_intersections(shape, pos, rot.as_radians(), &filter)
            .is_empty()
    };
    let max_x = config.cells(config.platform_width * 0.5 + 6.0);

    // Horizontal nudge: once on press, then repeating while held.
    let dir = (input.right_held || right_pressed) as i32 - (input.left_held || left_pressed) as i32;
    let mut nudge = left_pressed || right_pressed;
    if dir == 0 {
        control.repeat_timer = 0.0;
        control.repeating = false;
    } else if !nudge {
        control.repeat_timer += dt;
        let wait = if control.repeating {
            config.nudge_repeat_interval
        } else {
            config.nudge_repeat_delay
        };
        if control.repeat_timer >= wait {
            control.repeat_timer = 0.0;
            control.repeating = true;
            nudge = true;
        }
    } else {
        control.repeat_timer = 0.0;
        control.repeating = false;
    }
    if nudge && dir != 0 {
        let target = position.0 + Vec2::X * dir as f32 * config.cells(config.nudge_step);
        if target.x.abs() <= max_x && fits(target, rotation) {
            position.0 = target;
        }
    }

    // Rotation by 90 degrees, with small sideways kicks if the spot is taken.
    if rotate_pressed {
        let turned = rotation * Rotation::degrees(-90.0);
        let step = config.cells(config.nudge_step);
        for kick in [0.0, step, -step, 2.0 * step, -2.0 * step] {
            let target = position.0 + Vec2::X * kick;
            if fits(target, turned) {
                position.0 = target;
                rotation = turned;
                break;
            }
        }
    }

    // Fall, stopping at the first thing underneath.
    let speed = config.fall_speed
        * c
        * if input.fast_fall_held {
            config.fast_fall_multiplier
        } else {
            1.0
        };
    let distance = speed * dt;
    let hit = spatial.cast_shape(
        shape,
        position.0,
        rotation.as_radians(),
        Dir2::NEG_Y,
        &ShapeCastConfig::from_max_distance(distance),
        &filter,
    );
    match hit {
        Some(hit) => {
            position.0.y -= hit.distance;
            commands.entity(entity).insert((position, rotation));
            // Release the brick to the physics engine, keeping its momentum.
            commands
                .entity(entity)
                .remove::<ActiveBrick>()
                .insert((RigidBody::Dynamic, LinearVelocity(Vec2::NEG_Y * speed.min(c * 4.0))));
            round.bricks_placed += 1;
        }
        None => {
            position.0.y -= distance;
            commands.entity(entity).insert((position, rotation));
        }
    }

    // Steered off the side and past the kill line: it counts as dropped.
    if position.0.y < -config.cells(config.kill_depth) {
        commands.entity(entity).despawn();
        lose_brick(&mut round);
    }
}

fn lose_brick(round: &mut Round) {
    round.bricks_dropped += 1;
    if let Some(lives) = round.lives.as_mut() {
        *lives = lives.saturating_sub(1);
    }
}

/// Measures the settled tower and removes bricks that fell off.
fn track_tower(
    mut commands: Commands,
    config: Res<GameConfig>,
    mut round: ResMut<Round>,
    bricks: Query<(Entity, &Position, &LinearVelocity, &ColliderAabb), (With<Brick>, Without<ActiveBrick>)>,
) {
    let kill_y = -config.cells(config.kill_depth);
    let settled_speed = config.cells(0.5);
    let mut top: f32 = 0.0;
    let mut stack_top: f32 = 0.0;
    for (entity, position, velocity, aabb) in &bricks {
        if position.0.y < kill_y {
            commands.entity(entity).despawn();
            lose_brick(&mut round);
            continue;
        }
        // Bricks only count once they rest, and only if they are on the tower
        // rather than tumbling past the platform edge.
        let on_tower = position.0.x.abs() <= config.cells(config.platform_width * 0.5 + 2.0);
        if on_tower {
            stack_top = stack_top.max(aabb.max.y);
        }
        if velocity.0.length() < settled_speed && on_tower && aabb.min.y > -config.cell_size {
            top = top.max(aabb.max.y);
        }
    }
    round.tower_height = top / config.cell_size;
    round.stack_top = stack_top / config.cell_size;
}

fn judge_round(time: Res<Time>, config: Res<GameConfig>, mut round: ResMut<Round>) {
    if round.phase != Phase::Playing {
        return;
    }
    match config.mode {
        ModeConfig::Race {
            finish_height,
            hold_seconds,
        } => {
            if round.tower_height >= finish_height {
                round.hold_timer += time.delta_secs();
                if round.hold_timer >= hold_seconds {
                    round.phase = Phase::Won;
                }
            } else {
                round.hold_timer = 0.0;
            }
        }
        ModeConfig::Survival { bricks_to_place, .. } => {
            if round.lives == Some(0) {
                round.phase = Phase::Lost;
            } else if round.bricks_placed >= bricks_to_place {
                round.phase = Phase::Won;
            }
        }
    }
}
