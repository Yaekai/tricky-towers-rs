//! "Press Enter to start": the logo drops in and bounces, a brick lands on it
//! and shakes the rubble, then the prompt blinks under it. Layout and timings
//! from the original (docs/research/menus.md, "Start screen"), in pixels of a
//! 1920×1080 screen; the logo pieces are pictures at 26 px a unit.

use bevy::prelude::*;

use crate::art::Art;
use crate::config::{GameConfig, StartLook};
use crate::gameplay::{Match, Phase};
use crate::menu::MenuScreen;
use crate::select::{place, shown};

/// Pixels a unit of the logo's layout, at 1080 high.
const UNIT: f32 = 26.0;
/// The prompt bar's height and centre below the screen centre.
const PROMPT: (f32, f32) = (63.0, -218.0);

#[derive(Component)]
pub(crate) struct StartRoot;
#[derive(Component)]
pub(crate) struct StartPrompt;

/// The prompt's words, so they can change with the pad.
#[derive(Component)]
pub(crate) struct StartPromptWords;

/// The prompt's words: the key it takes, or the pad's button once a pad is plugged in
/// (the original shows the key or button picture; words stand in until checked).
pub(crate) fn prompt_words(pad: bool, playstation: bool) -> &'static str {
    match (pad, playstation) {
        (false, _) => "Press Space to start",
        (true, false) => "Press A to start",
        (true, true) => "Press Cross to start",
    }
}

/// What a logo piece is, for how it moves.
#[derive(Clone, Copy, PartialEq, Debug)]
enum PieceKind {
    Logo,
    Brick,
    Debris,
    Plant,
}

/// One picture of the logo, placed in units from the logo's centre.
#[derive(Component)]
pub(crate) struct LogoPiece {
    image: Handle<Image>,
    at: Vec2,
    scale: f32,
    kind: PieceKind,
}

/// Seconds the start screen has been up.
#[derive(Resource, Default)]
pub(crate) struct StartClock(pub f32);

pub(crate) fn spawn_start_screen(
    mut commands: Commands,
    art: Option<Res<Art>>,
    config: Res<GameConfig>,
) {
    let image = |name: &str| art.as_ref().and_then(|a| a.image(name));
    let mut root = commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.05, 0.06, 0.12)),
        GlobalZIndex(11),
        Visibility::Hidden,
        StartRoot,
    ));
    if let Some(background) = image("menu_background_bare") {
        root.with_child(crate::select::backdrop(background));
    }
    root.with_children(|parent| crate::menu::spawn_water_scene_into(parent, art.as_deref(), &config));
    // The pieces, back to front: plants behind the logo, then the logo, the
    // rubble and the brick. With only the combined logo staged it moves alone.
    let mut pieces: Vec<(String, Vec2, f32, PieceKind)> = Vec::new();
    let plain = image("logo_plain").is_some();
    if plain {
        for (i, at) in PLANTS.iter().enumerate() {
            pieces.push((format!("logo_plants_{}", i + 1), *at, 1.0, PieceKind::Plant));
        }
        pieces.push(("logo_plain".into(), Vec2::ZERO, 1.0, PieceKind::Logo));
        for (i, at) in DEBRIS.iter().enumerate() {
            pieces.push((
                format!("logo_debris_{}", i + 1),
                *at,
                1.0,
                PieceKind::Debris,
            ));
        }
        for &(at, scale) in SMALL_DEBRIS {
            pieces.push(("logo_debris_8".into(), at, scale, PieceKind::Debris));
        }
        pieces.push(("logo_brick".into(), BRICK, 1.0, PieceKind::Brick));
    } else {
        pieces.push(("logo".into(), Vec2::ZERO, 1.0, PieceKind::Logo));
    }
    let any = pieces.iter().any(|(name, ..)| image(name).is_some());
    root.with_children(|root| {
        for (name, at, scale, kind) in pieces {
            let Some(picture) = image(&name) else {
                continue;
            };
            root.spawn((
                ImageNode::new(picture.clone()),
                Node {
                    position_type: PositionType::Absolute,
                    ..default()
                },
                UiTransform::default(),
                Visibility::Hidden,
                LogoPiece {
                    image: picture,
                    at,
                    scale,
                    kind,
                },
            ));
        }
        if !any {
            root.spawn(place((1920.0, 200.0), (0.0, 182.0)))
                .with_child((
                    Text::new("Tricky Towers"),
                    TextFont {
                        font_size: FontSize::Vh(12.0),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
        }
        // The prompt's bar across the screen; its own box, so blinking hides the
        // letter-drawn copy too.
        let mut bar = root.spawn((
            place((1920.0, PROMPT.0), (0.0, PROMPT.1)),
            Visibility::Hidden,
            StartPrompt,
        ));
        if let Some(back) = image("start_message_background") {
            bar.insert(ImageNode::new(back));
        }
        bar.with_child((
            Text::new(prompt_words(false, false)),
            StartPromptWords,
            TextFont {
                font_size: FontSize::Vh(4.5),
                ..default()
            },
            TextColor(Color::WHITE),
            TextLayout::justify(Justify::Center),
            crate::letters::TextLook::OUTLINED,
        ));
    });
}

/// Brick, plants, rubble and the small rubble copies (with their sizes), in
/// units from the logo's centre, y up.
const BRICK: Vec2 = Vec2::new(-1.78, 10.11);
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

/// A value from (time, value) keys, straight between them and held past the ends.
pub fn keyed(keys: &[(f32, f32)], t: f32) -> f32 {
    let Some(&(first_t, first_v)) = keys.first() else {
        return 0.0;
    };
    if t <= first_t {
        return first_v;
    }
    for pair in keys.windows(2) {
        let ((t0, v0), (t1, v1)) = (pair[0], pair[1]);
        if t <= t1 {
            let s = if t1 > t0 { (t - t0) / (t1 - t0) } else { 1.0 };
            return v0 + (v1 - v0) * s;
        }
    }
    keys.last().map_or(0.0, |k| k.1)
}

/// Enter, Space, A or Start goes on to the title tiles.
pub(crate) fn start_input(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    gamepads: Query<&Gamepad>,
    game: Res<Match>,
    mut screen: ResMut<MenuScreen>,
) {
    if game.phase != Phase::Menu || *screen != MenuScreen::Start {
        return;
    }
    let key = keys.is_some_and(|k| {
        k.any_just_pressed([KeyCode::Enter, KeyCode::Space, KeyCode::NumpadEnter])
    });
    let pad = gamepads
        .iter()
        .any(|p| p.any_just_pressed([GamepadButton::South, GamepadButton::Start]));
    if key || pad {
        *screen = MenuScreen::Main;
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
    settings: Option<Res<crate::settings::Settings>>,
    pads: Query<&Gamepad>,
    mut prompt_text: Query<&mut Text, With<StartPromptWords>>,
    mut heard: Local<[bool; 3]>,
    mut root: Query<&mut Visibility, With<StartRoot>>,
    mut prompt: Query<&mut Visibility, (With<StartPrompt>, Without<StartRoot>)>,
    mut pieces: Query<
        (&LogoPiece, &mut Node, &mut UiTransform, &mut Visibility),
        (Without<StartRoot>, Without<StartPrompt>),
    >,
) {
    let on = game.phase == Phase::Menu && *screen == MenuScreen::Start;
    let playstation = pads.iter().next().is_some_and(crate::input::is_playstation);
    let words = prompt_words(!pads.is_empty(), playstation);
    for mut text in &mut prompt_text {
        if text.0 != words {
            text.0 = words.to_string();
        }
    }
    for mut v in &mut root {
        v.set_if_neq(shown(on));
    }
    if !on {
        clock.0 = 0.0;
        *heard = [false; 3];
        return;
    }
    clock.0 += time.delta_secs();
    let t = clock.0;
    let look: &StartLook = &config.start;
    // The original's sounds: the brick starts to fall, then lands with a nudge and a crumble.
    let (_, fall, land) = look.brick_fall;
    let cues = [
        (fall, "sfx_brick_fall_1", 0),
        (land, "sfx_brick_nudge", 1),
        (land, "sfx_menu_crumble", 2),
    ];
    for (at, name, slot) in cues {
        if !heard[slot] && t >= at {
            heard[slot] = true;
            if let Some(handle) = art.as_deref().and_then(|a| a.sounds.get(name)) {
                // The sound-effects volume from Settings, as every effect plays at.
                let volume = settings.as_deref().map_or(1.0, |s| s.volume);
                commands.spawn((
                    AudioPlayer(handle.clone()),
                    PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(volume)),
                ));
            }
        }
    }
    let drop = keyed(&look.drop, t) + keyed(&look.dip, t);
    let centre = Vec2::new(0.0, look.logo_y + drop * UNIT);
    for (piece, mut node, mut transform, mut visibility) in &mut pieces {
        let Some(size) = images
            .as_ref()
            .and_then(|all| all.get(&piece.image))
            .map(|i| i.size_f32() * piece.scale)
        else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };
        let mut at = piece.at;
        match piece.kind {
            PieceKind::Debris => at.y += keyed(&look.debris_hop, t),
            PieceKind::Brick => {
                // Falls in from above, speeding up, and lands at its spot.
                let (height, from, land) = look.brick_fall;
                let s = ((t - from) / (land - from).max(0.01)).clamp(0.0, 1.0);
                at.y += height * (1.0 - s * s);
            }
            PieceKind::Logo => {
                let tilt = keyed(&look.tilt, t);
                transform.rotation = Rot2::degrees(-tilt);
            }
            PieceKind::Plant => {}
        }
        let spot = centre + at * UNIT;
        let placed = place((size.x, size.y), (spot.x, spot.y));
        node.left = placed.left;
        node.top = placed.top;
        node.width = placed.width;
        node.height = placed.height;
        visibility.set_if_neq(Visibility::Inherited);
    }
    // The prompt shows once the logo is in place and blinks on and off.
    let (on_for, off_for) = look.blink;
    let cycle = (on_for + off_for).max(0.01);
    let lit = t >= look.prompt_after && (t - look.prompt_after) % cycle < on_for;
    for mut v in &mut prompt {
        v.set_if_neq(shown(lit));
    }
}

#[cfg(test)]
mod tests {
    use super::{keyed, prompt_words};

    #[test]
    fn the_prompt_names_the_key_or_the_plugged_pad_button() {
        assert_eq!(prompt_words(false, false), "Press Space to start");
        assert_eq!(prompt_words(true, false), "Press A to start");
        assert_eq!(prompt_words(true, true), "Press Cross to start");
    }

    #[test]
    fn keys_blend_straight_and_hold_at_the_ends() {
        let keys = [(0.0, 10.0), (1.0, 0.0), (2.0, 4.0)];
        assert_eq!(keyed(&keys, -1.0), 10.0);
        assert_eq!(keyed(&keys, 0.5), 5.0);
        assert_eq!(keyed(&keys, 1.5), 2.0);
        assert_eq!(keyed(&keys, 9.0), 4.0);
    }
}
