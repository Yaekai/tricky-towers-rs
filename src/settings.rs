//! The Settings screen from the title tiles, laid out from the PC readout
//! (docs/research/menus.md "Settings screen layout") with the options from the fan
//! wiki (docs/research/wiki-notes.md). Two pages side by side that slide:
//! page 1 has Sound effects, Music, Explanations, Stats, Credits and Keybinding;
//! page 2 has Full screen, Resolution, Quality, V-sync, Language and Apply, and
//! its changes wait for Apply. Kept between runs in `settings.ron`.
//!
//! Stats, Credits and Keybinding open their popups (stats.rs, credits.rs, keybinding.rs).
//! Text follows the picked language through `letters::Translations` (languages the letter
//! sheet can't draw stay English). Not done: Quality (kept but changes nothing),
//! language flags, slider repeat while held (every 0.2 s), and the tips that
//! Explanations turns on.

use std::path::PathBuf;

use bevy::audio::{AudioSink, AudioSinkPlayback, PlaybackSettings, Volume};
use bevy::prelude::*;
use bevy::window::{MonitorSelection, PresentMode, PrimaryWindow, WindowMode};
use serde::{Deserialize, Serialize};

use crate::art::Music;
use crate::letters::TextLook;
use crate::gameplay::{Match, Phase};
use crate::menu::MenuScreen;
use crate::select::{
    CORNER_SMALL, SCREEN, corners, label, label_with, place, read_taps, shown, sliced, spawn_page,
};

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Settings>()
            .init_resource::<SettingsMenu>()
            .init_resource::<crate::keybinding::KeybindingMenu>()
            .init_resource::<crate::keybinding::KeyBindings>()
            .add_plugins((
                crate::stats::StatsPlugin,
                crate::credits::CreditsPlugin,
                crate::explain::ExplainPlugin,
                crate::help::HelpPlugin,
                crate::online_lobby::OnlineLobbyPlugin,
            ))
            .add_systems(Startup, crate::keybinding::spawn_keybinding_popup)
            .add_systems(Update, crate::keybinding::show_keybinding_popup)
            .add_systems(Startup, spawn_settings_screen)
            .add_systems(Update, (apply_settings, read_screen_modes, pause_sfx))
            // Before the audio output (which runs after transform propagation) picks
            // new sounds up.
            .add_systems(
                PostUpdate,
                sfx_rules.before(bevy::transform::TransformSystems::Propagate),
            );
    }
}

/// Graphics quality, as the PC install names it (default High; the wiki's
/// Performance/Normal/Magical may be older names).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Quality {
    Low,
    Medium,
    #[default]
    High,
}

impl Quality {
    const ALL: [Quality; 3] = [Quality::Low, Quality::Medium, Quality::High];
}

/// Window sizes offered, from 512×384 up to 1920×1080 (the in-between ones are
/// common sizes, not read from the original, ?), plus Max's 3440×1440 monitor.
/// Used when the screen's own modes can't be read (the original lists Screen.resolutions,
/// the monitor's modes).
pub const RESOLUTIONS: [(u32, u32); 11] = [
    (512, 384),
    (640, 480),
    (800, 600),
    (1024, 768),
    (1280, 720),
    (1280, 1024),
    (1366, 768),
    (1600, 900),
    (1920, 1080),
    (2560, 1440),
    (3440, 1440),
];

/// The original's 19 language flags, in its order, by name (the original shows flags).
pub const LANGUAGES: [&str; 19] = [
    "English",
    "German",
    "French",
    "Spanish",
    "Portuguese",
    "Italian",
    "Chinese (Simplified)",
    "Chinese (Traditional)",
    "Japanese",
    "Korean",
    "Russian",
    "Polish",
    "Arabic",
    "Turkish",
    "Dutch",
    "Swedish",
    "Norwegian",
    "Finnish",
    "Romanian",
];

/// Each language's flag picture (`menu_language_<code>`), in `LANGUAGES` order.
const LANGUAGE_FLAGS: [&str; 19] = [
    "en", "de", "fr", "es", "pt", "it", "ch", "tw", "jp", "ko", "ru", "pl", "ar", "tr", "nl", "sv",
    "no", "fi", "ro",
];

/// The flag shown in place of the language's name (on settings page `0`).
#[derive(Component)]
pub(crate) struct LanguageFlag(usize);

/// Apply is a button centred under the rows, not a full row: SettingsListApplyItem's
/// background is the list's width less 400 px, 58 px tall, its title 30 px.
const APPLY_BUTTON: (f32, f32) = (ROW.0 - 400.0, 58.0);

/// The player's settings. Kept in `file` when one is set (the game sets it;
/// tests and offscreen pictures don't).
#[derive(Resource, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Sound effects, 0 to 1.
    pub volume: f32,
    /// Music, 0 to 1 (separate from the sound effects).
    pub music: f32,
    /// Tips and objectives.
    pub explanations: bool,
    pub fullscreen: bool,
    pub resolution: (u32, u32),
    pub quality: Quality,
    pub vsync: bool,
    pub language: String,
    #[serde(skip)]
    pub file: Option<PathBuf>,
    /// The monitor's own modes (Screen.resolutions), smallest first; empty until read.
    #[serde(skip)]
    pub screen_modes: Vec<(u32, u32)>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            volume: 1.0,
            music: 1.0,
            explanations: true,
            fullscreen: true,
            resolution: (1920, 1080),
            quality: Quality::High,
            vsync: false,
            language: LANGUAGES[0].to_string(),
            file: None,
            screen_modes: Vec::new(),
        }
    }
}

impl Settings {
    pub fn load(file: PathBuf) -> Self {
        let mut settings: Self = std::fs::read_to_string(&file)
            .ok()
            .and_then(|text| ron::from_str(&text).ok())
            .unwrap_or_default();
        settings.volume = settings.volume.clamp(0.0, 1.0);
        settings.music = settings.music.clamp(0.0, 1.0);
        settings.file = Some(file);
        settings
    }

    pub fn save(&self) {
        let Some(file) = &self.file else {
            return;
        };
        let written = file
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|_| {
                let text = ron::to_string(self).map_err(std::io::Error::other)?;
                std::fs::write(file, text)
            });
        if let Err(err) = written {
            warn!("could not save settings to {}: {err}", file.display());
        }
    }

    /// Moves `row`'s value one step left (-1) or right (+1); toggles flip either
    /// way. Returns true if anything changed.
    pub fn step(&mut self, row: Row, dir: i32) -> bool {
        let before = self.clone();
        let nudge = |v: f32| ((v + 0.1 * dir as f32) * 10.0).round().clamp(0.0, 10.0) / 10.0;
        let cycle = |at: usize, len: usize| (at as i32 + dir).rem_euclid(len as i32) as usize;
        match row {
            Row::Effects => self.volume = nudge(self.volume),
            Row::Music => self.music = nudge(self.music),
            Row::Explanations => self.explanations = !self.explanations,
            Row::Fullscreen => self.fullscreen = !self.fullscreen,
            Row::Vsync => self.vsync = !self.vsync,
            Row::Resolution => {
                let list = if self.screen_modes.is_empty() {
                    RESOLUTIONS.to_vec()
                } else {
                    self.screen_modes.clone()
                };
                let at = list.iter().position(|&r| r == self.resolution);
                let next = at.map_or(list.len() - 1, |i| cycle(i, list.len()));
                self.resolution = list[next];
            }
            Row::Quality => {
                let at = Quality::ALL
                    .iter()
                    .position(|&q| q == self.quality)
                    .unwrap_or(2);
                self.quality = Quality::ALL[cycle(at, Quality::ALL.len())];
            }
            Row::Language => {
                let at = LANGUAGES
                    .iter()
                    .position(|&l| l == self.language)
                    .unwrap_or(0);
                self.language = LANGUAGES[cycle(at, LANGUAGES.len())].to_string();
            }
            Row::Stats | Row::Credits | Row::Keybinding | Row::Apply => {}
        }
        *self != before
    }

    /// Takes page 2's values (the graphics ones) from `pending`.
    fn apply_page_two(&mut self, pending: &Settings) {
        self.fullscreen = pending.fullscreen;
        self.resolution = pending.resolution;
        self.quality = pending.quality;
        self.vsync = pending.vsync;
        self.language = pending.language.clone();
    }

    /// The words in a choice row's value area.
    fn choice(&self, row: Row) -> String {
        match row {
            Row::Resolution => format!("{} X {}", self.resolution.0, self.resolution.1),
            // The original's names for its three levels.
            Row::Quality => match self.quality {
                Quality::Low => "Performance",
                Quality::Medium => "Normal",
                Quality::High => "Magical",
            }
            .to_string(),
            Row::Language => self.language.clone(),
            _ => String::new(),
        }
    }
}

/// One row of the Settings screen.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Row {
    Effects,
    Music,
    Explanations,
    Stats,
    Credits,
    Keybinding,
    Fullscreen,
    Resolution,
    Quality,
    Vsync,
    Language,
    Apply,
}

/// How a row shows and changes its value.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Slider,
    Choice,
    Toggle,
    /// Just an icon and a title; picking it opens something.
    Popup,
}

impl Row {
    fn kind(self) -> Kind {
        match self {
            Row::Effects | Row::Music => Kind::Slider,
            Row::Resolution | Row::Quality | Row::Language => Kind::Choice,
            Row::Explanations | Row::Fullscreen | Row::Vsync => Kind::Toggle,
            Row::Stats | Row::Credits | Row::Keybinding | Row::Apply => Kind::Popup,
        }
    }

    fn title(self) -> &'static str {
        match self {
            Row::Effects => "Sound effects",
            Row::Music => "Music",
            Row::Explanations => "Explanations enabled",
            Row::Stats => "Statistics",
            Row::Credits => "Credits",
            Row::Keybinding => "Keybinding",
            Row::Fullscreen => "Fullscreen",
            Row::Resolution => "Resolution",
            Row::Quality => "Quality",
            Row::Vsync => "V-Sync",
            Row::Language => "Language",
            Row::Apply => "Apply",
        }
    }

    /// Its icon's size in the row's 100 px box (the <name>Icon prefabs).
    fn icon_size(self) -> (f32, f32) {
        match self {
            Row::Effects => (99.0, 76.0),
            Row::Music => (80.0, 74.0),
            Row::Explanations => (43.0, 73.0),
            Row::Stats => (107.0, 77.0),
            Row::Credits => (63.0, 71.0),
            Row::Keybinding => (103.0, 72.0),
            Row::Fullscreen => (94.0, 73.0),
            Row::Quality => (71.0, 70.0),
            Row::Vsync => (70.0, 74.0),
            Row::Language => (97.0, 81.0),
            _ => (100.0, 100.0),
        }
    }

    /// The staged icon, `settings_icon_<key>` (Apply has none, ?).
    fn icon(self) -> Option<&'static str> {
        Some(match self {
            Row::Effects => "settings_icon_sfx",
            Row::Music => "settings_icon_music",
            Row::Explanations => "settings_icon_explanations",
            Row::Stats => "settings_icon_stats",
            Row::Credits => "settings_icon_credits",
            Row::Keybinding => "settings_icon_keybinding",
            Row::Fullscreen => "settings_icon_fullscreen",
            Row::Resolution => "settings_icon_resolution",
            Row::Quality => "settings_icon_quality",
            Row::Vsync => "settings_icon_vsync",
            Row::Language => "settings_icon_language",
            Row::Apply => return None,
        })
    }
}

/// The two pages, top row first.
pub const PAGES: [[Row; 6]; 2] = [
    [
        Row::Effects,
        Row::Music,
        Row::Explanations,
        Row::Stats,
        Row::Credits,
        Row::Keybinding,
    ],
    [
        Row::Fullscreen,
        Row::Resolution,
        Row::Quality,
        Row::Vsync,
        Row::Language,
        Row::Apply,
    ],
];

/// Layout from the readout, in 1920×1080 pixels from the screen centre (y up).
const ROW: (f32, f32) = (800.0, 78.0);
const ROW_STEP: f32 = 100.0;
/// The rows' area runs from 220 px below the top to the bottom; its centre.
/// The original centres the rows on the screen (its pages sit around y 290 to 790 at 1080).
const AREA_CENTRE: f32 = 0.0;
const ICON: ((f32, f32), f32, f32) = ((100.0, 100.0), 79.0, 11.0);
const TITLE_X: f32 = 150.0;
const TITLE_SIZE: f32 = 36.0;
/// The smallest a row title auto-sizes to.
const TITLE_MIN: f32 = 24.0;
/// The row words: EnzoBlack, the display letters in black with no outline.
const ROW_TEXT: TextLook = TextLook {
    body: false,
    face: Some(Color::BLACK),
    outline: None,
    low: None,
    outline_edge: 0.0,
};
/// The value area at the row's right end, its arrows and the slider bar.
const VALUE_WIDTH: f32 = 360.0;
const VALUE_ARROW: (f32, f32) = (25.0, 31.0);
const VALUE_ARROW_INSET: f32 = 40.0;
const BAR: (f32, f32) = (190.0, 40.0);
/// The slider's pills.
const PILLS: usize = 10;
const BAR_INSET: f32 = 85.0;
const CHECKBOX: ((f32, f32), f32) = ((43.0, 43.0), 60.0);
const TICK: ((f32, f32), (f32, f32)) = ((61.0, 49.0), (9.0, 3.0));
const CROSS: (f32, f32) = (49.0, 49.0);
const CORNER_GAP: f32 = 10.0;
const PAGE_ARROW: ((f32, f32), f32) = ((94.0, 55.0), 500.0);
/// `menu_button_background`'s 9-slice borders, as on the Single Player list.
const BUTTON_SLICE: [f32; 4] = [18.0, 71.0, 15.0, 16.0];
/// Seconds a page slide takes (SettingsStateController: DOLocalMoveX over 0.3 s).
const SLIDE_SECONDS: f32 = 0.3;

fn row_y(index: usize) -> f32 {
    AREA_CENTRE + ROW_STEP / 2.0 * (PAGES[0].len() as f32 - 1.0) - ROW_STEP * index as f32
}

/// The settings row under a cursor (top-left origin) in a window `window` big.
fn row_at(cursor: Vec2, window: Vec2, rows: usize) -> Option<usize> {
    let p = screen_point(cursor, window);
    (0..rows).find(|&i| {
        (p.x.abs() <= ROW.0 / 2.0) && ((p.y - row_y(i)).abs() <= ROW.1 / 2.0)
    })
}

/// A cursor (top-left origin) in a window `window` big, on the 1920x1080 screen
/// with its centre at 0 and y up.
fn screen_point(cursor: Vec2, window: Vec2) -> Vec2 {
    Vec2::new(
        cursor.x / window.x * SCREEN.0 - SCREEN.0 / 2.0,
        SCREEN.1 / 2.0 - cursor.y / window.y * SCREEN.1,
    )
}

/// Steps a click at screen x `x` makes on a value row whose slider stands at
/// `now` (0-1): the arrows step once (ButtonPrevious/ButtonNext), a click on a
/// slider's bar sets the nearest tenth (SettingsSliderItem._HandleValueClicked).
/// `None` when the click isn't on a value control.
fn click_steps(row: Row, x: f32, now: f32) -> Option<i32> {
    let near = |at: f32| (x - value_x(at)).abs() <= VALUE_ARROW.0;
    if !matches!(row.kind(), Kind::Slider | Kind::Choice) {
        return None;
    }
    if near(VALUE_ARROW_INSET) {
        return Some(-1);
    }
    if near(VALUE_WIDTH - VALUE_ARROW_INSET) {
        return Some(1);
    }
    let along = x - value_x(BAR_INSET);
    (row.kind() == Kind::Slider && (0.0..=BAR.0).contains(&along))
        .then(|| (along / BAR.0 * 10.0).round() as i32 - (now * 10.0).round() as i32)
}

/// x of a point `inset` px into the value area from its left end.
fn value_x(inset: f32) -> f32 {
    ROW.0 / 2.0 - VALUE_WIDTH + inset
}

#[derive(Resource, Debug, Default)]
pub struct SettingsMenu {
    pub page: usize,
    pub selected: usize,
    /// Page 2's values until Apply; `None` until something there changes.
    pub pending: Option<Settings>,
    /// Where the pages are, sliding towards `page`.
    slide: f32,
}

impl SettingsMenu {
    pub fn row(&self) -> Row {
        PAGES[self.page][self.selected]
    }
}

#[derive(Component)]
pub(crate) struct SettingsRoot;

/// A Settings node: which page and row it belongs to and what it is.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub(crate) enum SettingsPart {
    Page(usize),
    Corner(usize, usize),
    /// One of a slider's ten filled pills (page, row, pill).
    Pill(usize, usize, usize),
    Tick(usize, usize),
    Cross(usize, usize),
    Value(usize, usize),
    PageArrow(bool),
}

fn spawn_settings_screen(
    mut commands: Commands,
    art: Option<Res<crate::art::Art>>,
) {
    let art = art.as_deref();
    let image = |name: &str| art.and_then(|a| a.image(name));
    let root = spawn_page(&mut commands, art, "SETTINGS");
    commands
        .entity(root)
        .insert(SettingsRoot)
        .with_children(|root| {
            for (p, rows) in PAGES.iter().enumerate() {
                root.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    SettingsPart::Page(p),
                ))
                .with_children(|page| {
                    for (i, &row) in rows.iter().enumerate() {
                        spawn_row(page, &image, p, i, row);
                    }
                });
            }
            for right in [false, true] {
                let x = if right { PAGE_ARROW.1 } else { -PAGE_ARROW.1 };
                let mut arrow = root.spawn((
                    place(PAGE_ARROW.0, (x, 0.0)),
                    SettingsPart::PageArrow(right),
                ));
                match image("menu_arrow") {
                    Some(art) => {
                        // The staged arrow points up; the original's point out sideways.
                        arrow.insert((
                            ImageNode::new(art),
                            UiTransform::from_rotation(Rot2::degrees(if right {
                                90.0
                            } else {
                                -90.0
                            })),
                        ));
                    }
                    None => {
                        arrow.with_children(|a| {
                            a.spawn(label(4.0, if right { ">" } else { "<" }));
                        });
                    }
                }
            }
        });
}

fn spawn_row(
    page: &mut ChildSpawnerCommands,
    image: &impl Fn(&str) -> Option<Handle<Image>>,
    p: usize,
    i: usize,
    row: Row,
) {
    let y = row_y(i);
    let left = -ROW.0 / 2.0;
    if row == Row::Apply {
        let mut back = page.spawn((
            place(APPLY_BUTTON, (0.0, y)),
            BackgroundColor(Color::srgba(0.3, 0.24, 0.55, 0.8)),
        ));
        if let Some(art) = image("menu_button_background") {
            back.insert((sliced(art, BUTTON_SLICE), BackgroundColor(Color::NONE)));
        }
        back.with_children(|b| {
            b.spawn(label_with(30.0 / SCREEN.1 * 100.0, row.title(), ROW_TEXT));
        });
        // Its Selected corners sit 10 px out from the button, as on the other rows.
        if let Some(corner) = image("menu_corner_small") {
            let frame = (APPLY_BUTTON.0 + 2.0 * CORNER_GAP, APPLY_BUTTON.1 + 2.0 * CORNER_GAP);
            corners(page, &corner, frame, (0.0, y), CORNER_SMALL, SettingsPart::Corner(p, i));
        }
        return;
    }
    let mut back = page.spawn((
        place(ROW, (0.0, y)),
        BackgroundColor(Color::srgba(0.3, 0.24, 0.55, 0.8)),
    ));
    if let Some(art) = image("menu_button_background") {
        back.insert((sliced(art, BUTTON_SLICE), BackgroundColor(Color::NONE)));
    }
    if let Some(icon) = row.icon().and_then(image) {
        // Each icon at its own prefab's size, centred in the 100 px box.
        page.spawn((ImageNode::new(icon), place(row.icon_size(), (left + ICON.1, y + ICON.2))));
    }
    let title_box = (ROW.0 - TITLE_X, ROW.1);
    page.spawn(Node {
        justify_content: JustifyContent::FlexStart,
        ..place(title_box, (left + TITLE_X + title_box.0 / 2.0, y))
    })
    .with_children(|t| {
        // Titles auto-size from 36 down to 24 px in their box: the row less 510 px beside a
        // slider or a choice, less 263.8 beside a toggle or alone (SettingsList*Item).
        let free = if matches!(row.kind(), Kind::Slider | Kind::Choice) { 510.0 } else { 263.8 };
        t.spawn((
            label_with(TITLE_SIZE / SCREEN.1 * 100.0, row.title(), ROW_TEXT),
            crate::letters::AutoSize {
                min: TITLE_MIN / SCREEN.1 * 100.0,
                width: (ROW.0 - free) / SCREEN.1 * 100.0,
            },
        ));
    });
    let arrows = |page: &mut ChildSpawnerCommands| {
        for right in [false, true] {
            let x = if right {
                value_x(VALUE_WIDTH - VALUE_ARROW_INSET)
            } else {
                value_x(VALUE_ARROW_INSET)
            };
            let mut arrow = page.spawn(place(VALUE_ARROW, (x, y)));
            match image("settings_arrow") {
                // The arrow art (gph_menuHorizontalListArrow) points right; the left one is flipped.
                Some(art) => {
                    arrow.insert(ImageNode {
                        image: art,
                        flip_x: !right,
                        ..default()
                    });
                }
                None => {
                    arrow.with_children(|a| {
                        a.spawn(label(3.0, if right { ">" } else { "<" }));
                    });
                }
            }
        }
    };
    match row.kind() {
        Kind::Slider => {
            arrows(page);
            // Ten pills (the 19x40 slider pictures side by side), filled up to
            // the value, as the original draws its sliders.
            let pill = (BAR.0 / PILLS as f32, BAR.1);
            for k in 0..PILLS {
                let x = value_x(BAR_INSET) + pill.0 * (k as f32 + 0.5);
                let mut empty = page.spawn((
                    place(pill, (x, y)),
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                ));
                if let Some(art) = image("settings_slider_empty") {
                    empty.insert((ImageNode::new(art), BackgroundColor(Color::NONE)));
                }
                let mut fill = page.spawn((
                    place(pill, (x, y)),
                    BackgroundColor(Color::srgb(0.95, 0.8, 0.3)),
                    SettingsPart::Pill(p, i, k),
                ));
                if let Some(art) = image("settings_slider_fill") {
                    fill.insert((ImageNode::new(art), BackgroundColor(Color::NONE)));
                }
            }
        }
        Kind::Choice => {
            arrows(page);
            let width = VALUE_WIDTH - 2.0 * (VALUE_ARROW_INSET + VALUE_ARROW.0);
            page.spawn(place((width, ROW.1), (value_x(VALUE_WIDTH / 2.0), y)))
                .with_children(|v| {
                    v.spawn((
                        label(TITLE_SIZE / SCREEN.1 * 100.0, ""),
                        SettingsPart::Value(p, i),
                    ));
                });
            if row == Row::Language {
                page.spawn((
                    ImageNode::default(),
                    place((62.0, 48.0), (value_x(VALUE_WIDTH / 2.0), y)),
                    LanguageFlag(p),
                    Visibility::Hidden,
                ));
            }
        }
        Kind::Toggle => {
            let x = ROW.0 / 2.0 - CHECKBOX.1;
            let mut boxed = page.spawn((
                place(CHECKBOX.0, (x, y)),
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            ));
            if let Some(art) = image("settings_checkbox") {
                boxed.insert((ImageNode::new(art), BackgroundColor(Color::NONE)));
            }
            let (size, (dx, dy)) = TICK;
            let mut tick = page.spawn((place(size, (x + dx, y + dy)), SettingsPart::Tick(p, i)));
            match image("settings_tick") {
                Some(art) => {
                    tick.insert(ImageNode::new(art));
                }
                None => {
                    tick.with_children(|t| {
                        t.spawn(label(3.5, "V"));
                    });
                }
            }
            let mut cross = page.spawn((place(CROSS, (x, y)), SettingsPart::Cross(p, i)));
            match image("settings_cross") {
                Some(art) => {
                    cross.insert(ImageNode::new(art));
                }
                None => {
                    cross.with_children(|t| {
                        t.spawn(label(3.5, "X"));
                    });
                }
            }
        }
        Kind::Popup => {}
    }
    if let Some(corner) = image("menu_corner_small") {
        let frame = (ROW.0 + 2.0 * CORNER_GAP, ROW.1 + 2.0 * CORNER_GAP);
        corners(
            page,
            &corner,
            frame,
            (0.0, y),
            CORNER_SMALL,
            SettingsPart::Corner(p, i),
        );
    }
}

/// Up/Down pick a row, Left/Right change it (or turn the page on a row without
/// a value), the bumpers turn the page, Enter toggles, applies or opens, Esc or
/// B goes back to the title tiles.
#[allow(clippy::too_many_arguments)]
pub(crate) fn settings_input(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    pads: Query<(Entity, &Gamepad)>,
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    game: Res<Match>,
    mut screen: ResMut<MenuScreen>,
    mut menu: ResMut<SettingsMenu>,
    mut settings: ResMut<Settings>,
    mut binds: ResMut<crate::keybinding::KeybindingMenu>,
    mut bindings: ResMut<crate::keybinding::KeyBindings>,
    mut stats: ResMut<crate::stats::StatsMenu>,
    mut credits: ResMut<crate::credits::CreditsMenu>,
    mut last_cursor: Local<Option<Vec2>>,
) {
    if game.phase != Phase::Menu || *screen != MenuScreen::Settings {
        stats.open = false;
        binds.open = false;
        binds.waiting = false;
        // Page 2's unapplied changes are dropped on leaving.
        if menu.pending.is_some() {
            menu.pending = None;
        }
        return;
    }
    // The Credits popup takes the keys while it runs (credits::run_credits).
    if credits.open {
        return;
    }
    let mut taps = read_taps(keys.as_deref(), &pads);
    let mut clicked = 0;
    // The Keybinding list's mouse: hover selects a row, a click takes it (waits for a key, or
    // resets); the Settings rows behind the list don't hear it.
    let list_mouse = binds.open && !binds.waiting;
    if list_mouse
        && let Ok(window) = windows.single()
        && let Some(c) = window.cursor_position()
        && let Some(i) = crate::keybinding::row_at(screen_point(c, window.size()))
    {
        binds.selected = i;
        if mouse.as_deref().is_some_and(|m| m.just_pressed(MouseButton::Left)) {
            taps.key_pick = true;
        }
    }
    // The mouse: a row under the cursor is selected as it moves, and a click acts on it.
    if !binds.open
        && let Ok(window) = windows.single()
    {
        let cursor = window.cursor_position();
        let over = cursor.and_then(|c| row_at(c, window.size(), PAGES[menu.page].len()));
        if over.is_some() && cursor != *last_cursor {
            menu.selected = over.unwrap_or(menu.selected);
        }
        *last_cursor = cursor;
        if let Some(i) = over
            && mouse.is_some_and(|m| m.just_pressed(MouseButton::Left))
        {
            menu.selected = i;
            let row = menu.row();
            let now = match row {
                Row::Music => settings.music,
                _ => settings.volume,
            };
            let x = cursor.map_or(0.0, |c| screen_point(c, window.size()).x);
            match click_steps(row, x, now) {
                Some(steps) => clicked = steps,
                None => taps.key_pick = true,
            }
        }
    }
    // The Stats popup only closes: Enter, B or Esc (in the menu's Esc handling).
    if stats.open {
        if taps.key_pick || !taps.pad_pick.is_empty() || !taps.pad_back.is_empty() {
            stats.open = false;
        }
        return;
    }
    // The Keybinding list takes the keys while it's open; B closes it.
    if binds.open {
        if !taps.pad_back.is_empty() {
            crate::keybinding::keybinding_escape(&mut binds);
        } else if let Some(keys) = keys.as_deref()
            && crate::keybinding::keybinding_input(keys, &taps, &mut binds, &mut bindings)
        {
            bindings.save();
        }
        return;
    }
    if !taps.pad_back.is_empty() {
        *screen = MenuScreen::Main;
        return;
    }
    let bumper = |button| pads.iter().any(|(_, pad)| pad.just_pressed(button));
    let mut turn =
        bumper(GamepadButton::RightTrigger) as i32 - bumper(GamepadButton::LeftTrigger) as i32;
    let count = PAGES[menu.page].len();
    if taps.up {
        menu.selected = (menu.selected + count - 1) % count;
    }
    if taps.down {
        menu.selected = (menu.selected + 1) % count;
    }
    let pick = taps.key_pick || !taps.pad_pick.is_empty();
    let dir = taps.right as i32 - taps.left as i32;
    let row = menu.row();
    let mut changed = false;
    match row.kind() {
        Kind::Popup if row == Row::Apply && pick => {
            if let Some(pending) = menu.pending.take() {
                settings.apply_page_two(&pending);
                changed = true;
            }
        }
        Kind::Popup if row == Row::Keybinding && pick => {
            binds.open = true;
            binds.selected = 0;
            binds.waiting = false;
        }
        Kind::Popup if row == Row::Stats && pick => stats.open = true,
        Kind::Popup if row == Row::Credits && pick => {
            credits.open = true;
            credits.time = 0.0;
        }
        Kind::Popup => turn += dir,
        Kind::Toggle if pick || dir != 0 => changed = step(&mut menu, &mut settings, row, 1),
        _ if dir != 0 => changed = step(&mut menu, &mut settings, row, dir),
        _ if clicked != 0 => {
            for _ in 0..clicked.abs() {
                changed |= step(&mut menu, &mut settings, row, clicked.signum());
            }
        }
        _ => {}
    }
    if turn != 0 {
        let page = (menu.page as i32 + turn).clamp(0, PAGES.len() as i32 - 1) as usize;
        if page != menu.page {
            menu.page = page;
            menu.selected = menu.selected.min(PAGES[page].len() - 1);
        }
    }
    if changed {
        settings.save();
    }
}

/// Changes `row` on the settings (page 1) or on page 2's pending copy. Returns
/// true when the saved settings changed.
fn step(menu: &mut SettingsMenu, settings: &mut Settings, row: Row, dir: i32) -> bool {
    if PAGES[1].contains(&row) {
        menu.pending
            .get_or_insert_with(|| settings.clone())
            .step(row, dir);
        false
    } else {
        settings.step(row, dir)
    }
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(crate) fn show_settings_screen(
    time: Res<Time>,
    game: Res<Match>,
    screen: Res<MenuScreen>,
    mut menu: ResMut<SettingsMenu>,
    settings: Res<Settings>,
    mut root: Query<&mut Visibility, With<SettingsRoot>>,
    mut parts: Query<(&SettingsPart, &mut Node, Option<&mut Visibility>), Without<SettingsRoot>>,
    mut texts: Query<(&SettingsPart, &mut Text)>,
    art: Option<Res<crate::art::Art>>,
    mut flags: Query<
        (&LanguageFlag, &mut ImageNode, &mut Visibility),
        (Without<SettingsPart>, Without<SettingsRoot>),
    >,
) {
    let on = game.phase == Phase::Menu && *screen == MenuScreen::Settings;
    for mut v in &mut root {
        v.set_if_neq(shown(on));
    }
    if !on {
        return;
    }
    let target = menu.page as f32;
    let step = time.delta_secs() / SLIDE_SECONDS;
    menu.slide = if (menu.slide - target).abs() <= step {
        target
    } else {
        menu.slide + step * (target - menu.slide).signum()
    };
    let shown_values = |p: usize| -> &Settings {
        match (&menu.pending, p) {
            (Some(pending), 1) => pending,
            _ => &settings,
        }
    };
    for (part, mut node, visibility) in &mut parts {
        let visible = match *part {
            SettingsPart::Page(p) => {
                let left = Val::Percent((p as f32 - menu.slide) * 100.0);
                if node.left != left {
                    node.left = left;
                }
                true
            }
            SettingsPart::Corner(p, i) => p == menu.page && i == menu.selected,
            SettingsPart::Pill(p, i, k) => {
                let values = shown_values(p);
                let v = match PAGES[p][i] {
                    Row::Effects => values.volume,
                    _ => values.music,
                };
                k < (v * PILLS as f32).round() as usize
            }
            SettingsPart::Tick(p, i) | SettingsPart::Cross(p, i) => {
                let values = shown_values(p);
                let value = match PAGES[p][i] {
                    Row::Explanations => values.explanations,
                    Row::Fullscreen => values.fullscreen,
                    _ => values.vsync,
                };
                value == matches!(part, SettingsPart::Tick(..))
            }
            SettingsPart::PageArrow(right) => (menu.page + 1 < PAGES.len()) == right,
            SettingsPart::Value(..) => true,
        };
        if let Some(mut v) = visibility {
            v.set_if_neq(shown(visible));
        }
    }
    // The language shows as its flag when the flags are staged.
    let flag_of = |p: usize| {
        let values = shown_values(p);
        let at = LANGUAGES.iter().position(|&l| l == values.language)?;
        art.as_ref()?
            .image(&format!("menu_language_{}", LANGUAGE_FLAGS[at]))
    };
    for (flag, mut node, mut visibility) in &mut flags {
        match flag_of(flag.0) {
            Some(image) => {
                if node.image != image {
                    node.image = image;
                }
                visibility.set_if_neq(Visibility::Inherited);
            }
            None => {
                visibility.set_if_neq(Visibility::Hidden);
            }
        }
    }
    for (part, mut words) in &mut texts {
        if let SettingsPart::Value(p, i) = *part {
            let line = if PAGES[p][i] == Row::Language && flag_of(p).is_some() {
                String::new()
            } else {
                shown_values(p).choice(PAGES[p][i])
            };
            if words.0 != line {
                words.0 = line;
            }
        }
    }
}

/// Puts changed settings into effect: the music already playing and the window.
fn apply_settings(
    settings: Res<Settings>,
    mut music: Query<&mut AudioSink, With<Music>>,
    mut ambience: Query<&mut AudioSink, (With<crate::art::Ambience>, Without<Music>)>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if !settings.is_changed() {
        return;
    }
    for mut sink in &mut music {
        sink.set_volume(Volume::Linear(settings.music));
    }
    for mut sink in &mut ambience {
        sink.set_volume(Volume::Linear(settings.volume));
    }
    let mode = if settings.fullscreen {
        WindowMode::BorderlessFullscreen(MonitorSelection::Current)
    } else {
        WindowMode::Windowed
    };
    let present = if settings.vsync {
        PresentMode::AutoVsync
    } else {
        PresentMode::AutoNoVsync
    };
    for mut window in &mut windows {
        if window.mode != mode {
            window.mode = mode;
        }
        if window.present_mode != present {
            window.present_mode = present;
        }
        let (w, h) = settings.resolution;
        // The resolution is in screen pixels (as Unity's Screen.SetResolution), whatever the
        // display's scaling.
        if !settings.fullscreen
            && (window.resolution.physical_width(), window.resolution.physical_height()) != (w, h)
        {
            window.resolution.set_physical_resolution(w, h);
        }
    }
}

/// Reads the primary monitor's modes once it shows up, for the Resolution row.
fn read_screen_modes(
    monitors: Query<&bevy::window::Monitor, With<bevy::window::PrimaryMonitor>>,
    mut settings: ResMut<Settings>,
    mut menu: Option<ResMut<SettingsMenu>>,
) {
    if !settings.screen_modes.is_empty() {
        return;
    }
    let Ok(monitor) = monitors.single() else { return };
    let mut modes: Vec<(u32, u32)> =
        monitor.video_modes.iter().map(|m| (m.physical_size.x, m.physical_size.y)).collect();
    modes.sort_unstable();
    modes.dedup();
    if modes.is_empty() {
        return;
    }
    if let Some(pending) = menu.as_mut().and_then(|m| m.pending.as_mut()) {
        pending.screen_modes = modes.clone();
    }
    settings.screen_modes = modes;
}

/// Sounds that don't start again while one is playing (InitAudioResourcesCommand's
/// ConcurrentAudioRules: SFX_GRAS_GROW_1..4 and SFX_SPAWN_MIST).
const ONE_AT_A_TIME: [&str; 5] = [
    "sfx_grass_grow_01",
    "sfx_grass_grow_02",
    "sfx_grass_grow_03",
    "sfx_grass_grow_04",
    "sfx_spawn_mist",
];

/// AudioManager.PlaySfx for every new sound effect: a sound asked for twice in a frame
/// plays once, the `ONE_AT_A_TIME` sounds wait for their last one to end, and effects and
/// the ambience play at the sound-effects volume times their own (SettingsStateController
/// gives the ambience the effects' volume). Music and stingers set the music volume
/// where they start.
#[allow(clippy::type_complexity)]
fn sfx_rules(
    mut commands: Commands,
    settings: Res<Settings>,
    art: Option<Res<crate::art::Art>>,
    mut sounds: Query<
        (Entity, Ref<AudioPlayer>, &mut PlaybackSettings),
        (Without<Music>, Without<crate::art::Stinger>),
    >,
) {
    if !sounds.iter().any(|(_, p, _)| p.is_added()) {
        return;
    }
    let held: Vec<AssetId<AudioSource>> = art
        .iter()
        .flat_map(|a| ONE_AT_A_TIME.iter().filter_map(|n| a.sounds.get(n)).map(|h| h.id()))
        .collect();
    let playing: Vec<AssetId<AudioSource>> = sounds
        .iter()
        .filter(|(_, p, _)| !p.is_added())
        .map(|(_, p, _)| p.0.id())
        .collect();
    let mut this_frame = Vec::new();
    for (entity, player, mut playback) in &mut sounds {
        if !player.is_added() {
            continue;
        }
        let id = player.0.id();
        if this_frame.contains(&id) || (held.contains(&id) && playing.contains(&id)) {
            commands.entity(entity).despawn();
            continue;
        }
        this_frame.push(id);
        playback.volume = Volume::Linear(playback.volume.to_linear() * settings.volume);
    }
}

/// PauseMenu pauses every sound effect while it is up and lets them go on as it closes
/// (AudioManager.PauseAllSfx / ResumeAllSfx); the music, its stingers and the ambience
/// play on.
#[allow(clippy::type_complexity)]
fn pause_sfx(
    game: Res<crate::gameplay::Match>,
    screen: Res<crate::menu::MenuScreen>,
    pause: Res<crate::menu::PauseMenu>,
    sinks: Query<
        (Entity, &AudioSink),
        (Without<Music>, Without<crate::art::Stinger>, Without<crate::art::Ambience>),
    >,
    mut held: Local<Option<Vec<Entity>>>,
) {
    let paused = game.phase == crate::gameplay::Phase::Menu
        && *screen == crate::menu::MenuScreen::Pause
        && pause.closing.is_none();
    if paused && held.is_none() {
        let mut list = Vec::new();
        for (entity, sink) in &sinks {
            if !sink.is_paused() {
                sink.pause();
                list.push(entity);
            }
        }
        *held = Some(list);
    } else if !paused && let Some(list) = held.take() {
        for entity in list {
            if let Ok((_, sink)) = sinks.get(entity) {
                sink.play();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sound_asked_for_twice_in_a_frame_plays_once_at_the_effects_volume() {
        let mut app = App::new();
        app.insert_resource(Settings { volume: 0.5, ..default() })
            .add_systems(Update, sfx_rules);
        let sound = || {
            (
                AudioPlayer(Handle::<AudioSource>::default()),
                PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.8)),
            )
        };
        app.world_mut().spawn(sound());
        app.world_mut().spawn(sound());
        app.update();
        let world = app.world_mut();
        let left: Vec<f32> = world
            .query::<&PlaybackSettings>()
            .iter(world)
            .map(|p| p.volume.to_linear())
            .collect();
        assert_eq!(left.len(), 1);
        assert!((left[0] - 0.4).abs() < 1e-4, "{left:?}");
    }

    #[test]
    fn steps_stay_in_range_and_toggle() {
        let mut s = Settings::default();
        assert!(s.step(Row::Effects, -1));
        assert_eq!(s.volume, 0.9);
        for _ in 0..20 {
            s.step(Row::Effects, 1);
        }
        assert_eq!(s.volume, 1.0);
        assert!(!s.step(Row::Effects, 1), "already at the top");
        for _ in 0..20 {
            s.step(Row::Music, -1);
        }
        assert_eq!(s.music, 0.0);
        // Full screen starts on, like the original, and toggles.
        assert!(s.step(Row::Fullscreen, 1));
        assert!(!s.fullscreen);
        assert!(!s.step(Row::Apply, 1), "Apply has nothing to change");
        // Choices wrap around.
        assert_eq!(s.resolution, (1920, 1080));
        s.step(Row::Resolution, 1);
        assert_eq!(s.resolution, (2560, 1440));
        s.step(Row::Resolution, 1);
        assert_eq!(s.resolution, (3440, 1440));
        s.step(Row::Resolution, 1);
        assert_eq!(s.resolution, RESOLUTIONS[0]);
        assert_eq!(s.quality, Quality::High);
        s.step(Row::Quality, -1);
        assert_eq!(s.quality, Quality::Medium);
    }

    #[test]
    fn clicks_on_a_slider_step_or_set_the_nearest_tenth() {
        let left = value_x(BAR_INSET);
        assert_eq!(click_steps(Row::Effects, value_x(VALUE_ARROW_INSET), 0.5), Some(-1));
        assert_eq!(click_steps(Row::Music, value_x(VALUE_WIDTH - VALUE_ARROW_INSET), 0.5), Some(1));
        assert_eq!(click_steps(Row::Effects, left + BAR.0 * 0.3, 1.0), Some(-7));
        assert_eq!(click_steps(Row::Effects, left + BAR.0, 0.2), Some(8));
        assert_eq!(click_steps(Row::Resolution, left + BAR.0 * 0.5, 0.0), None);
        assert_eq!(click_steps(Row::Fullscreen, value_x(VALUE_ARROW_INSET), 0.0), None);
    }

    #[test]
    fn a_cursor_over_a_settings_row_picks_it() {
        let window = Vec2::new(1920.0, 1080.0);
        for i in 0..6 {
            let at = Vec2::new(960.0, SCREEN.1 / 2.0 - row_y(i));
            assert_eq!(row_at(at, window, 6), Some(i), "row {i}");
        }
        assert_eq!(row_at(Vec2::new(10.0, 10.0), window, 6), None);
    }

    #[test]
    fn rows_sit_100_px_apart_around_the_area_centre() {
        // 6 rows centred on the screen: the original's rows run from 290 to 790 px down.
        assert_eq!(row_y(0), 250.0);
        assert_eq!(row_y(5), -250.0);
    }

    #[test]
    fn settings_survive_a_restart() {
        let file = std::env::temp_dir()
            .join(format!("tt-settings-{}", std::process::id()))
            .join("settings.ron");
        let mut s = Settings::load(file.clone());
        s.music = 0.3;
        s.vsync = true;
        s.save();
        let again = Settings::load(file.clone());
        assert_eq!((again.music, again.vsync), (0.3, true));
        let _ = std::fs::remove_dir_all(file.parent().unwrap());
    }
}
