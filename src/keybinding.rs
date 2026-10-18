//! Player 1's keyboard keys, rebindable from Settings → Keybinding like the original
//! (docs/research/gap-scan.md, section 7): a list of actions, each with its key in a
//! box on the right, the small corners on the picked row, and a row that resets them.
//! Enter on an action waits for the next key; Esc cancels the wait or closes the list.
//! Kept between runs in `keys.ron`.

use std::path::PathBuf;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::art::Art;
use crate::select::{CORNER_SMALL, SCREEN, corners, label_with, place, shown, sliced};

/// What a key does in a match.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    MoveLeft,
    MoveRight,
    Drop,
    TurnRight,
    TurnLeft,
    NudgeLeft,
    NudgeRight,
    LightMagic,
    DarkMagic,
    GameInfo,
}

impl Action {
    pub const ALL: [Action; 10] = [
        Action::MoveLeft,
        Action::MoveRight,
        Action::Drop,
        Action::TurnLeft,
        Action::TurnRight,
        Action::NudgeLeft,
        Action::NudgeRight,
        Action::LightMagic,
        Action::DarkMagic,
        Action::GameInfo,
    ];

    /// The row's words, as Max's clip of the original's list shows them.
    pub fn label(self) -> &'static str {
        match self {
            Action::MoveLeft => "Move left",
            Action::MoveRight => "Move right",
            Action::Drop => "Drop down",
            Action::TurnRight => "Rotate right",
            Action::TurnLeft => "Rotate left",
            Action::NudgeLeft => "Nudge left",
            Action::NudgeRight => "Nudge right",
            Action::LightMagic => "Light magic",
            Action::DarkMagic => "Dark magic",
            Action::GameInfo => "Online player info",
        }
    }

    /// The original's default key (docs/research/wiki-notes.md, checked on the PC).
    pub fn default_key(self) -> KeyCode {
        match self {
            Action::MoveLeft => KeyCode::ArrowLeft,
            Action::MoveRight => KeyCode::ArrowRight,
            Action::Drop => KeyCode::ArrowDown,
            Action::TurnRight => KeyCode::ArrowUp,
            Action::TurnLeft => KeyCode::Space,
            Action::NudgeLeft => KeyCode::KeyU,
            Action::NudgeRight => KeyCode::KeyI,
            Action::LightMagic => KeyCode::KeyJ,
            Action::DarkMagic => KeyCode::KeyK,
            Action::GameInfo => KeyCode::Tab,
        }
    }
}

/// Keys that can be bound, with the name shown in the box and saved in `keys.ron`.
const KEY_NAMES: &[(KeyCode, &str)] = &[
    (KeyCode::ArrowLeft, "Left"),
    (KeyCode::ArrowRight, "Right"),
    (KeyCode::ArrowUp, "Up"),
    (KeyCode::ArrowDown, "Down"),
    (KeyCode::Space, "Space"),
    (KeyCode::Tab, "Tab"),
    (KeyCode::Enter, "Enter"),
    (KeyCode::Backspace, "Backspace"),
    (KeyCode::ShiftLeft, "Left Shift"),
    (KeyCode::ShiftRight, "Right Shift"),
    (KeyCode::ControlLeft, "Left Ctrl"),
    (KeyCode::ControlRight, "Right Ctrl"),
    (KeyCode::AltLeft, "Left Alt"),
    (KeyCode::AltRight, "Right Alt"),
    (KeyCode::KeyA, "A"),
    (KeyCode::KeyB, "B"),
    (KeyCode::KeyC, "C"),
    (KeyCode::KeyD, "D"),
    (KeyCode::KeyE, "E"),
    (KeyCode::KeyF, "F"),
    (KeyCode::KeyG, "G"),
    (KeyCode::KeyH, "H"),
    (KeyCode::KeyI, "I"),
    (KeyCode::KeyJ, "J"),
    (KeyCode::KeyK, "K"),
    (KeyCode::KeyL, "L"),
    (KeyCode::KeyM, "M"),
    (KeyCode::KeyN, "N"),
    (KeyCode::KeyO, "O"),
    (KeyCode::KeyP, "P"),
    (KeyCode::KeyQ, "Q"),
    (KeyCode::KeyR, "R"),
    (KeyCode::KeyS, "S"),
    (KeyCode::KeyT, "T"),
    (KeyCode::KeyU, "U"),
    (KeyCode::KeyV, "V"),
    (KeyCode::KeyW, "W"),
    (KeyCode::KeyX, "X"),
    (KeyCode::KeyY, "Y"),
    (KeyCode::KeyZ, "Z"),
    (KeyCode::Digit0, "0"),
    (KeyCode::Digit1, "1"),
    (KeyCode::Digit2, "2"),
    (KeyCode::Digit3, "3"),
    (KeyCode::Digit4, "4"),
    (KeyCode::Digit5, "5"),
    (KeyCode::Digit6, "6"),
    (KeyCode::Digit7, "7"),
    (KeyCode::Digit8, "8"),
    (KeyCode::Digit9, "9"),
    (KeyCode::Numpad0, "Num 0"),
    (KeyCode::Numpad1, "Num 1"),
    (KeyCode::Numpad2, "Num 2"),
    (KeyCode::Numpad3, "Num 3"),
    (KeyCode::Numpad4, "Num 4"),
    (KeyCode::Numpad5, "Num 5"),
    (KeyCode::Numpad6, "Num 6"),
    (KeyCode::Numpad7, "Num 7"),
    (KeyCode::Numpad8, "Num 8"),
    (KeyCode::Numpad9, "Num 9"),
    (KeyCode::Comma, ","),
    (KeyCode::Period, "."),
    (KeyCode::Slash, "/"),
    (KeyCode::Semicolon, ";"),
    (KeyCode::Quote, "'"),
    (KeyCode::BracketLeft, "["),
    (KeyCode::BracketRight, "]"),
    (KeyCode::Minus, "-"),
    (KeyCode::Equal, "="),
];

/// A key's name as the original shows it (Unity's KeyCode names for the arrows).
pub fn cap_name(name: &str) -> &str {
    match name {
        "Left" => "LeftArrow",
        "Right" => "RightArrow",
        "Up" => "UpArrow",
        "Down" => "DownArrow",
        name => name,
    }
}

pub fn key_name(key: KeyCode) -> &'static str {
    KEY_NAMES
        .iter()
        .find(|(k, _)| *k == key)
        .map_or("?", |(_, name)| name)
}

fn key_from_name(name: &str) -> Option<KeyCode> {
    KEY_NAMES.iter().find(|(_, n)| *n == name).map(|(k, _)| *k)
}

/// Player 1's keys. Kept in `file` when one is set (the game sets it; tests and
/// offscreen pictures don't).
#[derive(Resource, Debug, Clone, PartialEq)]
pub struct KeyBindings {
    keys: Vec<(Action, KeyCode)>,
    pub file: Option<PathBuf>,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            keys: Action::ALL.iter().map(|&a| (a, a.default_key())).collect(),
            file: None,
        }
    }
}

impl KeyBindings {
    pub fn key(&self, action: Action) -> KeyCode {
        self.keys
            .iter()
            .find(|(a, _)| *a == action)
            .map_or(action.default_key(), |(_, k)| *k)
    }

    /// Binds `key` to `action`. An action already on that key takes `action`'s old
    /// key, so no key does two things (?: the original's rule for clashes not read).
    pub fn set(&mut self, action: Action, key: KeyCode) {
        let old = self.key(action);
        for (a, k) in &mut self.keys {
            if *a == action {
                *k = key;
            } else if *k == key {
                *k = old;
            }
        }
    }

    pub fn reset(&mut self) {
        self.keys = Self::default().keys;
    }

    pub fn load(file: PathBuf) -> Self {
        let mut bindings = Self::default();
        let saved: Vec<(Action, String)> = std::fs::read_to_string(&file)
            .ok()
            .and_then(|text| ron::from_str(&text).ok())
            .unwrap_or_default();
        for (action, name) in saved {
            if let Some(key) = key_from_name(&name) {
                bindings.set(action, key);
            }
        }
        bindings.file = Some(file);
        bindings
    }

    pub fn save(&self) {
        let Some(file) = &self.file else {
            return;
        };
        let named: Vec<(Action, &str)> =
            self.keys.iter().map(|(a, k)| (*a, key_name(*k))).collect();
        let written = file
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|_| {
                let text = ron::to_string(&named).map_err(std::io::Error::other)?;
                std::fs::write(file, text)
            });
        if let Err(err) = written {
            warn!("could not save keys to {}: {err}", file.display());
        }
    }
}

/// The Keybinding list's state: open, the picked row (the actions, then Reset) and
/// whether it waits for a key.
#[derive(Resource, Debug, Default)]
pub struct KeybindingMenu {
    pub open: bool,
    pub selected: usize,
    pub waiting: bool,
}

/// Rows: one per action, then Reset.
const ROWS: usize = Action::ALL.len() + 1;
/// Layout from KeyBindingPopup, ActionItem and ResetItem (px of a 1920×1080 screen, y up):
/// the 1000 x 800 Content 50 px over the middle, items 85 px apart from its top
/// (KeybindingPopup's ListSelectComponent), each 65 tall across it; the action's name in
/// EnzoBlack 32 from 50 px in, its key in EnzoBlack 24.55 in a 300 px box 30 px from the
/// right, the box being edited the checkbox picture (300 x 45) with the key in
/// EnzoBlackWhite32 28 blinking every 0.25 s; Reset's 600 x 58 background 29 px under its
/// top, its words EnzoBlack 30; "Action" and "Key" headings (EnzoBlackWhite32 36) 50 px
/// over the content; the choosing corners 10 px out.
const CONTENT: (f32, f32) = (1000.0, 800.0);
const CONTENT_Y: f32 = 50.0;
const ROW: (f32, f32) = (1000.0, 65.0);
const ROW_STEP: f32 = 85.0;
const RESET: (f32, f32) = (600.0, 58.0);
const NAME_X: f32 = 50.0;
const NAME_SIZE: f32 = 32.0;
const KEY_BOX: (f32, f32) = (300.0, 45.0);
const KEY_RIGHT: f32 = 30.0;
const KEY_SIZE: f32 = 24.55;
const KEY_EDIT_SIZE: f32 = 28.0;
const HEAD_SIZE: f32 = 36.0;
/// The corners sit this far out from the picked row (measured on Max's clip).
const CORNER_GAP: f32 = 4.0;
const BLINK: f32 = 0.25;
/// EnzoBlack: black display letters, no outline.
const ENZO_BLACK: crate::letters::TextLook = crate::letters::TextLook {
    body: false,
    face: Some(Color::BLACK),
    outline: None,
    low: None,
    outline_edge: 0.0,
};
/// EnzoBlackWhite32: white face, dark outline (dilate and outline 0.3).
const ENZO_WHITE: crate::letters::TextLook = crate::letters::TextLook {
    body: false,
    face: Some(Color::srgb(0.918, 0.933, 1.0)),
    outline: Some(Color::srgb(0.169, 0.035, 0.137)),
    low: None,
    outline_edge: 0.5 - 0.6 * 0.8333 / 2.0,
};
/// `menu_button_background`'s 9-slice borders, as on the Settings rows.
const BUTTON_SLICE: [f32; 4] = [18.0, 71.0, 15.0, 16.0];

/// The middle of row `i` (Reset's background for the last).
fn row_y(i: usize) -> f32 {
    let top = CONTENT_Y + CONTENT.1 / 2.0 - ROW_STEP * i as f32;
    if i < Action::ALL.len() { top - ROW.1 / 2.0 } else { top - RESET.1 / 2.0 }
}

/// The list's row under a point (screen centre origin, y up): an action or Reset (the last).
pub(crate) fn row_at(point: Vec2) -> Option<usize> {
    (0..ROWS).find(|&i| {
        let size = if i < Action::ALL.len() { ROW } else { RESET };
        (point.x).abs() <= size.0 / 2.0 && (point.y - row_y(i)).abs() <= size.1 / 2.0
    })
}

fn vh(px: f32) -> f32 {
    px / SCREEN.1 * 100.0
}

#[derive(Component)]
pub(crate) struct KeybindingRoot;

#[derive(Component, Clone, Copy)]
pub(crate) enum BindPart {
    Corner(usize),
    Key(usize),
    /// The box shown while a key is being waited for, and its blinking key.
    EditBox(usize),
    EditKey(usize),
}

pub(crate) fn spawn_keybinding_popup(mut commands: Commands, art: Option<Res<Art>>) {
    let art = art.as_deref();
    let image = |name: &str| art.and_then(|a| a.image(name));
    let mut root = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        GlobalZIndex(12),
        Visibility::Hidden,
        KeybindingRoot,
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
    root.with_children(|page| {
        let left = -CONTENT.0 / 2.0;
        let head_y = CONTENT_Y + CONTENT.1 / 2.0 + 25.0;
        page.spawn(Node {
            justify_content: JustifyContent::FlexStart,
            ..place((560.0, 50.0), (left + NAME_X + 280.0, head_y))
        })
        .with_children(|t| {
            t.spawn(label_with(vh(HEAD_SIZE), "Action", ENZO_WHITE));
        });
        page.spawn(place((KEY_BOX.0, 50.0), (CONTENT.0 / 2.0 - KEY_RIGHT - KEY_BOX.0 / 2.0, head_y)))
            .with_children(|t| {
                t.spawn(label_with(vh(HEAD_SIZE), "Key", ENZO_WHITE));
            });
        for i in 0..ROWS {
            let y = row_y(i);
            let reset = i == Action::ALL.len();
            let size = if reset { RESET } else { ROW };
            let mut row = page.spawn((
                place(size, (0.0, y)),
                BackgroundColor(Color::srgba(0.3, 0.24, 0.55, 0.9)),
            ));
            if let Some(back) = image("menu_button_background") {
                row.insert((sliced(back, BUTTON_SLICE), BackgroundColor(Color::NONE)));
            }
            if reset {
                page.spawn(place(RESET, (0.0, y))).with_children(|t| {
                    t.spawn(label_with(vh(30.0), "Reset to default", ENZO_BLACK));
                });
            } else {
                let name = Action::ALL[i].label();
                page.spawn(Node {
                    justify_content: JustifyContent::FlexStart,
                    ..place((560.0, ROW.1), (left + NAME_X + 280.0, y))
                })
                .with_children(|t| {
                    t.spawn(label_with(vh(NAME_SIZE), name, ENZO_BLACK));
                });
                let x = CONTENT.0 / 2.0 - KEY_RIGHT - KEY_BOX.0 / 2.0;
                page.spawn(place(KEY_BOX, (x, y))).with_children(|b| {
                    b.spawn((label_with(vh(KEY_SIZE), "", ENZO_BLACK), BindPart::Key(i)));
                });
                let mut edit = page.spawn((
                    place(KEY_BOX, (x, y)),
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                    Visibility::Hidden,
                    BindPart::EditBox(i),
                ));
                if let Some(back) = image("settings_checkbox") {
                    edit.insert((sliced(back, [12.0; 4]), BackgroundColor(Color::NONE)));
                }
                edit.with_children(|b| {
                    b.spawn((label_with(vh(KEY_EDIT_SIZE), "", ENZO_WHITE), BindPart::EditKey(i)));
                });
            }
            // ItemHighlight: corner centres 10 px out from the item's corners.
            let frame = (size.0 + 2.0 * CORNER_GAP + CORNER_SMALL.0, size.1 + 2.0 * CORNER_GAP + CORNER_SMALL.1);
            match image("menu_corner_small") {
                Some(corner) => {
                    corners(
                        page,
                        &corner,
                        frame,
                        (0.0, y),
                        CORNER_SMALL,
                        BindPart::Corner(i),
                    );
                }
                None => {
                    page.spawn((
                        place(frame, (0.0, y)),
                        Outline::new(Val::Px(3.0), Val::ZERO, Color::WHITE),
                        BindPart::Corner(i),
                    ));
                }
            }
        }
    });
}

/// The list's keys, called from the Settings screen while the list is open.
/// Returns true when the bindings changed.
pub(crate) fn keybinding_input(
    keys: &ButtonInput<KeyCode>,
    taps: &crate::select::Taps,
    menu: &mut KeybindingMenu,
    bindings: &mut KeyBindings,
) -> bool {
    if menu.waiting {
        // The next key (Esc is handled as cancel by the menu) becomes the action's.
        let Some(key) = keys
            .get_just_pressed()
            .find(|k| **k != KeyCode::Escape && key_from_name(key_name(**k)).is_some())
        else {
            return false;
        };
        menu.waiting = false;
        if let Some(&action) = Action::ALL.get(menu.selected) {
            bindings.set(action, *key);
            return true;
        }
        return false;
    }
    if taps.up {
        menu.selected = (menu.selected + ROWS - 1) % ROWS;
    }
    if taps.down {
        menu.selected = (menu.selected + 1) % ROWS;
    }
    if taps.key_pick || !taps.pad_pick.is_empty() {
        if menu.selected < Action::ALL.len() {
            menu.waiting = true;
        } else {
            bindings.reset();
            return true;
        }
    }
    false
}

/// Esc in the list: cancels a wait, else closes the list. Returns false when the
/// list wasn't open, so Esc does its usual thing.
pub(crate) fn keybinding_escape(menu: &mut KeybindingMenu) -> bool {
    if !menu.open {
        return false;
    }
    if menu.waiting {
        menu.waiting = false;
    } else {
        menu.open = false;
    }
    true
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(crate) fn show_keybinding_popup(
    time: Res<Time>,
    menu: Res<KeybindingMenu>,
    on_settings: Res<crate::menu::MenuScreen>,
    game: Res<crate::gameplay::Match>,
    bindings: Res<KeyBindings>,
    mut root: Query<&mut Visibility, With<KeybindingRoot>>,
    mut parts: Query<(&BindPart, &mut Visibility), Without<KeybindingRoot>>,
    mut texts: Query<(&BindPart, &mut Text)>,
) {
    let on = menu.open
        && game.phase == crate::gameplay::Phase::Menu
        && *on_settings == crate::menu::MenuScreen::Settings;
    for mut v in &mut root {
        v.set_if_neq(shown(on));
    }
    if !on {
        return;
    }
    let editing = |i: usize| menu.waiting && i == menu.selected;
    for (part, mut v) in &mut parts {
        match *part {
            BindPart::Corner(i) => v.set_if_neq(shown(i == menu.selected)),
            BindPart::EditBox(i) => v.set_if_neq(shown(editing(i))),
            _ => false,
        };
    }
    // The key being edited shows on the checkbox, blinking every 0.25 s; the row's own
    // key text hides under it.
    let blink = ((time.elapsed_secs() / BLINK) as u32).is_multiple_of(2);
    for (part, mut text) in &mut texts {
        let (i, shows) = match *part {
            BindPart::Key(i) => (i, !editing(i)),
            BindPart::EditKey(i) => (i, editing(i) && blink),
            _ => continue,
        };
        let words = if shows {
            Action::ALL
                .get(i)
                .map_or(String::new(), |a| cap_name(key_name(bindings.key(*a))).to_string())
        } else {
            String::new()
        };
        if text.0 != words {
            text.0 = words;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_default_to_the_original_and_swap_on_a_clash() {
        let mut keys = KeyBindings::default();
        assert_eq!(keys.key(Action::NudgeLeft), KeyCode::KeyU);
        assert_eq!(keys.key(Action::TurnLeft), KeyCode::Space);
        // Putting light magic on K takes dark magic's key; dark magic gets J.
        keys.set(Action::LightMagic, KeyCode::KeyK);
        assert_eq!(keys.key(Action::LightMagic), KeyCode::KeyK);
        assert_eq!(keys.key(Action::DarkMagic), KeyCode::KeyJ);
        keys.reset();
        assert_eq!(keys, KeyBindings::default());
    }

    #[test]
    fn keys_survive_a_restart() {
        let file = std::env::temp_dir()
            .join(format!("tt-keys-{}", std::process::id()))
            .join("keys.ron");
        let mut keys = KeyBindings::load(file.clone());
        keys.set(Action::Drop, KeyCode::KeyS);
        keys.save();
        let again = KeyBindings::load(file.clone());
        assert_eq!(again.key(Action::Drop), KeyCode::KeyS);
        assert_eq!(again.key(Action::MoveLeft), KeyCode::ArrowLeft);
        let _ = std::fs::remove_dir_all(file.parent().unwrap());
    }

    #[test]
    fn every_key_name_reads_back() {
        for (key, name) in KEY_NAMES {
            assert_eq!(key_from_name(name), Some(*key));
        }
    }
}
