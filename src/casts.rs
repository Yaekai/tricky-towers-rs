//! How casting looks, from the PC's readout of the original: a dark cast sends a
//! beam up from the caster, a beam down onto each victim's brick and a wave over
//! the victim's lane; a light cast throws lightning from the wizard to the brick.
//! Each wizard carries clouds holding its spells, which pop when one is used.
//! Large makes the brick in hand grow with a camera shake. Timings live in
//! [`GameConfig::casts`]; pictures are used when staged (docs/art-layout.md),
//! plain shapes stand in for them otherwise.

use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use crate::art::Art;
use crate::config::{GameConfig, ModeKind, SpellKind};
use crate::effects::CameraShake;
use crate::gameplay::{ActiveBrick, Brick, Match, Phase, Tower, TowerState};
use crate::spells::SpellLog;
use crate::wizards::Wizard;

pub struct CastsPlugin;

impl Plugin for CastsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                start_casts,
                run_beams,
                run_lightning,
                track_clouds,
                animate_clouds,
                grow_large_bricks,
            )
                .chain(),
        );
    }
}

const DARK_FALLBACK: Color = Color::srgba(0.45, 0.15, 0.65, 0.85);
const LIGHT_FALLBACK: Color = Color::srgba(1.0, 0.95, 0.6, 0.95);

/// Where a tower's wizard floats, in world pixels, and whether it faces left
/// (alone, it floats on the left of the tower). Falls back to the configured spot
/// when the wizard's art isn't staged.
fn wizard_spot(
    config: &GameConfig,
    tower_entity: Entity,
    tower: &Tower,
    state: &TowerState,
    solo: bool,
    wizards: &Query<(&Wizard, &GlobalTransform)>,
) -> (Vec2, bool) {
    if let Some((_, at)) = wizards
        .iter()
        .find(|(w, _)| !w.enemy() && w.tower() == tower_entity)
    {
        return (at.translation().truncate(), solo);
    }
    let look = &config.wizards;
    let x = if solo {
        look.solo_offset_x
    } else {
        look.offset_x
    };
    (
        Vec2::new(
            tower.origin_x + config.cells(x),
            config.cells(state.height + look.lift),
        ),
        solo,
    )
}

/// One stage of a dark cast's beam, or the lane wave.
#[derive(Component)]
struct Beam {
    kind: BeamKind,
    /// The beam's foot (world pixels) and the lane centre it ends over.
    foot: Vec2,
    lane_x: f32,
    target: usize,
    time: f32,
}

#[derive(Clone, Copy, PartialEq)]
enum BeamKind {
    Up,
    Down,
    Lane,
}

/// The lightning of a light cast, redrawn every frame from wizard to brick.
#[derive(Component)]
struct Lightning {
    from: Vec2,
    to: Vec2,
    time: f32,
    segments: Vec<Entity>,
}

#[allow(clippy::too_many_arguments)]
fn start_casts(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    game: Res<Match>,
    log: Res<SpellLog>,
    towers: Query<(Entity, &Tower, &TowerState)>,
    wizards: Query<(&Wizard, &GlobalTransform)>,
    bricks: Query<&GlobalTransform, With<ActiveBrick>>,
    mut seen: Local<u64>,
) {
    let fresh = (log.1 - (*seen).min(log.1)) as usize;
    *seen = log.1;
    if fresh == 0 || game.phase != Phase::Playing {
        return;
    }
    let solo = towers.iter().count() == 1;
    let find = |index: usize| towers.iter().find(|(_, t, _)| t.index == index);
    let brick_at = |state: &TowerState, tower: &Tower| {
        state
            .active
            .and_then(|e| bricks.get(e).ok())
            .map(|g| g.translation().truncate())
            .unwrap_or(Vec2::new(
                tower.origin_x,
                config.cells(state.stack_top + config.spawn_gap),
            ))
    };
    // A dark cast logs one entry per victim; one beam goes up per caster.
    let mut beamed_up: Vec<usize> = Vec::new();
    let mut lit: Vec<usize> = Vec::new();
    for cast in log.0.iter().rev().take(fresh) {
        let Some((caster_entity, caster, caster_state)) = find(cast.caster) else {
            // A trial's enemy wizard has no tower: only the beam down onto the victim
            // (where the original draws the enemy wizard isn't read yet, ?).
            if cast.caster == crate::spells::ENEMY_WIZARD
                && let Some((_, victim, victim_state)) = find(cast.target)
            {
                let foot = brick_at(victim_state, victim);
                spawn_beam(
                    &mut commands,
                    &config,
                    &art,
                    BeamKind::Down,
                    foot,
                    victim.origin_x,
                    cast.target,
                );
            }
            continue;
        };
        let (from, _) = wizard_spot(&config, caster_entity, caster, caster_state, solo, &wizards);
        if cast.spell.is_dark() {
            if !beamed_up.contains(&cast.caster) {
                beamed_up.push(cast.caster);
                spawn_beam(
                    &mut commands,
                    &config,
                    &art,
                    BeamKind::Up,
                    from,
                    caster.origin_x,
                    cast.caster,
                );
            }
            if let Some((_, victim, victim_state)) = find(cast.target) {
                let foot = brick_at(victim_state, victim);
                let beam = spawn_beam(
                    &mut commands,
                    &config,
                    &art,
                    BeamKind::Down,
                    foot,
                    victim.origin_x,
                    cast.target,
                );
                // The down beam waits for the up beam to grow in.
                commands
                    .entity(beam)
                    .insert((Visibility::Hidden, DelayedBy(config.casts.beam_up.0)));
            }
        } else if !lit.contains(&cast.caster) {
            lit.push(cast.caster);
            let to = brick_at(caster_state, caster);
            commands.spawn((
                Lightning {
                    from,
                    to,
                    time: 0.0,
                    segments: Vec::new(),
                },
                Transform::default(),
                Visibility::default(),
            ));
        }
    }
}

/// Seconds before a beam starts.
#[derive(Component)]
struct DelayedBy(f32);

fn beam_image(art: &Art, kind: BeamKind, mode: ModeKind) -> Option<Handle<Image>> {
    match kind {
        BeamKind::Lane if mode.look() == ModeKind::Survival => art
            .image("cast_lane_beam_survival")
            .or_else(|| art.image("cast_lane_beam")),
        BeamKind::Lane => art.image("cast_lane_beam"),
        _ => art.image("cast_beam"),
    }
}

fn spawn_beam(
    commands: &mut Commands,
    config: &GameConfig,
    art: &Art,
    kind: BeamKind,
    foot: Vec2,
    lane_x: f32,
    target: usize,
) -> Entity {
    let image = beam_image(art, kind, config.mode);
    let sprite = match image {
        Some(image) => Sprite {
            image,
            custom_size: Some(Vec2::ZERO),
            ..default()
        },
        None => Sprite::from_color(DARK_FALLBACK, Vec2::ZERO),
    };
    let mut beam = commands.spawn((
        Beam {
            kind,
            foot,
            lane_x,
            target,
            time: 0.0,
        },
        sprite,
        Transform::from_translation(foot.extend(16.0)),
    ));
    if art.image("cast_circle_gradient").is_some() && kind != BeamKind::Lane {
        let glow = art.image("cast_circle_gradient").unwrap();
        beam.with_children(|parent| {
            parent.spawn((
                BeamFoot,
                Sprite {
                    image: glow,
                    custom_size: Some(Vec2::splat(config.cells(config.casts.beam_width * 2.0))),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.1),
            ));
        });
    }
    beam.id()
}

/// The glow at a beam's foot; it stays at the foot, not stretched with the beam.
#[derive(Component)]
struct BeamFoot;

#[allow(clippy::type_complexity)]
fn run_beams(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    art: Res<Art>,
    camera: Query<(&Transform, &Projection), (With<Camera2d>, Without<Beam>)>,
    mut beams: Query<(
        Entity,
        &mut Beam,
        &mut Sprite,
        &mut Transform,
        &mut Visibility,
        Option<&mut DelayedBy>,
        Option<&Children>,
    )>,
    mut feet: Query<&mut Transform, (With<BeamFoot>, Without<Beam>, Without<Camera2d>)>,
) {
    let look = &config.casts;
    let dt = time.delta_secs();
    // The lane wave covers the whole view's height.
    let (view_y, view_h) = camera
        .single()
        .map(|(t, p)| {
            let scale = match p {
                Projection::Orthographic(o) => o.scale,
                _ => 1.0,
            };
            (t.translation.y, 2000.0 * scale)
        })
        .unwrap_or((0.0, 2000.0));
    let width = config.cells(look.beam_width);
    let reach = config.cells(look.beam_reach);
    for (entity, mut beam, mut sprite, mut transform, mut visibility, delay, children) in &mut beams
    {
        if let Some(mut delay) = delay {
            delay.0 -= dt;
            if delay.0 > 0.0 {
                continue;
            }
            commands.entity(entity).remove::<DelayedBy>();
            *visibility = Visibility::Inherited;
        }
        beam.time += dt;
        let (grow, last) = match beam.kind {
            BeamKind::Up => look.beam_up,
            BeamKind::Down => look.beam_down,
            BeamKind::Lane => look.lane_wave,
        };
        let t = beam.time;
        if beam.kind == BeamKind::Lane {
            // Fades in, then out.
            let alpha = if t < grow {
                t / grow
            } else {
                1.0 - (t - grow) / last.max(0.01)
            };
            if t >= grow + last {
                commands.entity(entity).despawn();
                continue;
            }
            let lane = config.cells(config.lane_half_width * 2.0 + 2.0);
            sprite.custom_size = Some(Vec2::new(lane, view_h));
            sprite.color.set_alpha(
                alpha.clamp(0.0, 1.0)
                    * if art.image("cast_lane_beam").is_some() {
                        1.0
                    } else {
                        0.35
                    },
            );
            transform.translation = Vec2::new(beam.lane_x, view_y).extend(15.0);
            continue;
        }
        // Grows out of its foot, holds, then squeezes to nothing.
        let length = reach * (t / grow).clamp(0.0, 1.0);
        let squeeze = ((t - last) / look.beam_squeeze.max(0.001)).clamp(0.0, 1.0);
        if squeeze >= 1.0 {
            if beam.kind == BeamKind::Down {
                // The lane wave follows the beam down.
                spawn_beam(
                    &mut commands,
                    &config,
                    &art,
                    BeamKind::Lane,
                    beam.foot,
                    beam.lane_x,
                    beam.target,
                );
            }
            commands.entity(entity).despawn();
            continue;
        }
        sprite.custom_size = Some(Vec2::new(width * (1.0 - 0.99 * squeeze), length));
        // Both beams stand on their foot: up from the wizard, down from the sky
        // onto the brick.
        transform.translation = (beam.foot + Vec2::Y * length * 0.5).extend(16.0);
        if let Some(children) = children {
            for child in children.iter() {
                if let Ok(mut foot) = feet.get_mut(child) {
                    foot.translation.y = -length * 0.5;
                }
            }
        }
    }
}

/// Number of pieces a lightning bolt is drawn with.
const BOLT_PIECES: usize = 14;

fn run_lightning(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    art: Res<Art>,
    mut bolts: Query<(Entity, &mut Lightning)>,
    mut pieces: Query<(&mut Transform, &mut Sprite), Without<Lightning>>,
) {
    let look = &config.casts;
    let image = art.image("cast_lightning");
    for (entity, mut bolt) in &mut bolts {
        bolt.time += time.delta_secs();
        if bolt.time >= look.lightning {
            for &piece in &bolt.segments {
                commands.entity(piece).despawn();
            }
            commands.entity(entity).despawn();
            continue;
        }
        if bolt.segments.is_empty() {
            bolt.segments = (0..BOLT_PIECES)
                .map(|_| {
                    let sprite = match &image {
                        Some(image) => Sprite {
                            image: image.clone(),
                            custom_size: Some(Vec2::ZERO),
                            ..default()
                        },
                        None => Sprite::from_color(LIGHT_FALLBACK, Vec2::ZERO),
                    };
                    commands.spawn((sprite, Transform::default())).id()
                })
                .collect();
            continue;
        }
        // The original's wobble: sin(phase + 5s) × amplitude × sin(πs) along the bolt, the
        // phase running 0 → -5 over the cast and the amplitude 0 → peak at 1/6 of it → 0.
        let (from, to) = (bolt.from, bolt.to);
        let side = (to - from).perp().normalize_or(Vec2::X);
        let t = (bolt.time / look.lightning).clamp(0.0, 1.0);
        let phase = -5.0 * t;
        let rise = 1.0 / 2.4;
        let envelope = if t < rise {
            t / rise
        } else {
            (1.0 - t) / (1.0 - rise)
        };
        let wobble = config.cells(look.lightning_wobble) * envelope;
        let point = |i: usize| {
            let s = i as f32 / BOLT_PIECES as f32;
            let swing = (s * std::f32::consts::PI).sin() * wobble * (s * 5.0 + phase).sin();
            from.lerp(to, s) + side * swing
        };
        let fade = 1.0 - bolt.time / look.lightning;
        for (i, &piece) in bolt.segments.iter().enumerate() {
            let Ok((mut transform, mut sprite)) = pieces.get_mut(piece) else {
                continue;
            };
            let (a, b) = (point(i), point(i + 1));
            let along = b - a;
            sprite.custom_size = Some(Vec2::new(along.length() + 2.0, config.cells(0.35)));
            sprite.color.set_alpha(fade);
            *transform = Transform::from_translation(((a + b) * 0.5).extend(16.5))
                .with_rotation(Quat::from_rotation_z(along.to_angle()));
        }
    }
}

/// A spell cloud beside a wizard, holding one spell.
#[derive(Component)]
struct SpellCloud {
    tower: Entity,
    dark: bool,
    stage: CloudStage,
    time: f32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum CloudStage {
    Show,
    Float,
    Used,
    Hide,
}

fn icon_spell(state: &TowerState, dark: bool) -> Option<SpellKind> {
    if dark {
        state
            .dark_spell
            .as_ref()
            .and_then(|c| c.spells.first().copied())
    } else {
        state.light_spell
    }
}

/// Starts a cloud when a tower gets a spell, and sends it off (Used after a cast,
/// Hide otherwise) when the spell goes.
#[allow(clippy::too_many_arguments)]
fn track_clouds(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    game: Res<Match>,
    log: Res<SpellLog>,
    towers: Query<(Entity, &Tower, &TowerState)>,
    mut clouds: Query<(Entity, &mut SpellCloud)>,
    mut held: Local<HashMap<(Entity, bool), SpellKind>>,
    mut seen_casts: Local<u64>,
) {
    let fresh = (log.1 - (*seen_casts).min(log.1)) as usize;
    *seen_casts = log.1;
    if game.phase == Phase::Menu {
        for (entity, _) in &clouds {
            commands.entity(entity).despawn();
        }
        held.clear();
        return;
    }
    let cast_by = |index: usize, dark: bool| {
        log.0
            .iter()
            .rev()
            .take(fresh)
            .any(|c| c.caster == index && c.spell.is_dark() == dark)
    };
    held.retain(|(tower, _), _| towers.contains(*tower));
    for (tower_entity, tower, state) in &towers {
        for dark in [false, true] {
            let now = icon_spell(state, dark);
            let before = held.get(&(tower_entity, dark)).copied();
            if now == before {
                continue;
            }
            if before.is_some() {
                let used = cast_by(tower.index, dark);
                for (_, mut cloud) in &mut clouds {
                    if cloud.tower == tower_entity
                        && cloud.dark == dark
                        && matches!(cloud.stage, CloudStage::Show | CloudStage::Float)
                    {
                        cloud.stage = if used {
                            CloudStage::Used
                        } else {
                            CloudStage::Hide
                        };
                        cloud.time = 0.0;
                    }
                }
            }
            match now {
                Some(spell) => {
                    held.insert((tower_entity, dark), spell);
                    spawn_cloud(&mut commands, &config, &art, tower_entity, dark, spell);
                }
                None => {
                    held.remove(&(tower_entity, dark));
                }
            }
        }
    }
}

fn spawn_cloud(
    commands: &mut Commands,
    config: &GameConfig,
    art: &Art,
    tower: Entity,
    dark: bool,
    spell: SpellKind,
) {
    let look = &config.casts;
    let size = config.cells(look.cloud_size);
    let cloud = match art.image("spell_cloud") {
        Some(image) => Sprite {
            image,
            custom_size: Some(Vec2::splat(size)),
            ..default()
        },
        None => Sprite::from_color(Color::srgba(0.05, 0.03, 0.12, 0.7), Vec2::splat(size * 0.8)),
    };
    let icon = match art.image(&format!("spell_icon_{spell:?}")) {
        Some(image) => Sprite {
            image,
            custom_size: Some(Vec2::splat(size * look.cloud_icon)),
            ..default()
        },
        None => Sprite::from_color(
            if dark { DARK_FALLBACK } else { LIGHT_FALLBACK },
            Vec2::splat(size * look.cloud_icon * 0.7),
        ),
    };
    let hint = art.image(if dark {
        "spell_cloud_button_dark"
    } else {
        "spell_cloud_button_light"
    });
    commands
        .spawn((
            SpellCloud {
                tower,
                dark,
                stage: CloudStage::Show,
                time: 0.0,
            },
            Transform::from_scale(Vec3::ZERO),
            Visibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((cloud, Transform::default()));
            parent.spawn((icon, Transform::from_xyz(0.0, 0.0, 0.1)));
            if let Some(hint) = hint {
                parent.spawn((
                    Sprite {
                        image: hint,
                        custom_size: Some(Vec2::splat(size * 0.3)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, -size * 0.5, 0.2),
                ));
            }
        });
}

fn ease_out_back(x: f32) -> f32 {
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    1.0 + c3 * (x - 1.0).powi(3) + c1 * (x - 1.0).powi(2)
}

/// Places each cloud beside its wizard and plays Show, Floating, Used and Hide.
fn animate_clouds(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    towers: Query<(Entity, &Tower, &TowerState)>,
    wizards: Query<(&Wizard, &GlobalTransform)>,
    mut clouds: Query<(Entity, &mut SpellCloud, &mut Transform)>,
) {
    let look = &config.casts;
    let dt = time.delta_secs();
    let solo = towers.iter().count() == 1;
    for (entity, mut cloud, mut transform) in &mut clouds {
        let Ok((tower_entity, tower, state)) = towers.get(cloud.tower) else {
            commands.entity(entity).despawn();
            continue;
        };
        cloud.time += dt;
        let (at, flip) = wizard_spot(&config, tower_entity, tower, state, solo, &wizards);
        let spots = look
            .cloud_spots
            .get(tower.index)
            .or(look.cloud_spots.first())
            .copied()
            .unwrap_or_default();
        let spot = Vec2::from(if cloud.dark { spots.1 } else { spots.0 });
        let spot = Vec2::new(if flip { -spot.x } else { spot.x }, spot.y);
        let t = cloud.time;
        let (scale, lift) = match cloud.stage {
            CloudStage::Show => {
                if t >= look.cloud_show {
                    cloud.stage = CloudStage::Float;
                    cloud.time = 0.0;
                }
                (ease_out_back((t / look.cloud_show).min(1.0)), 0.0)
            }
            CloudStage::Float => {
                let bob = (t / look.cloud_float * std::f32::consts::TAU).sin() * look.cloud_bob;
                (1.0, bob)
            }
            CloudStage::Used => {
                // Swells, then pops away, rising as the spell leaves it (?).
                let s = (t / look.cloud_used).min(1.0);
                let size = if s < 0.3 {
                    1.0 + s
                } else {
                    1.3 * (1.0 - (s - 0.3) / 0.7)
                };
                (size, 0.5 * s)
            }
            CloudStage::Hide => (1.0 - (t / look.cloud_hide).min(1.0), 0.0),
        };
        let done = match cloud.stage {
            CloudStage::Used => t >= look.cloud_used,
            CloudStage::Hide => t >= look.cloud_hide,
            _ => false,
        };
        if done {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation = (at + config.cells(1.0) * (spot + Vec2::Y * lift)).extend(21.0);
        transform.scale = Vec3::splat(scale.max(0.0));
    }
}

/// Grows the brick in hand when Large makes it bigger: its cells are drawn at
/// the new size at once, so they are scaled down to the old size and eased up.
/// Also shakes it while No Rotate refuses a turn.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn grow_large_bricks(
    time: Res<Time>,
    config: Res<GameConfig>,
    art: Res<Art>,
    mut commands: Commands,
    mut shake: ResMut<CameraShake>,
    bricks: Query<(Entity, &Brick, &ActiveBrick, Option<&Children>)>,
    mut cells: Query<(&mut Transform, Option<&GrowBase>), Without<Brick>>,
    mut sizes: Local<HashMap<Entity, (f32, f32, f32)>>,
    mut shook: Local<bevy::platform::collections::HashSet<Entity>>,
) {
    let look = &config.casts;
    let dt = time.delta_secs();
    sizes.retain(|entity, _| bricks.contains(*entity));
    shook.retain(|entity| bricks.contains(*entity));
    for (entity, brick, control, children) in &bricks {
        let entry = sizes
            .entry(entity)
            .or_insert((brick.scale, brick.scale, 1.0));
        // A bigger brick than last seen: start growing from the old size.
        if brick.scale > entry.1 + 1e-4 {
            *entry = (entry.1, brick.scale, 0.0);
            let (x, y, seconds) = look.large_shake;
            shake
                .kicks
                .push((Vec2::new(x, y) * config.cell_size, seconds));
            if let Some(sound) = art.sounds.get("sfx_large") {
                commands.spawn((AudioPlayer(sound.clone()), PlaybackSettings::DESPAWN));
            }
        } else if brick.scale < entry.1 - 1e-4 {
            *entry = (brick.scale, brick.scale, 1.0);
        }
        let (from, to, progress) = *entry;
        // No Rotate refusing a turn shakes the brick sideways; a curse wave's lift
        // makes its picture jump about.
        let shaking = control.refused > 0.0 || control.picture_shake > 0.0;
        let was_shaking = if shaking {
            !shook.insert(entity)
        } else {
            shook.remove(&entity)
        };
        // Only touch the cells while a grow or shake runs (the last step puts them
        // back).
        if progress >= 1.0 && !shaking && !was_shaking {
            continue;
        }
        let progress = (progress + dt / look.large_grow.max(0.001)).min(1.0);
        entry.2 = progress;
        let factor = (from + (to - from) * progress) / to;
        // A refused turn makes the picture jump like a wave's lift, smaller on a
        // big brick (the original's shake modifier).
        let jump = if control.picture_shake > 0.0 {
            control.picture_shake
        } else {
            config.cells(config.magic.no_rotate_jump) / brick.scale.max(1.0)
        };
        let wobble = if shaking {
            // A fresh random jump every frame, ±reach on each axis.
            let mut seed = (time.elapsed_secs().to_bits() ^ entity.index_u32()) | 1;
            let mut unit = || {
                seed ^= seed << 13;
                seed ^= seed >> 17;
                seed ^= seed << 5;
                (seed % 2001) as f32 / 1000.0 - 1.0
            };
            Vec2::new(unit(), unit()) * jump
        } else {
            Vec2::ZERO
        };
        let Some(children) = children else {
            continue;
        };
        for child in children.iter() {
            let Ok((mut transform, base)) = cells.get_mut(child) else {
                continue;
            };
            let base = match base {
                Some(base) => base.0,
                None => {
                    commands.entity(child).insert(GrowBase(*transform));
                    *transform
                }
            };
            transform.translation =
                (base.translation.truncate() * factor + wobble).extend(base.translation.z);
            transform.scale = base.scale * factor;
        }
    }
}

/// A brick cell's transform before the Large grow scales it.
#[derive(Component)]
struct GrowBase(Transform);

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::time::TimeUpdateStrategy;

    use super::*;
    use crate::config::{Curse, ModeKind};
    use crate::gameplay::PlayerInput;

    fn app() -> App {
        let mut config = GameConfig {
            mode: ModeKind::Race,
            opponents: 1,
            ..default()
        };
        config.magic.enabled = true;
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, TransformPlugin, AssetPlugin::default()))
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 64.0,
            )))
            .insert_resource(config)
            .init_resource::<Art>()
            .init_resource::<CameraShake>()
            .add_plugins((crate::TowerCorePlugin, CastsPlugin));
        app.finish();
        app.cleanup();
        for _ in 0..10 {
            app.update();
        }
        app
    }

    fn you(app: &mut App) -> Entity {
        let world = app.world_mut();
        world
            .query::<(Entity, &Tower)>()
            .iter(world)
            .find(|(_, t)| t.human)
            .unwrap()
            .0
    }

    fn count<C: Component>(app: &mut App) -> usize {
        let world = app.world_mut();
        world.query::<&C>().iter(world).count()
    }

    fn step(app: &mut App, n: usize) {
        for _ in 0..n {
            app.update();
        }
    }

    #[test]
    fn clouds_show_spells_and_casts_draw_lightning_and_beams() {
        let mut app = app();
        let tower = you(&mut app);
        {
            let mut state = app.world_mut().get_mut::<TowerState>(tower).unwrap();
            state.light_spell = Some(SpellKind::Ivy);
            state.dark_spell = Some(Curse::new(&[SpellKind::Large], 1, None));
        }
        step(&mut app, 2);
        assert_eq!(count::<SpellCloud>(&mut app), 2, "one cloud per held spell");

        // Casting light throws the dark half away: lightning, one Used, one Hide.
        app.world_mut()
            .get_mut::<PlayerInput>(tower)
            .unwrap()
            .cast_light_pressed = true;
        step(&mut app, 3);
        assert_eq!(count::<Lightning>(&mut app), 1);
        let world = app.world_mut();
        let mut stages: Vec<(bool, CloudStage)> = world
            .query::<&SpellCloud>()
            .iter(world)
            .map(|c| (c.dark, c.stage))
            .collect();
        stages.sort_by_key(|s| s.0);
        assert_eq!(
            stages,
            vec![(false, CloudStage::Used), (true, CloudStage::Hide)]
        );
        // Everything is gone after its clip.
        step(&mut app, 100);
        assert_eq!(count::<SpellCloud>(&mut app), 0);
        assert_eq!(count::<Lightning>(&mut app), 0);

        // A dark cast: a beam up from you, then one down onto the opponent.
        {
            let mut state = app.world_mut().get_mut::<TowerState>(tower).unwrap();
            state.light_spell = Some(SpellKind::Ivy);
            state.dark_spell = Some(Curse::new(&[SpellKind::Large], 1, None));
        }
        step(&mut app, 2);
        app.world_mut()
            .get_mut::<PlayerInput>(tower)
            .unwrap()
            .cast_dark_pressed = true;
        step(&mut app, 3);
        let world = app.world_mut();
        let kinds: Vec<bool> = world
            .query::<&Beam>()
            .iter(world)
            .map(|b| b.kind == BeamKind::Up)
            .collect();
        assert!(kinds.contains(&true) && kinds.contains(&false), "{kinds:?}");
        // The down beam ends in a wave over the victim's lane (all over by ~2.5 s).
        let mut saw_wave = false;
        for _ in 0..200 {
            app.update();
            let world = app.world_mut();
            saw_wave |= world
                .query::<&Beam>()
                .iter(world)
                .any(|b| b.kind == BeamKind::Lane);
        }
        assert!(saw_wave);
        assert_eq!(count::<Beam>(&mut app), 0);
    }
}
