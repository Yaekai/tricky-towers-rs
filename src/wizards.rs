//! The wizards floating beside each tower, drawn from the original's parts and
//! animated by what happens to their tower (see docs/research/wizards.md and the
//! wizard names in docs/art-layout.md). Without staged wizard art nothing is drawn.

use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::art::Art;
use crate::config::{GameConfig, ModeKind, WizardLook};
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
    LaughHard,
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
            Clip::LaughHard => "laugh_hard",
            Clip::Startle => "startle",
            Clip::Dying => "dying",
        }
    }
}

/// The original's voice line kind for a clip: a dark cast says a cast_other line,
/// and a wizard going out a startle line.
fn voice_kind(clip: Clip) -> &'static str {
    match clip {
        Clip::Casting => "cast_other",
        Clip::Dying => "startle",
        other => other.name(),
    }
}

/// Volume and pitch of wizard `number`'s voice lines of `kind`, from the original's
/// speech table: wizards 1-4 laugh at 0.8; packs 5 (0.65), 9 (0.3) and 10-14 (1.1) have
/// their own volume; guest characters 9-14 play at their own pitch, the rest at 1.13.
pub(crate) fn voice_mix(number: u32, kind: &str) -> (f32, f32) {
    let pitch = if number >= 9 { 1.0 } else { 1.13 };
    let volume = match number {
        5 => 0.65,
        9 => 0.3,
        10..=14 => 1.1,
        1..=4 if kind.starts_with("laugh") => 0.8,
        _ => 1.0,
    };
    (volume, pitch)
}

/// What the wizard last saw of its tower, to notice events.
#[derive(Default)]
struct Seen {
    placed: u32,
    lives: Option<u32>,
    lines: u32,
    out: bool,
    /// The tower finished (the Survival roof went on).
    won: bool,
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
    /// Seconds since the match was lost; the wizard shrinks away.
    vanish: f32,
    seen: Seen,
    rng: u64,
    /// The trial's enemy wizard: floats by the player's tower, only laughs.
    enemy: bool,
}

impl Wizard {
    /// The tower this wizard belongs to.
    pub fn tower(&self) -> Entity {
        self.tower
    }

    /// Whether this is a trial's enemy wizard rather than the tower's own.
    pub fn enemy(&self) -> bool {
        self.enemy
    }

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
    towers: Query<(
        Entity,
        &Tower,
        &TowerState,
        Option<&crate::gameplay::TowerSkin>,
    )>,
    wizards: Query<(Entity, &Wizard)>,
) {
    for (entity, wizard) in &wizards {
        if !towers.contains(wizard.tower) {
            commands.entity(entity).despawn();
        }
    }
    let cell = config.cell_size;
    for (tower_entity, tower, state, skin) in &towers {
        let has = |enemy: bool| {
            wizards
                .iter()
                .any(|(_, w)| w.tower == tower_entity && w.enemy == enemy)
        };
        // A trial's enemy wizard by the player's tower: Wizard02 in Survival trials,
        // Wizard03 in Race and Wizard04 in Puzzle (docs/research/trials.md).
        let enemy = config.trial.is_some() && tower.index == 0 && !has(true);
        if has(false) && !enemy {
            continue;
        }
        let number = if has(false) {
            match config.mode.rules() {
                ModeKind::Survival => 2,
                ModeKind::Puzzle => 4,
                _ => 3,
            }
        } else {
            // The tower's picked wizard (Select Players), else one per lane.
            skin.map_or(tower.index, |s| s.wizard) + 1
        };
        let enemy = has(false);
        let key = format!("wizard{number}");
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
            vanish: 0.0,
            seen: Seen {
                placed: state.bricks_placed,
                lives: state.lives,
                lines: state.lines_reached,
                out: state.status == TowerStatus::Out,
                won: state.status == TowerStatus::Won,
                casts: 0,
            },
            rng: config.seed ^ ((tower.index as u64 + 1) * 0x9E37_79B9),
            enemy,
        };
        wizard.action_countdown = wizard.random(look.action_every.0, look.action_every.1);
        if enemy {
            // It laughs hard in the trial's intro.
            wizard.clip = Clip::LaughHard;
        }
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
    game: Res<crate::gameplay::Match>,
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
        if wizard.enemy {
            let failed = state.status == TowerStatus::Out;
            if failed && wizard.clip == Clip::Idle && wizard.laugh_cooldown <= 0.0 {
                let wait = wizard.random(300, 400) as f32 / 100.0;
                wizard.laugh_cooldown = wait;
                play(&mut wizard, Clip::LaughHard, None);
            }
        } else {
            wizard.startle_cooldown -= dt;
            wizard.shake -= dt;

            if state.bricks_placed > wizard.seen.placed {
                wizard.action_countdown = wizard.action_countdown.saturating_sub(1);
                if wizard.action_countdown == 0 {
                    play(&mut wizard, Clip::Action, None);
                    wizard.action_countdown =
                        wizard.random(look.action_every.0, look.action_every.1);
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
            // The Survival roof going on: Action, then Cheer.
            let won = state.status == TowerStatus::Won;
            if won && !wizard.seen.won && game.mode.rules() == ModeKind::Survival {
                play(&mut wizard, Clip::Action, Some(Clip::Cheer));
            }
            wizard.seen = Seen {
                placed: state.bricks_placed,
                lives: state.lives,
                lines: state.lines_reached,
                out,
                won,
                casts: log.1,
            };
        }

        // Say the line that goes with a clip that just started.
        if wizard.clip != Clip::Idle && (wizard.clip != before || wizard.clip_time < started) {
            // The original picks one of the character's lines of that kind at random
            // (docs/research/wizards.md, "Voice lines"); older staging has one per clip.
            let kind = voice_kind(wizard.clip);
            let count = (1..)
                .take_while(|i| {
                    art.sounds
                        .contains_key(&format!("{}_say_{kind}_{i}", wizard.key))
                })
                .count() as u32;
            let line = if count > 0 {
                let i = wizard.random(1, count);
                format!("{}_say_{kind}_{i}", wizard.key)
            } else {
                format!("{}_say_{}", wizard.key, wizard.clip.name())
            };
            if let Some(sound) = art.sounds.get(&line) {
                let number: u32 = wizard.key.trim_start_matches("wizard").parse().unwrap_or(1);
                let (volume, pitch) = voice_mix(number, kind);
                commands.spawn((
                    AudioPlayer(sound.clone()),
                    PlaybackSettings::DESPAWN
                        .with_volume(bevy::audio::Volume::Linear(volume))
                        .with_speed(pitch),
                ));
            }
        }
    }
}

fn move_wizards(
    time: Res<Time>,
    config: Res<GameConfig>,
    game: Res<crate::gameplay::Match>,
    towers: Query<(&Tower, &TowerState)>,
    rivals: Query<&crate::gameplay::RivalLane>,
    mut wizards: Query<(&mut Wizard, &mut Transform, &mut Visibility)>,
) {
    let look = &config.wizards;
    // A trial's enemy wizard floats by the enemy lane (right of the player's).
    let enemy_lane = rivals.iter().next().map(|r| r.origin_x);
    let dt = time.delta_secs();
    let solo = towers.iter().count() == 1;
    for (mut wizard, mut transform, mut visibility) in &mut wizards {
        let Ok((tower, state)) = towers.get(wizard.tower) else {
            continue;
        };
        // A lost match: the wizard shrinks away in a moment, like a poof.
        let gone = if !wizard.enemy && crate::hud::lost(&game, tower, state) {
            wizard.vanish += dt;
            (wizard.vanish / config.hud.wizard_vanish.max(0.01)).min(1.0)
        } else {
            wizard.vanish = 0.0;
            0.0
        };
        visibility.set_if_neq(if gone >= 1.0 {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
        wizard.time += dt;
        // The enemy floats 4 cells over the tower top (the original's EnemyWizard).
        let target = state.height + if wizard.enemy { 4.0 } else { look.lift };
        let y = wizard.y.get_or_insert(target);
        *y += (target - *y) * dt.min(1.0);
        let y = *y;
        let offset = if wizard.enemy {
            // 7 cells left of the lane centre, 2 right in Puzzle trials.
            if config.mode.rules() == ModeKind::Puzzle {
                2.0
            } else {
                -7.0
            }
        } else if solo {
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
        let lane_x = if wizard.enemy {
            enemy_lane.unwrap_or(tower.origin_x)
        } else {
            tower.origin_x
        };
        transform.translation.x = lane_x + config.cells(at.x);
        transform.translation.y = config.cells(at.y);
        // Alone, the wizard floats on the left and turns to face the tower.
        let size = 1.0 - gone * gone;
        transform.scale = Vec3::new(if solo { -size } else { size }, size, 1.0);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn voice_lines_follow_the_original_speech_table() {
        assert_eq!(voice_kind(Clip::Casting), "cast_other");
        assert_eq!(voice_kind(Clip::Dying), "startle");
        assert_eq!(voice_mix(2, "laugh_evil"), (0.8, 1.13));
        assert_eq!(voice_mix(2, "cheer"), (1.0, 1.13));
        assert_eq!(voice_mix(5, "laugh"), (0.65, 1.13));
        assert_eq!(voice_mix(9, "startle"), (0.3, 1.0));
        assert_eq!(voice_mix(12, "cheer"), (1.1, 1.0));
    }
}
