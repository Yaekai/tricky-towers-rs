//! Match moments: the race hold countdown, the WINNER! and WIZARD OUT! banners, the
//! winner's spotlight, the veil over losing lanes and the trophy. Drawn with the
//! original's letters and art when staged (see `docs/research/match-moments.md`).

use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;

use crate::art::Art;
use crate::letters::TextLook;
use crate::brick_material::BrickFlash;
use crate::config::{curve, keyed, GameConfig, ModeKind};
use crate::gameplay::{Brick, Match, Phase, Placed, Tower, TowerState, TowerStatus};
use crate::spells::Zapped;
use crate::visuals::{FramedCamera, ViewSize};

pub struct ResultsPlugin;

impl Plugin for ResultsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PuzzleCountClock>().add_systems(
            Update,
            (
                // The count's clock first: WINNER! waits for it.
                show_puzzle_scores,
                start_moments,
                show_hold_counts,
                place_puzzle_scores,
                fade_flashes,
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
pub(crate) enum Tone {
    Gold,
    Red,
    /// EnzoBlackBronzeFront / Back (COUNT_MESSAGE_TEXT_DRAW_*, DRAW!).
    Bronze,
}

impl Tone {
    fn gradient(self) -> &'static str {
        match self {
            Tone::Gold => "text_gradient_gold",
            Tone::Red => "text_gradient_red",
            Tone::Bronze => "text_gradient_bronze",
        }
    }

    /// The front layer (EnzoBlackGoldFront / RedFront): the gradient face, no outline.
    pub(crate) fn front(self) -> TextLook {
        let face = match self {
            Tone::Gold => TextLook::GOLD,
            Tone::Red => TextLook::PINK,
            // gph_gradientTextureFont04: its top and bottom rows.
            Tone::Bronze => TextLook {
                face: Some(Color::srgb_u8(246, 147, 103)),
                low: Some(Color::srgb_u8(229, 96, 66)),
                ..TextLook::GOLD
            },
        };
        TextLook {
            face: face.face,
            low: face.low,
            ..default()
        }
    }

    /// The back layer (EnzoBlackGoldBack / RedBack): the face dilated 0.2 in wine and an
    /// outline 0.25 wide, drawn a little lower. The face isn't dilated here (ponytail: the
    /// letters cut the face at 0.5 only), so the wine band is a little thinner.
    pub(crate) fn back(self) -> TextLook {
        let wine = match self {
            Tone::Gold | Tone::Bronze => Color::srgb_u8(158, 23, 64),
            Tone::Red => Color::srgb_u8(160, 24, 101),
        };
        TextLook {
            face: Some(wine),
            outline: Some(Color::srgb_u8(37, 13, 40)),
            // The material's 0.5 - (0.2 + 0.25) x 0.833 / 2 = 0.31 drew the outline about
            // two thirds as wide as Max's Puzzle clip shows; 0.22 matches it.
            outline_edge: 0.22,
            ..default()
        }
    }

    fn fallback(self) -> Color {
        match self {
            Tone::Gold => Color::srgb_u8(255, 210, 80),
            Tone::Red => Color::srgb_u8(240, 70, 110),
            Tone::Bronze => Color::srgb_u8(235, 114, 79),
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

/// The back layer of a piece of match text (its EnzoBlack<Tone>Back material).
#[derive(Component)]
struct BackLayer;

/// A banner over a lane, and how long it has shown.
#[derive(Component)]
struct Banner {
    tower: Entity,
    out: bool,
    time: f32,
    /// WIZARD OUT!: seconds since it began to move out, once the match is over.
    leave: Option<f32>,
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

/// A winner's beam drawn as SpriteBlendModeOverlay over the background (the Race and Puzzle
/// worlds' WinnerHighlight material); its quad child is made once the grab picture exists.
#[derive(Component)]
struct OverlayBeam(Option<(Entity, Handle<crate::post::SpotOverlay>)>);

/// Match text in the original's style: a shadow a little below, the face on top.
/// Returns the container, which is positioned and scaled as one.
fn spawn_match_text(
    commands: &mut Commands,
    config: &GameConfig,
    text: &str,
    size: f32,
    color: Color,
    tone: Tone,
) -> Entity {
    let look = &config.moments;
    let [r, g, b] = look.shadow_color;
    let layer = |color: Color, drop: f32, letters: TextLook| {
        (
            MatchLabel,
            letters,
            // WinMessage, LoseMessage, the counts and the trial messages all space -1.
            crate::letters::LetterSpacing(-1.0),
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
            parent.spawn((layer(Color::srgb_u8(r, g, b), size * look.shadow_drop, tone.back()), BackLayer));
            parent.spawn(layer(color, 0.0, tone.front()));
        })
        .id()
}

/// A text look at `alpha` (its face, gradient and outline alike).
fn faded(look: TextLook, alpha: f32) -> TextLook {
    let fade = |c: Option<Color>| c.map(|c| c.with_alpha(alpha));
    TextLook {
        face: fade(look.face),
        low: fade(look.low),
        outline: fade(look.outline),
        ..look
    }
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
    camera: Query<&Projection, With<crate::visuals::MainCamera>>,
    puzzle: Res<PuzzleCountClock>,
    mut seen: Local<(u64, HashSet<(Entity, bool)>)>,
) {
    // World text: a cell's size on the screen now.
    let cell_px = config.cell_size
        / match camera.single() {
            Ok(Projection::Orthographic(o)) => o.scale.max(0.001),
            _ => 1.0,
        };
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
    // Endless runs end on FINISH too (SinglePlayerLeaderboardBaskController).
    let (win_words, win_size, out_words, out_size) = if config.trial.is_some() || game.mode.is_endless() {
        ("FINISH", look.banner_size, "FAILED", look.banner_size)
    } else {
        (
            "WINNER!",
            look.banner_cells * cell_px,
            "WIZARD OUT!",
            look.out_cells * cell_px,
        )
    };
    // A Puzzle battle's WINNER!, spotlight and veils wait for the count to reach the most
    // bricks (PuzzleGameBaskController._DoBaskEnd).
    if puzzle.counting() {
        return;
    }
    // The winner's roof goes on before WINNER!, the spotlight and the veils.
    if towers
        .iter()
        .any(|(_, tower, state)| crate::gameplay::roof_waiting(&game, tower, state))
    {
        return;
    }
    for (entity, tower, state) in &towers {
        let won = state.status == TowerStatus::Won || (over && game.winner == Some(tower.index));
        // A drawn Puzzle round: every tower with the most bricks says DRAW! (DrawMessage: font 40
        // against WINNER!'s 32.75, bronze, no sunrays) and gets the winner's light.
        let drew = puzzle.tied.contains(&entity);
        if (won || drew) && seen.1.insert((entity, true)) {
            let (words, size, tone) = if drew {
                ("DRAW!", look.banner_cells * 40.0 / 32.75 * cell_px, Tone::Bronze)
            } else {
                (win_words, win_size, Tone::Gold)
            };
            let color = tone_color(&art, &images, tone);
            let banner = spawn_match_text(&mut commands, &config, words, size, color, tone);
            commands.entity(banner).insert(Banner {
                tower: entity,
                out: false,
                time: 0.0,
                leave: None,
            });
            if let Some(image) = art.image("winner_sunrays").filter(|_| !drew) {
                let rays = commands
                    .spawn((
                        Sunrays,
                        ImageNode::new(image),
                        Node {
                            grid_row: GridPlacement::start(1),
                            grid_column: GridPlacement::start(1),
                            width: Val::Px(win_size * 4.0),
                            height: Val::Px(win_size * 4.0),
                            ..default()
                        },
                        UiTransform::default(),
                    ))
                    .id();
                commands.entity(banner).insert_children(0, &[rays]);
            }
            // No cup after a single match: cups only end a cup (not built yet).
            let beam = spawn_light(
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
            // Survival's world (World02) draws its beam as a plain sprite.
            if key != "survival" {
                commands.entity(beam).insert((OverlayBeam(None), crate::post::NeedsGrab));
            }
        }
        // Only Survival says WIZARD OUT! (its finish message); trials say FAILED.
        if state.status == TowerStatus::Out
            && (config.trial.is_some() || game.mode.rules() == ModeKind::Survival)
            && seen.1.insert((entity, false))
        {
            let color = tone_color(&art, &images, Tone::Red);
            let banner = spawn_match_text(&mut commands, &config, out_words, out_size, color, Tone::Red);
            commands.entity(banner).insert(Banner {
                tower: entity,
                out: true,
                time: 0.0,
                leave: None,
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
) -> Entity {
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
        // Winner/LoserHighlightBackground draw on their own background camera (depth 7):
        // over the scenery, under the towers.
        Transform::from_xyz(0.0, 0.0, -100.0),
        strip,
    ))
    .id()
}

#[allow(clippy::type_complexity)]
/// Fades the lights in and keeps them covering their lane from the bottom of the
/// screen to the top.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn animate_lights(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    view: Res<ViewSize>,
    camera: Query<(&Transform, &Projection), (With<crate::visuals::MainCamera>, Without<Light>)>,
    towers: Query<&Tower>,
    mut lights: Query<(Entity, &mut Light, &Strip, &mut Sprite, &mut Transform, Option<&mut OverlayBeam>)>,
    mut quads: Query<&mut Transform, (Without<Light>, Without<crate::visuals::MainCamera>)>,
    grab: Option<Res<crate::post::GrabTarget>>,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut spots: Option<ResMut<Assets<crate::post::SpotOverlay>>>,
) {
    let Ok((eye, Projection::Orthographic(ortho))) = camera.single() else {
        return;
    };
    let height = view.0.y * ortho.scale;
    let bottom = eye.translation.y - height * 0.5;
    for (entity, mut light, strip, mut sprite, mut transform, overlay) in &mut lights {
        light.time += time.delta_secs();
        let Ok(tower) = towers.get(light.tower) else {
            continue;
        };
        let fade =
            ((light.time - light.delay) / config.moments.spotlight_fade.max(0.01)).clamp(0.0, 1.0);
        let size = Vec2::new(config.cells(strip.width), height);
        sprite.custom_size = Some(size);
        let mut drawn = false;
        if let Some(mut overlay) = overlay {
            if overlay.0.is_none()
                && let (Some(grab), Some(meshes), Some(spots)) = (grab.as_ref(), meshes.as_mut(), spots.as_mut())
            {
                let spot = spots.add(crate::post::SpotOverlay {
                    colour: Vec4::new(1.0, 1.0, 1.0, 0.0),
                    picture: sprite.image.clone(),
                    grab: grab.0.clone(),
                });
                let quad = commands
                    .spawn((
                        Mesh2d(meshes.add(Rectangle::new(1.0, 1.0))),
                        MeshMaterial2d(spot.clone()),
                        Transform::default(),
                    ))
                    .id();
                commands.entity(entity).add_child(quad);
                overlay.0 = Some((quad, spot));
            }
            if let Some((quad, spot)) = &overlay.0 {
                // The sprite stands on its bottom middle; the quad is centred.
                if let Ok(mut q) = quads.get_mut(*quad) {
                    q.translation = Vec3::new(0.0, size.y * 0.5, 0.0);
                    q.scale = size.extend(1.0);
                }
                if let Some(spots) = spots.as_mut()
                    && let Some(mut m) = spots.get_mut(spot)
                {
                    m.colour.w = light.alpha * fade;
                }
                sprite.color = sprite.color.with_alpha(0.0);
                drawn = true;
            }
        }
        if !drawn {
            sprite.color = sprite.color.with_alpha(light.alpha * fade);
        }
        transform.translation.x = if strip.screen {
            // The whole-screen strip follows the camera.
            eye.translation.x
        } else {
            tower.origin_x + config.cells(strip.from * strip.side)
        };
        transform.translation.y = bottom;
    }
}

/// Moves the banners in, keeps them over their lane, and moves WIZARD OUT! away again.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn animate_banners(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    view: Res<ViewSize>,
    game: Res<Match>,
    camera: Query<(&Camera, &GlobalTransform, &Projection), With<crate::visuals::MainCamera>>,
    towers: Query<(&Tower, &TowerState)>,
    mut banners: Query<(
        Entity,
        &mut Banner,
        &mut Node,
        &ComputedNode,
        &mut UiTransform,
        &mut Visibility,
        &Children,
    )>,
    mut looks: Query<(&mut TextLook, Has<BackLayer>), With<MatchLabel>>,
    mut rays: Query<&mut UiTransform, (With<Sunrays>, Without<Banner>, Without<Trophy>)>,
    mut cups: Query<&mut UiTransform, (With<Trophy>, Without<Banner>, Without<Sunrays>)>,
    scores: Query<&PuzzleScore>,
    art: Res<Art>,
    mut seed: Local<u32>,
) {
    let Ok((camera, eye, projection)) = camera.single() else {
        return;
    };
    let look = &config.moments;
    if *seed == 0 {
        *seed = 0x51A7_3C11;
    }
    let half_view = view.0.y
        * 0.5
        * match projection {
            Projection::Orthographic(o) => o.scale,
            _ => 1.0,
        };
    let view_top = eye.translation().y + half_view;
    let dt = time.delta_secs();
    for (entity, mut banner, mut node, computed, mut transform, mut visibility, children) in &mut banners {
        banner.time += dt;
        let Ok((tower, state)) = towers.get(banner.tower) else {
            continue;
        };
        // A Puzzle battle's WINNER! throws the winner's stars from its count once it has
        // moved in (PuzzleGameBaskController's "inComplete"); DRAW has none.
        let moved_in = look.winner_in_x.last().map_or(0.0, |k| k.0);
        if !banner.out
            && game.mode == ModeKind::Puzzle
            && config.trial.is_none()
            && game.winner.is_some()
            && banner.time - dt < moved_in
            && banner.time >= moved_in
            && let Some(to) = scores
                .iter()
                .find(|s| s.tower == banner.tower)
                .and_then(|s| s.rise.map(|r| r.1))
        {
            let rules = &config.puzzle;
            let low = (state.laser(&config) + rules.count_over_laser) * config.cell_size;
            let high = view_top - rules.count_under_top * config.cell_size;
            let at = Vec2::new(tower.origin_x, low + to * (high - low));
            crate::particles::winner_stars(&mut commands, &art, config.cell_size, at, &mut seed);
        }
        // WINNER! stands 10 cells over its tower's top, or in Puzzle 11 under the view's
        // top; WIZARD OUT! too, moved by its clips (GameFinishController).
        let t = banner.time;
        if banner.out && game.phase == Phase::Over && banner.leave.is_none() {
            banner.leave = Some(0.0);
        }
        if let Some(leave) = banner.leave.as_mut() {
            *leave += dt;
        }
        let drop = match (banner.out, banner.leave) {
            (false, _) => 0.0,
            (true, Some(leave)) => curve(&look.out_leave, leave),
            (true, None) => {
                let moved_in = look.out_in.last().map_or(0.0, |k| k.0);
                let idle = look.out_idle.last().map_or(1.0, |k| k.0).max(0.01);
                if t < moved_in {
                    curve(&look.out_in, t)
                } else {
                    curve(&look.out_idle, (t - moved_in) % idle)
                }
            }
        };
        // Puzzle battles place it under the view's top (PuzzleGameBask); single-player runs
        // and trials over the tower (SinglePlayerBask's _GetEndTextPos).
        let world_y = if game.mode == ModeKind::Puzzle && config.trial.is_none() {
            view_top - config.cells(look.winner_below_top)
        } else {
            config.cells(state.height.max(0.0) + look.winner_above + drop)
        };
        let Ok(at) = camera.world_to_viewport(eye, Vec3::new(tower.origin_x, world_y, 0.0)) else {
            continue;
        };
        centre_on(&mut node, computed, at, view.0, 1.4);
        // Shown once laid out, so it never flashes in the wrong place.
        if computed.size().x > 0.0 {
            visibility.set_if_neq(Visibility::Inherited);
        }
        // WIZARD OUT! fades in and out with its clips (both layers' font alpha).
        if banner.out {
            let share = |t: f32, (from, to): (f32, f32)| ((t - from) / (to - from).max(0.001)).clamp(0.0, 1.0);
            let alpha = match banner.leave {
                Some(leave) => 1.0 - share(leave, look.out_fade_out),
                None => share(t, look.out_fade_in),
            };
            for child in children.iter() {
                if let Ok((mut letters, back)) = looks.get_mut(child) {
                    let base = if back { Tone::Red.back() } else { Tone::Red.front() };
                    let want = faded(base, alpha);
                    if *letters != want {
                        *letters = want;
                    }
                }
            }
        }
        if banner.out {
            if banner.leave.is_some_and(|l| l > look.out_leave.last().map_or(0.0, |k| k.0)) {
                commands.entity(entity).despawn();
            }
        } else {
            let moved_in = look.winner_in_x.last().map_or(0.0, |k| k.0);
            let ((x, y), at) = match look.winner_idle_x.last() {
                Some(&(length, ..)) if t > moved_in && length > 0.0 => {
                    ((&look.winner_idle_x, &look.winner_idle_y), (t - moved_in) % length)
                }
                _ => ((&look.winner_in_x, &look.winner_in_y), t),
            };
            transform.scale = Vec2::new(curve(x, at), curve(y, at));
            // Unity turns counter-clockwise for a positive z; UI rotation runs the other way.
            transform.rotation = Rot2::degrees(-keyed(&look.winner_turn, t));
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
/// While counting, `rise` holds where the number is going (PuzzleGameBaskController): the
/// count's height as a share of the way from over the laser to under the view top, before
/// and after the last brick counted, the seconds since it, and whether it tips.
#[derive(Component)]
struct PuzzleScore {
    tower: Entity,
    rise: Option<(f32, f32, f32, bool)>,
}

/// A Puzzle battle's count clock: seconds since the round ended, and when the count reaches
/// the most bricks (`PuzzleGameBaskController._DoBaskEnd`: WINNER! then, the popup after
/// `count_end_wait`). `end` is `None` outside a Puzzle battle's end.
#[derive(Resource, Default, Debug)]
pub struct PuzzleCountClock {
    pub since: f32,
    pub end: Option<f32>,
    /// The towers sharing the most bricks in a drawn round (each says DRAW!).
    pub tied: Vec<Entity>,
}

impl PuzzleCountClock {
    /// Whether the count is still running (WINNER! and the popup wait for it).
    pub fn counting(&self) -> bool {
        self.end.is_some_and(|end| self.since < end)
    }
}

/// When the count reaches the most bricks: `delay`, then `step` a brick, `pause` after a
/// count that is some tower's total.
pub fn puzzle_count_end(totals: &[u32], delay: f32, step: f32, pause: f32) -> f32 {
    let most = totals.iter().copied().max().unwrap_or(0);
    let mut at = delay;
    let mut wait = step;
    for tick in 1..=most {
        at += wait;
        wait = if totals.contains(&tick) { pause } else { step };
    }
    at
}

/// The Puzzle battle's count: the 3-2-1 over the lane that touched the laser, then each
/// tower's bricks counted together, one at a time, each number riding up its lane. A tie
/// for the most bricks says DRAW.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn show_puzzle_scores(
    mut commands: Commands,
    config: Res<GameConfig>,
    game: Res<Match>,
    art: Res<Art>,
    images: Res<Assets<Image>>,
    towers: Query<(Entity, &Tower, &TowerState)>,
    bricks: Query<(Entity, &Brick, Option<&Placed>, Has<Zapped>)>,
    time: Res<Time>,
    mut counted: Local<Option<(u64, f32)>>,
    mut clock: ResMut<PuzzleCountClock>,
    mut labels: Local<HashMap<Entity, (String, Entity)>>,
    camera: Query<&Projection, With<crate::visuals::MainCamera>>,
) {
    let cell_px = config.cell_size
        / match camera.single() {
            Ok(Projection::Orthographic(o)) => o.scale.max(0.001),
            _ => 1.0,
        };
    let battle = game.mode == ModeKind::Puzzle
        && config.trial.is_none()
        && game.phase != Phase::Menu;
    if !battle {
        *counted = None;
        clock.end = None;
        for (_, (_, label)) in labels.drain() {
            if let Ok(mut label) = commands.get_entity(label) {
                label.despawn();
            }
        }
        return;
    }
    let rules = &config.puzzle;
    let over = game.phase == Phase::Over;
    // Each brick still in a tower (placed and not zapped away), counted one by one.
    let mut standing: HashMap<Entity, u32> = HashMap::default();
    for (_, brick, placed, zapped) in &bricks {
        if placed.is_some() && !zapped {
            *standing.entry(brick.tower).or_insert(0) += 1;
        }
    }
    if over {
        let since = match *counted {
            Some((round, t)) if round == game.round => t,
            _ => 0.0,
        };
        *counted = Some((game.round, since + time.delta_secs()));
    } else {
        *counted = None;
    }
    let counts: Vec<u32> = towers
        .iter()
        .map(|(entity, _, _)| standing.get(&entity).copied().unwrap_or(0))
        .collect();
    let most = counts.iter().copied().max().unwrap_or(0);
    match *counted {
        Some((_, t)) => {
            clock.since = t;
            clock.end = Some(puzzle_count_end(&counts, rules.count_delay, rules.count_step, rules.count_pause));
        }
        None => clock.end = None,
    }
    // DRAW only when two or more towers share the most bricks.
    let draw = over && puzzle_tie(&counts);
    clock.tied = if draw {
        towers
            .iter()
            .filter(|(entity, ..)| standing.get(entity).copied().unwrap_or(0) == most)
            .map(|(entity, ..)| entity)
            .collect()
    } else {
        Vec::new()
    };
    let ticks = counted.and_then(|(_, t)| {
        puzzle_ticks(
            t - rules.count_delay,
            &counts,
            rules.count_step,
            rules.count_pause,
        )
    });
    let height = |n: u32| {
        if most == 0 {
            rules.count_curve.exp() - 1.0
        } else {
            (rules.count_curve * n as f32 / most as f32).exp() - 1.0
        }
    };
    for (entity, _tower, state) in &towers {
        let total = standing.get(&entity).copied().unwrap_or(0);
        let shown = if over {
            ticks.map(|(raw, since)| {
                let n = raw.min(total);
                // A tower stops a tick after its own total; the most bricks stop at once.
                let done = if total == most {
                    raw >= most
                } else {
                    raw > total
                };
                let counting = raw > 0 && raw <= total;
                // The brick just counted flashes white, in the order they were placed.
                if counting && since < time.delta_secs() {
                    let mut order: Vec<_> = bricks
                        .iter()
                        .filter(|(_, b, p, z)| b.tower == entity && p.is_some() && !z)
                        .map(|(e, _, p, _)| (p.map_or(0, |p| p.0), e))
                        .collect();
                    order.sort_unstable();
                    if let Some(&(_, brick)) = order.get(n as usize - 1) {
                        // PuzzleGameBaskController: back to its pack's look (stone and
                        // grass go, ice stays), then the flash.
                        commands
                            .entity(brick)
                            .remove::<(crate::spells::Stone, crate::spells::Grassy)>()
                            .try_insert(BrickFlash(1.0))
                            .queue_silenced(|mut e: EntityWorldMut| {
                                if let Some(mut b) = e.get_mut::<crate::gameplay::Brick>() {
                                    b.set_changed();
                                }
                            });
                    }
                }
                let from = if counting { height(n - 1) } else { height(n) };
                // The count keeps its number; a tie turns it bronze and DRAW! comes over it
                // (PuzzleGameBaskController._ShowCountMessageFinalState / DRAW_TEXT).
                let words = n.to_string();
                let size = if done && total == most {
                    config.moments.score_cells
                } else {
                    config.moments.count_cells
                };
                // A loser's final count turns red (COUNT_MESSAGE_TEXT_LOSER_*).
                let tone = if done && total < most {
                    Tone::Red
                } else if done && draw {
                    Tone::Bronze
                } else {
                    Tone::Gold
                };
                (words, size, tone, Some((from, height(n), since, counting)))
            })
        } else {
            state
                .laser_count
                .filter(|t| *t < rules.count_seconds)
                .map(|t| {
                    let words = (rules.count_seconds - t).ceil().max(1.0).to_string();
                    (words, config.moments.countdown_cells, Tone::Gold, None)
                })
        };
        let Some((words, size, tone, rise)) = shown else {
            if let Some((_, label)) = labels.remove(&entity) {
                if let Ok(mut label) = commands.get_entity(label) {
                    label.despawn();
                }
            }
            continue;
        };
        let key = format!("{words}|{size}|{tone:?}");
        let label = match labels.get(&entity) {
            Some((k, label)) if *k == key => *label,
            _ => {
                if let Some((_, old)) = labels.remove(&entity) {
                    if let Ok(mut old) = commands.get_entity(old) {
                        old.despawn();
                    }
                }
                let label = spawn_match_text(
                    &mut commands,
                    &config,
                    &words,
                    size * cell_px,
                    tone_color(&art, &images, tone),
                    tone,
                );
                // Over WINNER!'s sunrays, which would otherwise wash it out.
                commands.entity(label).insert((Visibility::Hidden, GlobalZIndex(21)));
                labels.insert(entity, (key, label));
                label
            }
        };
        commands.entity(label).insert(PuzzleScore {
            tower: entity,
            rise,
        });
    }
}

/// Fades each brick's flash out over `PuzzleRules::count_flash` (FlashBrickModifier).
fn fade_flashes(
    mut commands: Commands,
    config: Res<GameConfig>,
    time: Res<Time>,
    mut flashes: Query<(Entity, &mut BrickFlash)>,
) {
    for (entity, mut flash) in &mut flashes {
        flash.0 -= time.delta_secs() / config.puzzle.count_flash.max(0.001);
        if flash.0 <= 0.0 {
            commands.entity(entity).remove::<BrickFlash>();
        }
    }
}

/// The shared count `t` seconds after it starts, for towers holding `totals` bricks:
/// how many ticks so far and the seconds since the last. A tick comes every `step`, or
/// `pause` after one that reached some tower's total, and stops at the most bricks.
pub fn puzzle_ticks(t: f32, totals: &[u32], step: f32, pause: f32) -> Option<(u32, f32)> {
    if t < 0.0 {
        return None;
    }
    let most = totals.iter().copied().max().unwrap_or(0);
    let (mut ticks, mut at, mut wait) = (0, 0.0, step);
    while ticks < most && at + wait <= t {
        at += wait;
        ticks += 1;
        wait = if totals.contains(&ticks) { pause } else { step };
    }
    Some((ticks, t - at))
}

/// Puts each Puzzle score over its own lane: the 3-2-1 near the top of the view, the
/// count riding up from over the laser towards the view top as it counts.
#[allow(clippy::type_complexity)]
fn place_puzzle_scores(
    config: Res<GameConfig>,
    camera: Query<(&Camera, &GlobalTransform), With<crate::visuals::MainCamera>>,
    view: Res<ViewSize>,
    towers: Query<(&Tower, &TowerState)>,
    mut scores: Query<(
        &PuzzleScore,
        &mut Node,
        &ComputedNode,
        &mut UiTransform,
        &mut Visibility,
    )>,
) {
    let Ok((camera, eye)) = camera.single() else {
        return;
    };
    let rules = &config.puzzle;
    let c = config.cell_size;
    let top = camera
        .viewport_to_world_2d(eye, Vec2::ZERO)
        .map_or(0.0, |p| p.y);
    for (score, mut node, computed, mut transform, mut visibility) in &mut scores {
        let Ok((tower, state)) = towers.get(score.tower) else {
            continue;
        };
        let at = match score.rise {
            Some((from, to, since, tips)) => {
                // DOTween's default OutQuad over the tip time, for both the rise and the tip.
                let k = (since / rules.count_tip_time.max(0.001)).clamp(0.0, 1.0);
                let k = 1.0 - (1.0 - k) * (1.0 - k);
                let low = (state.laser(&config) + rules.count_over_laser) * c;
                let high = top - rules.count_under_top * c;
                let y = low + (from + (to - from) * k) * (high - low);
                let tip = if tips {
                    rules.count_tip * (1.0 - k)
                } else {
                    0.0
                };
                // Unity's +z turn is anticlockwise; Bevy's UI turns clockwise for positive angles.
                transform.rotation = Rot2::degrees(-tip);
                camera.world_to_viewport(eye, Vec3::new(tower.origin_x, y, 0.0))
            }
            // The 3-2-1 over its own tower's top (CountDownEffect; measured on Max's Puzzle
            // clip, 7.6 cells over the top brick).
            None => {
                transform.rotation = Rot2::IDENTITY;
                let y = config.cells(state.height.max(0.0) + config.moments.countdown_above);
                camera.world_to_viewport(eye, Vec3::new(tower.origin_x, y, 0.0))
            }
        };
        let Ok(at) = at else {
            continue;
        };
        centre_on(&mut node, computed, at, view.0, 1.0);
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
    camera: Query<(&Camera, &GlobalTransform), With<crate::visuals::MainCamera>>,
    towers: Query<(Entity, &Tower, &TowerState)>,
    mut counts: Query<(
        Entity,
        &HoldCount,
        &mut Node,
        &ComputedNode,
        &Children,
        &mut Visibility,
        Option<&mut UiTransform>,
    )>,
    mut labels: Query<&mut Text, With<MatchLabel>>,
    mut faces: Query<&mut TextColor, With<MatchLabel>>,
    mut looks: Query<(&mut TextLook, Has<BackLayer>), With<MatchLabel>>,
    view: Res<ViewSize>,
) {
    let Ok((camera, eye)) = camera.single() else {
        return;
    };
    // CountDownEffect: the first count started is rank 1 (big, gold, its sound heard), the
    // others rank 2, and all stand over the highest tower (UpdatePosition).
    let leader = towers
        .iter()
        .filter(|t| t.2.hold_timer > 0.0)
        .max_by(|a, b| a.2.hold_timer.total_cmp(&b.2.hold_timer))
        .map(|(e, ..)| e);
    let highest = towers.iter().map(|t| t.2.height).fold(0.0f32, f32::max);
    for (entity, ..) in &towers {
        let counted = matches!(
            game.mode,
            ModeKind::Race | ModeKind::Survival | ModeKind::Puzzle | ModeKind::EndlessPuzzle
        );
        if counted && !counts.iter().any(|(_, c, ..)| c.0 == entity) {
            let color = tone_color(&art, &images, Tone::Gold);
            let count = spawn_match_text(
                &mut commands,
                &config,
                "",
                config.moments.countdown_size,
                color,
                Tone::Gold,
            );
            commands.entity(count).insert(HoldCount(entity));
        }
    }
    for (entity, count, mut node, computed, children, mut visibility, transform) in &mut counts {
        let Ok((_, tower, state)) = towers.get(count.0) else {
            commands.entity(entity).despawn();
            continue;
        };
        let left = state.count_left(&config).filter(|_| game.phase == Phase::Playing);
        let holding = left.is_some();
        let (lead, other) = config.race.countdown_above;
        let above = if leader == Some(count.0) { lead } else { other };
        let at = Vec3::new(
            tower.origin_x,
            config.cells(highest + above) + config.cell_size * 0.5,
            0.0,
        );
        let Some(at) = holding
            .then(|| camera.world_to_viewport(eye, at).ok())
            .flatten()
        else {
            node.display = Display::None;
            continue;
        };
        let left = left.unwrap_or(0.0).ceil().max(1.0);
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
        // Children include the letter twins, so each layer goes by its marker.
        for child in children.iter() {
            if let Ok((mut look, back)) = looks.get_mut(child) {
                let wanted = if back { tone.back() } else { tone.front() };
                if *look != wanted {
                    *look = wanted;
                }
            }
        }
        for child in children {
            if let Ok(mut text) = labels.get_mut(*child)
                && text.0 != label
            {
                text.0 = label.clone();
            }
        }
        node.display = Display::Grid;
        // Rank 2 counts are 36 against rank 1's 52.
        let scale = if leader == Some(count.0) { 1.0 } else { config.race.countdown_small };
        if let Some(mut transform) = transform
            && transform.scale != Vec2::splat(scale)
        {
            transform.scale = Vec2::splat(scale);
        }
        centre_on(&mut node, computed, at, view.0, 1.0);
        if computed.size().x > 0.0 {
            visibility.set_if_neq(Visibility::Inherited);
        }
    }
}

/// Whether a Puzzle round is a draw: the most bricks are shared by two or more towers
/// (a round with no bricks at all is not).
pub fn puzzle_tie(counts: &[u32]) -> bool {
    let most = counts.iter().copied().max().unwrap_or(0);
    most > 0 && counts.iter().filter(|&&n| n == most).count() > 1
}

#[cfg(test)]
mod tests {
    use super::{puzzle_ticks, puzzle_tie};

    #[test]
    fn the_count_waits_at_each_towers_total_and_stops_at_the_most() {
        let at = |t: f32| puzzle_ticks(t, &[2, 4], 0.15, 0.8).map(|(n, _)| n);
        assert_eq!(at(-0.1), None, "nothing before the count starts");
        assert_eq!(at(0.1), Some(0));
        assert_eq!(at(0.31), Some(2));
        // 2 is a tower's total, so the next brick waits 0.8 s.
        assert_eq!(at(0.31 + 0.7), Some(2));
        assert_eq!(at(0.31 + 0.8), Some(3));
        assert_eq!(at(100.0), Some(4), "stops at the most bricks");
    }

    #[test]
    fn the_count_ends_when_it_reaches_the_most_bricks() {
        // 1.5 s, then 0.15 to 1, 0.15 to 2 (a total: 0.8 to wait), 0.8 to 3, 0.15 to 4.
        let end = super::puzzle_count_end(&[2, 4], 1.5, 0.15, 0.8);
        assert!((end - 2.75).abs() < 1e-4, "{end}");
        assert_eq!(super::puzzle_count_end(&[0, 0], 1.5, 0.15, 0.8), 1.5);
    }

    #[test]
    fn only_a_shared_most_is_a_draw() {
        assert!(puzzle_tie(&[4, 4]));
        assert!(puzzle_tie(&[2, 5, 5]));
        assert!(!puzzle_tie(&[4, 3]), "one tower has the most");
        assert!(!puzzle_tie(&[0, 0]), "no bricks is not a draw");
        assert!(!puzzle_tie(&[]));
    }
}
