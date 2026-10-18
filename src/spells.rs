//! Spells, as the original awards them. Each tower has a spell line above it;
//! touching it collects a pair, one light spell and one dark curse, and casting
//! either half throws the other away. Light spells help the caster; dark ones
//! curse every other player. Rubber-banding rules in `MagicRules` give players
//! who are behind stronger spells. See `docs/research/spell-awards.md` and
//! `docs/research/spells.md`.

use avian2d::dynamics::solver::islands::{BodyIslandNode, WakeBody};
use avian2d::prelude::*;
use bevy::prelude::*;

use crate::config::{
    AwardRule, Curse, Difficulty, GameConfig, MagicRules, ModeKind, SpellKind, WaveRules,
};
use crate::gameplay::{
    ActiveBrick, Brick, BrickMod, FloorBlock, Match, Phase, Placed, PlayerInput, Tower, TowerState,
    TowerStatus, brick_body, brick_collider, piece_def,
};

/// Bricks held together by Ivy: one rigid body whose children are the bricks.
#[derive(Component, Debug)]
pub struct IvyCluster;

/// A brick turned to stone: static, like the ground.
#[derive(Component, Debug)]
pub struct Stone;

/// A brick wrapped in vines.
#[derive(Component, Debug)]
pub struct Vined;

/// When a newly joined brick's vines show (the game's clock, seconds): IvyChildBrickModifier
/// shows them a tenth of a second per unit away from the Ivy brick, so they spread out.
#[derive(Component, Debug)]
pub struct VineDelay(pub f32);

/// A placed Ivy brick waiting to be vined to what it touches.
#[derive(Component, Debug)]
pub struct IvyPending;

/// The brick Ivy was cast on: it wears its vines from the moment the spell takes it
/// (IvyBrickModifier.Apply adds its _IVY child), whether or not it joins anything.
#[derive(Component, Debug)]
pub struct IvyBrick;

/// A brick being zapped away (Undo, or the Puzzle laser): the time left, how long the whole
/// zap takes and its dissolving edge's colour.
#[derive(Component, Debug)]
pub struct Zapped {
    pub timer: f32,
    pub length: f32,
    pub edge: [u8; 3],
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

/// A brick covered in grass (looks only): it starts to grow `delay` seconds after the cast,
/// 0.1 s per cell away from the brick it was cast on (GrassyEffect).
#[derive(Component, Debug)]
pub struct Grassy {
    pub delay: f32,
}

/// A Minibase tower piece: which of `MagicRules::minibase_pieces` it is.
#[derive(Component, Debug, Clone, Copy)]
pub struct TowerPiece(pub usize);

/// A tower piece's collider from its shape: the polygon cut into convex parts, its boxes and
/// its circles.
pub fn tower_piece_collider(piece: &crate::config::TowerPieceShape, cell: f32) -> Collider {
    let v = |(x, y): (f32, f32)| Vec2::new(x, y) * cell;
    // A polygon piece (Race's) is its convex decomposition, itself a compound; compounds don't
    // nest, so it stands alone.
    if piece.polygon.len() >= 3 {
        let points: Vec<Vec2> = piece.polygon.iter().map(|&p| v(p)).collect();
        let n = points.len() as u32;
        let edges = (0..n).map(|i| [i, (i + 1) % n]).collect();
        return Collider::convex_decomposition(points, edges);
    }
    let mut parts: Vec<(Vec2, Rotation, Collider)> = Vec::new();
    for &(at, size) in &piece.boxes {
        parts.push((v(at), Rotation::IDENTITY, Collider::rectangle(size.0 * cell, size.1 * cell)));
    }
    for &(at, r) in &piece.circles {
        parts.push((v(at), Rotation::IDENTITY, Collider::circle(r * cell)));
    }
    Collider::compound(parts)
}

/// Extra Life's orb (SpawnExtraLifeEffect's FloatingSpellOrb). Touch it with one of your
/// bricks for a life. It bobs up and down while it drifts right, and is gone once it is
/// 8 cells right of its tower.
#[derive(Component, Debug)]
pub struct LifeOrb {
    pub tower: Entity,
    /// Cells a second.
    pub velocity: Vec2,
    pub ascending: bool,
}

/// FloatingSpellOrb: pushed (1, 25) while rising until it climbs at 4 cells/s, then
/// (1, 10) until it sinks at 4, on top of gravity; leaves past 8 cells right. Its trigger
/// circle is 1.2 cells across the middle (radius).
const ORB_PUSH_UP: Vec2 = Vec2::new(1.0, 25.0);
const ORB_PUSH_DOWN: Vec2 = Vec2::new(1.0, 10.0);
const ORB_TURN_SPEED: f32 = 4.0;
const ORB_GONE_X: f32 = 8.0;
const ORB_RADIUS: f32 = 1.2;

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
    let lines_off = game.mode.rules() == ModeKind::Puzzle
        || (game.mode == ModeKind::EndlessSurvival
            && config.endless_play.survival_light.is_empty());
    // A trial has its own lines, or none at all.
    let trial_lines = config
        .trial
        .and_then(|i| config.trials.get(i))
        .map(|t| t.lines.clone());
    if !magic.enabled
        || game.phase != Phase::Playing
        || lines_off
        || matches!(trial_lines, Some(None))
    {
        return;
    }
    let trial_lines = trial_lines.flatten();
    let spacing = match &trial_lines {
        Some(lines) => lines.every * (1.0 + 0.25 * game.players.saturating_sub(1) as f32),
        None => magic.spacing(game.players),
    };
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
        if trial_lines
            .as_ref()
            .is_some_and(|l| state.lines_reached as usize >= l.spells.len())
        {
            continue;
        }
        let Some(mut line) = state.spell_line else {
            // Endless Race's lines are checkpoints at set heights.
            state.spell_line = Some(if game.mode == ModeKind::Endless {
                config.endless.line_height(state.lines_reached)
            } else {
                state.next_line.max(height + spacing)
            });
            state.line_wait = magic.wait(game.players, config.difficulty.unwrap_or_default());
            continue;
        };
        // Race lines slide down to the tower; Endless Race's stay at their checkpoint.
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
        if game.mode == ModeKind::Endless {
            crate::gameplay::pacer_checkpoint(&config.endless, &mut state);
        }
        // A trial's lines give their spells in order; "(random)" ones are cast at once.
        if let Some(lines) = &trial_lines {
            let (spell, now) = lines.spells[state.lines_reached as usize - 1];
            state.light_spell = Some(spell);
            state.dark_spell = None;
            state.auto_cast_light = now;
            continue;
        }
        // Endless Race's award rule is EndlessRaceGameModeFactory's InstantSpellPicker: Ivy,
        // Undo, Ivy, Undo, Petrify at random, never the same twice in a row.
        let finish_gap = (game.mode == ModeKind::Race).then(|| game.finish_height - height);
        let behind = leader - height;
        let hurt = state.lives.is_some_and(|l| l < config.survival.lives);
        let Some(rule) = magic.award_rules.iter().find(|r| {
            rule_holds(
                r,
                game.mode.rules(),
                config.difficulty.unwrap_or_default(),
                hurt,
                finish_gap,
                behind,
            )
        }) else {
            continue;
        };
        award(
            &mut state,
            magic,
            rule,
            opponents && config.dark_magic(game.mode),
        );
    }
}

/// Survival Pro's curse waves: every so often a fixed set of curses hits every
/// tower for a while, cursing the brick in hand and each new brick (Mist fogs
/// the tower once; Ghost carries off the last placed brick with the brick in
/// hand and again with each new one).
#[derive(Default)]
pub(crate) struct WaveClock {
    round: u64,
    next_in: f32,
    /// A wave counted in bricks is running: the gap to the next starts when it ends.
    counting: bool,
    deck: WaveDeck,
    rng: u64,
}

impl WaveClock {
    fn random(&mut self) -> u64 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        self.rng
    }

    /// Whole seconds from `lo` up to but not including `hi` (or `lo` when they meet).
    fn seconds(&mut self, (lo, hi): (u32, u32)) -> f32 {
        let span = hi.saturating_sub(lo).max(1) as u64;
        (lo as u64 + self.random() % span) as f32
    }
}

/// Which wave comes next: the level, how many waves it has had, and the waves
/// it hasn't used yet.
#[derive(Default, Debug)]
pub struct WaveDeck {
    level: usize,
    dealt: u32,
    left: Option<Vec<usize>>,
}

impl WaveDeck {
    /// Draws the next wave as (level, index into that level).
    pub fn draw(
        &mut self,
        rules: &WaveRules,
        mut roll: impl FnMut() -> u64,
    ) -> Option<(usize, usize)> {
        let last = rules.levels.len().checked_sub(1)?;
        let full = |level: usize| (0..rules.levels[level].len()).collect::<Vec<_>>();
        let mut left = self.left.take().unwrap_or_else(|| full(self.level));
        if self.dealt >= rules.waves_per_level.max(1) || left.is_empty() {
            self.level = (self.level + 1).min(last);
            self.dealt = 0;
            left = full(self.level);
        }
        if left.is_empty() {
            return None;
        }
        let index = left.remove((roll() % left.len() as u64) as usize);
        self.dealt += 1;
        self.left = Some(left);
        Some((self.level, index))
    }
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(crate) fn run_curse_waves(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    game: Res<Match>,
    mut clock: Local<WaveClock>,
    mut towers: Query<(Entity, &Tower, &mut TowerState)>,
    mut active: Query<(&mut ActiveBrick, &mut Brick)>,
    placed: Query<
        (
            Entity,
            &Brick,
            &Position,
            Option<&RigidBody>,
            Has<Stone>,
            Has<Vined>,
            Option<&crate::gameplay::Placed>,
            Has<Bubbled>,
        ),
        Without<ActiveBrick>,
    >,
    held: Query<&Position, With<ActiveBrick>>,
    ghostable: Query<
        (Entity, &Brick, &Placed, &RigidBody),
        (
            Without<ActiveBrick>,
            Without<Ghosted>,
            Without<Stone>,
            Without<Vined>,
        ),
    >,
) {
    let dt = time.delta_secs();
    for (entity, tower, mut state) in &mut towers {
        if state.wave_left > 0.0 {
            state.wave_left -= dt;
            if state.wave_left <= 0.0 {
                state.wave_mod = None;
                state.wave_ghost = false;
            }
        }
        // Each Ghost hit takes the newest placed brick that is still loose and
        // isn't ghosted, stone or ivy already.
        // One hit per tower per step, so the next one sees this one land.
        if state.ghosts_due > 0 {
            state.ghosts_due -= 1;
            if state.status != TowerStatus::Building {
                state.ghosts_due = 0;
                continue;
            }
            let Some((victim, ..)) = ghostable
                .iter()
                .filter(|(_, b, _, body)| b.tower == entity && **body == RigidBody::Dynamic)
                .max_by_key(|(_, _, placed, _)| **placed)
            else {
                state.ghosts_due = 0;
                continue;
            };
            let kept = state.last_placed;
            state.last_placed = Some(victim);
            cast_on_tower(
                &mut commands,
                &config,
                SpellKind::Ghost,
                entity,
                tower,
                &mut state,
                &placed,
            );
            state.last_placed = kept.filter(|&e| e != victim);
        }
    }
    let rules = &config.survival.waves;
    let on = game.phase == Phase::Playing
        && game.mode.rules() == ModeKind::Survival
        && config.survival.curse_waves
        && !rules.levels.is_empty();
    if !on {
        return;
    }
    if clock.round != game.round {
        // One seed for everyone, so every tower gets the same waves.
        *clock = WaveClock {
            round: game.round,
            rng: (config.seed ^ 0xC0FF_EE00 ^ game.round.wrapping_mul(0x9E37_79B9)) | 1,
            ..default()
        };
        clock.next_in = clock.seconds(rules.gap);
    }
    // The next wave's wait starts once a wave is over (WorldCurseSpawner._CurseWaveCompleted):
    // a counted wave ends with its last cursed brick.
    if clock.counting
        && towers
            .iter()
            .all(|(_, _, s)| s.status != TowerStatus::Building || s.wave_bricks == 0)
    {
        clock.counting = false;
        clock.next_in = clock.seconds(rules.gap);
    }
    clock.next_in -= dt;
    if clock.next_in > 0.0 {
        return;
    }
    let mut deck = std::mem::take(&mut clock.deck);
    let drawn = deck.draw(rules, || clock.random());
    clock.deck = deck;
    let Some((level, index)) = drawn else {
        return;
    };
    let wave = &rules.levels[level][index];
    // A wave lasts some seconds, or (Endless Survival) some bricks, the one in
    // hand first; the next wave's wait starts when it ends.
    let counted = wave.bricks > 0;
    let length = if counted {
        0.0
    } else {
        clock.seconds(wave.seconds)
    };
    if counted {
        clock.counting = true;
        clock.next_in = f32::INFINITY;
    } else {
        clock.next_in = length + clock.seconds(rules.gap);
    }
    let mut effect = BrickMod::default();
    for &spell in &wave.curses {
        if let Some(e) = brick_effect(spell, &config.magic) {
            effect.merge(&e);
        }
    }
    let cursed = effect != BrickMod::default();
    let ghost = wave.curses.contains(&SpellKind::Ghost);
    // A wave's Mist is the Mist spell's own (SpawnMistEffect), however the wave is counted.
    let magic_mist = config.magic.mist_seconds;
    for (_, _, mut state) in &mut towers {
        if state.status != TowerStatus::Building {
            continue;
        }
        state.wave_left = length;
        state.wave_bricks = wave.bricks.saturating_sub(1);
        // The brick in hand rises to a little under the top of the view, passing
        // through everything on the way, and is held there.
        if let Some(brick) = state.active
            && let Ok(at) = held.get(brick)
        {
            let top = config.cells(config.camera_top(state.stack_top) - rules.lift_below_top);
            commands
                .entity(brick)
                .insert((LinearVelocity::ZERO, ColliderDisabled));
            state.wave_hold = rules.lift + rules.hold;
            state.wave_hold_x = at.0.x;
            state.wave_lift = (at.0.y, top);
        }
        let more = !counted || state.wave_bricks > 0;
        if cursed {
            // The brick in hand, then every new one while the wave lasts.
            curse_bricks(&mut commands, &config, &mut state, &mut active, &effect, 1);
            state.wave_mod = more.then_some(effect);
        }
        if ghost {
            // Once for the brick in hand, then with every new brick.
            state.wave_ghost = more;
            state.ghosts_due += 1;
        }
        if wave.curses.contains(&SpellKind::Mist) {
            let mist = if counted { magic_mist } else { length };
            state.mist_timer = state.mist_timer.max(mist);
        }
    }
}

fn rule_holds(
    rule: &AwardRule,
    mode: ModeKind,
    difficulty: Difficulty,
    hurt: bool,
    finish_gap: Option<f32>,
    behind: f32,
) -> bool {
    rule.mode.is_none_or(|m| m == mode)
        && (rule.difficulties.is_empty() || rule.difficulties.contains(&difficulty))
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

/// Caster index logged for the enemy wizard of a trial, who has no tower.
pub const ENEMY_WIZARD: usize = usize::MAX;

/// Curses the trial's enemy wizard casts this step (src/trials.rs).
#[derive(Resource, Debug, Default)]
pub struct EnemyCasts(pub Vec<Curse>);

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(crate) fn cast_spells(
    mut commands: Commands,
    config: Res<GameConfig>,
    game: Res<Match>,
    mut log: ResMut<SpellLog>,
    mut enemy: ResMut<EnemyCasts>,
    mut towers: Query<(Entity, &Tower, &mut TowerState, &mut PlayerInput)>,
    mut active: Query<(&mut ActiveBrick, &mut Brick)>,
    placed: Query<
        (
            Entity,
            &Brick,
            &Position,
            Option<&RigidBody>,
            Has<Stone>,
            Has<Vined>,
            Option<&crate::gameplay::Placed>,
            Has<Bubbled>,
        ),
        Without<ActiveBrick>,
    >,
) {
    let mut casts = Vec::new();
    for (entity, tower, mut state, mut input) in &mut towers {
        let light = std::mem::take(&mut input.cast_light_pressed)
            | std::mem::take(&mut state.auto_cast_light);
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

    // A trial's enemy wizard casts on every tower; it has no tower of its own.
    for curse in enemy.0.drain(..) {
        casts.push((Entity::PLACEHOLDER, ENEMY_WIZARD, Cast::Dark(curse)));
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
pub(crate) fn brick_effect(spell: SpellKind, magic: &MagicRules) -> Option<BrickMod> {
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
        if effect.petrify || effect.ivy {
            brick.set_changed();
        }
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
            brick.ice = true;
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
            Option<&RigidBody>,
            Has<Stone>,
            Has<Vined>,
            Option<&crate::gameplay::Placed>,
            Has<Bubbled>,
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
                    length: magic.undo_seconds,
                    edge: magic.undo_edge,
                });
                state.last_placed = None;
                state.bricks_placed = state.bricks_placed.saturating_sub(1);
            }
        }
        SpellKind::ExtraLife => {
            let y = state.height.max(magic.life_orb_floor) + magic.life_orb_lift;
            commands.spawn((
                LifeOrb {
                    tower: target,
                    velocity: Vec2::ZERO,
                    ascending: true,
                },
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
            let survival = config.mode.look() == ModeKind::Survival;
            let lift = if survival { magic.minibase_lift_survival } else { magic.minibase_lift };
            let y = state.height.max(magic.minibase_floor) + lift;
            // SpawnTowerPieceEffect: the world's left or right piece, by the side it lands on.
            let shape = usize::from(survival) * 2 + usize::from(side > 0.0);
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
                ice: false,
            };
            let collider = magic
                .minibase_pieces
                .get(shape)
                .map(|piece| tower_piece_collider(piece, config.cell_size))
                .unwrap_or_else(|| brick_body(config, &ledge).0);
            let position = Vec2::new(tower.origin_x + config.cells(offset), config.cells(y));
            commands.spawn((
                ledge,
                TowerPiece(shape),
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
        SpellKind::Slow => {
            // Over the tower's top part (GetTopTowerpartFromTowerEffect), 2 cells over
            // the tower's height.
            let x = last.map_or(tower.origin_x, |b| b.2.0.x);
            commands.spawn((
                BalloonOrb {
                    tower: target,
                    age: 0.0,
                    leaving: None,
                },
                Transform::from_xyz(x, config.cells(state.height + 2.0), 0.0),
            ));
        }
        SpellKind::Bubble => {
            // BubbleEffect on the tower's last brick not already in a bubble (GetLastBrick
            // skipping Bubble and BubbleFail): a vined or stone brick only gets a bubble
            // that pops 0.7 s later (BubbleFailModifier); a loose one floats off.
            let pick = placed
                .iter()
                .filter(|b| b.1.tower == target && !b.7)
                .filter_map(|b| b.6.map(|order| (order.0, b)))
                .max_by_key(|(order, _)| *order)
                .map(|(_, b)| b);
            let loose = matches!(pick, Some((_, _, _, Some(RigidBody::Dynamic), false, false, ..)));
            if let Some((entity, ..)) = pick.filter(|_| !loose) {
                commands.entity(entity).insert(BubbleFail { timer: BUBBLE_FAIL_SECONDS });
            }
            if let Some((entity, ..)) = pick.filter(|_| loose) {
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
            if let Some((entity, _, at, Some(RigidBody::Dynamic), false, false, ..)) = last {
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
                    let away = position.0.distance(centre.0);
                    if brick.tower == target && away <= reach {
                        let delay = away / config.cell_size.max(0.01) * magic.grassy_delay_per_cell;
                        commands.entity(entity).insert(Grassy { delay });
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
    layers: Query<&CollisionLayers>,
) {
    for (entity, mut control, brick, position, rotation) in &mut bricks {
        if !std::mem::take(&mut control.reshape) {
            continue;
        }
        let shape = crate::gameplay::steer_shape(&config, brick);
        let (_, density) = brick_body(&config, brick);
        let filter = SpatialQueryFilter::from_mask(
            layers.get(entity).map_or(LayerMask::ALL, |l| l.memberships),
        )
        .with_excluded_entities([entity]);
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

/// Ivy, as in the original: a freshly placed Ivy brick and every brick of its tower
/// it touches (with the clusters those already belong to) become one rigid body,
/// static if any of them is. The bricks become the cluster's children and keep
/// their own colliders and looks.
#[allow(clippy::type_complexity)]
pub(crate) fn grow_ivy(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    spatial: SpatialQuery,
    bricks: Query<
        (
            Entity,
            &Brick,
            &Position,
            &Rotation,
            Option<&RigidBody>,
            Has<Stone>,
            Has<BodyIslandNode>,
            Has<IvyPending>,
            Option<&ChildOf>,
        ),
        (Without<ActiveBrick>, Without<FloorBlock>),
    >,
    clusters: Query<(&Children, &RigidBody), With<IvyCluster>>,
    layers: Query<&CollisionLayers>,
) {
    for (entity, brick, position, rotation, body, _, in_island, pending, _) in &bricks {
        if !pending {
            continue;
        }
        if body.is_none_or(|b| *b == RigidBody::Static) {
            commands.entity(entity).remove::<IvyPending>();
            continue;
        }
        // Wait until the physics has taken the brick in.
        if !in_island {
            continue;
        }
        commands.entity(entity).remove::<IvyPending>();
        let cell = config.cell_size * brick.scale;
        let reach = brick_collider(
            &brick.cells,
            cell,
            config.collider_scale + config.magic.ivy_reach / brick.scale,
        );
        let filter = SpatialQueryFilter::from_mask(
            layers.get(entity).map_or(LayerMask::ALL, |l| l.memberships),
        )
        .with_excluded_entities([entity]);
        let mut members: Vec<Entity> = vec![entity];
        let mut old_clusters: Vec<Entity> = Vec::new();
        for other in spatial.shape_intersections(&reach, position.0, rotation.as_radians(), &filter)
        {
            let Ok(b) = bricks.get(other) else { continue };
            if b.1.tower != brick.tower {
                continue;
            }
            match b.8.map(|c| c.parent()).filter(|p| clusters.contains(*p)) {
                Some(cluster) if !old_clusters.contains(&cluster) => {
                    old_clusters.push(cluster);
                    if let Ok((children, _)) = clusters.get(cluster) {
                        members.extend(children.iter().filter(|c| bricks.contains(*c)));
                    }
                }
                Some(_) => {}
                None => members.push(other),
            }
        }
        members.sort();
        members.dedup();
        if members.len() < 2 {
            continue;
        }
        // A cluster is static only when one of its bricks is solid (stone, zapped or
        // petrified), as IvyBrickModifier's `solid`; otherwise it is one dynamic body.
        let solid = members.iter().any(|&m| {
            bricks
                .get(m)
                .is_ok_and(|b| b.5 || b.4.is_some_and(|r| *r == RigidBody::Static))
        }) || old_clusters
            .iter()
            .any(|&c| clusters.get(c).is_ok_and(|(_, r)| *r == RigidBody::Static));
        let cluster = commands
            .spawn((
                IvyCluster,
                if solid {
                    RigidBody::Static
                } else {
                    RigidBody::Dynamic
                },
                *position,
                *rotation,
                Transform::from_translation(position.0.extend(0.0))
                    .with_rotation(Quat::from_rotation_z(rotation.as_radians())),
                LinearVelocity::ZERO,
                AngularVelocity::ZERO,
                AngularDamping(config.angular_damping),
                Visibility::default(),
            ))
            .id();
        let inverse = rotation.inverse();
        for &m in &members {
            let Ok((_, brick_m, p, r, .., child_of)) = bricks.get(m) else {
                continue;
            };
            // Bricks joining now show their vines after their distance / 10 s; the ones
            // already in a cluster keep theirs.
            if child_of.is_none() {
                let units = (p.0 - position.0).length() / config.cell_size.max(0.01);
                commands
                    .entity(m)
                    .insert(VineDelay(time.elapsed_secs() + units * config.magic.ivy_spread));
            }
            let local = inverse * (p.0 - position.0);
            let turn = (inverse * *r).as_radians();
            // A new collider, so no contact still names the brick as its body.
            commands
                .entity(m)
                .remove::<(RigidBody, Collider)>()
                .insert((
                    brick_body(&config, brick_m),
                    ChildOf(cluster),
                    Transform::from_translation(local.extend(0.0))
                        .with_rotation(Quat::from_rotation_z(turn)),
                    LinearVelocity::ZERO,
                    Vined,
                ));
        }
        // Their bricks have moved over; the old clusters go.
        for old in old_clusters {
            commands.entity(old).despawn();
        }
    }
}

/// Ivy clusters fall apart into loose bricks when one of their bricks is lost,
/// zapped, bubbled, carried off by a ghost or blown up, or only one is left; a
/// brick turned to stone makes its whole cluster static.
#[allow(clippy::type_complexity)]
pub(crate) fn loosen_ivy(
    mut commands: Commands,
    config: Res<GameConfig>,
    clusters: Query<
        (
            Entity,
            &Children,
            &RigidBody,
            &Position,
            &LinearVelocity,
            &AngularVelocity,
        ),
        With<IvyCluster>,
    >,
    members: Query<(
        &Position,
        &Rotation,
        Has<Brick>,
        Has<Stone>,
        Has<Zapped>,
        Has<Bubbled>,
        Has<Ghosted>,
        Has<Sensor>,
    )>,
) {
    let kill_y = -config.cells(config.kill_depth);
    for (cluster, children, body, centre, velocity, spin) in &clusters {
        let kids: Vec<_> = children
            .iter()
            .filter_map(|c| members.get(c).ok().map(|m| (c, m)))
            .collect();
        let stone = kids.iter().any(|(_, m)| m.3);
        let loose = kids.len() < 2
            || kids
                .iter()
                .any(|(_, (p, _, brick, _, zapped, bubbled, ghosted, sensor))| {
                    !brick || *zapped || *bubbled || *ghosted || *sensor || p.0.y < kill_y
                });
        if !loose {
            if stone && *body != RigidBody::Static {
                commands.entity(cluster).insert(RigidBody::Static);
            }
            continue;
        }
        for (child, (p, r, _, stone, ..)) in kids {
            let offset = p.0 - centre.0;
            commands.entity(child).remove::<ChildOf>().insert((
                if stone {
                    RigidBody::Static
                } else {
                    RigidBody::Dynamic
                },
                *p,
                *r,
                Transform::from_translation(p.0.extend(0.0))
                    .with_rotation(Quat::from_rotation_z(r.as_radians())),
                LinearVelocity(velocity.0 + Vec2::new(-offset.y, offset.x) * spin.0),
                AngularVelocity(spin.0),
            ));
        }
        commands.entity(cluster).despawn();
    }
}

/// Removes bricks once Undo's zap is over.
pub(crate) fn zap_bricks(
    mut commands: Commands,
    time: Res<Time>,
    mut zapped: Query<(Entity, &mut Zapped)>,
) {
    for (entity, mut zap) in &mut zapped {
        zap.timer -= time.delta_secs();
        if zap.timer <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// A bubble on a brick that can't float (vined or stone): it pops after a moment.
#[derive(Component, Debug)]
pub struct BubbleFail {
    pub timer: f32,
}

/// BubbleFailModifier's time out.
const BUBBLE_FAIL_SECONDS: f32 = 0.7;

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
    mut fails: Query<(Entity, &mut BubbleFail)>,
) {
    for (entity, mut fail) in &mut fails {
        fail.timer -= time.delta_secs();
        if fail.timer <= 0.0 {
            commands.entity(entity).remove::<BubbleFail>();
        }
    }
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

/// Slow's homing balloon (SpawnBalloonOrbEffect, HomingBalloonOrb): it pops up over the
/// tower's top, rises and drifts towards the brick in hand; the brick it touches is
/// caught and slowed. It gives up 8 cells under where bricks appear.
#[derive(Component, Debug)]
pub struct BalloonOrb {
    pub tower: Entity,
    /// Seconds since it appeared, and since it started to leave (shrinking away).
    pub age: f32,
    pub leaving: Option<f32>,
}

/// HomingBalloonOrb: 2.5 cells/s up, x eased towards the brick at 1 a second, active once
/// its 0.5 s pop-in is over, gone 0.5 s after it starts leaving; its trigger circle sits
/// 1.1 cells up with a 1.05 radius. The catch holds the brick 0.6 s, shaking the first 0.3.
const BALLOON_RISE: f32 = 2.5;
const BALLOON_EASE: f32 = 1.0;
const BALLOON_POP_IN: f32 = 0.5;
const BALLOON_LEAVE: f32 = 0.5;
const BALLOON_MARGIN: f32 = 8.0;
const BALLOON_REACH: (Vec2, f32) = (Vec2::new(0.03, 1.1), 1.05);
const BALLOON_HOLD: (f32, f32) = (0.6, 0.3);

#[allow(clippy::type_complexity)]
pub(crate) fn home_balloons(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    spatial: SpatialQuery,
    mut orbs: Query<(Entity, &mut BalloonOrb, &mut Transform), Without<ActiveBrick>>,
    states: Query<&TowerState>,
    mut active: Query<(&mut ActiveBrick, &Position)>,
) {
    let dt = time.delta_secs();
    let c = config.cell_size;
    for (entity, mut orb, mut transform) in &mut orbs {
        orb.age += dt;
        if let Some(left) = orb.leaving.as_mut() {
            *left += dt;
            if *left >= BALLOON_LEAVE {
                commands.entity(entity).despawn();
            }
            continue;
        }
        let Ok(state) = states.get(orb.tower) else {
            commands.entity(entity).despawn();
            continue;
        };
        transform.translation.y += BALLOON_RISE * c * dt;
        let brick = state.active.and_then(|b| active.get(b).ok().map(|(_, p)| (b, p.0)));
        if let Some((_, at)) = brick {
            let x = transform.translation.x;
            transform.translation.x = x + (at.x - x) * (dt * BALLOON_EASE).min(1.0);
        }
        if orb.age < BALLOON_POP_IN {
            continue;
        }
        let (offset, radius) = BALLOON_REACH;
        let centre = transform.translation.truncate() + offset * c;
        let touched = brick.filter(|(b, _)| {
            spatial
                .shape_intersections(&Collider::circle(radius * c), centre, 0.0, &SpatialQueryFilter::default())
                .contains(b)
        });
        if let Some((b, _)) = touched
            && let Ok((mut control, _)) = active.get_mut(b)
        {
            control.mods.slow = true;
            control.hold = BALLOON_HOLD.0;
            control.hold_shake = BALLOON_HOLD.1;
            commands.entity(entity).despawn();
            continue;
        }
        let spawn = config.cells(state.stack_top + config.spawn_gap);
        if transform.translation.y >= spawn - config.cells(BALLOON_MARGIN) {
            orb.leaving = Some(0.0);
        }
    }
}

/// Gives a life to the tower whose brick touches its heart orb.
pub(crate) fn collect_life_orbs(
    mut commands: Commands,
    config: Res<GameConfig>,
    spatial: SpatialQuery,
    time: Res<Time>,
    mut orbs: Query<(Entity, &mut LifeOrb, &mut Transform)>,
    bricks: Query<&Brick>,
    homes: Query<&Tower>,
    mut towers: Query<&mut TowerState>,
) {
    let dt = time.delta_secs();
    let orb = Collider::circle(config.cell_size * ORB_RADIUS);
    for (entity, mut life, mut transform) in &mut orbs {
        // FloatingSpellOrb.FixedUpdate: the push, gravity, and a tilt of 2 degrees per cell/s
        // of climb.
        let push = if life.ascending { ORB_PUSH_UP } else { ORB_PUSH_DOWN };
        life.velocity += (push - Vec2::Y * config.gravity) * dt;
        if life.ascending && life.velocity.y > ORB_TURN_SPEED {
            life.ascending = false;
        } else if !life.ascending && life.velocity.y < -ORB_TURN_SPEED {
            life.ascending = true;
        }
        transform.translation += (life.velocity * config.cell_size * dt).extend(0.0);
        transform.rotation = Quat::from_rotation_z((life.velocity.y * 2.0).to_radians());
        let home = homes.get(life.tower).map_or(0.0, |t| t.origin_x);
        if transform.translation.x - home > config.cells(ORB_GONE_X) {
            commands.entity(entity).despawn();
            continue;
        }
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
