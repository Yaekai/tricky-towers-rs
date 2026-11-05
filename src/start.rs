//! "Press Enter to start": the logo drops in and bounces, a brick lands on it
//! and shakes the rubble, then the prompt blinks under it. Layout and timings
//! from the original (docs/research/menus.md, "Start screen"), in pixels of a
//! 1920×1080 screen; the logo pieces are pictures at 26 px a unit.

use bevy::input::gamepad::{GamepadRumbleIntensity, GamepadRumbleRequest};
use bevy::prelude::*;

use crate::art::Art;
use crate::config::{curve, GameConfig, StartLook};
use crate::gameplay::{Match, Phase};
use crate::menu::{MenuScreen, ScreenTransition};
use crate::select::{place, shown};

/// Pixels a unit of the logo's layout, at 1080 high.
const UNIT: f32 = 26.0;
/// The prompt bar's height and centre below the screen centre, its words' size, the
/// space each side of the words, and the key picture's height.
const PROMPT: (f32, f32) = (63.0, -218.0);
const PROMPT_SIZE: f32 = 46.0;
const PROMPT_PAD: f32 = 64.0;
const PROMPT_KEY: f32 = 46.0;
/// The gap between a word and the key picture (a space at 46 px).
const PROMPT_GAP: f32 = 12.0;

/// A piece of the prompt: the words before the key, the key, the words after.
#[derive(Component, Clone, Copy, PartialEq)]
pub(crate) enum PromptPart {
    Before,
    Key,
    After,
}

#[derive(Component)]
pub(crate) struct StartRoot;
#[derive(Component)]
pub(crate) struct StartPrompt;

/// What a logo piece is, for how it moves.
#[derive(Clone, Copy, PartialEq, Debug)]
enum PieceKind {
    Logo,
    Brick,
    Debris(usize),
    Plant(usize),
}

/// One picture of the logo, placed in units from the logo's centre.
#[derive(Component)]
pub(crate) struct LogoPiece {
    image: Option<Handle<Image>>,
    at: Vec2,
    scale: f32,
    kind: PieceKind,
    /// The sprite's pivot height (0 bottom .. 1 top): the brick and the plants stand on
    /// their foot (gph_logoBrick and gph_logoPlantsNN pivot at 0.5, 0).
    pivot_y: f32,
}

#[derive(Resource, Default)]
pub(crate) struct StartClock {
    intro: f32,
    heard: usize,
    rumble_at: Option<f32>,
    blink: f32,
    prompt_hidden: bool,
}

pub(crate) fn spawn_start_screen(
    mut commands: Commands,
    art: Option<Res<Art>>,
) {
    let image = |name: &str| art.as_ref().and_then(|a| a.image(name));
    let mut root = commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        // The live Race scene shows through (visuals::MenuScene).
        GlobalZIndex(11),
        Visibility::Hidden,
        StartRoot,
    ));
    // The pieces, back to front: the brick, the plants behind the logo, then the logo
    // and the rubble. With only the combined logo staged it moves alone.
    let mut pieces: Vec<(String, Vec2, f32, PieceKind)> = Vec::new();
    let plain = image("logo_plain").is_some();
    if plain {
        // The brick sits at z 0 behind the logo, plants and rubble (all at z -1): its foot
        // is hidden by the letters it lands on.
        pieces.push(("logo_brick".into(), BRICK, 1.0, PieceKind::Brick));
        for (i, at) in PLANTS.iter().enumerate() {
            pieces.push((format!("logo_plants_{}", i + 1), *at, 1.0, PieceKind::Plant(i)));
        }
        pieces.push(("logo_plain".into(), LOGO, 1.0, PieceKind::Logo));
        for (i, at) in DEBRIS.iter().enumerate() {
            pieces.push((
                format!("logo_debris_{}", i + 1),
                *at,
                1.0,
                PieceKind::Debris(i),
            ));
        }
        for (i, &(at, scale)) in SMALL_DEBRIS.iter().enumerate() {
            pieces.push(("logo_debris_8".into(), at, scale, PieceKind::Debris(DEBRIS.len() + i)));
        }
    } else {
        pieces.push(("logo".into(), LOGO, 1.0, PieceKind::Logo));
    }
    let any = pieces.iter().any(|(name, ..)| image(name).is_some());
    root.with_children(|root| {
        for (name, at, scale, kind) in pieces {
            let Some(picture) = image(&name) else {
                continue;
            };
            let pivot_y = art
                .as_ref()
                .and_then(|a| a.entries.get(&name))
                .and_then(|e| e.pivot)
                .map_or(0.5, |p| p.1);
            root.spawn((
                ImageNode::new(picture.clone()),
                Node {
                    position_type: PositionType::Absolute,
                    ..default()
                },
                UiTransform::default(),
                Visibility::Hidden,
                LogoPiece {
                    image: Some(picture),
                    at,
                    scale,
                    kind,
                    pivot_y,
                },
            ));
        }
        if !any {
            root.spawn((place((1920.0, 200.0), (0.0, 182.0)), LogoPiece {
                image: None, at: LOGO, scale: 1.0, kind: PieceKind::Logo, pivot_y: 0.5,
            }))
                .with_child((
                    Text::new("Tricky Towers"),
                    TextFont {
                        font_size: FontSize::Vh(12.0),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
        }
        // The prompt (StartMessage): "PRESS [key] TO START" in EnzoBlackGold46 on the
        // bar with the brick ends, as wide as its words and key plus 64 px a side
        // (its ContentSizeFitter and padding). Its own box, so blinking hides the
        // letter-drawn copies too; show_start_screen lays it out.
        let mut bar = root.spawn((
            place((PROMPT_PAD * 2.0, PROMPT.0), (0.0, PROMPT.1)),
            Visibility::Hidden,
            StartPrompt,
        ));
        if let Some(back) = image("start_message_background") {
            bar.insert(crate::select::sliced(back, [77.0, 79.0, 0.0, 0.0]));
        }
        bar.with_children(|b| {
            for (part, words) in [(PromptPart::Before, "PRESS"), (PromptPart::After, "TO START")] {
                b.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    part,
                ))
                .with_child((
                    Text::new(words),
                    TextFont {
                        font_size: FontSize::Vh(PROMPT_SIZE / 10.8),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    crate::letters::TextLook::GOLD,
                ));
            }
            b.spawn((
                ImageNode::default(),
                Node {
                    position_type: PositionType::Absolute,
                    ..default()
                },
                PromptPart::Key,
            ));
        });
    });
}

/// Brick, plants, rubble and the small rubble copies (with their sizes), in
/// units from the logo's centre, y up.
const BRICK: Vec2 = Vec2::new(-1.78, 10.11);
/// The logo picture sits 7 units over LogoContainer's origin (the Start scene's Logo).
const LOGO: Vec2 = Vec2::new(0.0, 7.0);
const PLANTS: [Vec2; 4] = [
    Vec2::new(-9.67, 11.71),
    Vec2::new(-5.38, 11.79),
    Vec2::new(3.33, 11.66),
    Vec2::new(9.795, 11.206),
];
const DEBRIS: [Vec2; 7] = [
    Vec2::new(-1.02, 0.49),
    Vec2::new(0.45, 1.12),
    Vec2::new(3.58, 0.13),
    Vec2::new(9.36, 0.35),
    Vec2::new(5.32, 1.7),
    Vec2::new(-6.14, 0.16),
    Vec2::new(-7.27, 1.19),
];
const SMALL_DEBRIS: &[(Vec2, f32)] = &[
    (Vec2::new(-5.59, 1.56), 0.846),
    (Vec2::new(-5.03, -0.92), 0.846),
    (Vec2::new(2.82, -1.29), 0.692),
    (Vec2::new(7.93, -0.46), 0.521),
    (Vec2::new(8.63, 1.01), 0.658),
    (Vec2::new(9.06, 1.35), 0.587),
    (Vec2::new(11.36, 1.36), 0.658),
    (Vec2::new(-9.03, 0.59), 0.587),
    (Vec2::new(-9.77, 0.94), 0.736),
];

/// Continue can skip MoveIn, then plays Rumble, then skips it on a second press.
pub(crate) fn start_input(
    mut commands: Commands,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    gamepads: Query<(Entity, &Gamepad)>,
    game: Res<Match>,
    art: Option<Res<Art>>,
    config: Res<GameConfig>,
    time: Res<Time>,
    screen: Res<MenuScreen>,
    mut transition: ResMut<ScreenTransition>,
    mut clock: ResMut<StartClock>,
    mut rumble: Option<ResMut<Messages<GamepadRumbleRequest>>>,
) {
    if game.phase != Phase::Menu || *screen != MenuScreen::Start {
        return;
    }
    let key = keys.is_some_and(|k| {
        k.any_just_pressed([KeyCode::Enter, KeyCode::Space, KeyCode::NumpadEnter])
    });
    let pad = gamepads.iter().find_map(|(entity, p)| {
        p.any_just_pressed([crate::input::continue_back(p).0, GamepadButton::Start]).then_some(entity)
    });
    let look = &config.start;
    let now = time.elapsed_secs();
    let pressed = key || pad.is_some();
    if mouse.is_some_and(|m| m.just_pressed(MouseButton::Left))
        || clock.rumble_at.is_some_and(|at| pressed || now - at >= look.rumble_end)
    {
        transition.menu(MenuScreen::Main);
        return;
    }
    if !pressed {
        return;
    }
    if clock.intro < look.prompt_after {
        clock.intro = look.prompt_after;
        clock.heard = look.sounds.len();
        return;
    }
    clock.rumble_at = Some(now);
    if let (Some(gamepad), Some(rumble)) = (pad, rumble.as_mut()) {
        let (seconds, strength) = look.rumble_pad;
        rumble.write(GamepadRumbleRequest::Add {
            gamepad,
            duration: std::time::Duration::from_secs_f32(seconds.max(0.0)),
            intensity: GamepadRumbleIntensity::strong_motor(strength),
        });
    }
    for name in ["sfx_start_game_v2", "sfx_brick_heavy", "sfx_menu_crumble"] {
        if let Some(handle) = art.as_deref().and_then(|a| a.sounds.get(name)) {
            commands.spawn((AudioPlayer(handle.clone()), PlaybackSettings::DESPAWN));
        }
    }
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(crate) fn show_start_screen(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    game: Res<Match>,
    screen: Res<MenuScreen>,
    art: Option<Res<Art>>,
    images: Option<Res<Assets<Image>>>,
    mut clock: ResMut<StartClock>,
    pads: Query<&Gamepad>,
    mut prompt_parts: Query<
        (&PromptPart, &mut Node, Option<&mut ImageNode>),
        (Without<StartPrompt>, Without<LogoPiece>),
    >,
    mut prompt_bar: Query<&mut Node, (With<StartPrompt>, Without<PromptPart>, Without<LogoPiece>)>,
    mut root: Query<&mut Visibility, With<StartRoot>>,
    mut prompt: Query<&mut Visibility, (With<StartPrompt>, Without<StartRoot>)>,
    mut pieces: Query<
        (&LogoPiece, &mut Node, &mut UiTransform, &mut Visibility),
        (Without<StartRoot>, Without<StartPrompt>, Without<PromptPart>),
    >,
) {
    let on = game.phase == Phase::Menu && *screen == MenuScreen::Start;
    let playstation = pads.iter().next().is_some_and(crate::input::is_playstation);
    // The key picture: Space, or the pad's A / Cross once one is plugged in.
    let swapped = pads.iter().next().is_some_and(crate::input::swaps_continue);
    let key = match (!pads.is_empty(), playstation) {
        (false, _) => "menu_key_space",
        (true, false) => "menu_pad_a",
        (true, true) if swapped => "menu_pad_circle",
        (true, true) => "menu_pad_cross",
    };
    if let Some(art) = art.as_deref() {
        let key_image = art.image(key);
        let key_w = key_image
            .as_ref()
            .and_then(|h| images.as_ref()?.get(h))
            .map_or(PROMPT_KEY, |i| PROMPT_KEY * i.width() as f32 / i.height().max(1) as f32);
        let before = crate::letters::translated_width(art, "PRESS", PROMPT_SIZE);
        let after = crate::letters::translated_width(art, "TO START", PROMPT_SIZE);
        let inner = before + PROMPT_GAP + key_w + PROMPT_GAP + after;
        let px = |v: f32| Val::Vh(v / 10.8);
        for mut bar in &mut prompt_bar {
            let want = inner + 2.0 * PROMPT_PAD;
            if bar.width != Val::Percent(want / 19.2) {
                bar.width = Val::Percent(want / 19.2);
                bar.left = Val::Percent(50.0 - want / 2.0 / 19.2);
            }
        }
        for (part, mut node, image) in &mut prompt_parts {
            let (left, width) = match part {
                PromptPart::Before => (PROMPT_PAD, before),
                PromptPart::Key => (PROMPT_PAD + before + PROMPT_GAP, key_w),
                PromptPart::After => (PROMPT_PAD + before + 2.0 * PROMPT_GAP + key_w, after),
            };
            if node.left != px(left) || node.width != px(width) {
                node.left = px(left);
                node.width = px(width);
            }
            if *part == PromptPart::Key {
                let want_top = px((PROMPT.0 - PROMPT_KEY) / 2.0);
                if node.top != want_top || node.height != px(PROMPT_KEY) {
                    node.top = want_top;
                    node.height = px(PROMPT_KEY);
                }
                if let (Some(mut img), Some(h)) = (image, key_image.clone())
                    && img.image != h
                {
                    img.image = h;
                }
            }
        }
    }
    for mut v in &mut root {
        v.set_if_neq(shown(on));
    }
    if !on {
        *clock = StartClock::default();
        return;
    }
    clock.intro += time.delta_secs();
    let t = clock.intro;
    let look: &StartLook = &config.start;
    let rumble = clock.rumble_at.map(|at| time.elapsed_secs() - at);
    let fall = look.brick_y.first().map_or(0.0, |k| k.0);
    // The clip's sound events, each once.
    while let Some((_, names)) = look.sounds.get(clock.heard).filter(|(at, _)| t >= *at) {
        clock.heard += 1;
        for name in names {
            if let Some(handle) = art.as_deref().and_then(|a| a.sounds.get(name)) {
                commands.spawn((AudioPlayer(handle.clone()), PlaybackSettings::DESPAWN));
            }
        }
    }
    let (offset, tilt) = match rumble {
        Some(at) => (Vec2::new(curve(&look.rumble_x, at), curve(&look.rumble_y, at)), curve(&look.rumble_tilt, at)),
        None => (Vec2::new(0.0, curve(&look.drop, t)), curve(&look.tilt, t)),
    };
    let turn = Rot2::degrees(tilt);
    let centre = Vec2::new(0.0, look.logo_y) + offset * UNIT;
    for (piece, mut node, mut transform, mut visibility) in &mut pieces {
        let mut size = match &piece.image {
            Some(image) => {
                let Some(image) = images.as_ref().and_then(|all| all.get(image)) else {
                    visibility.set_if_neq(Visibility::Hidden);
                    continue;
                };
                image.size_f32() * piece.scale
            }
            None => Vec2::new(1920.0, 200.0),
        };
        let mut at = piece.at;
        match piece.kind {
            PieceKind::Debris(i) => {
                if let Some(elapsed) = rumble {
                    let (end, drop) = look.rumble_debris;
                    let start = look.rumble_debris_after[i];
                    let k = ((elapsed - start) / (end - start).max(f32::EPSILON)).clamp(0.0, 1.0);
                    // The two drop keys have flat tangents.
                    at.y -= drop * k * k * (3.0 - 2.0 * k);
                } else {
                    at.y += curve(&look.debris_hop, t);
                }
                size *= curve(&look.debris_pop, t);
            }
            PieceKind::Brick => {
                // Hidden until it starts to fall, then squashes and stretches as it lands.
                if t < fall {
                    visibility.set_if_neq(Visibility::Hidden);
                    continue;
                }
                at.y += curve(&look.brick_y, t);
                size *= Vec2::new(curve(&look.brick_scale_x, t), curve(&look.brick_scale_y, t));
            }
            PieceKind::Logo => {}
            PieceKind::Plant(i) => {
                // Grows up from its foot.
                let grow = look.plant_grow.get(i).map_or(1.0, |keys| curve(keys, t));
                size.y *= grow;
            }
        }
        // Placed by the sprite's pivot: its middle sits half a picture from it.
        at.y += (0.5 - piece.pivot_y) * size.y / UNIT;
        transform.rotation = Rot2::degrees(-tilt);
        let spot = centre + turn * at * UNIT;
        let placed = place((size.x, size.y), (spot.x, spot.y));
        node.left = placed.left;
        node.top = placed.top;
        node.width = placed.width;
        node.height = placed.height;
        visibility.set_if_neq(Visibility::Inherited);
    }
    // The prompt shows once the logo is in place and blinks on and off.
    let ready = t >= look.prompt_after;
    if ready {
        let interval = if rumble.is_some() { look.rumble_blink } else if clock.prompt_hidden { look.blink.1 } else { look.blink.0 };
        clock.blink += time.delta_secs();
        if interval > 0.0 && clock.blink >= interval {
            clock.blink = 0.0;
            clock.prompt_hidden = !clock.prompt_hidden;
        }
    }
    for mut v in &mut prompt {
        v.set_if_neq(shown(ready && !clock.prompt_hidden));
    }
}

