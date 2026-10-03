//! The core loop: towers, bricks the players steer down, the hand-off to the
//! physics engine when a brick touches the tower, and the match rules.
//!
//! This module has no rendering or keyboard code, so it runs headless in tests
//! and mods can drive it with their own input, bots or replays. Every tower
//! (yours and each AI opponent's) runs through exactly the same systems; the
//! only difference is who writes its [`PlayerInput`].

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::config::{GameConfig, ModeKind, SpellKind};

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Match>()
            .init_resource::<MatchRequest>()
            .init_resource::<crate::spells::SpellLog>()
            .add_systems(Startup, setup_world)
            .add_systems(
                FixedUpdate,
                (
                    start_match,
                    spawn_next_brick,
                    control_active_brick,
                    track_towers,
                    crate::spells::grant_spells,
                    crate::spells::cast_spells,
                    judge_match,
                )
                    .chain()
                    .in_set(GameplaySystems),
            )
            .add_systems(Update, pause_physics_outside_play);
    }
}

/// System set for the fixed-step gameplay systems, so mods can order around it.
/// Anything that writes [`PlayerInput`] in `FixedUpdate` should run before it.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameplaySystems;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Waiting in the menu; physics is paused.
    Menu,
    Playing,
    /// Someone won or everyone is out. Bricks keep simulating.
    Over,
}

/// State of the whole match.
#[derive(Resource, Debug, Clone)]
pub struct Match {
    pub phase: Phase,
    pub mode: ModeKind,
    /// Index of the winning tower, if anyone won.
    pub winner: Option<usize>,
}

impl Default for Match {
    fn default() -> Self {
        Self {
            phase: Phase::Menu,
            mode: ModeKind::Race,
            winner: None,
        }
    }
}

/// Set `pending` to start (or restart) a match with the current
/// [`GameConfig::mode`] and [`GameConfig::opponents`].
#[derive(Resource, Debug, Default)]
pub struct MatchRequest {
    pub pending: bool,
}

/// One tower: a platform and the bricks stacked on it.
#[derive(Component, Debug, Clone)]
pub struct Tower {
    pub index: usize,
    /// World x of the platform centre.
    pub origin_x: f32,
    pub human: bool,
}

/// Marks towers driven by the computer.
#[derive(Component, Debug, Default)]
pub struct AiControlled;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TowerStatus {
    Building,
    Won,
    /// Out of lives, or failed the puzzle.
    Out,
}

/// Changes applied to a tower's next brick, set by spells.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BrickMod {
    pub scale: f32,
    pub friction: Option<f32>,
    pub tint: Option<Color>,
}

impl Default for BrickMod {
    fn default() -> Self {
        Self {
            scale: 1.0,
            friction: None,
            tint: None,
        }
    }
}

/// Score and progress of one tower.
#[derive(Component, Debug, Clone)]
pub struct TowerState {
    pub status: TowerStatus,
    pub bricks_placed: u32,
    pub bricks_dropped: u32,
    /// Remaining lives, for modes that have them.
    pub lives: Option<u32>,
    /// Height of the settled tower above the platform, in cells.
    pub height: f32,
    /// Top of any brick above this platform, settled or not, in cells. New
    /// bricks spawn above this so they never appear inside a falling one.
    pub stack_top: f32,
    /// True when no brick of this tower is moving.
    pub all_settled: bool,
    /// Seconds the tower has stayed past the race finish line.
    pub hold_timer: f32,
    /// Seconds a finished puzzle has rested below the line.
    pub settle_timer: f32,
    /// The brick being steered, if any.
    pub active: Option<Entity>,
    /// The spell waiting to be cast. One at a time, like the original.
    pub spell: Option<SpellKind>,
    /// Bricks placed towards the next spell.
    pub spell_progress: u32,
    pub next_mod: BrickMod,
    pub rng: u64,
}

impl TowerState {
    pub fn new(config: &GameConfig, seed: u64) -> Self {
        Self {
            status: TowerStatus::Building,
            bricks_placed: 0,
            bricks_dropped: 0,
            lives: (config.mode == ModeKind::Survival).then_some(config.survival.lives),
            height: 0.0,
            stack_top: 0.0,
            all_settled: true,
            hold_timer: 0.0,
            settle_timer: 0.0,
            active: None,
            spell: None,
            spell_progress: 0,
            next_mod: BrickMod::default(),
            rng: seed.max(1),
        }
    }

    pub fn random(&mut self) -> u64 {
        xorshift(&mut self.rng)
    }

    fn lose_brick(&mut self) {
        self.bricks_dropped += 1;
        if let Some(lives) = self.lives.as_mut() {
            *lives = lives.saturating_sub(1);
        }
    }
}

pub fn xorshift(state: &mut u64) -> u64 {
    // xorshift64*
    *state ^= *state >> 12;
    *state ^= *state << 25;
    *state ^= *state >> 27;
    state.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

/// Player intent for the current fixed step. Lives on each tower and is
/// written by the keyboard (your tower), the AI (opponents), or a test.
#[derive(Component, Default, Debug, Clone)]
pub struct PlayerInput {
    pub left_held: bool,
    pub right_held: bool,
    pub fast_fall_held: bool,
    /// One-shot presses, cleared once the gameplay step has used them.
    pub left_pressed: bool,
    pub right_pressed: bool,
    pub rotate_pressed: bool,
    pub cast_pressed: bool,
}

/// Seeded 7-bag randomizer: every piece appears once per bag. All towers use
/// the same seed, so everyone gets the same bricks in the same order.
#[derive(Component, Debug, Clone)]
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

    fn refill(&mut self) {
        let mut bag: Vec<usize> = (0..self.piece_count).collect();
        for i in (1..bag.len()).rev() {
            let j = (xorshift(&mut self.state) % (i as u64 + 1)) as usize;
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

/// Any brick in play, controlled or not.
#[derive(Component, Debug, Clone)]
pub struct Brick {
    pub tower: Entity,
    pub piece: usize,
    /// Cell centres in cell units, relative to the brick's origin, before `scale`.
    pub cells: Vec<Vec2>,
    pub color: Color,
    pub scale: f32,
}

/// The brick a player is currently steering.
#[derive(Component, Debug)]
pub struct ActiveBrick {
    /// Slightly smaller than the brick, used for "can I move here?" checks so
    /// a brick can slide into a gap exactly its own width.
    pub query_shape: Collider,
    /// How far the brick would fall before touching something (for the drop guide).
    pub drop_distance: Option<f32>,
    repeat_timer: f32,
    repeating: bool,
}

#[derive(Component, Debug)]
pub struct Platform {
    pub tower: Entity,
}

/// Builds a brick collider from its cells. `fill` is the fraction of each cell covered.
pub fn brick_collider(cells: &[Vec2], cell: f32, fill: f32) -> Collider {
    Collider::compound(
        cells
            .iter()
            .map(|&p| {
                (
                    p * cell,
                    Rotation::IDENTITY,
                    Collider::rectangle(cell * fill, cell * fill),
                )
            })
            .collect(),
    )
}

fn setup_world(
    mut commands: Commands,
    config: Res<GameConfig>,
    mut game: ResMut<Match>,
    mut request: ResMut<MatchRequest>,
) {
    commands.insert_resource(Gravity(Vec2::NEG_Y * config.gravity * config.cell_size));
    game.mode = config.mode;
    if config.start_in_menu {
        game.phase = Phase::Menu;
    } else {
        request.pending = true;
    }
}

fn pause_physics_outside_play(game: Res<Match>, mut time: ResMut<Time<Physics>>) {
    let paused = game.phase == Phase::Menu;
    if paused != time.is_paused() {
        if paused {
            time.pause();
        } else {
            time.unpause();
        }
    }
}

/// Clears the field and builds a tower for you and one per opponent.
#[allow(clippy::type_complexity)]
fn start_match(
    mut commands: Commands,
    mut request: ResMut<MatchRequest>,
    mut game: ResMut<Match>,
    config: Res<GameConfig>,
    mut log: ResMut<crate::spells::SpellLog>,
    old: Query<
        Entity,
        Or<(
            With<Tower>,
            With<Brick>,
            With<Platform>,
            With<crate::spells::VineJoint>,
        )>,
    >,
) {
    if !std::mem::take(&mut request.pending) {
        return;
    }
    for entity in &old {
        commands.entity(entity).despawn();
    }

    let count = 1 + config.opponents as usize;
    let c = config.cell_size;
    for index in 0..count {
        let origin_x =
            config.cells(config.tower_spacing) * (index as f32 - (count - 1) as f32 * 0.5);
        let human = index == 0;
        let mut tower = commands.spawn((
            Tower {
                index,
                origin_x,
                human,
            },
            TowerState::new(&config, config.seed.wrapping_add(index as u64 * 7919)),
            BrickBag::new(config.seed, config.pieces.len()),
            PlayerInput::default(),
        ));
        if !human {
            tower.insert(AiControlled);
        }
        let tower = tower.id();

        // The platform's top surface sits at y = 0.
        commands.spawn((
            Platform { tower },
            RigidBody::Static,
            Collider::rectangle(config.cells(config.platform_width), c),
            Friction::new(config.brick_friction),
            Transform::from_xyz(origin_x, -c * 0.5, 0.0),
        ));
    }

    log.0.clear();
    *game = Match {
        phase: Phase::Playing,
        mode: config.mode,
        winner: None,
    };
}

fn spawn_next_brick(
    mut commands: Commands,
    config: Res<GameConfig>,
    game: Res<Match>,
    spatial: SpatialQuery,
    mut towers: Query<(Entity, &Tower, &mut TowerState, &mut BrickBag)>,
) {
    if game.phase != Phase::Playing {
        return;
    }
    let c = config.cell_size;
    for (tower_entity, tower, mut state, mut bag) in &mut towers {
        if state.status != TowerStatus::Building || state.active.is_some() {
            continue;
        }
        if game.mode == ModeKind::Puzzle
            && state.bricks_placed + state.bricks_dropped >= config.puzzle.bricks
        {
            continue;
        }
        let piece = bag.draw();
        let def = &config.pieces[piece];
        let modifier = std::mem::take(&mut state.next_mod);
        let cells = def.centered_cells();
        let cell = c * modifier.scale;
        let query_shape = brick_collider(&cells, cell, config.collider_scale - 0.04);

        let floor = match game.mode {
            // Puzzle bricks come from above the laser line.
            ModeKind::Puzzle => state.stack_top.max(config.puzzle.line_height),
            _ => state.stack_top,
        };
        let mut position = Vec2::new(tower.origin_x, config.cells(floor + config.spawn_gap));
        // Bricks flying around can still be in the way; spawn above them.
        for _ in 0..32 {
            let hits = spatial.shape_intersections(
                &query_shape,
                position,
                0.0,
                &SpatialQueryFilter::default(),
            );
            if hits.is_empty() {
                break;
            }
            position.y += c;
        }

        let color = modifier
            .tint
            .unwrap_or(Color::srgb(def.color[0], def.color[1], def.color[2]));
        let brick = commands
            .spawn((
                Brick {
                    tower: tower_entity,
                    piece,
                    cells: cells.clone(),
                    color,
                    scale: modifier.scale,
                },
                ActiveBrick {
                    query_shape,
                    drop_distance: None,
                    repeat_timer: 0.0,
                    repeating: false,
                },
                RigidBody::Kinematic,
                brick_collider(&cells, cell, config.collider_scale),
                Friction::new(modifier.friction.unwrap_or(config.brick_friction)),
                Restitution::new(config.brick_restitution),
                ColliderDensity(config.brick_density),
                Transform::from_translation(position.extend(0.0)),
                Position(position),
                Rotation::IDENTITY,
            ))
            .id();
        state.active = Some(brick);
    }
}

/// Moves each controlled brick: nudges, rotation and falling. Movement is
/// checked against the world with shape queries rather than simulated, which
/// gives the crisp grid-like control of the original while the brick is held.
#[allow(clippy::type_complexity)]
fn control_active_brick(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    game: Res<Match>,
    spatial: SpatialQuery,
    mut towers: Query<(&Tower, &mut TowerState, &mut PlayerInput)>,
    // Position and Rotation are read here and written back through commands,
    // because SpatialQuery already reads them for every collider.
    mut bricks: Query<(&mut ActiveBrick, &Position, &Rotation)>,
) {
    let dt = time.delta_secs();
    let c = config.cell_size;
    for (tower, mut state, mut input) in &mut towers {
        let left_pressed = std::mem::take(&mut input.left_pressed);
        let right_pressed = std::mem::take(&mut input.right_pressed);
        let rotate_pressed = std::mem::take(&mut input.rotate_pressed);

        if game.phase != Phase::Playing || state.status != TowerStatus::Building {
            continue;
        }
        let Some(entity) = state.active else { continue };
        let Ok((mut control, position, rotation)) = bricks.get_mut(entity) else {
            state.active = None;
            continue;
        };
        let (mut position, mut rotation) = (*position, *rotation);
        let shape = control.query_shape.clone();
        let filter = SpatialQueryFilter::from_excluded_entities([entity]);
        let fits = |pos: Vec2, rot: Rotation| {
            spatial
                .shape_intersections(&shape, pos, rot.as_radians(), &filter)
                .is_empty()
        };
        let max_offset = config.cells(config.platform_width * 0.5 + 6.0);

        // Horizontal nudge: once on press, then repeating while held.
        let dir =
            (input.right_held || right_pressed) as i32 - (input.left_held || left_pressed) as i32;
        let mut nudge = left_pressed || right_pressed;
        if dir == 0 || nudge {
            control.repeat_timer = 0.0;
            control.repeating = false;
        } else {
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
        }
        if nudge && dir != 0 {
            let target = position.0 + Vec2::X * dir as f32 * config.cells(config.nudge_step);
            if (target.x - tower.origin_x).abs() <= max_offset && fits(target, rotation) {
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
        let multiplier = if input.fast_fall_held {
            config.fast_fall_multiplier
        } else {
            1.0
        };
        let speed = config.fall_speed * c * multiplier;
        let distance = speed * dt;
        let cast = |from: Vec2, max: f32| {
            spatial.cast_shape(
                &shape,
                from,
                rotation.as_radians(),
                Dir2::NEG_Y,
                &ShapeCastConfig::from_max_distance(max),
                &filter,
            )
        };
        match cast(position.0, distance) {
            Some(hit) => {
                position.0.y -= hit.distance;
                // Release the brick to the physics engine, keeping its momentum.
                commands.entity(entity).remove::<ActiveBrick>().insert((
                    position,
                    rotation,
                    RigidBody::Dynamic,
                    LinearVelocity(Vec2::NEG_Y * speed.min(c * 4.0)),
                ));
                state.active = None;
                state.bricks_placed += 1;
                state.spell_progress += 1;
                continue;
            }
            None => position.0.y -= distance,
        }

        // Steered off the side and past the kill line: it counts as dropped.
        if position.0.y < -config.cells(config.kill_depth) {
            commands.entity(entity).despawn();
            state.active = None;
            state.lose_brick();
            continue;
        }

        control.drop_distance = cast(position.0, config.cells(200.0)).map(|hit| hit.distance);
        commands.entity(entity).insert((position, rotation));
    }
}

/// Measures each settled tower and removes bricks that fell off.
#[allow(clippy::type_complexity)]
fn track_towers(
    mut commands: Commands,
    config: Res<GameConfig>,
    mut towers: Query<(&Tower, &mut TowerState)>,
    bricks: Query<
        (Entity, &Brick, &Position, &LinearVelocity, &ColliderAabb),
        Without<ActiveBrick>,
    >,
    joints: Query<(Entity, &crate::spells::VineJoint)>,
) {
    let kill_y = -config.cells(config.kill_depth);
    let settled_speed = config.cells(0.5);
    let mut tops: Vec<(Entity, f32, f32, bool)> = Vec::new();

    for (_, mut state) in &mut towers {
        state.height = 0.0;
        state.stack_top = 0.0;
        state.all_settled = true;
    }
    for (entity, brick, position, velocity, aabb) in &bricks {
        let Ok((tower, mut state)) = towers.get_mut(brick.tower) else {
            continue;
        };
        if position.0.y < kill_y {
            commands.entity(entity).despawn();
            for (joint, vine) in &joints {
                if vine.a == entity || vine.b == entity {
                    commands.entity(joint).despawn();
                }
            }
            state.lose_brick();
            continue;
        }
        let on_tower = (position.0.x - tower.origin_x).abs()
            <= config.cells(config.platform_width * 0.5 + 2.0);
        if !on_tower || aabb.min.y <= -config.cell_size {
            continue;
        }
        let settled = velocity.0.length() < settled_speed;
        tops.push((
            brick.tower,
            aabb.max.y,
            if settled { aabb.max.y } else { 0.0 },
            settled,
        ));
    }
    for (tower, any_top, settled_top, settled) in tops {
        if let Ok((_, mut state)) = towers.get_mut(tower) {
            state.stack_top = state.stack_top.max(any_top / config.cell_size);
            state.height = state.height.max(settled_top / config.cell_size);
            state.all_settled &= settled;
        }
    }
}

fn judge_match(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut game: ResMut<Match>,
    mut towers: Query<(&Tower, &mut TowerState)>,
) {
    if game.phase != Phase::Playing {
        return;
    }
    let dt = time.delta_secs();
    for (_, mut state) in &mut towers {
        if state.status != TowerStatus::Building {
            continue;
        }
        match game.mode {
            ModeKind::Race => {
                if state.height >= config.race.finish_height {
                    state.hold_timer += dt;
                    if state.hold_timer >= config.race.hold_seconds {
                        state.status = TowerStatus::Won;
                    }
                } else {
                    state.hold_timer = 0.0;
                }
            }
            ModeKind::Survival => {
                if state.lives == Some(0) {
                    state.status = TowerStatus::Out;
                } else if state.bricks_placed >= config.survival.bricks_to_place {
                    state.status = TowerStatus::Won;
                }
            }
            ModeKind::Puzzle => {
                let rules = &config.puzzle;
                if state.bricks_dropped > 0
                    || (state.all_settled && state.height > rules.line_height + 0.05)
                {
                    state.status = TowerStatus::Out;
                } else if state.bricks_placed >= rules.bricks
                    && state.active.is_none()
                    && state.all_settled
                {
                    state.settle_timer += dt;
                    if state.settle_timer >= rules.settle_seconds {
                        state.status = TowerStatus::Won;
                    }
                } else {
                    state.settle_timer = 0.0;
                }
            }
        }
    }

    let mut winner = None;
    let mut building = Vec::new();
    let mut human_out = false;
    let mut count = 0;
    for (tower, state) in &towers {
        count += 1;
        match state.status {
            TowerStatus::Won => {
                winner = Some(winner.map_or(tower.index, |w: usize| w.min(tower.index)))
            }
            TowerStatus::Building => building.push(tower.index),
            TowerStatus::Out => human_out |= tower.human,
        }
    }
    if winner.is_none() && count > 1 && building.len() == 1 && game.mode != ModeKind::Puzzle {
        // Last tower standing.
        winner = Some(building[0]);
    }
    if winner.is_some() || building.is_empty() || human_out {
        game.phase = Phase::Over;
        game.winner = winner;
    }
}
