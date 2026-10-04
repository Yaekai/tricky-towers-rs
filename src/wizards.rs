//! The wizards floating beside each tower, drawn from the original's parts and
//! animated by what happens to their tower (see docs/research/wizards.md and the
//! wizard names in docs/art-layout.md). Without staged wizard art nothing is drawn.

use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::art::Art;
use crate::config::{GameConfig, WizardLook};
use crate::gameplay::{Tower, TowerState, TowerStatus};
use crate::spells::SpellLog;

pub struct WizardsPlugin;

impl Plugin for WizardsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_wizards, react_wizards, move_wizards, pose_wizards).chain(),
        );
    }
}

/// What a wizard is doing; each is a set of part animations `<part>_<clip>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clip {
    Idle,
    Action,
    Cheer,
    CastSelf,
    Casting,
    LaughEvil,
    Startle,
    Dying,
}

impl Clip {
    pub fn name(self) -> &'static str {
        match self {
            Clip::Idle => "idle",
            Clip::Action => "action",
            Clip::Cheer => "cheer",
            Clip::CastSelf => "cast_self",
            Clip::Casting => "casting",
            Clip::LaughEvil => "laugh_evil",
            Clip::Startle => "startle",
            Clip::Dying => "dying",
        }
    }
}

/// What the wizard last saw of its tower, to notice events.
#[derive(Default)]
struct Seen {
    placed: u32,
    lives: Option<u32>,
    lines: u32,
    out: bool,
    casts: u64,
}

#[derive(Component)]
pub struct Wizard {
    tower: Entity,
    lane: usize,
    /// The art name, `wizard<lane + 1>`.
    key: String,
    pub clip: Clip,
    clip_time: f32,
    /// Clip to play when this one ends (Idle if none).
    then: Option<Clip>,
    time: f32,
    y: Option<f32>,
    action_countdown: u32,
    laugh_cooldown: f32,
    startle_cooldown: f32,
    shake: f32,
    seen: Seen,
    rng: u64,
}

impl Wizard {
    fn random(&mut self, lo: u32, hi: u32) -> u32 {
        self.rng = self
            .rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        lo + ((self.rng >> 33) as u32) % (hi.max(lo) - lo + 1)
    }
}

/// One part of a wizard (`wizard1_body`, ...).
#[derive(Component)]
struct WizardPart(String);

fn part_transform(art: &Art, image: &str, part: &str, cell: f32) -> (Anchor, Transform) {
    let entry = art.entries.get(image).or_else(|| art.entries.get(part));
    let place = art.entries.get(part);
    let (px, py) = entry.and_then(|e| e.pivot).unwrap_or((0.5, 0.5));
    let ppu = entry
        .and_then(|e| e.pixels_per_unit)
        .unwrap_or(100.0)
        .max(1.0);
    let (x, y) = place.and_then(|e| e.position).unwrap_or_default();
    (
        Anchor(Vec2::new(px - 0.5, py - 0.5)),
        Transform::from_xyz(x * cell, y * cell, 0.0).with_scale(Vec3::splat(cell / ppu)),
    )
}

fn spawn_wizards(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    towers: Query<(Entity, &Tower, &TowerState)>,
    wizards: Query<(Entity, &Wizard)>,
) {
    for (entity, wizard) in &wizards {
        if !towers.contains(wizard.tower) {
            commands.entity(entity).despawn();
        }
    }
    let cell = config.cell_size;
    for (tower_entity, tower, state) in &towers {
        if wizards.iter().any(|(_, w)| w.tower == tower_entity) {
            continue;
        }
        let key = format!("wizard{}", tower.index + 1);
        let Some(parts) = art.entries.get(&key).and_then(|e| e.parts.clone()) else {
            continue;
        };
        let look = &config.wizards;
        let mut wizard = Wizard {
            tower: tower_entity,
            lane: tower.index,
            key: key.clone(),
            clip: Clip::Idle,
            clip_time: 0.0,
            then: None,
            time: tower.index as f32 * 1.3,
            y: None,
            action_countdown: 0,
            laugh_cooldown: 0.0,
            startle_cooldown: 0.0,
            shake: 0.0,
            seen: Seen {
                placed: state.bricks_placed,
                lives: state.lives,
                lines: state.lines_reached,
                out: state.status == TowerStatus::Out,
                casts: 0,
            },
            rng: config.seed ^ ((tower.index as u64 + 1) * 0x9E37_79B9),
        };
        wizard.action_countdown = wizard.random(look.action_every.0, look.action_every.1);
        commands
            .spawn((
                wizard,
                Transform::from_xyz(tower.origin_x, 0.0, 20.0),
                Visibility::default(),
            ))
            .with_children(|wizard| {
                for (i, part) in parts.iter().enumerate() {
                    let name = format!("{key}_{part}");
                    let Some(image) = art
                        .image(&format!("{name}_idle"))
                        .or_else(|| art.image(&name))
                    else {
                        continue;
                    };
                    let (anchor, mut transform) = part_transform(&art, &name, &name, cell);
                    transform.translation.z = i as f32 * 0.01;
                    wizard.spawn((
                        WizardPart(name),
                        Sprite::from_image(image),
                        anchor,
                        transform,
                    ));
                }
            });
    }
}

/// Starts `clip` unless something more important is playing.
fn play(wizard: &mut Wizard, clip: Clip, then: Option<Clip>) {
    if wizard.clip == Clip::Dying {
        return;
    }
    wizard.clip = clip;
    wizard.clip_time = 0.0;
    wizard.then = then;
}

#[allow(clippy::too_many_arguments)]
fn react_wizards(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    art: Res<Art>,
    log: Res<SpellLog>,
    towers: Query<&TowerState>,
    mut wizards: Query<&mut Wizard>,
) {
    let look: &WizardLook = &config.wizards;
    let dt = time.delta_secs();
    for mut wizard in &mut wizards {
        let Ok(state) = towers.get(wizard.tower) else {
            continue;
        };
        let before = wizard.clip;
        let started = wizard.clip_time;
        wizard.laugh_cooldown -= dt;
        wizard.startle_cooldown -= dt;
        wizard.shake -= dt;

        if state.bricks_placed > wizard.seen.placed {
            wizard.action_countdown = wizard.action_countdown.saturating_sub(1);
            if wizard.action_countdown == 0 {
                play(&mut wizard, Clip::Action, None);
                wizard.action_countdown = wizard.random(look.action_every.0, look.action_every.1);
            }
        }
        if state.lines_reached > wizard.seen.lines {
            play(&mut wizard, Clip::Cheer, None);
        }
        // Spells: casting your own, or being hit by someone else's dark one.
        let new = (log.1 - wizard.seen.casts.min(log.1)) as usize;
        let casts: Vec<_> = log.0.iter().rev().take(new).cloned().collect();
        for cast in casts.iter().rev() {
            let dark = cast.spell.is_dark();
            if cast.caster == wizard.lane && (!dark || cast.target != wizard.lane) {
                if dark {
                    let laugh = wizard.laugh_cooldown <= 0.0;
                    if laugh {
                        wizard.laugh_cooldown = look.reaction_cooldown;
                    }
                    play(&mut wizard, Clip::Casting, laugh.then_some(Clip::LaughEvil));
                } else {
                    play(&mut wizard, Clip::CastSelf, None);
                }
            } else if dark && cast.target == wizard.lane && wizard.startle_cooldown <= 0.0 {
                wizard.startle_cooldown = look.reaction_cooldown;
                play(&mut wizard, Clip::Startle, None);
            }
        }
        if state.lives.unwrap_or(0) < wizard.seen.lives.unwrap_or(0) {
            // Losing a life always startles, cooldown or not.
            wizard.startle_cooldown = look.reaction_cooldown;
            wizard.shake = look.shake.2;
            play(&mut wizard, Clip::Startle, None);
        }
        let out = state.status == TowerStatus::Out;
        if out && !wizard.seen.out {
            play(&mut wizard, Clip::Dying, None);
        }
        wizard.seen = Seen {
            placed: state.bricks_placed,
            lives: state.lives,
            lines: state.lines_reached,
            out,
            casts: log.1,
        };

        // Say the line that goes with a clip that just started.
        if wizard.clip != Clip::Idle && (wizard.clip != before || wizard.clip_time < started) {
            let line = format!("{}_say_{}", wizard.key, wizard.clip.name());
            if let Some(sound) = art.sounds.get(&line) {
                commands.spawn((AudioPlayer(sound.clone()), PlaybackSettings::DESPAWN));
            }
        }
    }
}

fn move_wizards(
    time: Res<Time>,
    config: Res<GameConfig>,
    towers: Query<(&Tower, &TowerState)>,
    mut wizards: Query<(&mut Wizard, &mut Transform)>,
) {
    let look = &config.wizards;
    let dt = time.delta_secs();
    let solo = towers.iter().count() == 1;
    for (mut wizard, mut transform) in &mut wizards {
        let Ok((tower, state)) = towers.get(wizard.tower) else {
            continue;
        };
        wizard.time += dt;
        let target = state.height + look.lift;
        let y = wizard.y.get_or_insert(target);
        *y += (target - *y) * dt.min(1.0);
        let y = *y;
        let offset = if solo {
            look.solo_offset_x
        } else {
            look.offset_x
        };
        let t = wizard.time;
        let mut at = Vec2::new(
            offset + look.bob.0 * t.sin(),
            y + look.bob.1 * (2.0 * t).sin(),
        );
        if wizard.shake > 0.0 {
            at += Vec2::new(
                look.shake.0 * (t * 60.0).sin(),
                look.shake.1 * (t * 73.0).cos(),
            );
        }
        transform.translation.x = tower.origin_x + config.cells(at.x);
        transform.translation.y = config.cells(at.y);
        // Alone, the wizard floats on the left and turns to face the tower.
        transform.scale.x = if solo { -1.0 } else { 1.0 };
    }
}

/// Shows each part's frame for the current clip and moves on when it ends.
fn pose_wizards(
    time: Res<Time>,
    config: Res<GameConfig>,
    art: Res<Art>,
    mut wizards: Query<(&mut Wizard, &Children)>,
    mut parts: Query<(&WizardPart, &mut Sprite, &mut Anchor, &mut Transform)>,
) {
    let cell = config.cell_size;
    for (mut wizard, children) in &mut wizards {
        wizard.clip_time += time.delta_secs();
        let clip = wizard.clip.name();
        // The clip lasts as long as its longest part animation.
        let length = children
            .iter()
            .filter_map(|c| parts.get(c).ok())
            .filter_map(|(part, ..)| art.length(&format!("{}_{clip}", part.0)))
            .fold(0.0f32, f32::max);
        if wizard.clip != Clip::Idle && wizard.clip_time > length.max(0.3) {
            if wizard.clip == Clip::Dying {
                wizard.clip_time = length;
            } else {
                wizard.clip = wizard.then.take().unwrap_or(Clip::Idle);
                wizard.clip_time = 0.0;
            }
        }
        let clip = wizard.clip.name();
        let t = if wizard.clip == Clip::Idle {
            wizard.time
        } else {
            wizard.clip_time
        };
        for child in children.iter() {
            let Ok((part, mut sprite, mut anchor, mut transform)) = parts.get_mut(child) else {
                continue;
            };
            let animation = [format!("{}_{clip}", part.0), format!("{}_idle", part.0)]
                .into_iter()
                .find(|name| art.is_animation(name));
            let image = match &animation {
                Some(name) => art.frame_name(name, t).map(str::to_string),
                None => Some(part.0.clone()),
            };
            let Some(image) = image else {
                continue;
            };
            let Some(handle) = art.images.get(&image) else {
                continue;
            };
            if sprite.image != *handle {
                sprite.image = handle.clone();
                let (new_anchor, new_transform) = part_transform(&art, &image, &part.0, cell);
                *anchor = new_anchor;
                transform.translation.x = new_transform.translation.x;
                transform.translation.y = new_transform.translation.y;
                transform.scale = new_transform.scale;
            }
        }
    }
}
