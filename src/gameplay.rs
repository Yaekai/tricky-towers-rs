//! The core loop: towers, bricks the players steer down, the hand-off to the
//! physics engine when a brick touches the tower, and the match rules.
//!
//! This module has no rendering or keyboard code, so it runs headless in tests
//! and mods can drive it with their own input, bots or replays. Every tower
//! (yours and each AI opponent's) runs through exactly the same systems; the
//! only difference is who writes its [`PlayerInput`].

use avian2d::dynamics::solver::islands::WakeBody;
use avian2d::prelude::*;
use bevy::prelude::*;

use crate::config::{Curse, GameConfig, ModeKind, PieceDef, SpellKind};

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_hz(50.0))
            .init_resource::<Match>()
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
                    crate::spells::update_spell_lines,
                    crate::spells::cast_spells,
                    (
                        crate::spells::reshape_active_bricks,
                        crate::spells::grow_ivy,
                        crate::spells::zap_bricks,
                        crate::spells::float_bubbles,
                        crate::spells::carry_ghosts,
                        crate::spells::collect_life_orbs,
                    )
                        .chain(),
                    judge_match,
                )
                    .chain()
                    .in_set(GameplaySystems),
            )
            .add_systems(Update, pause_physics_outside_play);
    }
}

/// Most towers in one match, as in the original.
pub const MAX_TOWERS: usize = 4;

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
    /// Seconds since the match started.
    pub elapsed: f32,
    /// Current race finish height, in cells; it comes down late in a race.
    pub finish_height: f32,
    /// Towers in the match.
    pub players: usize,
    /// Matches started so far, so each round can differ.
    pub round: u64,
}

impl Default for Match {
    fn default() -> Self {
        Self {
            phase: Phase::Menu,
            mode: ModeKind::Race,
            winner: None,
            elapsed: 0.0,
            finish_height: 0.0,
            players: 1,
            round: 0,
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
    /// Which human player steers it (0 = player 1), if any.
    pub player: Option<usize>,
}

impl Tower {
    /// Player 1's tower: the keyboard's, and the one the HUD follows.
    pub fn is_first_player(&self) -> bool {
        self.player == Some(0)
    }
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

/// Spell effects on one brick. They stack: two Large curses make it larger.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BrickMod {
    /// Large curses on this brick.
    pub large: u32,
    /// Friction override (Ice).
    pub friction: Option<f32>,
    pub tint: Option<Color>,
    /// Falls at least this much faster than the drop speed, in cells/s (Fast).
    pub fast: f32,
    /// Caught by a balloon (Slow).
    pub slow: bool,
    /// The player cannot rotate this brick.
    pub no_rotate: bool,
    /// The brick turns by itself every this many seconds; manual turning is off.
    pub auto_rotate: Option<f32>,
    /// Becomes one of the Mystery shapes.
    pub mystery: bool,
    /// Turns to stone when it lands (Petrify).
    pub petrify: bool,
    /// Vines itself to what it lands on (Ivy).
    pub ivy: bool,
    /// Drops and petrifies by itself (Auto Build).
    pub auto_build: bool,
}

impl BrickMod {
    pub fn merge(&mut self, other: &BrickMod) {
        self.large += other.large;
        self.friction = match (self.friction, other.friction) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
        self.tint = other.tint.or(self.tint);
        self.fast = self.fast.max(other.fast);
        self.slow |= other.slow;
        self.no_rotate |= other.no_rotate;
        self.auto_rotate = other.auto_rotate.or(self.auto_rotate);
        self.mystery |= other.mystery;
        self.petrify |= other.petrify;
        self.ivy |= other.ivy;
        self.auto_build |= other.auto_build;
    }

    /// Size multiplier from Large curses.
    pub fn scale(&self, config: &GameConfig) -> f32 {
        match self.large {
            0 => 1.0,
            n => config.magic.large_scale + config.magic.large_stack * (n - 1) as f32,
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
    /// The light half of the collected spell pair. Casting either half
    /// throws the other away, like the original.
    pub light_spell: Option<SpellKind>,
    /// The dark half of the pair (one or more curses).
    pub dark_spell: Option<Curse>,
    /// Height of this tower's spell line, in cells, while one is out.
    pub spell_line: Option<f32>,
    /// Where the next spell line is due, in cells.
    pub next_line: f32,
    /// Race: seconds before the spell line starts sliding down.
    pub line_wait: f32,
    /// Spell lines collected so far.
    pub lines_reached: u32,
    /// The light spell collected last; it never comes twice in a row.
    pub last_light: Option<SpellKind>,
    /// The brick placed most recently (for Undo, Bubble and Grassy).
    pub last_placed: Option<Entity>,
    /// Seconds of Mist left over this tower.
    pub mist_timer: f32,
    /// Seconds before a dropped brick can cost another life.
    pub invincible: f32,
    /// This round's puzzle laser height, before dropped bricks lower it.
    pub laser_base: f32,
    /// Spell effects waiting for the coming bricks, next brick first.
    pub next_mods: Vec<BrickMod>,
    /// Mystery shapes still in the bag.
    pub mystery_bag: Vec<usize>,
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
            // Puzzle has no spell lines, only a spell to start with.
            light_spell: (config.mode == ModeKind::Puzzle && config.magic.enabled)
                .then_some(config.puzzle.start_spell)
                .flatten(),
            dark_spell: None,
            spell_line: None,
            next_line: 0.0,
            line_wait: 0.0,
            lines_reached: 0,
            last_light: None,
            last_placed: None,
            mist_timer: 0.0,
            invincible: 0.0,
            laser_base: config.puzzle.line_height,
            next_mods: Vec::new(),
            mystery_bag: Vec::new(),
            rng: seed.max(1),
        }
    }

    pub fn random(&mut self) -> u64 {
        xorshift(&mut self.rng)
    }

    /// Adds `effect` to the coming bricks, `index` 0 being the next one.
    pub fn queue_mod(&mut self, index: usize, effect: &BrickMod) {
        if self.next_mods.len() <= index {
            self.next_mods.resize(index + 1, BrickMod::default());
        }
        self.next_mods[index].merge(effect);
    }

    /// Draws a Mystery shape from the bag.
    pub fn draw_mystery(&mut self, shapes: usize) -> Option<usize> {
        if shapes == 0 {
            return None;
        }
        if self.mystery_bag.is_empty() {
            self.mystery_bag = (0..shapes).collect();
        }
        let pick = (self.random() % self.mystery_bag.len() as u64) as usize;
        Some(self.mystery_bag.swap_remove(pick))
    }

    /// A brick fell off. In Survival it costs a life, then a moment of grace.
    fn lose_brick(&mut self, config: &GameConfig) {
        self.bricks_dropped += 1;
        if self.invincible > 0.0 {
            return;
        }
        if let Some(lives) = self.lives.as_mut() {
            *lives = lives.saturating_sub(1);
            self.invincible = config.survival.invincible_seconds;
        }
    }

    /// Bricks used so far, placed or dropped.
    pub fn bricks_used(&self) -> u32 {
        self.bricks_placed + self.bricks_dropped
    }

    /// Puzzle laser height for this tower: each dropped brick brings it lower.
    pub fn laser(&self, config: &GameConfig) -> f32 {
        self.laser_base - config.puzzle.drop_penalty * self.bricks_dropped as f32
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
    /// Nudge: a one-cell shove, its own button.
    pub nudge_left_pressed: bool,
    pub nudge_right_pressed: bool,
    /// Rotate clockwise.
    pub rotate_pressed: bool,
    /// Rotate counter-clockwise.
    pub rotate_ccw_pressed: bool,
    pub cast_light_pressed: bool,
    pub cast_dark_pressed: bool,
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
    /// Mass relative to a normal brick.
    pub mass: f32,
    /// Iced by the Ice curse (slippery, drawn as ice).
    pub ice: bool,
}

/// The brick a player is currently steering.
#[derive(Component, Debug)]
pub struct ActiveBrick {
    /// The brick's shape, shrunk by `steer_inset`, for "can I move here?" checks.
    pub query_shape: Collider,
    /// How far the brick would fall before touching something (for the drop guide).
    pub drop_distance: Option<f32>,
    /// Spell effects on this brick.
    pub mods: BrickMod,
    /// Set when a spell changed the brick's shape or size while it was steered.
    pub reshape: bool,
    repeat_timer: f32,
    repeating: bool,
    /// Current fall speed, in cells per second.
    speed: f32,
    auto_rotate_timer: f32,
    /// Seconds before the balloon allows another sideways step.
    side_cooldown: f32,
}

impl ActiveBrick {
    fn new(query_shape: Collider, mods: BrickMod) -> Self {
        Self {
            query_shape,
            drop_distance: None,
            mods,
            reshape: false,
            repeat_timer: 0.0,
            repeating: false,
            speed: 0.0,
            auto_rotate_timer: 0.0,
            side_cooldown: 0.0,
        }
    }
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

/// The physics parts of a brick: its collider, and the density that gives it
/// `brick.mass` whatever its size.
pub fn brick_body(config: &GameConfig, brick: &Brick) -> (Collider, ColliderDensity) {
    let cell = config.cell_size * brick.scale;
    let fill = config.collider_scale;
    let area = brick.cells.len() as f32 * (cell * fill).powi(2);
    (
        brick_collider(&brick.cells, cell, fill),
        ColliderDensity(config.brick_mass * brick.mass / area.max(f32::EPSILON)),
    )
}

/// The shrunk shape a steered brick uses.
pub fn steer_shape(config: &GameConfig, brick: &Brick) -> Collider {
    let fill = (1.0 - 2.0 * config.steer_inset).clamp(0.1, 1.0);
    brick_collider(&brick.cells, config.cell_size * brick.scale, fill)
}

/// Shape and colour of a piece, by index into the normal pieces and then the
/// Mystery shapes.
pub fn piece_def(config: &GameConfig, piece: usize) -> &PieceDef {
    let normal = config.pieces.len();
    if piece < normal {
        &config.pieces[piece]
    } else {
        &config.magic.mystery_pieces[(piece - normal) % config.magic.mystery_pieces.len()]
    }
}

fn setup_world(
    mut commands: Commands,
    config: Res<GameConfig>,
    mut game: ResMut<Match>,
    mut request: ResMut<MatchRequest>,
) {
    commands.insert_resource(Gravity(Vec2::NEG_Y * config.gravity * config.cell_size));
    commands.insert_resource(Time::<Fixed>::from_hz(config.physics_hz));
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
            With<crate::spells::LifeOrb>,
        )>,
    >,
    joints: Query<(Entity, &crate::spells::VineJoint)>,
) {
    if !std::mem::take(&mut request.pending) {
        return;
    }
    // Joints go before the bricks they hold; avian's island bookkeeping
    // breaks if a jointed body disappears first.
    for (entity, _) in &joints {
        commands.entity(entity).despawn();
    }
    for entity in &old {
        commands.entity(entity).despawn();
    }

    // The puzzle laser is the same for everyone, and new each round.
    let round = game.round + 1;
    let mut roll = config.seed ^ round.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let laser = config.puzzle.line_height
        + (xorshift(&mut roll) % 1001) as f32 / 1000.0 * config.puzzle.line_random;
    let humans = config.players.clamp(1, MAX_TOWERS as u32) as usize;
    let count = (humans + config.opponents as usize).min(MAX_TOWERS);
    let c = config.cell_size;
    for index in 0..count {
        let origin_x =
            config.cells(config.tower_spacing) * (index as f32 - (count - 1) as f32 * 0.5);
        // Human players first, then the AI wizards.
        let human = index < humans;
        let mut tower = commands.spawn((
            Tower {
                index,
                origin_x,
                human,
                player: human.then_some(index),
            },
            TowerState {
                laser_base: laser,
                ..TowerState::new(&config, config.seed.wrapping_add(index as u64 * 7919))
            },
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
        elapsed: 0.0,
        finish_height: config.race.finish_height,
        players: count,
        round,
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
        let limit = match game.mode {
            ModeKind::Puzzle => Some(config.puzzle.bricks),
            ModeKind::Survival => Some(config.survival.bricks_to_place),
            ModeKind::Race => None,
        };
        if limit.is_some_and(|n| state.bricks_used() >= n) {
            continue;
        }
        let modifier = if state.next_mods.is_empty() {
            BrickMod::default()
        } else {
            state.next_mods.remove(0)
        };
        let mut piece = bag.draw();
        if modifier.mystery
            && let Some(shape) = state.draw_mystery(config.magic.mystery_pieces.len())
        {
            piece = config.pieces.len() + shape;
        }
        let def = piece_def(&config, piece);
        let brick = Brick {
            tower: tower_entity,
            piece,
            cells: def.centered_cells(),
            color: modifier
                .tint
                .unwrap_or(Color::srgb(def.color[0], def.color[1], def.color[2])),
            scale: modifier.scale(&config),
            mass: def.mass,
            ice: modifier.friction.is_some(),
        };
        let query_shape = steer_shape(&config, &brick);
        let (body, density) = brick_body(&config, &brick);
        let friction = Friction::new(modifier.friction.unwrap_or(config.brick_friction));

        if modifier.auto_build {
            // Auto Build: straight down onto a random spot over the tower, as stone.
            let half = config.cells(config.platform_width * 0.5);
            let x = tower.origin_x + ((state.random() % 1000) as f32 / 1000.0 * 2.0 - 1.0) * half;
            let from = Vec2::new(x, config.cells(state.stack_top + config.spawn_gap));
            let Some(hit) = spatial.cast_shape(
                &query_shape,
                from,
                0.0,
                Dir2::NEG_Y,
                &ShapeCastConfig::from_max_distance(config.cells(200.0)),
                &SpatialQueryFilter::default(),
            ) else {
                continue;
            };
            let position = from - Vec2::Y * hit.distance;
            let entity = commands
                .spawn((
                    brick,
                    RigidBody::Static,
                    crate::spells::Stone,
                    body,
                    density,
                    friction,
                    Restitution::new(config.brick_restitution),
                    Transform::from_translation(position.extend(0.0)),
                    Position(position),
                    Rotation::IDENTITY,
                ))
                .id();
            state.bricks_placed += 1;
            state.last_placed = Some(entity);
            continue;
        }

        let floor = match game.mode {
            // Puzzle bricks come from above the laser line.
            ModeKind::Puzzle => state.stack_top.max(state.laser(&config)),
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

        let entity = commands
            .spawn((
                brick,
                ActiveBrick::new(query_shape.clone(), modifier),
                RigidBody::Kinematic,
                // While steered, the brick is its shrunk self.
                query_shape,
                density,
                friction,
                Restitution::new(config.brick_restitution),
                AngularDamping(config.angular_damping),
                Transform::from_translation(position.extend(0.0)),
                Position(position),
                Rotation::IDENTITY,
            ))
            .id();
        state.active = Some(entity);
    }
}

/// A push to give a brick once it is placed, in cells per second.
struct Shove {
    entity: Entity,
    velocity: Vec2,
}

/// Moves each controlled brick: moves, nudges, rotation and falling. The
/// brick is moved directly and checked against the world with shape queries,
/// like the original; the first contact places it.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn control_active_brick(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    game: Res<Match>,
    spatial: SpatialQuery,
    mut towers: Query<(&Tower, &mut TowerState, &mut PlayerInput)>,
    // Position and Rotation are read here and written back through commands,
    // because SpatialQuery already reads them for every collider.
    mut bricks: Query<(&mut ActiveBrick, &Brick, &Position, &Rotation)>,
    mut velocities: Query<(&RigidBody, &mut LinearVelocity), Without<ActiveBrick>>,
) {
    let dt = time.delta_secs();
    let c = config.cell_size;
    // The drop speed rises with the tallest tower and with each player out.
    let (highest, out) = towers.iter().fold((0.0f32, 0u32), |(h, o), (_, s, _)| {
        (h.max(s.height), o + (s.status == TowerStatus::Out) as u32)
    });
    let drop_speed = config.drop_speed()
        + highest * config.fall_speed_per_height
        + out as f32 * config.fall_speed_per_out;

    for (tower, mut state, mut input) in &mut towers {
        let left_pressed = std::mem::take(&mut input.left_pressed);
        let right_pressed = std::mem::take(&mut input.right_pressed);
        let nudge_left = std::mem::take(&mut input.nudge_left_pressed);
        let nudge_right = std::mem::take(&mut input.nudge_right_pressed);
        let rotate_cw = std::mem::take(&mut input.rotate_pressed);
        let rotate_ccw = std::mem::take(&mut input.rotate_ccw_pressed);

        if game.phase != Phase::Playing || state.status != TowerStatus::Building {
            continue;
        }
        let Some(entity) = state.active else { continue };
        let Ok((mut control, brick, position, rotation)) = bricks.get_mut(entity) else {
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
        let cast = |from: Vec2, rot: Rotation, dir: Dir2, max: f32| {
            spatial.cast_shape(
                &shape,
                from,
                rot.as_radians(),
                dir,
                &ShapeCastConfig::from_max_distance(max),
                &filter,
            )
        };
        let in_lane = |x: f32| (x - tower.origin_x).abs() <= config.cells(config.lane_half_width);
        let mut placed = false;
        let mut shoves: Vec<Shove> = Vec::new();

        // Moving: one step on press, then repeating while held.
        let dir =
            (input.right_held || right_pressed) as i32 - (input.left_held || left_pressed) as i32;
        let mut step = left_pressed || right_pressed;
        if dir == 0 || step {
            control.repeat_timer = 0.0;
            control.repeating = false;
        } else {
            control.repeat_timer += dt;
            let wait = if control.repeating {
                config.move_repeat_interval
            } else {
                config.move_repeat_delay
            };
            if control.repeat_timer >= wait {
                control.repeat_timer = 0.0;
                control.repeating = true;
                step = true;
            }
        }
        // The balloon only lets the brick drift sideways slowly.
        control.side_cooldown = (control.side_cooldown - dt).max(0.0);
        if control.mods.slow && step {
            if control.side_cooldown > 0.0 {
                step = false;
            } else {
                control.side_cooldown =
                    config.move_step / config.magic.slow_side_speed.max(f32::EPSILON);
            }
        }
        if step && dir != 0 {
            let sign = dir as f32;
            let want = config.cells(config.move_step);
            let along = if dir > 0 { Dir2::X } else { Dir2::NEG_X };
            if in_lane(position.0.x + sign * want) {
                match cast(position.0, rotation, along, want) {
                    None => position.0.x += sign * want,
                    Some(hit) => {
                        position.0.x += sign * hit.distance;
                        // Blocked by more than a little: bump the brick in the
                        // way, and this one is placed.
                        if want - hit.distance > config.cells(config.bump_threshold) {
                            placed = true;
                            let push = Vec2::X * sign;
                            shoves.push(Shove {
                                entity,
                                velocity: push * config.bump_impulse,
                            });
                            shoves.push(Shove {
                                entity: hit.entity,
                                velocity: push * config.bump_impulse_other,
                            });
                        }
                    }
                }
            }
        }

        // Nudge: a full cell at once; hitting a brick shoves both and places this one.
        let nudge = nudge_right as i32 - nudge_left as i32;
        if !placed && nudge != 0 {
            let sign = nudge as f32;
            let want = config.cells(config.nudge_distance);
            let along = if nudge > 0 { Dir2::X } else { Dir2::NEG_X };
            if in_lane(position.0.x + sign * want) {
                match cast(position.0, rotation, along, want) {
                    None => position.0.x += sign * want,
                    Some(hit) => {
                        position.0.x += sign * hit.distance;
                        placed = true;
                        for target in [entity, hit.entity] {
                            shoves.push(Shove {
                                entity: target,
                                velocity: Vec2::X * sign * config.nudge_impulse,
                            });
                        }
                    }
                }
            }
        }

        // Rotation: 90 degrees, tried in steps. No wall kicks: if the turn is
        // blocked part way, the brick stops there and is placed.
        let mut turns: Vec<f32> = Vec::new();
        if !control.mods.no_rotate && control.mods.auto_rotate.is_none() {
            if rotate_cw {
                turns.push(-90.0);
            }
            if rotate_ccw {
                turns.push(90.0);
            }
        }
        if let Some(interval) = control.mods.auto_rotate {
            control.auto_rotate_timer += dt;
            if control.auto_rotate_timer >= interval {
                control.auto_rotate_timer = 0.0;
                turns.push(90.0);
            }
        }
        for turn in turns {
            if placed {
                break;
            }
            let steps = config.rotation_steps;
            let mut reached = rotation;
            let mut blocked = false;
            for k in 1..=steps {
                let next = rotation * Rotation::degrees(turn * k as f32 / steps as f32);
                if !fits(position.0, next) {
                    blocked = true;
                    break;
                }
                reached = next;
            }
            rotation = reached;
            placed = blocked;
        }

        // Falling: holding down accelerates, letting go slows back to the drop speed.
        let base = drop_speed;
        control.speed = if control.mods.slow {
            if input.fast_fall_held {
                config.magic.slow_fast_fall_speed
            } else {
                config.magic.slow_fall_speed
            }
        } else if input.fast_fall_held {
            (control.speed.max(base) + config.drop_acceleration * dt).min(config.drop_max_speed)
        } else {
            (control.speed - config.drop_deceleration * dt).max(base)
        };
        if control.mods.fast > 0.0 && !control.mods.slow {
            control.speed = control.speed.max(base + control.mods.fast);
        }
        if !placed {
            let distance = config.cells(control.speed) * dt;
            match cast(position.0, rotation, Dir2::NEG_Y, distance) {
                Some(hit) => {
                    position.0.y -= hit.distance;
                    placed = true;
                }
                None => position.0.y -= distance,
            }
        }

        if placed {
            // Hand the brick to the physics engine, full size and at rest.
            let (body, _) = brick_body(&config, brick);
            let mut placed_brick = commands.entity(entity);
            placed_brick.remove::<ActiveBrick>().insert((
                position,
                rotation,
                body,
                LinearVelocity::ZERO,
            ));
            if control.mods.petrify {
                placed_brick.insert((RigidBody::Static, crate::spells::Stone));
            } else {
                placed_brick.insert(RigidBody::Dynamic);
            }
            if control.mods.ivy {
                placed_brick.insert(crate::spells::IvyPending);
            }
            for shove in shoves {
                let velocity = shove.velocity * c;
                if shove.entity == entity {
                    if !control.mods.petrify {
                        commands.entity(entity).insert(LinearVelocity(velocity));
                    }
                } else if let Ok((RigidBody::Dynamic, mut v)) = velocities.get_mut(shove.entity) {
                    v.0 += velocity;
                    commands.queue_silenced(WakeBody(shove.entity));
                }
            }
            state.active = None;
            state.bricks_placed += 1;
            state.last_placed = Some(entity);
            continue;
        }

        // Steered off the side and past the kill line: it counts as dropped.
        if position.0.y < -config.cells(config.kill_depth) {
            commands.entity(entity).despawn();
            state.active = None;
            state.lose_brick(&config);
            continue;
        }

        control.drop_distance =
            cast(position.0, rotation, Dir2::NEG_Y, config.cells(200.0)).map(|hit| hit.distance);
        commands.entity(entity).insert((position, rotation));
    }
}

/// Measures each tower and removes bricks that fell off.
#[allow(clippy::type_complexity)]
fn track_towers(
    mut commands: Commands,
    config: Res<GameConfig>,
    mut towers: Query<(&Tower, &mut TowerState)>,
    bricks: Query<
        (
            Entity,
            &Brick,
            &Position,
            &LinearVelocity,
            &ColliderAabb,
            Has<crate::spells::Bubbled>,
            Has<crate::spells::Ghosted>,
        ),
        Without<ActiveBrick>,
    >,
    joints: Query<(Entity, &crate::spells::VineJoint)>,
) {
    let kill_y = -config.cells(config.kill_depth);
    // Two dropped bricks can share a vine; despawn it once.
    let mut cut: bevy::platform::collections::HashSet<Entity> = default();
    let settled_speed = config.cells(0.5);
    let mut tops: Vec<(Entity, f32, bool)> = Vec::new();

    for (_, mut state) in &mut towers {
        state.height = 0.0;
        state.stack_top = 0.0;
        state.all_settled = true;
    }
    for (entity, brick, position, velocity, aabb, bubbled, ghosted) in &bricks {
        let Ok((tower, mut state)) = towers.get_mut(brick.tower) else {
            continue;
        };
        if position.0.y < kill_y {
            for (joint, vine) in &joints {
                if (vine.a == entity || vine.b == entity) && cut.insert(joint) {
                    commands.entity(joint).despawn();
                }
            }
            commands.entity(entity).despawn();
            state.lose_brick(&config);
            continue;
        }
        let on_tower = (position.0.x - tower.origin_x).abs()
            <= config.cells(config.platform_width * 0.5 + 2.0);
        // Bricks held up by a bubble or a ghost don't count toward the height.
        if !on_tower || aabb.min.y <= -config.cell_size || bubbled || ghosted {
            continue;
        }
        let settled = velocity.0.x.abs() < settled_speed && velocity.0.y.abs() < settled_speed;
        tops.push((brick.tower, aabb.max.y, settled));
    }
    for (tower, top, settled) in tops {
        if let Ok((_, mut state)) = towers.get_mut(tower) {
            // Like the original, the height is the top of any placed brick,
            // settled or not.
            state.stack_top = state.stack_top.max(top / config.cell_size);
            state.height = state.stack_top;
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
    game.elapsed += dt;
    if game.mode == ModeKind::Race && game.elapsed > config.race.finish_drop_after {
        game.finish_height = (game.finish_height - config.race.finish_drop_speed * dt).max(1.0);
    }
    for (_, mut state) in &mut towers {
        if state.status != TowerStatus::Building {
            continue;
        }
        match game.mode {
            ModeKind::Race => {
                if state.height >= game.finish_height {
                    state.hold_timer += dt;
                    if state.hold_timer >= config.race.hold_seconds {
                        state.status = TowerStatus::Won;
                    }
                } else {
                    state.hold_timer = 0.0;
                }
            }
            ModeKind::Survival => {
                state.invincible = (state.invincible - dt).max(0.0);
                if state.lives == Some(0) {
                    state.status = TowerStatus::Out;
                } else if state.bricks_used() >= config.survival.bricks_to_place
                    && state.active.is_none()
                {
                    // Out of bricks: a short countdown, then the roof goes on.
                    state.settle_timer += dt;
                    if state.settle_timer >= config.survival.finish_countdown {
                        state.status = TowerStatus::Won;
                    }
                }
            }
            ModeKind::Puzzle => {
                let rules = &config.puzzle;
                if state.all_settled && state.height > state.laser(&config) + 0.05 {
                    state.status = TowerStatus::Out;
                } else if state.bricks_used() >= rules.bricks
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
    let mut humans_building = 0;
    let mut humans = 0;
    let mut count = 0;
    for (tower, state) in &towers {
        count += 1;
        match state.status {
            TowerStatus::Won => {
                winner = Some(winner.map_or(tower.index, |w: usize| w.min(tower.index)))
            }
            TowerStatus::Building => building.push(tower.index),
            TowerStatus::Out => {}
        }
        if tower.human {
            humans += 1;
            humans_building += (state.status == TowerStatus::Building) as usize;
        }
    }
    if game.mode == ModeKind::Puzzle && count > 1 {
        // Everyone plays out their bricks; the tallest pile of bricks wins.
        winner = None;
        if building.is_empty() {
            winner = towers
                .iter()
                .max_by_key(|(t, s)| (s.bricks_placed, std::cmp::Reverse(t.index)))
                .map(|(t, _)| t.index);
        }
    } else if winner.is_none() && count > 1 && building.len() == 1 {
        // Last tower standing.
        winner = Some(building[0]);
    }
    // Over when someone wins, nobody is left, or every human is out.
    if winner.is_some() || building.is_empty() || (humans > 0 && humans_building == 0) {
        game.phase = Phase::Over;
        game.winner = winner;
    }
}
