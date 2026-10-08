//! The explanation popup before a match (docs/research/menus.md, "Explanation
//! (tutorial) popups"): over `modal_tile`, the mode's title and rule, its picture,
//! the difficulty with what it takes away, CONTINUE and "Disable all explanations".
//! Battles show it for every mode; single player only before trials SR_10, SS_17
//! and SP_20. Only while Explanations is on, and only in the real game (tests,
//! scripts and offscreen pictures keep no settings file).
//! The first battle and the first single-player match of a session open with the
//! keyboard controls panel first (ControlsExplanation, sliding up in 0.3 s).

use bevy::prelude::*;

use crate::art::Art;
use crate::config::{Difficulty, GameConfig, ModeKind};
use crate::gameplay::{Match, Phase};
use crate::menu::MenuScreen;
use crate::select::{place, shown};

pub struct ExplainPlugin;

impl Plugin for ExplainPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Explain>()
            .add_systems(Startup, spawn_explain)
            .add_systems(Update, (open_explain, run_explain).chain());
    }
}

/// The popup's state: seconds since it opened, since it started closing, and the
/// screen to put back when the match goes on.
#[derive(Resource, Default)]
pub struct Explain {
    open: bool,
    time: f32,
    closing: Option<f32>,
    back_to: MenuScreen,
    /// Showing the controls step, and whether the mode's explanation follows it.
    controls: bool,
    mode_next: bool,
}

#[derive(Component)]
struct ControlsRoot;

/// The controls panel, sliding up from 1080 px below.
#[derive(Component)]
struct ControlsPanel;

/// Each part slides this far in over this long, after its delay (OutQuad).
const SLIDE: f32 = 1500.0;
const SLIDE_TIME: f32 = 0.5;
const TRIALS: [&str; 3] = ["SR_10", "SS_17", "SP_20"];
/// The controls panel slides up this fast (OutQuad).
const CONTROLS_IN: f32 = 0.3;

#[derive(Component)]
struct ExplainRoot;

/// A sliding part: its delay and where it rests (centre, px from the screen centre).
#[derive(Component)]
struct Slide {
    delay: f32,
    size: (f32, f32),
    at: (f32, f32),
}

/// What a part shows, filled in when the popup opens.
#[derive(Component, Clone, Copy, PartialEq)]
enum Part {
    Title,
    Subtitle,
    ModePicture,
    DifficultyPicture,
    DifficultyTitle,
    DifficultySubtitle,
    /// A spell slot's frame, and the icon inside it.
    Slot(usize),
    SlotIcon(usize),
    Buttons,
}

fn text(
    size: f32,
) -> (
    Text,
    crate::letters::TextLook,
    TextFont,
    TextColor,
    TextLayout,
) {
    (
        Text::new(""),
        crate::letters::TextLook::OUTLINED,
        TextFont {
            font_size: FontSize::Vh(size / 10.8),
            ..default()
        },
        TextColor(Color::WHITE),
        TextLayout::justify(Justify::Center),
    )
}

/// A sliding part: words of `words` px, or a picture when 0.
fn part(
    root: &mut ChildSpawnerCommands,
    part: Part,
    delay: f32,
    size: (f32, f32),
    at: (f32, f32),
    words: f32,
) -> Entity {
    let mut node = root.spawn((place(size, at), Slide { delay, size, at }, part));
    if words > 0.0 {
        node.with_child((text(words), part));
    } else {
        node.insert(ImageNode::default());
    }
    node.id()
}

fn spawn_explain(
    mut commands: Commands,
    art: Option<Res<Art>>,
    keys: Option<Res<crate::keybinding::KeyBindings>>,
) {
    spawn_controls(&mut commands, art.as_deref(), keys.as_deref());
    let art = art.as_deref();
    let image = |name: &str| art.and_then(|a| a.image(name));
    let mut root = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
        GlobalZIndex(13),
        Visibility::Hidden,
        ExplainRoot,
    ));
    if let Some(tile) = image("modal_tile") {
        root.insert((
            ImageNode::new(tile).with_mode(NodeImageMode::Tiled {
                tile_x: true,
                tile_y: true,
                stretch_value: 1.0,
            }),
            BackgroundColor(Color::NONE),
        ));
    }
    let slot_light = image("spell_slot_light");
    let slot_dark = image("spell_slot_dark");
    root.with_children(|root| {
        part(root, Part::Title, 0.1, (1600.0, 100.0), (0.0, 410.0), 75.0);
        part(
            root,
            Part::Subtitle,
            0.2,
            (1600.0, 50.0),
            (0.0, 333.5),
            26.0,
        );
        part(
            root,
            Part::ModePicture,
            0.3,
            (487.0, 414.0),
            (0.0, 61.0),
            0.0,
        );
        part(
            root,
            Part::DifficultyPicture,
            0.4,
            (209.0, 157.0),
            (0.0, -235.0),
            0.0,
        );
        part(
            root,
            Part::DifficultyTitle,
            0.4,
            (1200.0, 70.0),
            (0.0, -318.0),
            46.0,
        );
        part(
            root,
            Part::DifficultySubtitle,
            0.4,
            (1200.0, 70.0),
            (0.0, -380.0),
            26.0,
        );
        for (i, back) in [slot_light, slot_dark].into_iter().enumerate() {
            let x = if i == 0 { -35.0 } else { 35.0 };
            let id = part(root, Part::Slot(i), 0.5, (55.0, 55.0), (x, -450.0), 0.0);
            // The slot's frame, with what the difficulty takes away inside (50x50).
            root.commands().entity(id).remove::<ImageNode>();
            if let Some(back) = back {
                root.commands().entity(id).insert(ImageNode::new(back));
            }
            root.commands().entity(id).with_child((
                ImageNode::default(),
                Node {
                    width: Val::Percent(50.0 / 55.0 * 100.0),
                    height: Val::Percent(50.0 / 55.0 * 100.0),
                    ..default()
                },
                Part::SlotIcon(i),
            ));
        }
        // CONTINUE bottom-right and "Disable all explanations" bottom-left, 30 px in
        // and 50 px up, shown once everything is in.
        let half = (crate::select::SCREEN.0 / 2.0, crate::select::SCREEN.1 / 2.0);
        for (left, words, size) in [
            (false, "Continue  Space", (400.0, 50.0)),
            (true, "C  Disable all explanations", (850.0, 70.0)),
        ] {
            let x = if left {
                -half.0 + 30.0 + size.0 / 2.0
            } else {
                half.0 - 30.0 - size.0 / 2.0
            };
            let mut node = place(size, (x, -half.1 + 50.0 + size.1 / 2.0));
            node.justify_content = if left {
                JustifyContent::FlexStart
            } else {
                JustifyContent::FlexEnd
            };
            root.spawn((node, Part::Buttons)).with_children(|b| {
                b.spawn(text(46.0)).insert(Text::new(words.to_uppercase()));
            });
        }
    });
}

fn spawn_controls(
    commands: &mut Commands,
    art: Option<&Art>,
    keys: Option<&crate::keybinding::KeyBindings>,
) {
    let defaults = crate::keybinding::KeyBindings::default();
    let mut root = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
        GlobalZIndex(13),
        Visibility::Hidden,
        ControlsRoot,
    ));
    if let Some(tile) = art.and_then(|a| a.image("modal_tile")) {
        root.insert((
            ImageNode::new(tile).with_mode(NodeImageMode::Tiled {
                tile_x: true,
                tile_y: true,
                stretch_value: 1.0,
            }),
            BackgroundColor(Color::NONE),
        ));
    }
    root.with_children(|root| {
        let half = (crate::select::SCREEN.0 / 2.0, crate::select::SCREEN.1 / 2.0);
        let panel =
            crate::menu::spawn_controls_panel(root, art, keys.unwrap_or(&defaults), Some(half));
        root.commands()
            .entity(panel)
            .insert((ControlsPanel, UiTransform::default()));
        for (left, words, size) in [
            (false, "Continue  Space", (400.0, 50.0)),
            (true, "C  Disable all explanations", (935.0, 72.0)),
        ] {
            let x = if left {
                -half.0 + 30.0 + size.0 / 2.0
            } else {
                half.0 - 30.0 - size.0 / 2.0
            };
            let mut node = place(size, (x, -half.1 + 50.0 + size.1 / 2.0));
            node.justify_content = if left {
                JustifyContent::FlexStart
            } else {
                JustifyContent::FlexEnd
            };
            root.spawn(node).with_children(|b| {
                b.spawn(text(46.0)).insert(Text::new(words.to_uppercase()));
            });
        }
    });
}

/// The words and pictures for this match, or None when it shows no explanation.
struct Content {
    title: String,
    subtitle: String,
    picture: &'static str,
    difficulty: Option<Difficulty>,
}

fn content(config: &GameConfig, mode: ModeKind) -> Option<Content> {
    let trial = config.trial.and_then(|i| config.trials.get(i));
    if config.trial.is_some() && !trial.is_some_and(|t| TRIALS.contains(&t.name.as_str())) {
        return None;
    }
    let mode = trial.map_or(mode, |t| t.mode);
    let key = match mode {
        ModeKind::Race => "RACE",
        ModeKind::Survival => "SURVIVAL",
        ModeKind::Puzzle => "PUZZLE",
        // Endless shows none.
        _ => return None,
    };
    let who = if trial.is_some() {
        "SINGLE_PLAYER"
    } else {
        "MULTIPLAYER"
    };
    let picture = match (mode, trial.is_some()) {
        (ModeKind::Race, true) => "menu_explanation_mode_race_sp",
        (ModeKind::Race, false) => "menu_mode_race",
        (ModeKind::Survival, _) => "menu_mode_survivalsp",
        _ => "menu_mode_puzzle",
    };
    Some(Content {
        title: format!("EXPLANATION_TITLE_{who}_{key}"),
        subtitle: format!("EXPLANATION_{who}_{key}"),
        picture,
        // Single-player explanations hide the difficulty block.
        difficulty: trial
            .is_none()
            .then(|| config.difficulty.unwrap_or(Difficulty::Normal)),
    })
}

/// What a difficulty takes away, light slot then dark slot ("magic off" is
/// `menu_unavailble`).
fn slots(mode: ModeKind, difficulty: Difficulty) -> [Option<&'static str>; 2] {
    let off = Some("menu_unavailble");
    match (mode, difficulty) {
        (ModeKind::Puzzle, Difficulty::Easy) => [Some("spell_icon_Undo"), off],
        (ModeKind::Puzzle, Difficulty::Normal) => [Some("spell_icon_Ivy"), off],
        (ModeKind::Puzzle, Difficulty::Pro) => [off, off],
        (_, Difficulty::Easy) | (ModeKind::Survival, Difficulty::Pro) => [None, off],
        _ => [None, None],
    }
}

/// Opens the popup when a match starts from the menus.
#[allow(clippy::too_many_arguments)]
fn open_explain(
    mut game: ResMut<Match>,
    config: Res<GameConfig>,
    settings: Option<Res<crate::settings::Settings>>,
    art: Option<Res<Art>>,
    mut screen: ResMut<MenuScreen>,
    mut explain: ResMut<Explain>,
    mut last_round: Local<u64>,
    mut controls_seen: Local<[bool; 2]>,
    mut parts: Query<(
        &Part,
        Option<&mut Text>,
        Option<&mut ImageNode>,
        &mut Visibility,
    )>,
) {
    if game.round == *last_round {
        return;
    }
    *last_round = game.round;
    let on = settings.is_some_and(|s| s.explanations && s.file.is_some());
    if !on || game.phase != Phase::Playing {
        return;
    }
    // The controls come first once per session, for battles and for single player.
    let single = config.trial.is_some()
        || matches!(
            game.mode,
            ModeKind::Endless | ModeKind::EndlessSurvival | ModeKind::EndlessPuzzle
        );
    let controls_due = !controls_seen[single as usize];
    controls_seen[single as usize] = true;
    let content = content(&config, game.mode);
    if !controls_due && content.is_none() {
        return;
    }
    explain.controls = controls_due;
    explain.mode_next = content.is_some();
    let content = content.unwrap_or(Content {
        title: String::new(),
        subtitle: String::new(),
        picture: "",
        difficulty: None,
    });
    let words = crate::credits::english();
    let say = |key: &str| words.get(key).cloned().unwrap_or_default().to_uppercase();
    let image = |name: &str| art.as_deref().and_then(|a| a.image(name));
    let mode = config
        .trial
        .and_then(|i| config.trials.get(i))
        .map_or(game.mode, |t| t.mode);
    let diff_key = |d: Difficulty| match d {
        Difficulty::Easy => "EASY",
        Difficulty::Normal => "NORMAL",
        Difficulty::Pro => "PRO",
    };
    for (part, text, image_node, mut visibility) in &mut parts {
        let difficulty_part = matches!(
            part,
            Part::DifficultyPicture
                | Part::DifficultyTitle
                | Part::DifficultySubtitle
                | Part::Slot(_)
                | Part::SlotIcon(_)
        );
        if difficulty_part && content.difficulty.is_none() {
            *visibility = Visibility::Hidden;
            continue;
        }
        *visibility = Visibility::Inherited;
        let d = content.difficulty.unwrap_or(Difficulty::Normal);
        if let Some(mut text) = text {
            text.0 = match part {
                Part::Title => say(&content.title),
                Part::Subtitle => say(&content.subtitle),
                Part::DifficultyTitle => say(&format!("DIFFICULTY_{}", diff_key(d))),
                Part::DifficultySubtitle if d == Difficulty::Pro => {
                    let key = content.title.trim_start_matches("EXPLANATION_TITLE_");
                    say(&format!("EXPLANATION_{key}_PRO_SUBTITLE"))
                }
                Part::Buttons => continue,
                _ => String::new(),
            };
        }
        if let Some(mut node) = image_node {
            let name = match part {
                Part::ModePicture => Some(content.picture.to_string()),
                Part::DifficultyPicture => {
                    Some(format!("menu_difficulty_{}", diff_key(d).to_lowercase()))
                }
                Part::SlotIcon(i) => slots(mode, d)[*i].map(String::from),
                _ => continue,
            };
            match name.and_then(|n| image(&n)) {
                Some(handle) => {
                    node.image = handle;
                    node.color = Color::WHITE;
                }
                None => node.color = Color::NONE,
            }
        }
    }
    explain.open = true;
    explain.time = 0.0;
    explain.closing = None;
    explain.back_to = *screen;
    game.phase = Phase::Menu;
    *screen = MenuScreen::Explain;
}

/// Slides the parts in, waits for Continue (or K / pad Y, which also turns
/// explanations off), slides them out and lets the match go on.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn run_explain(
    time: Res<Time>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    pads: Query<(Entity, &Gamepad)>,
    mut game: ResMut<Match>,
    mut screen: ResMut<MenuScreen>,
    mut explain: ResMut<Explain>,
    mut settings: Option<ResMut<crate::settings::Settings>>,
    mut root: Query<&mut Visibility, (With<ExplainRoot>, Without<ControlsRoot>)>,
    mut controls_root: Query<&mut Visibility, (With<ControlsRoot>, Without<ExplainRoot>)>,
    mut controls_panel: Query<&mut UiTransform, With<ControlsPanel>>,
    mut slides: Query<(&Slide, &mut Node), Without<ExplainRoot>>,
    mut buttons: Query<
        (&Part, &mut Visibility),
        (Without<Slide>, Without<ExplainRoot>, Without<ControlsRoot>),
    >,
) {
    let on = explain.open && *screen == MenuScreen::Explain && game.phase == Phase::Menu;
    for mut v in &mut root {
        v.set_if_neq(shown(on && !explain.controls));
    }
    for mut v in &mut controls_root {
        v.set_if_neq(shown(on && explain.controls));
    }
    if !on {
        explain.open = false;
        return;
    }
    let dt = time.delta_secs();
    explain.time += dt;
    let taps = crate::select::read_taps(keys.as_deref(), &pads);
    let disable = keys
        .as_ref()
        .is_some_and(|k| k.any_just_pressed([KeyCode::KeyK, KeyCode::KeyC]))
        || pads
            .iter()
            .any(|(_, p)| p.just_pressed(GamepadButton::North));
    if explain.controls {
        let t = (explain.time / CONTROLS_IN).min(1.0);
        let below = 1080.0 * (1.0 - t) * (1.0 - t);
        for mut transform in &mut controls_panel {
            transform.translation = Val2::new(Val::Px(0.0), Val::Vh(below / 10.8));
        }
        let go = disable || taps.key_pick || !taps.pad_pick.is_empty();
        if explain.time >= CONTROLS_IN && go {
            if disable && let Some(settings) = settings.as_mut() {
                settings.explanations = false;
                settings.save();
            }
            explain.controls = false;
            explain.time = 0.0;
            if disable || !explain.mode_next {
                explain.open = false;
                *screen = explain.back_to;
                game.phase = Phase::Playing;
            }
        }
        return;
    }
    let last_in = 0.5 + SLIDE_TIME;
    let all_in = explain.time >= last_in;
    let out_quad = |t: f32| 1.0 - (1.0 - t) * (1.0 - t);
    if let Some(closing) = explain.closing.as_mut() {
        *closing += dt;
    } else if all_in {
        if disable && let Some(settings) = settings.as_mut() {
            settings.explanations = false;
            settings.save();
        }
        if disable || taps.key_pick || !taps.pad_pick.is_empty() {
            explain.closing = Some(0.0);
        }
    }
    let closing = explain.closing;
    for (slide, mut node) in &mut slides {
        let offset = match closing {
            // Out: the same delays, sliding up.
            Some(c) => SLIDE * out_quad(((c - slide.delay) / SLIDE_TIME).clamp(0.0, 1.0)),
            None => {
                -SLIDE
                    * (1.0 - out_quad(((explain.time - slide.delay) / SLIDE_TIME).clamp(0.0, 1.0)))
            }
        };
        let at = (slide.at.0, slide.at.1 + offset);
        let placed = place(slide.size, at);
        if node.top != placed.top {
            node.top = placed.top;
        }
    }
    let show_buttons = all_in && closing.is_none();
    for (part, mut v) in &mut buttons {
        if *part == Part::Buttons {
            v.set_if_neq(shown(show_buttons));
        }
    }
    if closing.is_some_and(|c| c >= last_in) {
        explain.open = false;
        *screen = explain.back_to;
        game.phase = Phase::Playing;
    }
}
