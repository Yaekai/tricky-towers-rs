//! Match moments: the race hold countdown, the WINNER! and WIZARD OUT! banners, the
//! winner's spotlight, the veil over losing lanes and the trophy. Drawn with the
//! original's letters and art when staged (see `docs/research/match-moments.md`).

use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;

use crate::art::Art;
use crate::config::{GameConfig, ModeKind};
use crate::gameplay::{Brick, Match, Phase, Placed, Tower, TowerState, TowerStatus};
use crate::spells::Zapped;
use crate::visuals::{FramedCamera, ViewSize};

pub struct ResultsPlugin;

impl Plugin for ResultsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                start_moments,
                show_hold_counts,
                show_puzzle_scores,
                place_puzzle_scores,
                animate_banners,
                animate_lights,
            )
                .chain()
                .after(FramedCamera),
        );
    }
}

/// The colour of match text: gold for winners and the countdown, red for losers,
/// bronze for a draw.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Tone {
    Gold,
    Red,
}

impl Tone {
    fn gradient(self) -> &'static str {
        match self {
            Tone::Gold => "text_gradient_gold",
            Tone::Red => "text_gradient_red",
        }
    }

    fn fallback(self) -> Color {
        match self {
            Tone::Gold => Color::srgb_u8(255, 210, 80),
            Tone::Red => Color::srgb_u8(240, 70, 110),
        }
    }
}

/// The text's colour: the middle of its staged gradient, else a plain stand-in.
fn tone_color(art: &Art, images: &Assets<Image>, tone: Tone) -> Color {
    art.image(tone.gradient())
        .and_then(|h| images.get(&h))
        .and_then(|image| {
            let size = image.size();
            image.get_color_at(size.x / 2, size.y / 2).ok()
        })
        .unwrap_or_else(|| tone.fallback())
}

/// Everything shown for one match; cleared when the next one starts.
#[derive(Component)]
struct Moment;

/// Both layers of a piece of match text, so its words can change.
#[derive(Component)]
struct MatchLabel;

/// A banner over a lane, and how long it has shown.
#[derive(Component)]
struct Banner {
    tower: Entity,
    out: bool,
    time: f32,
}

/// The countdown over a tower holding past the race finish.
#[derive(Component)]
struct HoldCount(Entity);

/// Sunrays turning behind WINNER!.
#[derive(Component)]
struct Sunrays;

/// The trophy popping in under WINNER!.
#[derive(Component)]
struct Trophy;

/// A beam over the winner's lane, or a veil over a loser's.
#[derive(Component)]
struct Light {
    tower: Entity,
    delay: f32,
    alpha: f32,
    time: f32,
}

/// Match text in the original's style: a shadow a little below, the face on top.
/// Returns the container, which is positioned and scaled as one.
fn spawn_match_text(
    commands: &mut Commands,
    config: &GameConfig,
    text: &str,
    size: f32,
    color: Color,
) -> Entity {
    let look = &config.moments;
    let [r, g, b] = look.shadow_color;
    let layer = |color: Color, drop: f32| {
        (
            MatchLabel,
            Text::new(text),
            TextFont {
                font_size: FontSize::Px(size),
                ..default()
            },
            TextColor(color),
            TextLayout::justify(Justify::Center),
            Node {
                grid_row: GridPlacement::start(1),
                grid_column: GridPlacement::start(1),
                top: Val::Px(drop),
                ..default()
            },
        )
    };
    commands
        .spawn((
            Moment,
            Node {
                position_type: PositionType::Absolute,
                display: Display::Grid,
                justify_items: JustifyItems::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            UiTransform::default(),
            GlobalZIndex(20),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn(layer(Color::srgb_u8(r, g, b), size * look.shadow_drop));
            parent.spawn(layer(color, 0.0));
        })
        .id()
}

/// Puts a text container centred on a point of the screen, kept inside it even
/// while it is scaled up by `scale` around its centre.
fn centre_on(node: &mut Node, computed: &ComputedNode, at: Vec2, view: Vec2, scale: f32) {
    let size = computed.size() * computed.inverse_scale_factor();
    let margin = size * (scale.max(1.0) - 1.0) * 0.5;
    let fit = |centre: f32, size: f32, margin: f32, view: f32| {
        (centre - size * 0.5).clamp(margin, (view - size - margin).max(margin))
    };
    node.left = Val::Px(fit(at.x, size.x, margin.x, view.x));
    node.top = Val::Px(fit(at.y, size.y, margin.y, view.y));
}

/// Starts the banners and lights as towers win or go out, and clears them when a
/// new match starts.
#[allow(clippy::too_many_arguments)]
fn start_moments(
    mut commands: Commands,
    config: Res<GameConfig>,
    game: Res<Match>,
    art: Res<Art>,
    images: Res<Assets<Image>>,
    towers: Query<(Entity, &Tower, &TowerState)>,
    moments: Query<Entity, With<Moment>>,
    mut seen: Local<(u64, HashSet<(Entity, bool)>)>,
) {
    if game.phase == Phase::Menu || seen.0 != game.round {
        for entity in &moments {
            commands.entity(entity).despawn();
        }
        *seen = (game.round, HashSet::default());
        if game.phase == Phase::Menu {
            return;
        }
    }
    let look = &config.moments;
    let key = match game.mode.look() {
        ModeKind::Survival => "survival",
        ModeKind::Puzzle => "puzzle",
        _ => "race",
    };
    let beam_alpha = match game.mode.look() {
        ModeKind::Survival => look.spotlight_alpha.1,
        ModeKind::Puzzle => look.spotlight_alpha.2,
        _ => look.spotlight_alpha.0,
    };
    let over = game.phase == Phase::Over;
    // Trials end on FINISH / FAILED instead (the original's CHALLENGE_FINISH_MESSAGE and
    // CHALLENGE_FAILED_MESSAGE; same size as WINNER! in Max's screenshots).
    let (win_words, win_size, out_words, out_size) = if config.trial.is_some() {
        ("FINISH", look.banner_size, "FAILED", look.banner_size)
    } else {
        (
            "WINNER!",
            look.banner_size,
            "WIZARD OUT!",
            look.banner_size * 0.75,
        )
    };
    // The winner's roof goes on before WINNER!, the spotlight and the veils.
    if towers
        .iter()
        .any(|(_, tower, state)| crate::gameplay::roof_waiting(&game, tower, state))
    {
        return;
    }
    for (entity, tower, state) in &towers {
        let won = state.status == TowerStatus::Won || (over && game.winner == Some(tower.index));
        if won && seen.1.insert((entity, true)) {
            let color = tone_color(&art, &images, Tone::Gold);
            let banner = spawn_match_text(&mut commands, &config, win_words, win_size, color);
            commands.entity(banner).insert(Banner {
                tower: entity,
                out: false,
                time: 0.0,
            });
            if let Some(image) = art.image("winner_sunrays") {
                let rays = commands
                    .spawn((
                        Sunrays,
                        ImageNode::new(image),
                        Node {
                            grid_row: GridPlacement::start(1),
                            grid_column: GridPlacement::start(1),
                            width: Val::Px(look.banner_size * 4.0),
                            height: Val::Px(look.banner_size * 4.0),
                            ..default()
                        },
                        UiTransform::default(),
                    ))
                    .id();
                commands.entity(banner).insert_children(0, &[rays]);
            }
            // No cup after a single match: cups only end a cup (not built yet).
            spawn_light(
                &mut commands,
                &art,
                &format!("spotlight_winner_{key}"),
                entity,
                look.spotlight_delay,
                beam_alpha,
                Strip {
                    from: 0.0,
                    side: 0.0,
                    width: look.spotlight_width,
                    screen: false,
                },
                Color::WHITE,
            );
        }
        // Puzzle has no WIZARD OUT!: a stopped tower just stops (its bricks are counted).
        if state.status == TowerStatus::Out
            && game.mode.rules() != ModeKind::Puzzle
            && seen.1.insert((entity, false))
        {
            let color = tone_color(&art, &images, Tone::Red);
            let banner = spawn_match_text(&mut commands, &config, out_words, out_size, color);
            commands.entity(banner).insert(Banner {
                tower: entity,
                out: true,
                time: 0.0,
            });
        }
    }
    // The veil: everything but a clear gap around the winner's lane darkens.
    let Some((winner, winner_tower, _)) = towers
        .iter()
        .find(|(_, t, _)| over && game.winner == Some(t.index))
    else {
        return;
    };
    if !seen.1.insert((winner, false)) {
        return;
    }
    let humans = towers.iter().filter(|(_, t, _)| t.human).count();
    let name = format!("spotlight_loser_{key}");
    if humans == 1 && !winner_tower.human {
        // The computer beat the only player: one strip over the whole screen.
        let strip = Strip {
            from: 0.0,
            side: 0.0,
            width: look.veil_solo_width,
            screen: true,
        };
        spawn_light(
            &mut commands,
            &art,
            &name,
            winner,
            0.0,
            look.veil_alpha,
            strip,
            Color::BLACK,
        );
    } else {
        for side in [-1.0, 1.0] {
            let strip = Strip {
                from: look.veil_gap,
                side,
                width: look.veil_width,
                screen: false,
            };
            spawn_light(
                &mut commands,
                &art,
                &name,
                winner,
                0.0,
                look.veil_alpha,
                strip,
                Color::BLACK,
            );
        }
    }
}

/// Where a light lies across its tower's lane, in cells: centred on the lane
/// (`side` 0), or starting `from` the lane centre and reaching `width` outward
/// to the left (-1) or right (1). A `screen` strip is centred on the view.
#[derive(Component, Clone, Copy)]
struct Strip {
    from: f32,
    side: f32,
    width: f32,
    screen: bool,
}

#[allow(clippy::too_many_arguments)]
fn spawn_light(
    commands: &mut Commands,
    art: &Art,
    name: &str,
    tower: Entity,
    delay: f32,
    alpha: f32,
    strip: Strip,
    fallback: Color,
) {
    let mut sprite = match art.image(name) {
        // The veil's art is drawn dark, like the original; the beam as it is.
        Some(image) => Sprite {
            image,
            color: fallback.with_alpha(0.0),
            ..default()
        },
        None => Sprite::from_color(fallback.with_alpha(0.0), Vec2::ONE),
    };
    // The art is dark along its length and fades out at its right end, so the
    // left strip is drawn as it is and the right one mirrored: both fade toward
    // the winner.
    sprite.flip_x = strip.side > 0.0;
    let anchor = if strip.side < 0.0 {
        bevy::sprite::Anchor::BOTTOM_RIGHT
    } else if strip.side > 0.0 {
        bevy::sprite::Anchor::BOTTOM_LEFT
    } else {
        bevy::sprite::Anchor::BOTTOM_CENTER
    };
    commands.spawn((
        Moment,
        Light {
            tower,
            delay,
            alpha,
            time: 0.0,
        },
        sprite,
        anchor,
        Transform::from_xyz(0.0, 0.0, 30.0),
        strip,
    ));
}

#[allow(clippy::type_complexity)]
/// Fades the lights in and keeps them covering their lane from the bottom of the
/// screen to the top.
fn animate_lights(
    time: Res<Time>,
    config: Res<GameConfig>,
    view: Res<ViewSize>,
    camera: Query<(&Transform, &Projection), (With<Camera2d>, Without<Light>)>,
    towers: Query<&Tower>,
    mut lights: Query<(&mut Light, &Strip, &mut Sprite, &mut Transform)>,
) {
    let Ok((eye, Projection::Orthographic(ortho))) = camera.single() else {
        return;
    };
    let height = view.0.y * ortho.scale;
    let bottom = eye.translation.y - height * 0.5;
    for (mut light, strip, mut sprite, mut transform) in &mut lights {
        light.time += time.delta_secs();
        let Ok(tower) = towers.get(light.tower) else {
            continue;
        };
        let fade =
            ((light.time - light.delay) / config.moments.spotlight_fade.max(0.01)).clamp(0.0, 1.0);
        sprite.color = sprite.color.with_alpha(light.alpha * fade);
        sprite.custom_size = Some(Vec2::new(config.cells(strip.width), height));
        transform.translation.x = if strip.screen {
            // The whole-screen strip follows the camera.
            eye.translation.x
        } else {
            tower.origin_x + config.cells(strip.from * strip.side)
        };
        transform.translation.y = bottom;
    }
}

fn ease_out_elastic(x: f32) -> f32 {
    if x >= 1.0 {
        return 1.0;
    }
    2f32.powf(-10.0 * x) * ((10.0 * x - 0.75) * std::f32::consts::TAU / 3.0).sin() + 1.0
}

fn ease_out(x: f32) -> f32 {
    1.0 - (1.0 - x.clamp(0.0, 1.0)).powi(3)
}

/// Moves the banners in, keeps them over their lane, and moves WIZARD OUT! away again.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn animate_banners(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    view: Res<ViewSize>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    towers: Query<&Tower>,
    mut banners: Query<(
        Entity,
        &mut Banner,
        &mut Node,
        &ComputedNode,
        &mut UiTransform,
        &mut Visibility,
    )>,
    mut rays: Query<&mut UiTransform, (With<Sunrays>, Without<Banner>, Without<Trophy>)>,
    mut cups: Query<&mut UiTransform, (With<Trophy>, Without<Banner>, Without<Sunrays>)>,
) {
    let Ok((camera, eye)) = camera.single() else {
        return;
    };
    let look = &config.moments;
    let dt = time.delta_secs();
    for (entity, mut banner, mut node, computed, mut transform, mut visibility) in &mut banners {
        banner.time += dt;
        let Ok(tower) = towers.get(banner.tower) else {
            continue;
        };
        let Ok(at) = camera.world_to_viewport(eye, Vec3::new(tower.origin_x, 0.0, 0.0)) else {
            continue;
        };
        centre_on(
            &mut node,
            computed,
            Vec2::new(at.x, view.0.y * 0.35),
            view.0,
            1.4,
        );
        // Shown once laid out, so it never flashes in the wrong place.
        if computed.size().x > 0.0 {
            visibility.set_if_neq(Visibility::Inherited);
        }
        let t = banner.time;
        if banner.out {
            let (enter, stay, leave) = look.out_times;
            let drop = view.0.y * 0.6;
            let y = if t < enter {
                -drop * (1.0 - ease_out(t / enter))
            } else if t < enter + stay {
                0.0
            } else {
                -drop * ease_out((t - enter - stay) / leave)
            };
            transform.translation = Val2::px(0.0, y);
            if t > enter + stay + leave {
                commands.entity(entity).despawn();
            }
        } else {
            transform.scale = Vec2::splat(ease_out_elastic(t / look.winner_in.max(0.01)).max(0.0));
        }
    }
    for mut ray in &mut rays {
        ray.rotation *= Rot2::degrees(look.sunray_spin * dt);
    }
    for mut cup in &mut cups {
        let (from, over) = look.trophy_pop;
        let next = (cup.scale.x + (1.0 - from) * dt / over.max(0.01)).min(1.0);
        cup.scale = Vec2::splat(next);
    }
}

/// A Puzzle battle's score over a lane, or the 3-2-1 over the lane that touched the laser.
#[derive(Component)]
struct PuzzleScore {
    tower: Entity,
}

/// The Puzzle battle's count: the 3-2-1 over the lane that touched the laser, then each
/// tower's brick count at the top of its lane (the winner's gold, the others pink).
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn show_puzzle_scores(
    mut commands: Commands,
    config: Res<GameConfig>,
    game: Res<Match>,
    art: Res<Art>,
    images: Res<Assets<Image>>,
    towers: Query<(Entity, &Tower, &TowerState)>,
    bricks: Query<(&Brick, Has<Placed>, Has<Zapped>)>,
    time: Res<Time>,
    mut counted: Local<Option<(u64, f32)>>,
    mut labels: Local<HashMap<Entity, (String, Entity)>>,
) {
    let battle = game.mode.rules() == ModeKind::Puzzle
        && config.trial.is_none()
        && game.phase != Phase::Menu;
    if !battle {
        *counted = None;
        for (_, (_, label)) in labels.drain() {
            if let Ok(mut label) = commands.get_entity(label) {
                label.despawn();
            }
        }
        return;
    }
    let count = config.puzzle.count_seconds;
    let over = game.phase == Phase::Over;
    // Each brick still in a tower (placed and not zapped away), counted one by one.
    let mut standing: HashMap<Entity, u32> = HashMap::default();
    for (brick, placed, zapped) in &bricks {
        if placed && !zapped {
            *standing.entry(brick.tower).or_insert(0) += 1;
        }
    }
    if over {
        let since = match *counted {
            Some((round, t)) if round == game.round => t,
            _ => {
                *counted = Some((game.round, 0.0));
                0.0
            }
        };
        let t = since + time.delta_secs();
        *counted = Some((game.round, t));
    } else {
        *counted = None;
    }
    let gold = tone_color(&art, &images, Tone::Gold);
    let pink = Color::srgb(1.0, 0.45, 0.7);
    let counted_so_far = counted.map_or(0.0, |(_, t)| t);
    let most = standing.values().copied().max().unwrap_or(0);
    let draw = over && game.winner.is_none();
    for (entity, tower, state) in &towers {
        let total = standing.get(&entity).copied().unwrap_or(0);
        let shown = if over {
            let winner = game.winner == Some(tower.index);
            let step = config.puzzle.count_step.max(0.01);
            let so_far = ((counted_so_far / step).floor() as u32).min(total);
            if draw && total == most {
                Some((format!("{so_far} DRAW"), pink))
            } else {
                Some((so_far.to_string(), if winner { gold } else { pink }))
            }
        } else {
            state
                .laser_count
                .filter(|t| *t < count)
                .map(|t| ((count - t).ceil().max(1.0).to_string(), gold))
        };
        let Some((words, colour)) = shown else {
            if let Some((_, label)) = labels.remove(&entity) {
                if let Ok(mut label) = commands.get_entity(label) {
                label.despawn();
            }
            }
            continue;
        };
        if labels.get(&entity).is_some_and(|(w, _)| *w == words) {
            continue;
        }
        if let Some((_, old)) = labels.remove(&entity) {
            if let Ok(mut old) = commands.get_entity(old) {
                old.despawn();
            }
        }
        let label = spawn_match_text(&mut commands, &config, &words, config.moments.banner_size, colour);
        commands
            .entity(label)
            .insert((PuzzleScore { tower: entity }, Visibility::Hidden));
        labels.insert(entity, (words, label));
    }
}

/// Puts each Puzzle score over its own lane, near the top of the view.
fn place_puzzle_scores(
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    view: Res<ViewSize>,
    towers: Query<&Tower>,
    mut scores: Query<(&PuzzleScore, &mut Node, &ComputedNode, &mut Visibility)>,
) {
    let Ok((camera, eye)) = camera.single() else {
        return;
    };
    for (score, mut node, computed, mut visibility) in &mut scores {
        let Ok(tower) = towers.get(score.tower) else {
            continue;
        };
        let Ok(at) = camera.world_to_viewport(eye, Vec3::new(tower.origin_x, 0.0, 0.0)) else {
            continue;
        };
        centre_on(&mut node, computed, Vec2::new(at.x, view.0.y * 0.2), view.0, 1.0);
        if computed.size().x > 0.0 {
            visibility.set_if_neq(Visibility::Inherited);
        }
    }
}

/// The race countdown above the finish line of each tower holding past it, higher
/// over the leader, like the original.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn show_hold_counts(
    mut commands: Commands,
    config: Res<GameConfig>,
    game: Res<Match>,
    art: Res<Art>,
    images: Res<Assets<Image>>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    towers: Query<(Entity, &Tower, &TowerState)>,
    mut counts: Query<(
        Entity,
        &HoldCount,
        &mut Node,
        &ComputedNode,
        &Children,
        &mut Visibility,
    )>,
    mut labels: Query<&mut Text, With<MatchLabel>>,
    mut faces: Query<&mut TextColor, With<MatchLabel>>,
    view: Res<ViewSize>,
) {
    let Ok((camera, eye)) = camera.single() else {
        return;
    };
    let leader = towers
        .iter()
        .max_by(|a, b| a.2.height.total_cmp(&b.2.height))
        .map(|(e, ..)| e);
    for (entity, ..) in &towers {
        if game.mode == ModeKind::Race && !counts.iter().any(|(_, c, ..)| c.0 == entity) {
            let color = tone_color(&art, &images, Tone::Gold);
            let count = spawn_match_text(
                &mut commands,
                &config,
                "",
                config.moments.countdown_size,
                color,
            );
            commands.entity(count).insert(HoldCount(entity));
        }
    }
    for (entity, count, mut node, computed, children, mut visibility) in &mut counts {
        let Ok((_, tower, state)) = towers.get(count.0) else {
            commands.entity(entity).despawn();
            continue;
        };
        let holding = game.phase == Phase::Playing
            && game.mode == ModeKind::Race
            && state.status == TowerStatus::Building
            && state.hold_timer > 0.0;
        let (lead, other) = config.race.countdown_above;
        let above = if leader == Some(count.0) { lead } else { other };
        let at = Vec3::new(
            tower.origin_x,
            config.cells(game.finish_height + above) + config.cell_size * 0.5,
            0.0,
        );
        let Some(at) = holding
            .then(|| camera.world_to_viewport(eye, at).ok())
            .flatten()
        else {
            node.display = Display::None;
            continue;
        };
        let left = (config.race.hold_seconds - state.hold_timer)
            .ceil()
            .max(1.0);
        let label = format!("{left:.0}");
        // The leader's countdown is gold, the others' pink.
        let tone = if leader == Some(count.0) {
            Tone::Gold
        } else {
            Tone::Red
        };
        let face = tone_color(&art, &images, tone);
        if let Some(mut color) = children.get(1).and_then(|c| faces.get_mut(*c).ok())
            && color.0 != face
        {
            color.0 = face;
        }
        for child in children {
            if let Ok(mut text) = labels.get_mut(*child)
                && text.0 != label
            {
                text.0 = label.clone();
            }
        }
        node.display = Display::Grid;
        centre_on(&mut node, computed, at, view.0, 1.0);
        if computed.size().x > 0.0 {
            visibility.set_if_neq(Visibility::Inherited);
        }
    }
}
