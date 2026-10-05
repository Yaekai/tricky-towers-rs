//! The end of a match for every tower that didn't win, as the original plays it:
//! a short wait, the tower stops being solid and falls apart while some of its
//! bricks blow up one after another, then a rumble and a heavy thud. The Game
//! Completed popup waits until every loser is done (docs/research/match-moments.md).
//!
//! Headless-safe: this only changes bricks and writes [`CollapseEvent`]s; the
//! effects layer turns those into blasts, camera shakes and sounds.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::gameplay::{
    ActiveBrick, Brick, GameplaySystems, Match, Phase, Placed, Tower, TowerState, TowerStatus,
    xorshift,
};

pub struct CollapsePlugin;

impl Plugin for CollapsePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<CollapseEvent>().add_systems(
            FixedUpdate,
            (start_collapse, run_collapse)
                .chain()
                .after(GameplaySystems),
        );
    }
}

/// Where a losing tower is in its end sequence.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CollapseStage {
    /// Waiting before the tower gives way.
    Wait,
    /// Bricks are blowing up, one every `gap` seconds.
    Blast,
    /// The long rumble after the last blast.
    Rumble,
    Done,
}

/// A losing tower's end sequence; added to the tower when the match ends.
#[derive(Component, Debug)]
pub struct Collapse {
    pub stage: CollapseStage,
    /// Seconds left in the current stage (or before the next blast).
    pub clock: f32,
    /// Bricks still to blow up, next first.
    pub queue: Vec<Entity>,
    roll: u64,
}

/// Something the effects layer shows or plays for a collapsing tower.
#[derive(Message, Clone, Debug)]
pub struct CollapseEvent {
    pub tower: usize,
    pub kind: CollapseKind,
}

#[derive(Clone, Debug)]
pub enum CollapseKind {
    /// The sequence starts: the wait before the tower gives way.
    Begin,
    /// A brick blew up: where (world pixels), its turn, cells, colour and scale.
    Blast {
        at: Vec2,
        rotation: f32,
        cells: Vec<Vec2>,
        color: Color,
        scale: f32,
    },
    /// The rumble starts and lasts this many seconds.
    Rumble(f32),
    /// The heavy thud that ends the sequence.
    Thud,
}

/// True for a tower that plays no end sequence: the winner, or one that won on its own.
fn is_winner(game: &Match, tower: &Tower, state: &TowerState) -> bool {
    game.winner == Some(tower.index) || state.status == TowerStatus::Won
}

/// True once every losing tower has finished its end sequence (winners count as
/// done straight away), so the popup can open.
pub fn all_done<'a>(
    game: &Match,
    towers: impl IntoIterator<Item = (&'a Tower, &'a TowerState, Option<&'a Collapse>)>,
) -> bool {
    towers.into_iter().all(|(tower, state, collapse)| {
        is_winner(game, tower, state) || collapse.is_some_and(|c| c.stage == CollapseStage::Done)
    })
}

fn start_collapse(
    mut commands: Commands,
    config: Res<crate::config::GameConfig>,
    game: Res<Match>,
    mut towers: Query<(Entity, &Tower, &mut TowerState), Without<Collapse>>,
    mut events: MessageWriter<CollapseEvent>,
) {
    if game.phase != Phase::Over {
        return;
    }
    for (entity, tower, mut state) in &mut towers {
        if is_winner(&game, tower, &state) {
            continue;
        }
        events.write(CollapseEvent {
            tower: tower.index,
            kind: CollapseKind::Begin,
        });
        // The brick in hand goes down with the rest.
        if let Some(active) = state.active.take() {
            commands
                .entity(active)
                .remove::<ActiveBrick>()
                .insert(RigidBody::Dynamic);
        }
        commands.entity(entity).insert(Collapse {
            stage: CollapseStage::Wait,
            clock: config.moments.collapse_wait,
            queue: Vec::new(),
            roll: config.seed
                ^ game.round.wrapping_mul(0x9E37_79B9_7F4A_7C15)
                ^ (tower.index as u64 + 1),
        });
    }
}

#[allow(clippy::type_complexity)]
fn run_collapse(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<crate::config::GameConfig>,
    mut towers: Query<(Entity, &Tower, &TowerState, &mut Collapse)>,
    bricks: Query<(Entity, &Brick, &Position, &Rotation, Option<&Placed>)>,

    mut events: MessageWriter<CollapseEvent>,
) {
    let look = &config.moments;
    let dt = time.delta_secs();
    for (tower_entity, tower, state, mut collapse) in &mut towers {
        if collapse.stage == CollapseStage::Done {
            continue;
        }
        collapse.clock -= dt;
        match collapse.stage {
            CollapseStage::Wait if collapse.clock <= 0.0 => {
                // Every brick stops being solid; every n-th one (or all of them if
                // the wizard was already out) is queued to blow up, oldest first.
                let mut own: Vec<(Option<Placed>, Entity)> = bricks
                    .iter()
                    .filter(|(_, brick, ..)| brick.tower == tower_entity)
                    .map(|(entity, _, _, _, placed)| (placed.copied(), entity))
                    .collect();
                own.sort_by_key(|(placed, entity)| (placed.is_none(), *placed, *entity));
                for &(_, brick) in &own {
                    commands.entity(brick).insert(Sensor);
                }
                let every = if state.status == TowerStatus::Out {
                    1
                } else {
                    look.blast_every.max(1)
                };
                collapse.queue = own
                    .iter()
                    .step_by(every)
                    .take(look.blast_limit)
                    .map(|&(_, entity)| entity)
                    .collect();
                collapse.queue.reverse();
                collapse.stage = CollapseStage::Blast;
                collapse.clock = 0.0;
            }
            CollapseStage::Blast => {
                // Several blasts can fall in one step near the end, when the gaps
                // are shorter than a step.
                while collapse.clock <= 0.0 {
                    let Some(next) = collapse.queue.pop() else {
                        break;
                    };
                    // Bricks that already sank are out of play; skip them.
                    let Ok((entity, brick, position, rotation, _)) = bricks.get(next) else {
                        continue;
                    };

                    commands.entity(entity).despawn();
                    events.write(CollapseEvent {
                        tower: tower.index,
                        kind: CollapseKind::Blast {
                            at: position.0,
                            rotation: rotation.as_radians(),
                            cells: brick.cells.clone(),
                            color: brick.color,
                            scale: brick.scale,
                        },
                    });
                    // The gap shrinks as the queue runs down.
                    let r = (xorshift(&mut collapse.roll) % 10_000) as f32 / 10_000.0;
                    let (lo, hi) = look.blast_gap;
                    let left = collapse.queue.len().min(look.blast_gap_cap) as f32;
                    collapse.clock += left * (lo + (hi - lo) * r);
                }
                if collapse.queue.is_empty() {
                    collapse.stage = CollapseStage::Rumble;
                    collapse.clock = look.rumble.2;
                    events.write(CollapseEvent {
                        tower: tower.index,
                        kind: CollapseKind::Rumble(look.rumble.2),
                    });
                }
            }
            CollapseStage::Rumble if collapse.clock <= 0.0 => {
                collapse.stage = CollapseStage::Done;
                events.write(CollapseEvent {
                    tower: tower.index,
                    kind: CollapseKind::Thud,
                });
            }
            _ => {}
        }
    }
}
