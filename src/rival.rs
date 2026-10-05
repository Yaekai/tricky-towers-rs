//! A trial's enemy lane (`gameplay::RivalLane`): in race trials the race house with
//! the rival tower popping in brick by brick at the start, as in the original
//! (docs/research/trials-obstacles-roofs.md, "Rival towers"). The towers' layouts are
//! staged from the install as `rival_towers.ron` (tools/extract/rival_towers.ps1);
//! without it the lane shows the house only. Looks only.

use std::collections::HashMap;

use bevy::prelude::*;
use serde::Deserialize;

use crate::art::Art;
use crate::config::GameConfig;
use crate::gameplay::RivalLane;
use crate::visuals::art_sprite;

pub struct RivalPlugin;

impl Plugin for RivalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (build_rival_lane, pop_in));
    }
}

/// One stone piece of a rival tower, in cells from the lane centre at the base top.
#[derive(Deserialize)]
struct Piece {
    art: String,
    x: f32,
    y: f32,
    turn: f32,
    scale: f32,
    /// Pieces of a group pop in together, groups in order.
    group: u32,
}

/// Seconds the whole tower takes to pop in, and the least time between pop sounds.
const BUILD_SECONDS: f32 = 1.5;
const POP_SOUND_GAP: f32 = 0.075;

/// The pop-in under way: time so far, groups shown, time since the last pop sound.
#[derive(Component)]
struct PopIn {
    time: f32,
    groups: u32,
    shown: u32,
    since_sound: f32,
}

#[derive(Component)]
struct RivalPiece(u32);

fn read_layouts() -> HashMap<u32, Vec<Piece>> {
    let path = crate::art::extracted_dir().join("rival_towers.ron");
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| ron::from_str(&text).ok())
        .unwrap_or_default()
}

fn build_rival_lane(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    game: Res<crate::gameplay::Match>,
    lanes: Query<(Entity, &RivalLane), Added<RivalLane>>,
    mut layouts: Local<Option<HashMap<u32, Vec<Piece>>>>,
) {
    let c = config.cell_size;
    for (entity, lane) in &lanes {
        let mut node = commands.entity(entity);
        node.insert((
            Transform::from_xyz(lane.origin_x, 0.0, 0.0),
            Visibility::default(),
        ));
        // The lane's own background, placed like a platform's (centre half a cell down).
        node.with_children(|parent| {
            parent
                .spawn((
                    Transform::from_xyz(0.0, -c * 0.5, 0.0),
                    Visibility::default(),
                ))
                .with_children(|layers| {
                    crate::visuals::spawn_lane_layers(layers, &art, game.mode, c)
                });
        });
        let Some(rival) = lane.rival else { continue };
        // The race house, its top at the lane's y = 0.
        if let Some((sprite, anchor, transform)) = art_sprite(&art, "platform_race", c) {
            node.with_child((sprite, anchor, transform));
        }
        let layouts = layouts.get_or_insert_with(read_layouts);
        let Some(pieces) = layouts.get(&rival) else {
            continue;
        };
        for piece in pieces {
            let key = match piece.art.strip_prefix("ivy_") {
                Some(shape) => format!("vine_overlay_{shape}"),
                None => piece.art.clone(),
            };
            let Some((mut sprite, anchor, mut transform)) = art_sprite(&art, &key, c) else {
                continue;
            };
            // Ivy on a rival brick is fully grown: the last frame.
            if let Some(frame) = art.frame(&key, 1.0e6) {
                sprite.image = frame;
            }
            transform.translation = Vec3::new(piece.x * c, piece.y * c, 0.5);
            transform.rotation = Quat::from_rotation_z(piece.turn.to_radians());
            transform.scale *= Vec3::new(piece.scale, piece.scale, 1.0);
            node.with_child((
                sprite,
                anchor,
                transform,
                Visibility::Hidden,
                RivalPiece(piece.group),
            ));
        }
        let groups = pieces.iter().map(|p| p.group).max().unwrap_or(0);
        node.insert(PopIn {
            time: 0.0,
            groups,
            shown: 0,
            since_sound: POP_SOUND_GAP,
        });
    }
}

/// Shows round(groups × t / 1.5) groups at time t, with a bubble pop at most every
/// 0.075 s, its pitch rising 1 → 2.5 over the build.
fn pop_in(
    mut commands: Commands,
    time: Res<Time>,
    art: Res<Art>,
    mut lanes: Query<(&mut PopIn, &Children)>,
    mut pieces: Query<(&RivalPiece, &mut Visibility)>,
) {
    for (mut pop, children) in &mut lanes {
        if pop.shown >= pop.groups {
            continue;
        }
        pop.time += time.delta_secs();
        pop.since_sound += time.delta_secs();
        let progress = (pop.time / BUILD_SECONDS).min(1.0);
        let due = ((pop.groups as f32 * progress).round() as u32).min(pop.groups);
        if due > pop.shown {
            for child in children.iter() {
                if let Ok((piece, mut visible)) = pieces.get_mut(child)
                    && piece.0 <= due
                {
                    *visible = Visibility::Inherited;
                }
            }
            pop.shown = due;
            if pop.since_sound > POP_SOUND_GAP
                && let Some(sound) = art.sounds.get("sfx_bubble_pop")
            {
                pop.since_sound = 0.0;
                commands.spawn((
                    AudioPlayer(sound.clone()),
                    PlaybackSettings::DESPAWN.with_speed(1.0 + progress * 1.5),
                ));
            }
        }
    }
}
