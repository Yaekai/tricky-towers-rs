//! Settings → Credits (docs/research/menus.md, "Credits popup"): over `modal_tile`, a
//! first screen with the logo and THANK YOU FOR PLAYING!, then the original's
//! credits list (`credits_content`, staged from the install) and the Weirdbeard
//! logo, scrolling up in 60 s. Down holds it at 10×, Up runs it back at 10×, any
//! other button closes it. Without the staged list it shows the first screen only.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::art::Art;
use crate::gameplay::{Match, Phase};
use crate::menu::MenuScreen;
use crate::select::shown;

pub struct CreditsPlugin;

impl Plugin for CreditsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CreditsMenu>()
            .add_systems(Startup, spawn_credits)
            .add_systems(Update, run_credits);
    }
}

/// Whether the popup is open and how far it has scrolled (seconds of the 60 s run).
#[derive(Resource, Debug, Default)]
pub struct CreditsMenu {
    pub open: bool,
    pub time: f32,
}

/// Seconds before the scroll starts, how long it takes, and the Up/Down speed.
const DELAY: f32 = 0.5;
const SCROLL: f32 = 60.0;
const FAST: f32 = 10.0;
/// Heights in px of a 1080-high screen: the first screen, a title line and the gap
/// after it, a name line, between groups, the end logo's slot.
const FIRST: f32 = 1080.0;
const TITLE: f32 = 78.6;
const TITLE_GAP: f32 = 10.0;
const NAME: f32 = 70.0;
const GROUP_GAP: f32 = 50.0;
const END_LOGO: (f32, f32) = (601.0, 434.0);

#[derive(Component)]
struct CreditsRoot;

/// The scrolling column and its full height in px (list + first screen).
#[derive(Component)]
struct CreditsContent(f32);

/// White display letters with the dark outline, in capitals, `size` px of 1080.
fn line(
    size: f32,
    text: &str,
) -> (
    Text,
    crate::letters::TextLook,
    TextFont,
    TextColor,
    TextLayout,
    crate::letters::LetterSpacing,
) {
    (
        Text::new(text.to_uppercase()),
        crate::letters::TextLook::OUTLINED,
        TextFont {
            font_size: FontSize::Vh(size / 10.8),
            ..default()
        },
        TextColor(Color::WHITE),
        TextLayout::justify(Justify::Center),
        // CreditsName, CreditsTitle and CreditsPopup's texts space -1.
        crate::letters::LetterSpacing(-1.0),
    )
}

fn vh(px: f32) -> Val {
    Val::Vh(px / 10.8)
}

/// The English wording by key, from the staged `text_en` (the original's language XML).
pub(crate) fn english() -> HashMap<String, String> {
    let text = std::fs::read_to_string(crate::art::extracted_dir().join("language/en.txt"))
        .unwrap_or_default();
    let mut words = HashMap::new();
    for block in text.split("<lookup>").skip(1) {
        let between = |open: &str, close: &str| {
            let start = block.find(open)? + open.len();
            let end = block[start..].find(close)? + start;
            Some(block[start..end].to_string())
        };
        if let (Some(key), Some(value)) = (
            between("<key>", "</key>"),
            between("<translation>", "</translation>"),
        ) {
            let value = value
                .replace("&amp;", "&")
                .replace("&lt;", "<")
                .replace("&gt;", ">")
                .replace("&quot;", "\"")
                .replace("&apos;", "'");
            words.insert(key, value);
        }
    }
    words
}

/// The credits groups: titles and names, and whether the names come first.
fn groups() -> Vec<(Vec<String>, Vec<String>, bool)> {
    let text = std::fs::read_to_string(crate::art::extracted_dir().join("language/credits.json"))
        .unwrap_or_default();
    // The JSON reads as RON maps and lists.
    let Ok(ron::Value::Seq(list)) = ron::from_str::<ron::Value>(&text) else {
        return Vec::new();
    };
    let strings = |v: Option<&ron::Value>| -> Vec<String> {
        match v {
            Some(ron::Value::Seq(items)) => items
                .iter()
                .filter_map(|i| match i {
                    ron::Value::String(s) => Some(s.clone()),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        }
    };
    list.iter()
        .filter_map(|group| match group {
            ron::Value::Map(map) => {
                let get = |k: &str| map.get(&ron::Value::String(k.into()));
                let first = matches!(get("nameFirst"), Some(ron::Value::Bool(true)));
                Some((strings(get("titles")), strings(get("names")), first))
            }
            _ => None,
        })
        .collect()
}

fn spawn_credits(mut commands: Commands, art: Option<Res<Art>>) {
    let art = art.as_deref();
    let image = |name: &str| art.and_then(|a| a.image(name));
    let words = english();
    let say = |key: &str| words.get(key).cloned().unwrap_or_else(|| key.to_string());
    let mut root = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
        GlobalZIndex(13),
        Visibility::Hidden,
        CreditsRoot,
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
    let row = |height: f32| Node {
        width: Val::Percent(100.0),
        height: vh(height),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        flex_shrink: 0.0,
        ..default()
    };
    let groups = groups();
    let mut list = 0.0;
    root.with_children(|root| {
        root.spawn(Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .with_children(|col| {
            // The first screen: logo in the middle, the thanks 260 px under the centre.
            col.spawn(Node {
                width: Val::Percent(100.0),
                height: vh(FIRST),
                flex_shrink: 0.0,
                ..default()
            })
            .with_children(|first| {
                if let Some(logo) = image("logo") {
                    first.spawn((
                        ImageNode::new(logo),
                        Node {
                            position_type: PositionType::Absolute,
                            width: vh(818.0),
                            height: vh(420.0),
                            left: Val::Percent(50.0),
                            top: vh(FIRST / 2.0 - 210.0),
                            margin: UiRect::left(vh(-409.0)),
                            ..default()
                        },
                    ));
                }
                first
                    .spawn(Node {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(100.0),
                        top: vh(FIRST / 2.0 + 260.0 - 40.0),
                        height: vh(80.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_child(line(60.0, &say("CREDITS_THANKS_FOR_PLAYING")));
            });
            for (titles, names, names_first) in &groups {
                let mut lines: Vec<(f32, f32, String)> = Vec::new();
                let title_lines: Vec<_> = titles.iter().map(|t| (TITLE, 60.0, say(t))).collect();
                let name_lines: Vec<_> = names.iter().map(|n| (NAME, 46.0, n.clone())).collect();
                let gap = (TITLE_GAP, 0.0, String::new());
                if *names_first {
                    lines.extend(name_lines);
                    lines.push(gap);
                    lines.extend(title_lines);
                } else {
                    lines.extend(title_lines);
                    lines.push(gap);
                    lines.extend(name_lines);
                }
                lines.push((GROUP_GAP, 0.0, String::new()));
                for (height, size, text) in lines {
                    list += height;
                    let mut node = col.spawn(row(height));
                    if !text.is_empty() {
                        node.with_child(line(size, &text));
                    }
                }
            }
            if !groups.is_empty() {
                list += END_LOGO.1 + 100.0 + GROUP_GAP;
                let mut slot = col.spawn(row(END_LOGO.1 + 100.0 + GROUP_GAP));
                if let Some(logo) = image("menu_weirdbeard_logo") {
                    slot.with_child((
                        ImageNode::new(logo),
                        Node {
                            width: vh(END_LOGO.0),
                            height: vh(END_LOGO.1),
                            ..default()
                        },
                    ));
                }
            }
        })
        .insert(CreditsContent(list + FIRST));
    });
}

#[allow(clippy::type_complexity)]
fn run_credits(
    time: Res<Time>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    pads: Query<(Entity, &Gamepad)>,
    game: Res<Match>,
    screen: Res<MenuScreen>,
    mut menu: ResMut<CreditsMenu>,
    mut root: Query<&mut Visibility, With<CreditsRoot>>,
    mut content: Query<(&CreditsContent, &mut Node)>,
) {
    if game.phase != Phase::Menu || *screen != MenuScreen::Settings {
        menu.open = false;
    }
    for mut v in &mut root {
        v.set_if_neq(shown(menu.open));
    }
    if !menu.open {
        menu.time = 0.0;
        return;
    }
    let held = |key: KeyCode, button: GamepadButton| {
        keys.as_ref().is_some_and(|k| k.pressed(key)) || pads.iter().any(|(_, p)| p.pressed(button))
    };
    let speed = if held(KeyCode::ArrowDown, GamepadButton::DPadDown) {
        FAST
    } else if held(KeyCode::ArrowUp, GamepadButton::DPadUp) {
        -FAST
    } else {
        1.0
    };
    // Any other button closes it (Esc is handled with the menu's Esc).
    let taps = crate::select::read_taps(keys.as_deref(), &pads);
    let closing = taps.key_pick || !taps.pad_pick.is_empty() || !taps.pad_back.is_empty();
    let before = menu.time;
    menu.time = (menu.time + time.delta_secs() * speed).max(0.0);
    if closing || (before >= DELAY + SCROLL) {
        menu.open = false;
        return;
    }
    for (content, mut node) in &mut content {
        let run = ((menu.time - DELAY) / SCROLL).clamp(0.0, 1.0);
        node.top = vh(-content.0 * run);
    }
}
