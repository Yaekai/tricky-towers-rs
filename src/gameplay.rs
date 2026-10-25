//! The core loop: towers, bricks the players steer down, the hand-off to the
//! physics engine when a brick touches the tower, and the match rules.
//!
//! This module has no rendering or keyboard code, so it runs headless in tests
//! and mods can drive it with their own input, bots or replays. Every tower
//! (yours and each AI opponent's) runs through exactly the same systems; the
//! only difference is who writes its [`PlayerInput`].

use avian2d::dynamics::solver::islands::WakeBody;
use avian2d::prelude::*;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use crate::config::{Curse, GameConfig, ModeKind, PieceDef, PuzzleFloor, SpellKind};

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_hz(50.0))
            .init_resource::<Match>()
            .init_resource::<MatchRequest>()
            .init_resource::<WindGusts>()
            .add_message::<BrickLost>()
            .add_message::<RoofPlaced>()
            .add_message::<BrickNudged>()
            .add_message::<BrickRespawned>()
            .add_observer(put_in_tower_layer)
            .add_systems(FixedPreUpdate, layer_stragglers)
            .init_resource::<crate::spells::SpellLog>()
            .init_resource::<crate::spells::PendingCasts>()
            .init_resource::<crate::spells::EnemyCasts>()
            .add_systems(Startup, setup_world)
            .add_systems(
                FixedUpdate,
                (
                    start_match,
                    spawn_next_brick,
                    // Before the brick controls, which take the presses.
                    steer_roofs,
                    control_active_brick,
                    blow_wind,
                    push_wind,
                    track_towers,
                    crate::spells::update_spell_lines,
                    crate::spells::run_curse_waves,
                    crate::spells::cast_spells,
                    crate::spells::land_casts,
                    (
                        crate::spells::reshape_active_bricks,
                        crate::spells::loosen_ivy,
                        crate::spells::grow_ivy,
                        crate::spells::zap_bricks,
                        crate::spells::float_bubbles,
                        crate::spells::carry_ghosts,
                        crate::spells::collect_life_orbs,
                        crate::spells::home_balloons,
                    )
                        .chain(),
                    judge_match,
                    still_caught_towers,
                    petrify_out_towers,
                    raise_bases,
                    zap_over_laser,
                    place_obstacles,
                    drain_water,
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
    /// A battle's opening (mode title, camera sweep, 3-2-1); src/intro.rs runs it.
    /// Nothing spawns or moves until it hands over to `Playing`.
    Intro,
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
    /// Race Pro wind's side: 1 blows right, -1 left, 0 no wind.
    pub wind: f32,
    /// Started by Rematch or Retry.
    pub retry: bool,
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
            wind: 0.0,
            retry: false,
        }
    }
}

/// The winner's roof was set down on its tower (ROOF_PLACED), its bottom middle at this
/// world point (pixels).
#[derive(Message, Clone, Copy, Debug)]
pub struct RoofPlaced {
    pub tower: Entity,
    pub at: Vec2,
}

/// The brick in hand was nudged one way (+1 right, -1 left), for its look (BrickNudgeModifier).
#[derive(Message, Clone, Copy, Debug)]
pub struct BrickNudged {
    pub brick: Entity,
    pub dir: f32,
}

/// A brick fell into the water and was lost, at this world point (pixels).
#[derive(Message, Clone, Copy, Debug)]
pub struct BrickLost {
    pub tower: usize,
    pub at: Vec2,
    /// It cost a life (LIFE_LOST / LIFE_LOST_PUZZLE in the original), not just a brick.
    pub life: bool,
}

/// A lost brick in the water: no longer part of any tower, sinking slowly until it
/// goes.
#[derive(Component, Debug)]
pub struct Sunk;

/// Takes a brick that hit the water out of play, as the original does: it almost
/// stops, its spin is damped, and (unless it was in a bubble) the water lifts it.
fn sink(
    commands: &mut Commands,
    config: &GameConfig,
    entity: Entity,
    velocity: Vec2,
    spin: f32,
    floats: bool,
    water: bool,
) {
    let rules = &config.water;
    let mut brick = commands.entity(entity);
    brick.remove::<(
        Brick,
        ActiveBrick,
        Cursed,
        crate::spells::Bubbled,
        crate::spells::Ghosted,
    )>();
    // Out of view in Survival and Puzzle it just falls on; Race's water slows it.
    let keep = if water { 1.0 - rules.stop } else { 1.0 };
    let spin_keep = if water { 1.0 - rules.spin_damp } else { 1.0 };
    brick.insert((
        Sunk,
        RigidBody::Dynamic,
        LinearVelocity(velocity * keep),
        AngularVelocity(spin * spin_keep),
    ));
    if floats && water {
        brick.insert(ConstantLinearAcceleration(
            Vec2::Y * config.cells(rules.lift),
        ));
    }
}

/// Whether bricks are lost by leaving the view (Survival and Puzzle) rather than in
/// Race's water.
fn lost_out_of_view(game: &Match) -> bool {
    matches!(game.mode.rules(), ModeKind::Survival | ModeKind::Puzzle)
}

/// The bottom of the towers' view and its half width around a tower, in cells, as the
/// camera frames them (visuals::frame_towers: SingleTowerCamera, Puzzle's WidthTowerCamera
/// on the floor); a lone tower's view is taken as wide as the screen.
fn tower_view(config: &GameConfig, game: &Match, lowest: f32, highest: f32, towers: usize) -> (f32, f32) {
    let floor = -config.camera_floor();
    let bottom = if game.mode.rules() == ModeKind::Puzzle || game.phase == Phase::Over {
        floor
    } else {
        let look = &config.hud;
        let top = (highest + look.camera_height - look.camera_tower_room).max(floor + look.camera_height);
        (lowest - look.camera_below_lowest).max(floor).min(top - look.camera_height)
    };
    // ponytail: a lane's view is its share of a 16:9 screen; a trial's split isn't modelled.
    let lanes = if config.trial.is_some() { 1 } else { towers };
    (bottom, config.lanes.width(lanes) / 2.0)
}

/// TowerPartRemoveOutsideCameraChecker and its Survival and Puzzle variants: whether a
/// brick at `at` (cells from the tower's base centre) has left the view.
fn out_of_view(
    config: &GameConfig,
    game: &Match,
    view: (f32, f32),
    at: Vec2,
    fall: f32,
    scale: f32,
    ghosted: bool,
) -> bool {
    let reach = config.lost_reach * scale;
    let battle_puzzle = game.mode == ModeKind::Puzzle && config.trial.is_none();
    let offset = if battle_puzzle { config.lost_puzzle_offset } else { 0.0 };
    let out = at.y + reach < view.0 + offset || at.x.abs() - reach > view.1;
    // Survival battles and Endless Survival also drop a ghost-carried brick that leaves.
    let ghost_rule = matches!(game.mode, ModeKind::Survival | ModeKind::EndlessSurvival)
        && config.trial.is_none()
        && ghosted;
    out && (battle_puzzle || fall < -config.lost_fall_speed || ghost_rule)
}

/// Lost bricks float in Race's water for a while, or fall on out of view, then go.
fn drain_water(
    mut commands: Commands,
    config: Res<GameConfig>,
    sunk: Query<(Entity, &Position), With<Sunk>>,
) {
    for (entity, position) in &sunk {
        if position.0.y < -config.cells(config.water.remove_depth) {
            commands.entity(entity).despawn();
        }
    }
}

/// Present in the full game (src/intro.rs): battles open with their intro instead
/// of going straight into play. Headless runs leave it out.
#[derive(Resource)]
pub struct IntroOn;

/// Set `pending` to start (or restart) a match with the current
/// [`GameConfig::mode`] and [`GameConfig::opponents`].
#[derive(Resource, Debug, Default)]
pub struct MatchRequest {
    pub pending: bool,
    /// Rematch or Retry: the next match skips the intro's camera sweep and count-up.
    pub retry: bool,
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

/// A block of the Puzzle floor (static, part of no one's tower count).
#[derive(Component, Debug)]
pub struct FloorBlock {
    pub part: FloorPart,
}

/// Which piece of the original's floor a block is drawn as.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloorPart {
    /// An Easy or Normal column, in one of the 1-wide looks (0 or 1).
    Column(u32),
    /// An Easy or Normal piece two cells wide (the original's piece 2).
    Wide,
    /// One of the Pro pillar's columns.
    Pillar,
    /// A Pro side ledge in one of 3 looks, or the highest ledge on its side.
    Ledge(u32),
    TopLedge,
}

/// The Puzzle floor's pieces for one round: each is (x of its centre, top, bottom, width in
/// cells), in cells from the middle of the base's top. The same for every tower. Easy and
/// Normal build it as the original does (docs/research/puzzle-floor.md): a cell is skipped
/// with the gap chance, otherwise a 1- or 2-cell piece goes down; one piece stays flat and
/// the rest drop by a random amount.
pub fn puzzle_floor(config: &GameConfig, roll: &mut u64) -> Vec<(f32, f32, f32, u32)> {
    let depth = config.puzzle.floor_depth.max(1.0).round();
    fn chance(roll: &mut u64, percent: u32) -> bool {
        xorshift(roll) % 100 < percent as u64
    }
    let mut columns = Vec::new();
    match config.puzzle.floor {
        PuzzleFloor::Plain => {}
        PuzzleFloor::Easy | PuzzleFloor::Normal => {
            let (width, gaps, lowest) = if config.puzzle.floor == PuzzleFloor::Easy {
                config.puzzle.easy_floor
            } else {
                config.puzzle.normal_floor
            };
            let mut pieces: Vec<(f32, u32)> = Vec::new();
            let mut num = 0;
            while num < width {
                // The first cell with nothing placed yet always takes a piece.
                let place = !chance(roll, gaps) || (num + 1 >= width && pieces.is_empty());
                if place {
                    let wide = if xorshift(roll) % 3 == 0 { 2 } else { 1 };
                    let wide = if num + wide > width { 1 } else { wide };
                    // From RandomPuzzleFloor's _minX, floor(-width / 2): a 5-wide floor sits
                    // half a cell left of the lane centre.
                    let x = num as f32 + wide as f32 * 0.5 + (-(width as f32) / 2.0).floor();
                    pieces.push((x, wide));
                    num += wide;
                } else {
                    num += 1;
                }
            }
            // One piece stays flat; with more than one, another drops by 1 to `lowest`;
            // the rest by 0 to `lowest`.
            let mut left: Vec<usize> = (0..pieces.len()).collect();
            if !left.is_empty() {
                left.remove((xorshift(roll) % left.len() as u64) as usize);
            }
            let mut drops = vec![0u32; pieces.len()];
            if pieces.len() > 1 && lowest > 0 {
                let k = left.remove((xorshift(roll) % left.len() as u64) as usize);
                drops[k] = 1 + (xorshift(roll) % lowest as u64) as u32;
            }
            for &k in &left {
                drops[k] = (xorshift(roll) % (lowest as u64 + 1)) as u32;
            }
            for (&(x, wide), &drop) in pieces.iter().zip(&drops) {
                columns.push((x, -(drop as f32), -depth, wide));
            }
        }
        // ProRandomPuzzleFloor: a 1-wide pillar; beside it a column on each side from 4
        // cells above the original's floor bottom; then on each side, from the top row
        // down through all but the last 3 rows, a 1-cell ledge on `ledges`% of rows.
        // Only what reaches above our (shallower) floor's bottom is kept.
        PuzzleFloor::Pro => {
            let (height, ledges) = config.puzzle.pro_floor;
            let height = height.max(4) as f32;
            columns.push((0.0, 0.0, -depth, 1));
            for side in [1.0, -1.0] {
                let top = -(height - 4.0);
                if top > -depth {
                    columns.push((side, top, -depth, 1));
                }
            }
            for side in [1.0, -1.0] {
                for row in 0..(height as i32 - 3) {
                    let top = -(row as f32);
                    if chance(roll, ledges) && top > -depth {
                        columns.push((side, top, top - 1.0, 1));
                    }
                }
            }
        }
    }
    columns
}

/// A lost cursed brick on its way back.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Respawn {
    pub piece: usize,
    pub mods: BrickMod,
    pub times: u32,
    /// Where it was lost (pixels), for the trail to the brick dealt again.
    pub from: Vec2,
}

/// A lost cursed brick was dealt again (BrickDroppedChecker's RespawnTrailController): the
/// new brick and where the old one was lost (pixels).
#[derive(Message, Clone, Copy, Debug)]
pub struct BrickRespawned {
    pub brick: Entity,
    pub from: Vec2,
}

/// On a placed brick that carries a curse: what it would come back as if lost.
#[derive(Component, Debug, Clone, Copy)]
pub struct Cursed {
    pub mods: BrickMod,
    pub times: u32,
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
    /// Whether a lost brick with these effects comes back (curses and Mystery do).
    pub fn comes_back(&self) -> bool {
        self.large > 0
            || self.fast > 0.0
            || self.friction.is_some()
            || self.no_rotate
            || self.auto_rotate.is_some()
            || self.slow
            || self.mystery
    }

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
    /// Puzzle: seconds since a brick touched the laser; the round's 3-2-1 before the
    /// bricks are counted.
    pub laser_count: Option<f32>,
    /// Puzzle: how far the base has risen so far, in cells; it eases up to `base_rise`.
    pub base_shown: f32,
    /// Puzzle: the base's current move (from, to, seconds in), and how many moves have
    /// started (the effects rumble with each; TowerMoveController).
    pub base_move: Option<(f32, f32, f32)>,
    pub base_moves: u32,
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
    /// When (match seconds) the last brick was placed, for BrickDroppedChecker's 2.5 s.
    pub last_placed_at: f32,
    /// Nothing was within a cell under it when it was placed (BrickPlacedStruct.placedMidAir).
    pub last_placed_mid_air: bool,
    /// Seconds of Mist left over this tower.
    pub mist_timer: f32,
    /// Seconds before a dropped brick can cost another life.
    pub invincible: f32,
    /// Match time when the tower went out, for the final placings.
    pub out_at: Option<f32>,
    /// This round's puzzle laser height, before dropped bricks lower it.
    pub laser_base: f32,
    /// Spell effects waiting for the coming bricks, next brick first.
    pub next_mods: Vec<BrickMod>,
    /// Mystery shapes still in the bag.
    pub mystery_bag: Vec<usize>,
    /// A lost cursed brick coming back next: its piece, effects, how many times it
    /// has come back, and the x it was lost at.
    pub respawn: Option<Respawn>,
    /// A curse wave's effect on every new brick, and seconds it still lasts.
    pub wave_mod: Option<BrickMod>,
    pub wave_left: f32,
    /// The wave holds Ghost: each new brick carries off the last placed one.
    pub wave_ghost: bool,
    /// Ghost hits the curse wave still owes this tower.
    pub ghosts_due: u32,
    /// Bricks placed so far, counting up; stamps each brick's `Placed`.
    pub placed_count: u32,
    /// Seconds the brick in hand is still lifted and held by a wave's start, and
    /// where it rises from and to (world pixels; it shakes around that path).
    pub wave_hold: f32,
    pub wave_hold_x: f32,
    /// A trial's "(random)" spell line: cast the light spell next tick.
    pub auto_cast_light: bool,
    /// Bricks still to curse in an Endless Survival wave, after the brick in hand.
    pub wave_bricks: u32,
    /// Bricks that fell out of the tower after being placed.
    pub tower_losses: u32,
    /// Endless Puzzle: the level, the bricks used when it ends, and the time into
    /// a level-up while one runs.
    pub puzzle_level: u32,
    pub level_end: u32,
    pub level_up: Option<f32>,
    /// Blocks placed so far in the running level-up.
    pub obstacles_placed: u32,
    /// The winner's roof.
    pub roof: RoofStage,
    pub wave_lift: (f32, f32),
    /// Endless Race: the pacer line's height in cells, checkpoints reached, its speed
    /// and the step the next checkpoint adds, seconds left of the checkpoint stop, the
    /// height it stopped at, and seconds since the run started.
    /// seconds it still waits after the last stage.
    pub pacer: f32,
    pub pacer_stages: u32,
    pub pacer_speed: f32,
    pub pacer_step: f32,
    pub pacer_pause: f32,
    pub pacer_from: f32,
    pub pacer_clock: f32,
    /// Endless Race once the pacer has caught the tower: its end sweep, before the roof.
    pub pacer_end: Option<PacerEnd>,
    /// How far left and right of the lane centre the tower's top `auto_build_top` cells reach
    /// (Tower.GetTowerRect), in cells; `None` with nothing there.
    pub top_span: Option<(f32, f32)>,
    /// How far the widest settled brick reaches from the lane centre, in cells.
    pub half_width: f32,
    /// The tallest this tower has been, in cells (Endless Race's score).
    pub best_height: f32,
    pub rng: u64,
}

impl TowerState {
    pub fn new(config: &GameConfig, seed: u64) -> Self {
        Self {
            status: TowerStatus::Building,
            bricks_placed: 0,
            bricks_dropped: 0,
            laser_count: None,
            base_shown: 0.0,
            base_move: None,
            base_moves: 0,
            // Endless Puzzle has no health (no HEALTH model): a lost brick lifts the tower.
            lives: (config.mode.rules() == ModeKind::Survival
                && config.mode != ModeKind::EndlessPuzzle)
                .then_some(config.survival.lives),
            height: 0.0,
            stack_top: 0.0,
            all_settled: true,
            hold_timer: 0.0,
            settle_timer: 0.0,
            active: None,
            // Puzzle has no spell lines, only a spell to start with.
            light_spell: (config.mode.rules() == ModeKind::Puzzle && config.magic.enabled)
                .then_some(config.puzzle.start_spell)
                .flatten(),
            dark_spell: None,
            spell_line: None,
            next_line: 0.0,
            line_wait: 0.0,
            lines_reached: 0,
            last_light: None,
            last_placed: None,
            last_placed_at: 0.0,
            last_placed_mid_air: false,
            mist_timer: 0.0,
            invincible: 0.0,
            out_at: None,
            laser_base: config.puzzle.line_height,
            next_mods: Vec::new(),
            respawn: None,
            wave_mod: None,
            wave_left: 0.0,
            wave_ghost: false,
            ghosts_due: 0,
            placed_count: 0,
            wave_hold: 0.0,
            wave_hold_x: 0.0,
            auto_cast_light: false,
            wave_bricks: 0,
            tower_losses: 0,
            puzzle_level: 1,
            level_end: config.endless_play.level_bricks(1),
            level_up: None,
            obstacles_placed: 0,
            roof: RoofStage::None,
            wave_lift: (0.0, 0.0),
            pacer: config.endless.intro_from,
            pacer_stages: 0,
            pacer_speed: config.endless.start_speed,
            pacer_step: config.endless.first_step,
            pacer_pause: 0.0,
            pacer_from: 0.0,
            pacer_clock: 0.0,
            pacer_end: None,
            top_span: None,
            best_height: 0.0,
            half_width: 0.0,
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
    /// Counts a lost brick; true when it cost a life.
    fn lose_brick(&mut self, config: &GameConfig) -> bool {
        self.bricks_dropped += 1;
        if self.invincible > 0.0 {
            return false;
        }
        match self.lives.as_mut() {
            Some(lives) => {
                *lives = lives.saturating_sub(1);
                self.invincible = config.survival.invincible_seconds;
                true
            }
            None => false,
        }
    }

    /// True while the winner's roof is still being steered or settling.
    pub fn roof_pending(&self) -> bool {
        matches!(self.roof, RoofStage::Steering | RoofStage::Resting(_))
    }

    /// Bricks used so far, placed or dropped.
    pub fn bricks_used(&self) -> u32 {
        self.bricks_placed + self.bricks_dropped
    }

    /// Puzzle laser height for this tower: each dropped brick brings it lower.
    pub fn laser(&self, config: &GameConfig) -> f32 {
        if config.mode == ModeKind::EndlessPuzzle {
            // Endless Puzzle: the laser rises at each level-up.
            let rules = &config.endless_play;
            // DOTween's default ease, out-quad.
            let rising = self.level_up.map_or(0.0, |t| {
                let k = ((t - rules.level_wait.0) / rules.rise_seconds.max(0.01)).clamp(0.0, 1.0);
                1.0 - (1.0 - k) * (1.0 - k)
            });
            return self.laser_base + rules.laser_rise * rising;
        }
        self.laser_base
    }

    /// Seconds left of this tower's CountDownEffect (a race's hold, Survival's last brick
    /// used, Endless Puzzle's laser touch) while it runs.
    pub fn count_left(&self, config: &GameConfig) -> Option<f32> {
        if self.hold_timer <= 0.0 || self.status != TowerStatus::Building {
            return None;
        }
        let length = if config.mode == ModeKind::Survival {
            config.survival.finish_countdown
        } else {
            config.race.hold_seconds
        };
        Some((length - self.hold_timer).max(0.0))
    }

    /// Puzzle and Endless Puzzle (TowerMoveController): how far the base has risen, in
    /// cells, for the bricks that fell off.
    pub fn base_rise(&self, config: &GameConfig) -> f32 {
        if matches!(config.mode, ModeKind::Puzzle | ModeKind::EndlessPuzzle) {
            config.puzzle.drop_penalty * self.bricks_dropped as f32
        } else {
            0.0
        }
    }

    /// Endless Survival and Endless Puzzle score: the bricks in the tower.
    pub fn endless_score(&self) -> u32 {
        self.bricks_placed.saturating_sub(self.tower_losses)
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

/// When a brick was placed: its tower's running count at the time.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Placed(pub u32);

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
    /// Times this brick has already come back after being lost.
    pub comebacks: u32,
    repeat_timer: f32,
    repeating: bool,
    /// Current fall speed, in cells per second.
    speed: f32,
    pub(crate) auto_rotate_timer: f32,
    /// The balloon's cap on held-down falling, and whether down was held last step.
    slow_cap: f32,
    down_before: bool,
    /// Seconds a refused turn (No Rotate) still shakes the brick; drawn by the
    /// visuals.
    pub refused: f32,
    /// How far (world units) the brick's picture jumps about while a curse wave
    /// lifts and holds it; 0 when not. Only the picture shakes, not the body.
    pub picture_shake: f32,
    /// Seconds the brick still hangs where it is, taking no input (Slow's balloon catching
    /// it: PauseBrickControlEffect ... ResumeBrickControlEffect); it shakes while
    /// `hold_shake` is above 0.
    pub hold: f32,
    pub hold_shake: f32,
    /// Seconds the victim's pad still rumbles while a dark cast hangs its brick
    /// (DependantControllerRumbleEffect 128/128, until the shake comes off).
    pub hold_rumble: f32,
    /// Carried back up by a spell (MoveBrickUpEffect): from and to height (pixels),
    /// seconds so far and in all; it passes through everything on the way.
    pub rise: Option<(f32, f32, f32, f32)>,
}

impl ActiveBrick {
    fn new(query_shape: Collider, mods: BrickMod) -> Self {
        Self {
            query_shape,
            drop_distance: None,
            mods,
            reshape: false,
            comebacks: 0,
            repeat_timer: 0.0,
            repeating: false,
            speed: 0.0,
            auto_rotate_timer: 0.0,
            slow_cap: 0.0,
            down_before: false,
            refused: 0.0,
            picture_shake: 0.0,
            hold: 0.0,
            hold_shake: 0.0,
            hold_rumble: 0.0,
            rise: None,
        }
    }
}

#[derive(Component, Debug)]
pub struct Platform {
    pub tower: Entity,
}

/// A piece of a tower's base that rises with its drops: where it stands with no
/// bricks lost.
#[derive(Component, Debug)]
pub struct Raised {
    pub tower: Entity,
    pub rest: f32,
}

/// Puzzle: the tower's base (and its floor blocks) rise as bricks fall off it, while
/// the laser stays one line for everyone.
fn raise_bases(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut towers: Query<(Entity, &mut TowerState)>,
    mut bases: Query<(&Raised, &mut Transform), Without<RigidBody>>,
    mut bodies: Query<
        (&mut Position, &mut Transform, Option<&Brick>, Option<&Raised>, Option<&Children>),
        (With<RigidBody>, Without<ChildOf>),
    >,
    members: Query<&Brick, Without<RigidBody>>,
) {
    let dt = time.delta_secs().max(f32::EPSILON);
    let rules = &config.puzzle;
    let mut shown: HashMap<Entity, f32> = HashMap::new();
    let mut moved: HashMap<Entity, f32> = HashMap::new();
    for (entity, mut state) in &mut towers {
        let before = state.base_shown;
        // Each brick lost before the round's bricks are zapped moves the base up from where
        // it is to the new height over `base_rise_time` (DOTween's out-quad); once a tower
        // touched the laser it stays put.
        let target = state.base_rise(&config);
        let heading = state.base_move.map_or(state.base_shown, |m| m.1);
        if state.laser_count.is_none() && state.hold_timer == 0.0 && target != heading {
            state.base_move = Some((state.base_shown, target, 0.0));
            state.base_moves += 1;
        }
        if let Some((from, to, t)) = state.base_move.as_mut() {
            *t += dt;
            let k = (*t / rules.base_rise_time.max(0.001)).min(1.0);
            let at = *from + (*to - *from) * (1.0 - (1.0 - k) * (1.0 - k));
            if k >= 1.0 {
                state.base_move = None;
            }
            state.base_shown = at;
        }
        shown.insert(entity, state.base_shown * config.cell_size);
        if state.base_shown != before {
            moved.insert(entity, (state.base_shown - before) * config.cell_size);
        }
    }
    // The original moves nothing: TowerMoveController only shifts the tower's camera offset,
    // so the tower and the laser part by half a cell while every brick keeps its contacts.
    // With one camera for all lanes the whole tower (floor, bricks, Ivy clusters, the brick in
    // hand) is moved by the same amount instead; pushing the floor up under the bricks threw
    // the ones near an edge off.
    if !moved.is_empty() {
        for (mut position, mut transform, brick, raised, children) in &mut bodies {
            let owner = brick.map(|b| b.tower).or(raised.map(|r| r.tower)).or_else(|| {
                children?.iter().find_map(|c| members.get(c).ok()).map(|b| b.tower)
            });
            if let Some(&dy) = owner.and_then(|t| moved.get(&t)) {
                position.0.y += dy;
                transform.translation.y += dy;
            }
        }
    }
    // A part with no body (the platform holding the lines and art) is simply put there.
    for (raised, mut transform) in &mut bases {
        let want = raised.rest + shown.get(&raised.tower).copied().unwrap_or(0.0);
        if transform.translation.y != want {
            transform.translation.y = want;
        }
    }
}

/// Merges a brick's cells into solid boxes as the original's prefabs have them (rows of
/// cells joined, equal rows stacked): `(centre, width, height)` in cells.
fn brick_boxes(cells: &[Vec2]) -> Vec<(Vec2, f32, f32)> {
    let mut keys: Vec<(i32, i32)> = cells
        .iter()
        .map(|p| ((p.x * 2.0).round() as i32, (p.y * 2.0).round() as i32))
        .collect();
    keys.sort_by_key(|&(x, y)| (y, x));
    // Row runs: (y, x_start, x_end) in half cells, cells 2 apart.
    let mut runs: Vec<(i32, i32, i32)> = Vec::new();
    for (x, y) in keys {
        match runs.last_mut() {
            Some(r) if r.0 == y && r.2 + 2 == x => r.2 = x,
            _ => runs.push((y, x, x)),
        }
    }
    // Stack runs with the same extent on neighbouring rows: (y_lo, y_hi, x_start, x_end).
    let mut boxes: Vec<(i32, i32, i32, i32)> = Vec::new();
    for (y, x0, x1) in runs {
        match boxes.iter_mut().find(|b| b.2 == x0 && b.3 == x1 && b.1 + 2 == y) {
            Some(b) => b.1 = y,
            None => boxes.push((y, y, x0, x1)),
        }
    }
    boxes
        .into_iter()
        .map(|(y0, y1, x0, x1)| {
            let centre = Vec2::new((x0 + x1) as f32 / 4.0, (y0 + y1) as f32 / 4.0);
            (centre, (x1 - x0) as f32 / 2.0 + 1.0, (y1 - y0) as f32 / 2.0 + 1.0)
        })
        .collect()
}

/// Builds a brick collider from its cells, as the original's solid boxes. `fill` is the
/// fraction of each cell left after trimming `1 - fill` of a cell off every box's width and
/// height (the original's shrink amount).
pub fn brick_collider(cells: &[Vec2], cell: f32, fill: f32) -> Collider {
    let trim = (1.0 - fill) * cell;
    Collider::compound(
        brick_boxes(cells)
            .into_iter()
            .map(|(c, w, h)| {
                (
                    c * cell,
                    Rotation::IDENTITY,
                    Collider::rectangle((w * cell - trim).max(0.01), (h * cell - trim).max(0.01)),
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
    let trim = (1.0 - fill) * cell;
    let area: f32 = brick_boxes(&brick.cells).iter().map(|&(_, w, h)| (w * cell - trim) * (h * cell - trim)).sum();
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

/// The mode rules as loaded, before a match's difficulty, trial or Endless mode.
type ModeRules = (
    crate::config::RaceRules,
    crate::config::SurvivalRules,
    crate::config::PuzzleRules,
    crate::config::MagicRules,
);

/// Clears the field and builds a tower for you and one per opponent.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn start_match(
    mut commands: Commands,
    intro: Option<Res<IntroOn>>,
    mut request: ResMut<MatchRequest>,
    mut game: ResMut<Match>,
    mut config: ResMut<GameConfig>,
    mut log: ResMut<crate::spells::SpellLog>,
    old: Query<
        Entity,
        Or<(
            With<Tower>,
            With<Brick>,
            With<Platform>,
            With<crate::spells::LifeOrb>,
            With<crate::spells::BalloonOrb>,
            With<RoofPiece>,
            With<RivalLane>,
            With<crate::spells::IvyCluster>,
        )>,
    >,
    mut base: Local<Option<ModeRules>>,
) {
    if !std::mem::take(&mut request.pending) {
        return;
    }
    let retry = std::mem::take(&mut request.retry);
    // Trials and the Endless modes change the mode rules for one match; start
    // each match from the rules as loaded.
    let base = base.get_or_insert_with(|| {
        (
            config.race.clone(),
            config.survival.clone(),
            config.puzzle.clone(),
            config.magic.clone(),
        )
    });
    config.race = base.0.clone();
    config.survival = base.1.clone();
    config.puzzle = base.2.clone();
    config.magic = base.3.clone();
    // The picked difficulty's values replace the mode rules.
    config.apply_difficulty();
    // A trial's own rules go over the difficulty's.
    if let Some(trial) = config.trial.and_then(|i| config.trials.get(i)).cloned() {
        trial.apply(&mut config);
    }
    let endless = config.endless_play.clone();
    endless.apply(&mut config);
    let config = &*config;
    // An Ivy cluster takes its bricks with it, so some may already be gone.
    for entity in &old {
        commands.entity(entity).try_despawn();
    }

    // The puzzle laser is the same for everyone, and new each round.
    let round = game.round + 1;
    // Every match deals its own bricks (the original seeds each match at random); the config's
    // seed only fixes the first one, so runs stay reproducible.
    let match_seed = config.seed ^ game.round.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let mut roll = config.seed ^ round.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let laser = config.puzzle.line_height
        + (xorshift(&mut roll) % 1001) as f32 / 1000.0 * config.puzzle.line_random;
    // Race wind blows to a random side for the whole match.
    let wind = match config.race.wind {
        Some(_) if config.mode == ModeKind::Race => {
            if xorshift(&mut roll).is_multiple_of(2) {
                1.0
            } else {
                -1.0
            }
        }
        _ => 0.0,
    };
    // A trial's own floor replaces the round's Puzzle columns.
    let floor = if config.mode.rules() == ModeKind::Puzzle && config.trial_floor().is_none() {
        puzzle_floor(config, &mut roll)
    } else {
        Vec::new()
    };
    let humans = config.players.clamp(1, MAX_TOWERS as u32) as usize;
    let mut count = (humans + config.opponents as usize).min(MAX_TOWERS);
    // A trial is one player against the enemy wizard, who has no tower.
    let humans = if config.trial.is_some() { 1 } else { humans };
    if config.trial.is_some() {
        count = 1;
    }
    let c = config.cell_size;
    for index in 0..count {
        // The screen is shared out into equal lanes, wider with fewer towers.
        let spacing = config.lanes.spacing(count);
        let mut origin_x = config.cells(spacing) * (index as f32 - (count - 1) as f32 * 0.5);
        if config.trial.is_some() {
            // A trial splits the screen with the enemy wizard's lane on the right;
            // a Puzzle trial's lane takes the left two thirds, centred in them.
            origin_x = if config.mode == ModeKind::Puzzle {
                -config.cells(config.lanes.screen_width) / 6.0
            } else {
                -config.cells(config.lanes.spacing(2)) * 0.5
            };
            if index == 0 {
                let rival = config
                    .trial
                    .and_then(|i| config.trials.get(i))
                    .and_then(|t| t.rival);
                commands.spawn((
                    RivalLane {
                        origin_x: config.cells(config.lanes.spacing(2)) * 0.5,
                        rival,
                    },
                    Transform::default(),
                ));
            }
        }
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
                ..TowerState::new(config, match_seed.wrapping_add(index as u64 * 7919))
            },
            // Everyone shares one brick order on Pro and in Puzzle, else each their own.
            BrickBag::new(
                if config.same_bricks(config.mode) {
                    match_seed
                } else {
                    match_seed.wrapping_add((index as u64 + 1) * 0x2545_F491_4F6C_DD1D)
                },
                config.pieces.len(),
            ),
            PlayerInput::default(),
        ));
        if !human {
            tower.insert(AiControlled);
        }
        let (wizard, pack) = config
            .skins
            .get(index)
            .copied()
            .unwrap_or((index, crate::config::BrickPack::Default));
        tower.insert(TowerSkin { wizard, pack });
        let tower = tower.id();

        // The platform's top surface sits at y = 0. In Puzzle the round's floor
        // takes its place.
        let mut platform = commands.spawn((
            Platform { tower },
            Raised {
                tower,
                rest: -c * 0.5,
            },
            Transform::from_xyz(origin_x, -c * 0.5, 0.0),
        ));
        if floor.is_empty() {
            // The base's boxes, placed from its top (half a cell above the centre),
            // or a trial's own floor in place of or on top of them.
            let trial_floor = config.trial_floor();
            let mut boxes = match trial_floor {
                Some(f) if !f.keep_base => Vec::new(),
                _ => config.base_boxes(config.mode),
            };
            boxes.extend(trial_floor.iter().flat_map(|f| f.boxes.iter().copied()));
            let parts = boxes
                .iter()
                .map(|b| {
                    let size = Vec2::new(b.x.1 - b.x.0, b.y.1 - b.y.0) * c;
                    let centre = Vec2::new(b.x.0 + b.x.1, b.y.0 + b.y.1) * 0.5 * c;
                    (
                        centre + Vec2::Y * c * 0.5,
                        Rotation::IDENTITY,
                        Collider::rectangle(size.x, size.y),
                    )
                })
                .collect();
            // Puzzle's bases rise under the bricks, so they move kinematically there.
            let body = if config.mode.rules() == ModeKind::Puzzle {
                RigidBody::Kinematic
            } else {
                RigidBody::Static
            };
            platform.insert((
                body,
                Collider::compound(parts),
                Friction::new(config.brick_friction),
            ));
        }
        // The highest ledge on each side of a Pro pillar has its own look.
        let top_ledge = |side: f32| {
            floor
                .iter()
                .filter(|&&(x, top, bottom, _)| x != 0.0 && top - bottom <= 1.0 && x.signum() == side)
                .map(|&(_, top, _, _)| top)
                .fold(f32::MIN, f32::max)
        };
        let (left_top, right_top) = (top_ledge(-1.0), top_ledge(1.0));
        let pro = config.puzzle.floor == PuzzleFloor::Pro;
        for (n, &(x, top, bottom, wide)) in floor.iter().enumerate() {
            let part = if !pro {
                if wide == 2 {
                    FloorPart::Wide
                } else {
                    FloorPart::Column((n as u64 ^ round) as u32 % 2)
                }
            } else if x == 0.0 || top - bottom > 1.0 {
                FloorPart::Pillar
            } else if top == if x < 0.0 { left_top } else { right_top } {
                FloorPart::TopLedge
            } else {
                FloorPart::Ledge(((n as u64).wrapping_mul(2654435761) ^ round) as u32 % 3)
            };
            let rows = (top - bottom).round().max(1.0) as i32;
            let cells: Vec<Vec2> = (0..rows)
                .map(|i| Vec2::new(0.0, (rows - 1) as f32 * 0.5 - i as f32))
                .collect();
            let block = Brick {
                tower,
                piece: 0,
                cells,
                color: Color::srgb(0.62, 0.52, 0.40),
                scale: 1.0,
                mass: 1.0,
                ice: false,
            };
            // Solid like the base's boxes, not the shrunk brick shape: a brick hangs over a
            // floor block the way it does over a base, and not over a brick.
            let collider = Collider::rectangle(wide as f32 * c, rows as f32 * c);
            let centre = Vec2::new(
                origin_x + config.cells(x),
                config.cells((top + bottom) * 0.5),
            );
            commands.spawn((
                block,
                FloorBlock { part },
                Raised {
                    tower,
                    rest: centre.y,
                },
                RigidBody::Kinematic,
                collider,
                Friction::new(config.brick_friction),
                Transform::from_translation(centre.extend(0.0)),
            ));
        }
    }

    log.0.clear();
    // Battles open with their intro (mode title, 3-2-1) in the full game.
    *game = Match {
        phase: if intro.is_some() {
            Phase::Intro
        } else {
            Phase::Playing
        },
        mode: config.mode,
        winner: None,
        elapsed: 0.0,
        finish_height: config.race.finish_height,
        players: count,
        round,
        wind,
        retry,
    };
}

fn spawn_next_brick(
    mut commands: Commands,
    config: Res<GameConfig>,
    game: Res<Match>,
    spatial: SpatialQuery,
    mut towers: Query<(Entity, &Tower, &mut TowerState, &mut BrickBag)>,
    mut respawned: MessageWriter<BrickRespawned>,
) {
    if game.phase != Phase::Playing {
        return;
    }
    let c = config.cell_size;
    for (tower_entity, tower, mut state, mut bag) in &mut towers {
        if state.status != TowerStatus::Building || state.active.is_some() || state.pacer_end.is_some() {
            continue;
        }
        // The Endless modes have no brick limit.
        let limit = match game.mode {
            // A Puzzle battle has no brick limit: a round ends with the laser.
            ModeKind::Puzzle if config.trial.is_some() => Some(config.puzzle.bricks),
            ModeKind::Puzzle => None,
            ModeKind::Survival => Some(config.survival.bricks_to_place),
            // Endless Puzzle stops at the end of each level until the level-up.
            ModeKind::EndlessPuzzle => Some(state.level_end),
            _ => None,
        };
        if limit.is_some_and(|n| state.bricks_used() >= n) {
            continue;
        }
        // A single-player run stops dealing bricks while its finish countdown runs, and deals
        // again if the countdown aborts (AbstractSinglePlayerGameMode's brickSpawningActive).
        if config.trial.is_some() && game.mode == ModeKind::Race && state.hold_timer > 0.0 {
            continue;
        }
        // Endless Puzzle deals no more once a brick crossed the laser.
        if game.mode == ModeKind::EndlessPuzzle && state.hold_timer > 0.0 {
            continue;
        }
        let respawn = state.respawn.take();
        let (mut modifier, piece) = match respawn {
            // A lost cursed brick comes back as it was.
            Some(back) => (back.mods, back.piece),
            None => {
                let mut modifier = if state.next_mods.is_empty() {
                    BrickMod::default()
                } else {
                    state.next_mods.remove(0)
                };
                if let Some(wave) = &state.wave_mod {
                    modifier.merge(wave);
                }
                let mut piece = bag.draw();
                // A trial with a fixed brick order deals exactly that order.
                if let Some(name) = config
                    .trial
                    .and_then(|i| config.trials.get(i))
                    .and_then(|t| t.order.get(state.bricks_used() as usize))
                    && let Some(fixed) = config.pieces.iter().position(|p| &p.name == name)
                {
                    piece = fixed;
                }
                if modifier.mystery
                    && let Some(shape) = state.draw_mystery(config.magic.mystery_pieces.len())
                {
                    piece = config.pieces.len() + shape;
                }
                (modifier, piece)
            }
        };
        // Ghost in a curse wave strikes again with every new brick.
        if state.wave_ghost {
            state.ghosts_due += 1;
        }
        // An Endless Survival wave ends after its last cursed brick.
        if respawn.is_none() && state.wave_bricks > 0 {
            state.wave_bricks -= 1;
            if state.wave_bricks == 0 {
                state.wave_mod = None;
                state.wave_ghost = false;
            }
        }
        // A Mystery piece is a curse of its own when it comes back.
        modifier.mystery |= piece >= config.pieces.len();
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
        let (_, density) = brick_body(&config, &brick);
        let friction = Friction::new(modifier.friction.unwrap_or(config.brick_friction));

        // Auto Build (AutoBuildBrickModifier): over a random spot across the tower's top, as
        // stone, falling at `auto_build_speed`.
        let auto_x = modifier.auto_build.then(|| {
            let (range, limit) = (config.magic.auto_build_range, config.lane_half_width);
            let (lo, hi) = state.top_span.map_or((0.0, 0.0), |(lo, hi)| {
                let (lo, hi) = (lo.max(-range), hi.min(range));
                (lo.clamp(-limit, limit), hi.max(lo).clamp(-limit, limit))
            });
            let k = (state.random() % 1000) as f32 / 1000.0;
            tower.origin_x + config.cells(lo + (hi - lo) * k)
        });
        if modifier.auto_build {
            modifier.petrify = true;
        }

        let floor = match game.mode.rules() {
            // Puzzle bricks come from above the laser line.
            ModeKind::Puzzle => state.stack_top.max(state.laser(&config)),
            _ => state.stack_top,
        };
        // A returning brick is dealt where every brick is (LocalGamePlayController._SpawnBrick).
        let x = auto_x.unwrap_or(tower.origin_x);
        let mut position = Vec2::new(x, config.cells(floor + config.spawn_gap));
        // Bricks flying around can still be in the way; spawn above them.
        for _ in 0..32 {
            let hits = spatial.shape_intersections(
                &query_shape,
                position,
                0.0,
                &tower_filter(tower.index),
            );
            if hits.is_empty() {
                break;
            }
            position.y += c;
        }

        let entity = commands
            .spawn((
                brick,
                ActiveBrick {
                    comebacks: respawn.map_or(0, |back| back.times),
                    ..ActiveBrick::new(query_shape.clone(), modifier)
                },
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
        if let Some(back) = respawn {
            respawned.write(BrickRespawned { brick: entity, from: back.from });
        }
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
    mut lost: MessageWriter<BrickLost>,
    mut nudged: MessageWriter<BrickNudged>,
) {
    let dt = time.delta_secs();
    let c = config.cell_size;
    // DropSpeedController: the drop speed rises with the tallest tower still playing (its
    // dead-player share is never fed in the original); a Puzzle battle keeps one pace
    // (PuzzleGameModeFactory: no tower height), and Endless Race tops out.
    let (highest, out) = towers.iter().fold((0.0f32, 0u32), |(h, o), (_, s, _)| {
        let playing = s.status == TowerStatus::Building;
        (if playing { h.max(s.height) } else { h }, o + (s.status == TowerStatus::Out) as u32)
    });
    let steady = game.mode == ModeKind::Puzzle && config.trial.is_none();
    let mut drop_speed = config.drop_speed()
        + if steady {
            0.0
        } else {
            highest * config.fall_speed_per_height + out as f32 * config.fall_speed_per_out
        };
    if game.mode == ModeKind::Endless {
        drop_speed = drop_speed.min(config.endless.max_drop_speed);
    }
    let view = {
        let (mut low, mut high, mut n) = (f32::MAX, f32::MIN, 0);
        for (_, s, _) in towers.iter() {
            (low, high, n) = (low.min(s.stack_top), high.max(s.stack_top), n + 1);
        }
        tower_view(&config, &game, low, high, n)
    };

    for (tower, mut state, mut input) in &mut towers {
        let left_pressed = std::mem::take(&mut input.left_pressed);
        let right_pressed = std::mem::take(&mut input.right_pressed);
        let nudge_left = std::mem::take(&mut input.nudge_left_pressed);
        let nudge_right = std::mem::take(&mut input.nudge_right_pressed);
        let rotate_cw = std::mem::take(&mut input.rotate_pressed);
        let rotate_ccw = std::mem::take(&mut input.rotate_ccw_pressed);

        // ponytail: the original only stops dealing bricks for Endless Race's end sweep; the
        // brick in hand stops here too rather than land on the stilled tower.
        if game.phase != Phase::Playing
            || state.status != TowerStatus::Building
            || state.pacer_end.is_some()
        {
            continue;
        }
        let Some(entity) = state.active else { continue };
        let Ok((mut control, brick, position, rotation)) = bricks.get_mut(entity) else {
            state.active = None;
            continue;
        };
        // A curse wave just started: the brick rises near the top of the view
        // (eased in, passing through everything), hangs there, shakes the whole
        // time, and takes no input until it is let go.
        if state.wave_hold > 0.0 {
            let rules = &config.survival.waves;
            state.wave_hold = (state.wave_hold - dt).max(0.0);
            let elapsed = rules.lift + rules.hold - state.wave_hold;
            let t = (elapsed / rules.lift.max(0.01)).min(1.0);
            let (from, to) = state.wave_lift;
            let at = Vec2::new(state.wave_hold_x, from + (to - from) * t * t);
            if state.wave_hold > 0.0 {
                // Only the picture shakes (drawn by the visuals), smaller on an
                // enlarged brick.
                control.picture_shake = config.cells(rules.shake) / brick.scale.max(1.0);
            } else {
                control.picture_shake = 0.0;
                commands.entity(entity).remove::<ColliderDisabled>();
            }
            commands
                .entity(entity)
                .insert((Position(at), LinearVelocity::ZERO));
            continue;
        }
        // Carried back up by a spell (Brick.MoveUp: in-quad over sqrt(distance) / 5 s), then
        // it hangs 0.3 s before its colliders come back (the default post-cast effects).
        if let Some((from, to, t, secs)) = control.rise {
            let t = t + dt;
            let k = (t / secs.max(0.01)).min(1.0);
            let at = Vec2::new(position.0.x, from + (to - from) * k * k);
            control.rise = (k < 1.0).then_some((from, to, t, secs));
            if k >= 1.0 {
                control.hold = control.hold.max(config.magic.move_up_settle);
                commands.entity(entity).remove::<ColliderDisabled>();
            }
            commands.entity(entity).insert((Position(at), LinearVelocity::ZERO));
            continue;
        }
        // Caught by Slow's balloon: it hangs still a moment, shaking at first.
        if control.hold > 0.0 {
            control.hold = (control.hold - dt).max(0.0);
            control.hold_shake = (control.hold_shake - dt).max(0.0);
            control.hold_rumble = (control.hold_rumble - dt).max(0.0);
            control.picture_shake = if control.hold_shake > 0.0 {
                config.cells(config.magic.no_rotate_jump) / brick.scale.max(1.0)
            } else {
                0.0
            };
            commands.entity(entity).insert(LinearVelocity::ZERO);
            continue;
        }
        let (mut position, mut rotation) = (*position, *rotation);
        let shape = control.query_shape.clone();
        let filter = tower_filter(tower.index).with_excluded_entities([entity]);
        let fits = |pos: Vec2, rot: Rotation| {
            spatial
                .shape_intersections(&shape, pos, rot.as_radians(), &filter)
                .is_empty()
        };
        // The steering shape is inset, so like the original's BrickMovementController.Move
        // it looks the inset further and stops that much short: the full-size brick then
        // just touches what it met (stopping at the shrunk shape's contact sank it 0.15 cells
        // into its neighbours, and the physics popped it out by a varying amount).
        let inset = config.cells(config.steer_inset) * brick.scale;
        let cast = |from: Vec2, rot: Rotation, dir: Dir2, max: f32| {
            spatial
                .cast_shape(
                    &shape,
                    from,
                    rot.as_radians(),
                    dir,
                    &ShapeCastConfig::from_max_distance(max + inset),
                    &filter,
                )
                .map(|mut hit| {
                    hit.distance -= inset;
                    hit
                })
        };
        let reach = config.cells(config.lane_reach(state.half_width));
        let in_lane = |x: f32| (x - tower.origin_x).abs() <= reach;
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
                nudged.write(BrickNudged { brick: entity, dir: sign });
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
        control.refused = (control.refused - dt).max(0.0);
        if !control.mods.no_rotate && control.mods.auto_rotate.is_none() {
            if rotate_cw {
                turns.push(-90.0);
            }
            if rotate_ccw {
                turns.push(90.0);
            }
        } else if control.mods.no_rotate && (rotate_cw || rotate_ccw) {
            // No Rotate: the turn is refused with a shake.
            control.refused = config.magic.no_rotate_shake;
        }
        if let Some(cycle) = control.mods.auto_rotate {
            // Auto Rotate winds back, swings 90° net the other way, then waits.
            let before = auto_rotate_angle(&config.magic, cycle, control.auto_rotate_timer);
            control.auto_rotate_timer += dt;
            let after = auto_rotate_angle(&config.magic, cycle, control.auto_rotate_timer);
            if after != before {
                turns.push(after - before);
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
        let magic = &config.magic;
        let pressed = input.fast_fall_held && !control.down_before;
        control.down_before = input.fast_fall_held;
        control.speed = if control.mods.slow {
            // SlowBrickModifier: a press lets it fall up to 6 again, easing back to 3.
            if pressed {
                control.slow_cap = magic.slow_fast_fall_speed;
            }
            control.slow_cap = (control.slow_cap - magic.slow_fast_ease * dt).max(magic.slow_fast_floor);
            if input.fast_fall_held {
                (control.speed.max(base) + config.drop_acceleration * dt).min(control.slow_cap)
            } else {
                (control.speed - config.drop_deceleration * dt).max(base).min(magic.slow_fall_speed)
            }
        } else if input.fast_fall_held {
            (control.speed.max(base) + config.drop_acceleration * dt).min(config.drop_max_speed)
        } else {
            (control.speed - config.drop_deceleration * dt).max(base)
        };
        if control.mods.fast > 0.0 && !control.mods.slow {
            control.speed = control.speed.max(base + control.mods.fast);
        }
        // Auto Build's bricks fall at one speed (its BrickControl min, max and drop speeds).
        if control.mods.auto_build {
            control.speed = magic.auto_build_speed;
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
                placed_brick.insert((crate::spells::IvyPending, crate::spells::IvyBrick));
            }
            if control.mods.comes_back() {
                placed_brick.insert(Cursed {
                    mods: control.mods,
                    times: control.comebacks,
                });
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
            state.last_placed_at = game.elapsed;
            state.last_placed_mid_air = cast(position.0, rotation, Dir2::NEG_Y, c).is_none();
            state.placed_count += 1;
            commands.entity(entity).insert(Placed(state.placed_count));
            if control.mods.auto_build {
                commands.entity(entity).insert(AutoBuilt);
            }
            continue;
        }

        // Steered off the side and past the kill line (out of view in Survival and Puzzle):
        // it counts as dropped.
        let gone = if lost_out_of_view(&game) {
            let at = Vec2::new(position.0.x - tower.origin_x, position.0.y) / c;
            out_of_view(&config, &game, view, at, -drop_speed, brick.scale, false)
        } else {
            position.0.y < -config.cells(config.kill_depth)
        };
        if gone {
            let falling = Vec2::NEG_Y * config.cells(drop_speed);
            sink(&mut commands, &config, entity, falling, 0.0, true, !lost_out_of_view(&game));
            state.active = None;
            let life = state.lose_brick(&config);
            lost.write(BrickLost {
                tower: tower.index,
                at: position.0,
                life,
            });
            // BrickDroppedChecker (Race only): a cursed or Mystery brick dropped while steered
            // is dealt again as the next brick.
            if game.mode.rules() == ModeKind::Race
                && control.mods.comes_back()
                && control.comebacks < config.magic.respawn_limit
            {
                state.respawn = Some(Respawn {
                    piece: brick.piece,
                    mods: control.mods,
                    times: control.comebacks + 1,
                    from: position.0,
                });
            }
            continue;
        }

        control.drop_distance =
            cast(position.0, rotation, Dir2::NEG_Y, config.cells(200.0)).map(|hit| hit.distance);
        commands.entity(entity).insert((position, rotation));
    }
}

/// Race Pro wind, as in the original: rows of wind 1 cell apart down from the top of
/// each tower, each pushing the first brick it meets on the windward side. Lower bricks
/// shelter behind the tower; the steered brick and bubbled bricks are never pushed.
#[allow(clippy::type_complexity)]
fn blow_wind(
    config: Res<GameConfig>,
    game: Res<Match>,
    spatial: SpatialQuery,
    towers: Query<(&Tower, &TowerState)>,
    skip: Query<(), Or<(With<ActiveBrick>, With<crate::spells::Bubbled>)>>,
    mut gusts: ResMut<WindGusts>,
) {
    gusts.0.clear();
    let Some((low, high)) = config.race.wind else {
        return;
    };
    if game.wind == 0.0 || game.phase == Phase::Menu {
        return;
    }
    let rules = &config.race;
    let swing = (game.elapsed * std::f32::consts::TAU / rules.wind_period.max(0.01)).sin();
    let strength = low + (high - low) * (swing + 1.0) * 0.5;
    let along = if game.wind > 0.0 {
        Dir2::X
    } else {
        Dir2::NEG_X
    };
    for (tower, state) in &towers {
        for row in 0..rules.wind_rows {
            let y = state.height - 0.5 - row as f32;
            let from = Vec2::new(
                tower.origin_x - game.wind * config.cells(rules.wind_reach),
                config.cells(y),
            );
            let hit = spatial.cast_ray_predicate(
                from,
                along,
                config.cells(rules.wind_reach * 2.0),
                true,
                &tower_filter(tower.index),
                &|entity| !skip.contains(entity),
            );
            if let Some(hit) = hit {
                let force =
                    strength * rules.wind_force * (1.0 - rules.wind_row_falloff).powi(row as i32);
                let point = from + along * hit.distance;
                gusts
                    .0
                    .push((hit.entity, point, along * config.cells(force)));
            }
        }
    }
}

/// This step's wind pushes: the brick, where it's hit, and the force.
#[derive(Resource, Default)]
pub struct WindGusts(pub Vec<(Entity, Vec2, Vec2)>);

/// Applies the wind found by [`blow_wind`] to the bricks it hit.
fn push_wind(gusts: Res<WindGusts>, mut bricks: Query<(&RigidBody, Forces), With<Brick>>) {
    for &(entity, point, force) in &gusts.0 {
        if let Ok((body, mut forces)) = bricks.get_mut(entity)
            && body.is_dynamic()
        {
            forces.apply_force_at_point(force, point);
        }
    }
}

/// Each tower is its own physics world, like the original's: everything of tower `index` (its
/// bricks, base, floor, roof and pieces) is in this layer and only meets the same layer, so
/// another player's falling bricks never touch it.
pub fn tower_layer(index: usize) -> u32 {
    1 << (index + 1)
}

/// The query filter that only sees tower `index`'s world.
pub fn tower_filter(index: usize) -> SpatialQueryFilter {
    SpatialQueryFilter::from_mask(tower_layer(index))
}

/// The tower layer for a collider that has none yet: a brick by its tower, anything else (base,
/// floor, roof, pieces) by the nearest tower to where it is.
fn layer_for(
    towers: &Query<(Entity, &Tower)>,
    x: f32,
    brick: Option<&Brick>,
) -> Option<CollisionLayers> {
    let index = brick
        .and_then(|b| towers.get(b.tower).ok())
        .map(|(_, t)| t.index)
        .or_else(|| {
            towers
                .iter()
                .min_by(|a, b| (a.1.origin_x - x).abs().total_cmp(&(b.1.origin_x - x).abs()))
                .map(|(_, t)| t.index)
        })?;
    let layer = tower_layer(index);
    Some(CollisionLayers::new(layer, layer))
}

/// Puts every new collider into its tower's layer.
fn put_in_tower_layer(
    add: On<Add, Collider>,
    mut commands: Commands,
    towers: Query<(Entity, &Tower)>,
    things: Query<(&Transform, Option<&Brick>)>,
) {
    let Ok((transform, brick)) = things.get(add.entity) else {
        return;
    };
    if let Some(layers) = layer_for(&towers, transform.translation.x, brick) {
        commands.entity(add.entity).try_insert(layers);
    }
}

/// Colliders that appeared before their tower existed (the platform) join their layer here.
fn layer_stragglers(
    mut commands: Commands,
    towers: Query<(Entity, &Tower)>,
    things: Query<(Entity, &Transform, Option<&Brick>, &CollisionLayers), With<Collider>>,
) {
    for (entity, transform, brick, current) in &things {
        // Still the default layer: nobody has placed it yet.
        if current.memberships != LayerMask::DEFAULT {
            continue;
        }
        if let Some(layers) = layer_for(&towers, transform.translation.x, brick) {
            commands.entity(entity).try_insert(layers);
        }
    }
}

/// Measures each tower and removes bricks that fell off.
#[allow(clippy::type_complexity)]
fn track_towers(
    mut commands: Commands,
    config: Res<GameConfig>,
    game: Res<Match>,
    mut towers: Query<(Entity, &Tower, &mut TowerState)>,
    bricks: Query<
        (
            Entity,
            &Brick,
            &Position,
            &LinearVelocity,
            &Collider,
            &Rotation,
            Has<crate::spells::Bubbled>,
            Has<crate::spells::Ghosted>,
            Option<&Cursed>,
            Option<&AngularVelocity>,
            Has<ChildOf>,
        ),
        (Without<ActiveBrick>, Without<FloorBlock>),
    >,
    mut lost: MessageWriter<BrickLost>,
) {
    let kill_y = -config.cells(config.kill_depth);
    let view = {
        let (mut low, mut high, mut n) = (f32::MAX, f32::MIN, 0);
        for (_, _, s) in towers.iter() {
            (low, high, n) = (low.min(s.stack_top), high.max(s.stack_top), n + 1);
        }
        tower_view(&config, &game, low, high, n)
    };
    let by_view = lost_out_of_view(&game);

    let settled_speed = config.cells(0.5);
    let mut tops: Vec<(Entity, f32, bool)> = Vec::new();
    // Each counted part's tower, left and right edge (from the lane centre) and top, in cells.
    let mut spans: Vec<(Entity, f32, f32, f32)> = Vec::new();

    for (_, _, mut state) in &mut towers {
        state.height = 0.0;
        state.stack_top = 0.0;
        state.half_width = 0.0;
        state.all_settled = true;
    }
    for (entity, brick, position, velocity, collider, rotation, bubbled, ghosted, cursed, spin, in_ivy) in
        &bricks
    {
        let Ok((_, tower, mut state)) = towers.get_mut(brick.tower) else {
            continue;
        };
        // Tight bounds like Unity's collider.bounds: ColliderAabb is padded by the
        // speculative margin while a brick moves (the trial 9 laser 0.27 over).
        let aabb = collider.aabb(position.0, *rotation);
        // A lost brick of an Ivy cluster is counted once the cluster lets go.
        let gone = if by_view {
            let at = Vec2::new(position.0.x - tower.origin_x, position.0.y) / config.cell_size;
            let fall = velocity.0.y / config.cell_size;
            out_of_view(&config, &game, view, at, fall, brick.scale, ghosted)
        } else {
            position.0.y < kill_y
        };
        if gone && !in_ivy {
            let spin = spin.map_or(0.0, |s| s.0);
            sink(&mut commands, &config, entity, velocity.0, spin, !bubbled, !by_view);
            // Once the match is over (and losing towers fall apart), losses no
            // longer change the score.
            let playing = game.phase == Phase::Playing;
            let life = playing && state.lose_brick(&config);
            lost.write(BrickLost {
                tower: tower.index,
                at: position.0,
                life,
            });
            if !playing {
                continue;
            }
            state.tower_losses += 1;
            // BrickDroppedChecker (Race only): the brick placed last, lost within 2.5 s of
            // being placed, takes the place of the one in hand at once.
            // Race Pro's checker is forgiving: only a brick placed mid-air (nothing a cell
            // under it) comes back; the others take any brick lost in time.
            let pro = config.difficulty == Some(crate::config::Difficulty::Pro);
            let just_placed = state.last_placed == Some(entity)
                && game.elapsed - state.last_placed_at <= config.magic.respawn_window
                && (!pro || state.last_placed_mid_air);
            if game.mode.rules() == ModeKind::Race
                && just_placed
                && let Some(cursed) = cursed.filter(|c| c.times < config.magic.respawn_limit)
            {
                state.respawn = Some(Respawn {
                    piece: brick.piece,
                    mods: cursed.mods,
                    times: cursed.times + 1,
                    from: position.0,
                });
                if let Some(current) = state.active.take() {
                    commands.entity(current).despawn();
                }
            }
            continue;
        }
        let on_tower = (position.0.x - tower.origin_x).abs()
            <= config.cells(config.base_half_width(config.mode) + 2.0);
        // Bricks held up by a bubble or a ghost don't count toward the height.
        if !on_tower || aabb.min.y <= -config.cell_size || bubbled || ghosted {
            continue;
        }
        let settled = velocity.0.x.abs() < settled_speed && velocity.0.y.abs() < settled_speed;
        if settled {
            let reach = (aabb.max.x - tower.origin_x).max(tower.origin_x - aabb.min.x);
            state.half_width = state.half_width.max(reach / config.cell_size);
        }
        // DEBUG-TOOL: a trial's bricks over the laser, for comparing with the original.
        if config.trial.is_some() && aabb.max.y / config.cell_size > state.laser_base + 0.05 {
            info!("trial brick over laser: piece {:?} at ({:.2}, {:.2}) top {:.2} settled {}", brick.piece, position.0.x / config.cell_size, position.0.y / config.cell_size, aabb.max.y / config.cell_size, settled);
        }
        tops.push((brick.tower, aabb.max.y, settled));
        let c = config.cell_size;
        spans.push((
            brick.tower,
            (aabb.min.x - tower.origin_x) / c,
            (aabb.max.x - tower.origin_x) / c,
            aabb.max.y / c,
        ));
    }
    for (tower, top, settled) in tops {
        if let Ok((_, _, mut state)) = towers.get_mut(tower) {
            // Like the original, the height is the top of any placed brick,
            // settled or not.
            state.stack_top = state.stack_top.max(top / config.cell_size);
            state.height = state.stack_top;
            state.all_settled &= settled;
        }
    }
    // The top of each tower: its parts within `auto_build_top` of its height, the base too.
    let near = config.magic.auto_build_top;
    let base = config.base_half_width(config.mode);
    for (entity, _, mut state) in &mut towers {
        let floor = state.height - near;
        let base_span = (0.0 > floor).then_some((-base, base));
        state.top_span = spans
            .iter()
            .filter(|s| s.0 == entity && s.3 > floor)
            .map(|s| (s.1, s.2))
            .chain(base_span)
            .reduce(|a, b| (a.0.min(b.0), a.1.max(b.1)));
    }
}

/// Puzzle battles: once a tower goes out over the laser, its bricks above the laser
/// are zapped away like Undo's (the original's MaximumHeightGamePlayRule). Bricks
/// joined by Ivy stay: removing one from a cluster isn't safe.
#[allow(clippy::type_complexity)]
/// A brick of a tower that went out, turning to stone in this many seconds.
#[derive(Component)]
pub struct PetrifyIn(pub f32);

/// A Survival tower that goes out turns to stone from the bottom up (GameFinishController
/// petrifies every brick of a finished tower: PetrifyBrickModifier(instant) after a random
/// 0.05 to 0.1 s plus its height / 40).
fn petrify_out_towers(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    game: Res<Match>,
    mut towers: Query<&mut TowerState>,
    fresh: Query<
        (Entity, &Brick, &Position),
        (
            Without<ActiveBrick>,
            Without<FloorBlock>,
            Without<crate::spells::Stone>,
            Without<PetrifyIn>,
            Without<ChildOf>,
        ),
    >,
    mut waiting: Query<(Entity, &mut PetrifyIn)>,
) {
    let dt = time.delta_secs();
    for (entity, mut wait) in &mut waiting {
        wait.0 -= dt;
        if wait.0 <= 0.0 {
            commands
                .entity(entity)
                .remove::<PetrifyIn>()
                .insert((RigidBody::Static, crate::spells::Stone));
        }
    }
    if game.mode.rules() != ModeKind::Survival || game.phase != Phase::Playing {
        return;
    }
    let (low, high, per) = config.survival.out_petrify;
    for (entity, brick, position) in &fresh {
        let Ok(mut state) = towers.get_mut(brick.tower) else {
            continue;
        };
        if state.status != TowerStatus::Out {
            continue;
        }
        let r = (state.random() % 1000) as f32 / 1000.0;
        let height = position.0.y / config.cell_size;
        commands
            .entity(entity)
            .insert(PetrifyIn(low + (high - low) * r + height.max(0.0) / per.max(0.01)));
    }
}

fn zap_over_laser(
    mut commands: Commands,
    config: Res<GameConfig>,
    game: Res<Match>,
    towers: Query<&TowerState>,
    bricks: Query<
        (Entity, &Brick, &ColliderAabb),
        (
            Without<ActiveBrick>,
            Without<FloorBlock>,
            Without<crate::spells::Zapped>,
            Without<ChildOf>,
        ),
    >,
) {
    let battle = game.mode == ModeKind::Puzzle && config.trial.is_none();
    if !battle && game.mode != ModeKind::EndlessPuzzle {
        return;
    }
    for (entity, brick, aabb) in &bricks {
        let Ok(state) = towers.get(brick.tower) else {
            continue;
        };
        let crossed = if battle {
            state.status == TowerStatus::Out
        } else {
            state.hold_timer > 0.0
        };
        if crossed
            && aabb.max.y / config.cell_size > state.laser(&config) + 0.05
        {
            commands.entity(entity).insert(crate::spells::Zapped {
                timer: config.puzzle.zap_seconds,
                length: config.puzzle.zap_seconds,
                edge: config.puzzle.zap_edge,
            });
        }
    }
}

fn in_out_sine(t: f32) -> f32 {
    (1.0 - (std::f32::consts::PI * t.clamp(0.0, 1.0)).cos()) / 2.0
}

/// EndlessRaceGameModeIntroController: the pacer slides in from `intro_from` to its start
/// over `intro_seconds` (in-out sine); the intro runs it before the 3-2-1.
pub fn slide_pacer(rules: &crate::config::EndlessRules, state: &mut TowerState, dt: f32) {
    state.pacer_clock = (state.pacer_clock + dt).min(rules.intro_seconds);
    let t = in_out_sine(state.pacer_clock / rules.intro_seconds.max(f32::EPSILON));
    state.pacer = rules.intro_from + (rules.start_height - rules.intro_from) * t;
}

/// Moves Endless Race's pacer line (EndlessRaceGameModePlayController): it slides in if
/// no intro did, then rises; at a checkpoint it stops, eases down over the first part of
/// its wait (in-out sine), and rises again when the wait is over.
pub fn advance_pacer(rules: &crate::config::EndlessRules, state: &mut TowerState, mut dt: f32) {
    if state.pacer_clock < rules.intro_seconds {
        let left = rules.intro_seconds - state.pacer_clock;
        slide_pacer(rules, state, dt);
        if dt <= left {
            return;
        }
        dt -= left;
    }
    state.pacer_clock += dt;
    if state.pacer_pause > 0.0 {
        state.pacer_pause = (state.pacer_pause - dt).max(0.0);
        let since = rules.wait_seconds - state.pacer_pause;
        let t = in_out_sine(since / rules.ease_seconds.max(f32::EPSILON));
        state.pacer = state.pacer_from - rules.ease_drop * t;
        return;
    }
    if state.pacer_clock - dt <= rules.intro_seconds {
        state.pacer = rules.start_height;
    }
    state.pacer += state.pacer_speed * dt;
}

/// A brick Auto Build put down (its landing sounds are the auto-build hits).
#[derive(Component)]
pub struct AutoBuilt;

/// Endless Race's end (EndlessRaceGameModePlayController._StartEndAnimation): once the pacer
/// catches the tower no bricks come and the tower stands still, while the pacer rises to
/// `end_over` cells over the tower (out-sine), waits, and drops to `end_under` cells under
/// the camera's bottom (in-expo); then the run is won and the roof comes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PacerEnd {
    /// Seconds since the pacer caught the tower.
    pub clock: f32,
    from: f32,
    high: f32,
    low: f32,
}

impl PacerEnd {
    fn new(config: &GameConfig, state: &TowerState) -> Self {
        let rules = &config.endless;
        // SingleTowerCamera's bottom for this tower (visuals::frame_towers).
        let look = &config.hud;
        let floor = -config.camera_floor();
        let top = (state.stack_top + look.camera_height - look.camera_tower_room)
            .max(floor + look.camera_height);
        let bottom = (state.stack_top - look.camera_below_lowest)
            .max(floor)
            .min(top - look.camera_height);
        Self {
            clock: 0.0,
            from: state.pacer,
            high: state.height + rules.end_over,
            low: bottom - rules.end_under,
        }
    }

    /// The pacer's height `clock` seconds in, or `None` once the sweep is over.
    fn pacer(&self, rules: &crate::config::EndlessRules) -> Option<f32> {
        let (rise, wait, drop) = rules.end_times;
        let t = self.clock;
        if t < rise {
            let k = (t / rise.max(f32::EPSILON) * std::f32::consts::FRAC_PI_2).sin();
            Some(self.from + (self.high - self.from) * k)
        } else if t < rise + wait {
            Some(self.high)
        } else if t < rise + wait + drop {
            let x = (t - rise - wait) / drop.max(f32::EPSILON);
            let k = if x <= 0.0 { 0.0 } else { 2f32.powf(10.0 * x - 10.0) };
            Some(self.high + (self.low - self.high) * k)
        } else {
            None
        }
    }
}

/// The tower the Endless Race pacer caught stands still for the end sweep (Tower.StopPhysics),
/// the brick in hand too.
fn still_caught_towers(
    mut commands: Commands,
    towers: Query<(Entity, &TowerState)>,
    mut bricks: Query<(Entity, &Brick, &RigidBody, &mut LinearVelocity, &mut AngularVelocity)>,
) {
    for (tower, state) in &towers {
        if state.pacer_end.is_none() {
            continue;
        }
        for (entity, brick, body, mut linear, mut angular) in &mut bricks {
            if brick.tower == tower && *body != RigidBody::Static {
                commands.entity(entity).insert(RigidBody::Static);
                linear.0 = Vec2::ZERO;
                angular.0 = 0.0;
            }
        }
    }
}

/// A spell line picked up in Endless Race: the pacer stops, and rises faster after.
pub fn pacer_checkpoint(rules: &crate::config::EndlessRules, state: &mut TowerState) {
    state.pacer_stages += 1;
    state.pacer_from = state.pacer;
    state.pacer_pause = rules.wait_seconds.max(rules.ease_seconds);
    state.pacer_speed = (state.pacer_speed + state.pacer_step).min(rules.max_speed);
    state.pacer_step = (state.pacer_step + rules.step_growth).min(rules.max_step);
}

fn judge_match(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut game: ResMut<Match>,
    mut towers: Query<(&Tower, &mut TowerState)>,
) {
    if game.phase == Phase::Intro && game.mode == ModeKind::Endless {
        for (_, mut state) in &mut towers {
            slide_pacer(&config.endless, &mut state, time.delta_secs());
        }
    }
    if game.phase != Phase::Playing {
        return;
    }
    let dt = time.delta_secs();
    // SinglePlayerGameModePlayController: a trial's or Endless run's time (a race trial's
    // clock) stands still while a 3-2-1 runs.
    let single = config.trial.is_some() || game.mode.is_endless();
    if !(single && towers.iter().any(|(_, s)| s.hold_timer > 0.0)) {
        game.elapsed += dt;
    }
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
                } else if state.bricks_used() >= config.survival.bricks_to_place {
                    // The last brick used (placed or lost): the 3-2-1 (CountDownEffect, as
                    // a race's hold), then the roof goes on.
                    state.hold_timer += dt;
                    if state.hold_timer >= config.survival.finish_countdown {
                        state.status = TowerStatus::Won;
                    }
                }
            }
            ModeKind::Endless => {
                state.best_height = state.best_height.max(state.height);
                // Caught: the pacer's end sweep, then the run is over, and like any finished
                // single-player run the tower gets its roof and FINISH (EndlessRaceGameMode's
                // ROOF and BASK states).
                if let Some(mut end) = state.pacer_end {
                    end.clock += dt;
                    state.pacer_end = Some(end);
                    match end.pacer(&config.endless) {
                        Some(pacer) => state.pacer = pacer,
                        None => {
                            state.pacer = config.endless.intro_from;
                            state.status = TowerStatus::Won;
                        }
                    }
                } else {
                    advance_pacer(&config.endless, &mut state, dt);
                    if state.pacer >= state.height {
                        state.pacer_end = Some(PacerEnd::new(&config, &state));
                    }
                }
            }
            ModeKind::EndlessSurvival => {
                // No brick limit: the run ends with the last life.
                state.invincible = (state.invincible - dt).max(0.0);
                state.best_height = state.best_height.max(state.height);
                // The last life ends the run with the roof (ROOF_02) and FINISH.
                if state.lives == Some(0) {
                    state.status = TowerStatus::Won;
                }
            }
            ModeKind::EndlessPuzzle => {
                // Levels of bricks, the laser rising between them; a brick over
                // the laser or a lost one ends the run.
                state.best_height = state.best_height.max(state.height);
                let rules = &config.endless_play;
                if let Some(t) = state.level_up.as_mut() {
                    *t += dt;
                    if *t >= rules.level_up_seconds() {
                        state.level_up = None;
                        state.laser_base += rules.laser_rise;
                        state.puzzle_level += 1;
                        state.level_end += rules.level_bricks(state.puzzle_level);
                        if config.magic.enabled && rules.level_spell.is_some() {
                            state.light_spell = rules.level_spell;
                            state.dark_spell = None;
                        }
                    }
                } else if state.hold_timer > 0.0 || state.height > state.laser(&config) + 0.05 {
                    // A brick over the laser is zapped (MaximumHeightGamePlayRule): no more
                    // bricks come and a 3-2-1 runs (CountDownEffect, as a race's hold); then
                    // the run is over, a win for the leaderboard: PuzzleRoofController puts no
                    // roof on but makes the wizard the winner, so the bask says FINISH.
                    state.hold_timer += dt;
                    if state.hold_timer >= config.race.hold_seconds {
                        state.status = TowerStatus::Won;
                    }
                } else if state.bricks_used() >= state.level_end && state.active.is_none() {
                    // EndlessPuzzleGameModePlayController levels up as soon as the level's
                    // bricks are used (placed or lost).
                    state.level_up = Some(0.0);
                    state.obstacles_placed = 0;
                }
            }
            ModeKind::Puzzle => {
                let rules = &config.puzzle;
                // In a puzzle trial a dropped brick fails it (1 life, trials.md).
                let trial_drop = config.trial.is_some() && state.bricks_dropped > 0;
                // A brick touching the laser stops that player's bricks at once; the
                // round's 3-2-1 runs, then the bricks are counted (judge_match).
                let slack = if config.trial.is_some() { rules.trial_laser_slack } else { 0.0 };
                if state.laser_count.is_none()
                    && !trial_drop
                    && state.height > state.laser(&config) + 0.05 + slack
                {
                    state.laser_count = Some(0.0);
                    info!(
                        "puzzle: a brick touched the laser (height {:.2}, laser {:.2}, {} placed, settled {})",
                        state.height,
                        state.laser(&config),
                        state.bricks_placed,
                        state.all_settled
                    );
                }
                if trial_drop && state.status == TowerStatus::Building {
                    info!(
                        "puzzle trial: a brick was lost ({} dropped, {} placed, height {:.2})",
                        state.bricks_dropped, state.bricks_placed, state.height
                    );
                }
                if trial_drop || state.laser_count.is_some() {
                    state.status = TowerStatus::Out;
                } else if config.trial.is_some() && state.hold_timer > 0.0 {
                    // SinglePlayerPuzzleGameModePlayController: no more bricks, the 3-2-1
                    // (CountDownEffect), then the trial is won.
                    state.hold_timer += dt;
                    if state.hold_timer >= config.race.hold_seconds {
                        state.status = TowerStatus::Won;
                    }
                } else if config.trial.is_some()
                    && state.bricks_used() >= rules.bricks
                    && state.active.is_none()
                    && state.all_settled
                {
                    // Its bricks all in and the tower still for `settle_seconds`.
                    state.settle_timer += dt;
                    if state.settle_timer > rules.settle_seconds {
                        state.hold_timer = f32::EPSILON;
                    }
                } else {
                    state.settle_timer = 0.0;
                }
            }
        }
        if state.status == TowerStatus::Out && state.out_at.is_none() {
            state.out_at = Some(game.elapsed);
        }
    }

    // A Puzzle round is counted once every touched tower's 3-2-1 has run.
    let counting = game.mode.rules() == ModeKind::Puzzle
        && towers.iter().any(|(_, s)| s.laser_count.is_some_and(|t| t < config.puzzle.count_seconds));
    for (_, mut state) in &mut towers {
        if let Some(t) = state.laser_count.as_mut() {
            *t += dt;
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
    if game.mode.rules() == ModeKind::Puzzle && count > 1 {
        // Everyone plays out their bricks; the tallest pile of bricks wins.
        winner = None;
        // A tie for the most bricks is a draw: no winner, and both lanes say DRAW.
        if building.is_empty() {
            let most = towers.iter().map(|(_, s)| s.bricks_placed).max().unwrap_or(0);
            let leaders: Vec<usize> = towers
                .iter()
                .filter(|(_, s)| s.bricks_placed == most)
                .map(|(t, _)| t.index)
                .collect();
            if leaders.len() == 1 {
                winner = leaders.first().copied();
            }
        }
    } else if winner.is_none() && count > 1 && building.len() == 1 {
        // Last tower standing: with two players the match ends when one goes out, and the
        // other gets its roof and WINNER! (Max's Survival clips); with more it goes on.
        winner = Some(building[0]);
    }
    // Over when someone wins, nobody is left, or every human is out (a Puzzle round
    // waits for its count).
    if !counting && (winner.is_some() || building.is_empty() || (humans > 0 && humans_building == 0)) {
        game.phase = Phase::Over;
        game.winner = winner;
    }
}

/// Endless Puzzle: once the laser has risen in a level-up, the blocks pop in one
/// by one, static and left out of the tower height.
fn place_obstacles(
    mut commands: Commands,
    config: Res<GameConfig>,
    game: Res<Match>,
    mut towers: Query<(Entity, &Tower, &mut TowerState)>,
) {
    if game.mode != ModeKind::EndlessPuzzle || game.phase != Phase::Playing {
        return;
    }
    let rules = &config.endless_play;
    for (entity, tower, mut state) in &mut towers {
        let Some(t) = state.level_up else { continue };
        let since = t - rules.level_wait.0 - rules.rise_seconds;
        let due = ((since / rules.obstacle_delay.max(0.001)).floor() as i64)
            .clamp(0, rules.obstacles as i64) as u32;
        if since < 0.0 || state.obstacles_placed >= due {
            continue;
        }
        // Every spot is drawn at the first block, so none repeats.
        let laser = state.laser_base + rules.laser_rise;
        let height = state.height;
        let spots = rules.obstacle_spots(height, laser, || {
            (state.random() % 10_000) as f32 / 10_000.0
        });
        for spot in spots
            .iter()
            .skip(state.obstacles_placed as usize)
            .take((due - state.obstacles_placed) as usize)
        {
            let look = rules
                .obstacle_looks
                .get(state.random() as usize % rules.obstacle_looks.len().max(1));
            let block = Brick {
                tower: entity,
                piece: 0,
                cells: vec![Vec2::ZERO],
                color: Color::srgb(0.62, 0.52, 0.40),
                scale: 1.0,
                mass: 1.0,
                ice: false,
            };
            let (collider, _) = brick_body(&config, &block);
            let at = Vec2::new(tower.origin_x + config.cells(spot.x), config.cells(spot.y));
            commands.spawn((
                block,
                FloorBlock {
                    part: FloorPart::Ledge(look.copied().unwrap_or(0)),
                },
                Obstacle,
                RigidBody::Static,
                collider,
                Friction::new(config.brick_friction),
                Transform::from_translation(at.extend(0.0)),
            ));
        }
        state.obstacles_placed = due;
    }
}

/// A trial's enemy lane, right of the player's: the race house and a rival tower
/// (1-13) in race trials, empty otherwise. Scenery only, no tower of its own.
#[derive(Component, Debug)]
pub struct RivalLane {
    pub origin_x: f32,
    pub rival: Option<u32>,
}

/// An Endless Puzzle block put down at a level-up.
#[derive(Component, Debug)]
pub struct Obstacle;

/// True while this tower won and its roof isn't on yet: the match moments and
/// the popup wait for it.
/// The modes whose finished tower gets a roof: Race and Survival battles, and Endless Race
/// and Survival runs (LocalRoofController with ROOF_01 / ROOF_02).
fn roofed(mode: ModeKind) -> bool {
    matches!(
        mode,
        ModeKind::Race | ModeKind::Survival | ModeKind::Endless | ModeKind::EndlessSurvival
    )
}

pub fn roof_waiting(game: &Match, tower: &Tower, state: &TowerState) -> bool {
    roofed(game.mode)
        && game.phase == Phase::Over
        && game.winner == Some(tower.index)
        && state.roof != RoofStage::Done
}

/// The roof the winner of a Race or Survival match steers onto the tower.
#[derive(Component, Debug)]
pub struct RoofPiece {
    pub tower: Entity,
    /// `roofs.race` or `roofs.survival`.
    pub survival: bool,
    speed: f32,
    repeat_timer: f32,
    repeating: bool,
}

/// Where a tower's roof is: none yet, steered, resting before the match moves on,
/// or done.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum RoofStage {
    #[default]
    None,
    Steering,
    Resting(f32),
    Done,
}

/// The winner of a Race or Survival match gets a roof to place, as in the
/// original: it comes from the brick spawn height over the lane centre, moves and
/// nudges like a brick but never turns, falls at `roofs.drop_speed` (faster while
/// held), and the winner's tower stops moving meanwhile. The first touch sets it
/// down; `roofs.rest_seconds` later the match moves on.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn steer_roofs(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    game: Res<Match>,
    spatial: SpatialQuery,
    mut towers: Query<(Entity, &Tower, &mut TowerState, &mut PlayerInput)>,
    mut roofs: Query<(Entity, &mut RoofPiece, &Position, &Collider)>,
    bricks: Query<(Entity, &Brick, &RigidBody), Without<RoofPiece>>,
    mut placed: MessageWriter<RoofPlaced>,
) {
    let dt = time.delta_secs();
    let rules = &config.roofs;
    let roofed = roofed(game.mode);
    for (entity, tower, mut state, mut input) in &mut towers {
        match state.roof {
            RoofStage::None => {
                if !roofed || game.phase != Phase::Over || game.winner != Some(tower.index) {
                    continue;
                }
                if let Some(active) = state.active.take() {
                    commands.entity(active).despawn();
                }
                // The winner's tower stops moving for the roof.
                for (brick, b, body) in &bricks {
                    if b.tower == entity && *body == RigidBody::Dynamic {
                        commands.entity(brick).insert(RigidBody::Static);
                    }
                }
                let survival = game.mode.rules() == ModeKind::Survival;
                let boxes = if survival {
                    &rules.survival
                } else {
                    &rules.race
                };
                // Steered like a brick, its boxes shrunk by the steering inset on every side
                // (Brick's shrink amount 0.15), so its feet straddle a brick that is a little
                // off the grid.
                let parts = roof_collider(config.cell_size, boxes, config.steer_inset);
                let at = Vec2::new(
                    tower.origin_x,
                    config.cells(state.stack_top + config.spawn_gap),
                );
                commands.spawn((
                    RoofPiece {
                        tower: entity,
                        survival,
                        speed: rules.drop_speed,
                        repeat_timer: 0.0,
                        repeating: false,
                    },
                    RigidBody::Kinematic,
                    parts,
                    Transform::from_translation(at.extend(0.0)),
                    Position(at),
                    Rotation::IDENTITY,
                ));
                state.roof = RoofStage::Steering;
            }
            RoofStage::Steering => {
                let Some((roof_entity, mut roof, position, shape)) =
                    roofs.iter_mut().find(|(_, r, _, _)| r.tower == entity)
                else {
                    continue;
                };
                let left = std::mem::take(&mut input.left_pressed);
                let right = std::mem::take(&mut input.right_pressed);
                let nudge_left = std::mem::take(&mut input.nudge_left_pressed);
                let nudge_right = std::mem::take(&mut input.nudge_right_pressed);
                let filter = tower_filter(tower.index).with_excluded_entities([roof_entity]);
                // Like BrickMovementController.Move: look the inset further, stop that much short.
                let inset = config.cells(config.steer_inset);
                let cast = |from: Vec2, dir: Dir2, max: f32| {
                    spatial
                        .cast_shape(
                            shape,
                            from,
                            0.0,
                            dir,
                            &ShapeCastConfig::from_max_distance(max + inset),
                            &filter,
                        )
                        .map(|mut hit| {
                            hit.distance = (hit.distance - inset).max(0.0);
                            hit
                        })
                        .filter(|hit| hit.distance <= max)
                };
                let mut at = position.0;
                let reach = config.cells(config.lane_reach(state.half_width));
                // Moves: a step on press, repeating while held; nudges a full cell.
                let dir = (input.right_held || right) as i32 - (input.left_held || left) as i32;
                let mut step = left || right;
                if dir == 0 || step {
                    roof.repeat_timer = 0.0;
                    roof.repeating = false;
                } else {
                    roof.repeat_timer += dt;
                    let wait = if roof.repeating {
                        config.move_repeat_interval
                    } else {
                        config.move_repeat_delay
                    };
                    if roof.repeat_timer >= wait {
                        roof.repeat_timer = 0.0;
                        roof.repeating = true;
                        step = true;
                    }
                }
                let nudge = nudge_right as i32 - nudge_left as i32;
                for (go, sign, want) in [
                    (step && dir != 0, dir as f32, config.move_step),
                    (nudge != 0, nudge as f32, config.nudge_distance),
                ] {
                    let want = config.cells(want);
                    if go && (at.x + sign * want - tower.origin_x).abs() <= reach {
                        let along = if sign > 0.0 { Dir2::X } else { Dir2::NEG_X };
                        at.x += sign * cast(at, along, want).map_or(want, |hit| hit.distance);
                    }
                }
                roof.speed = if input.fast_fall_held {
                    (roof.speed + config.drop_acceleration * dt).min(config.drop_max_speed)
                } else {
                    (roof.speed - config.drop_deceleration * dt).max(rules.drop_speed)
                };
                let fall = config.cells(roof.speed) * dt;
                match cast(at, Dir2::NEG_Y, fall) {
                    Some(hit) => {
                        at.y -= hit.distance;
                        // LocalRoofController: on its first contact it is made physical (full
                        // size less Brick's 0.005 shrink, falling, mass 1) and joins the tower,
                        // which stays stopped; the physics settles it from there.
                        let survival = roof.survival;
                        let boxes = if survival { &rules.survival } else { &rules.race };
                        let solid = roof_collider(config.cell_size, boxes, 0.005);
                        let area: f32 = boxes.iter().map(|b| (b.x.1 - b.x.0 - 0.01) * (b.y.1 - b.y.0 - 0.01)).sum();
                        commands.entity(roof_entity).insert((
                            solid,
                            ColliderDensity(config.brick_mass / (area * config.cell_size * config.cell_size).max(f32::EPSILON)),
                            RigidBody::Dynamic,
                            Friction::new(config.brick_friction),
                            Restitution::new(config.brick_restitution),
                            AngularDamping(config.angular_damping),
                        ));
                        state.roof = RoofStage::Resting(0.0);
                        let bottom = shape.aabb(at, 0.0).min.y;
                        placed.write(RoofPlaced {
                            tower: entity,
                            at: Vec2::new(at.x, bottom),
                        });
                    }
                    None => at.y -= fall,
                }
                // Steered past the tower and out of sight: it counts as set down.
                // (It is gone, so nothing more goes to it: an insert after the despawn
                // panicked, Max's crash when steering the roof into the void.)
                if at.y < -config.cells(config.kill_depth) {
                    commands.entity(roof_entity).despawn();
                    state.roof = RoofStage::Resting(0.0);
                    continue;
                }
                commands.entity(roof_entity).insert(Position(at));
            }
            RoofStage::Resting(t) => {
                state.roof = if t + dt >= rules.rest_seconds {
                    RoofStage::Done
                } else {
                    RoofStage::Resting(t + dt)
                };
            }
            RoofStage::Done => {}
        }
    }
}

/// A roof's compound collider from its boxes (cells), each shrunk by `shrink` cells a side.
fn roof_collider(cell: f32, boxes: &[crate::config::BaseBox], shrink: f32) -> Collider {
    Collider::compound(
        boxes
            .iter()
            .map(|b| {
                let size = (Vec2::new(b.x.1 - b.x.0, b.y.1 - b.y.0) - Vec2::splat(2.0 * shrink)).max(Vec2::splat(0.01)) * cell;
                let centre = Vec2::new(b.x.0 + b.x.1, b.y.0 + b.y.1) * 0.5 * cell;
                (centre, Rotation::IDENTITY, Collider::rectangle(size.x, size.y))
            })
            .collect(),
    )
}

/// The wizard (0 for wizard 1) and brick pack a tower is drawn with. Looks only.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct TowerSkin {
    pub wizard: usize,
    pub pack: crate::config::BrickPack,
}

/// Final placings, first place first: the winner, then towers still standing
/// (by height in Race, lives then bricks in Survival, bricks in Puzzle, best
/// height in Endless), then towers that went out, the last to go out first.
pub fn placings(game: &Match, towers: &[(usize, &TowerState)]) -> Vec<usize> {
    let mut order: Vec<(usize, &TowerState)> = towers.to_vec();
    let score = |s: &TowerState| match game.mode {
        ModeKind::Race => s.height,
        ModeKind::Survival => s.lives.unwrap_or(0) as f32 * 1000.0 + s.bricks_placed as f32,
        ModeKind::Puzzle | ModeKind::EndlessPuzzle => s.bricks_placed as f32,
        ModeKind::EndlessSurvival => s.bricks_placed as f32,
        ModeKind::Endless => s.best_height,
    };
    let key = |(index, s): &(usize, &TowerState)| {
        let won = game.winner == Some(*index);
        let standing = s.status != TowerStatus::Out;
        (won, standing, s.out_at.unwrap_or(0.0), score(s))
    };
    order.sort_by(|a, b| {
        let (ka, kb) = (key(a), key(b));
        kb.0.cmp(&ka.0)
            .then(kb.1.cmp(&ka.1))
            .then(kb.2.total_cmp(&ka.2))
            .then(kb.3.total_cmp(&ka.3))
            .then(a.0.cmp(&b.0))
    });
    order.into_iter().map(|(i, _)| i).collect()
}

/// DOTween's InOutElastic with its default amplitude (1.70158) and period, `t` in 0..1.
pub fn in_out_elastic(t: f32) -> f32 {
    use std::f32::consts::TAU;
    if t <= 0.0 {
        return 0.0;
    }
    if t >= 1.0 {
        return 1.0;
    }
    let (a, p) = (1.70158f32, 0.45f32);
    let s = p / TAU * (1.0 / a).asin();
    let t = t * 2.0 - 1.0;
    if t < 0.0 {
        -0.5 * a * 2f32.powf(10.0 * t) * ((t - s) * TAU / p).sin()
    } else {
        a * 2f32.powf(-10.0 * t) * ((t - s) * TAU / p).sin() * 0.5 + 1.0
    }
}

/// How far Auto Rotate has turned a brick `t` seconds after it took hold, in
/// degrees (counter-clockwise): each `cycle` it winds back, then swings 90° net,
/// both with an in-out elastic ease.
pub fn auto_rotate_angle(magic: &crate::config::MagicRules, cycle: f32, t: f32) -> f32 {
    let (back, back_time) = magic.auto_rotate_back;
    let (swing, swing_time) = magic.auto_rotate_swing;
    let cycle = cycle.max(back_time + swing_time).max(0.01);
    let done = (t / cycle).floor();
    let c = t - done * cycle;
    let within = if c < back_time {
        -back * in_out_elastic(c / back_time.max(1e-4))
    } else if c < back_time + swing_time {
        -back + swing * in_out_elastic((c - back_time) / swing_time.max(1e-4))
    } else {
        swing - back
    };
    done * (swing - back) + within
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn survival_bricks_are_lost_as_they_leave_the_view() {
        let config = GameConfig::default();
        let game = Match { phase: Phase::Playing, mode: ModeKind::Survival, ..default() };
        // Towers 30 cells tall: the view (41.5 cells, its top 21.5 over them) ends at 10.
        let view = tower_view(&config, &game, 30.0, 30.0, 2);
        assert!((view.0 - 10.0).abs() < 1e-4);
        let falling = -5.0;
        // Lost once the brick's reach is under the bottom, while it falls...
        assert!(out_of_view(&config, &game, view, Vec2::new(0.0, 7.0), falling, 1.0, false));
        assert!(!out_of_view(&config, &game, view, Vec2::new(0.0, 8.0), falling, 1.0, false));
        // ...not while it rests or drifts down slowly, unless a ghost carries it off.
        assert!(!out_of_view(&config, &game, view, Vec2::new(0.0, 7.0), -1.0, 1.0, false));
        assert!(out_of_view(&config, &game, view, Vec2::new(0.0, 7.0), -1.0, 1.0, true));
        // Past a lane's side.
        assert!(out_of_view(&config, &game, view, Vec2::new(view.1 + 2.3, 15.0), falling, 1.0, false));
        // A Puzzle battle: the floor plus 4, falling or not.
        let puzzle = Match { mode: ModeKind::Puzzle, ..game.clone() };
        let view = tower_view(&config, &puzzle, 0.0, 0.0, 2);
        let edge = -config.camera_floor() + config.lost_puzzle_offset - config.lost_reach;
        assert!(out_of_view(&config, &puzzle, view, Vec2::new(0.0, edge - 0.1), 0.0, 1.0, false));
        assert!(!out_of_view(&config, &puzzle, view, Vec2::new(0.0, edge + 0.1), 0.0, 1.0, false));
    }
}
