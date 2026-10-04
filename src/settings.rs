//! The Settings screen from the title tiles: volume, music volume and full
//! screen, applied at once and kept between runs. The original's own list of
//! options and its layout aren't read yet (?), so this is a plain list.

use std::path::PathBuf;

use bevy::audio::{AudioSink, AudioSinkPlayback, GlobalVolume, Volume};
use bevy::prelude::*;
use bevy::window::{MonitorSelection, PrimaryWindow, WindowMode};
use serde::{Deserialize, Serialize};

use crate::art::Music;
use crate::gameplay::{Match, Phase};
use crate::menu::MenuScreen;
use crate::select::{label, place, read_taps, shown, spawn_page};

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Settings>()
            .init_resource::<SettingsMenu>()
            .add_systems(Startup, spawn_settings_screen)
            .add_systems(Update, apply_settings);
    }
}

/// The player's settings. Kept in `file` when one is set (the game sets it;
/// tests and offscreen pictures don't).
#[derive(Resource, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Every sound, 0 to 1.
    pub volume: f32,
    /// Music on top of that, 0 to 1.
    pub music: f32,
    pub fullscreen: bool,
    #[serde(skip)]
    pub file: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            volume: 1.0,
            music: 0.7,
            fullscreen: false,
            file: None,
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

    /// Moves the setting on `row` one step left (-1) or right (+1). Returns true
    /// if anything changed.
    pub fn step(&mut self, row: usize, dir: i32) -> bool {
        let before = self.clone();
        let nudge = |v: f32| ((v + 0.1 * dir as f32) * 10.0).round().clamp(0.0, 10.0) / 10.0;
        match ROWS.get(row) {
            Some(Row::Volume) => self.volume = nudge(self.volume),
            Some(Row::Music) => self.music = nudge(self.music),
            Some(Row::Fullscreen) => self.fullscreen = !self.fullscreen,
            _ => {}
        }
        *self != before
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Row {
    Volume,
    Music,
    Fullscreen,
    Back,
}

const ROWS: [Row; 4] = [Row::Volume, Row::Music, Row::Fullscreen, Row::Back];
const ROW: (f32, f32) = (800.0, 70.0);
const ROW_TOP: f32 = 180.0;
const ROW_STEP: f32 = 90.0;

#[derive(Resource, Debug, Default)]
pub struct SettingsMenu {
    pub selected: usize,
}

#[derive(Component)]
pub(crate) struct SettingsRoot;
#[derive(Component)]
pub(crate) struct SettingsRow(usize);
#[derive(Component)]
pub(crate) struct SettingsText(usize);

fn spawn_settings_screen(mut commands: Commands, art: Option<Res<crate::art::Art>>) {
    let page = spawn_page(&mut commands, art.as_deref(), "Settings");
    commands
        .entity(page)
        .insert(SettingsRoot)
        .with_children(|page| {
            for (i, _) in ROWS.iter().enumerate() {
                page.spawn((
                    place(ROW, (0.0, ROW_TOP - ROW_STEP * i as f32)),
                    BackgroundColor(Color::srgba(0.3, 0.24, 0.55, 0.8)),
                    SettingsRow(i),
                ))
                .with_children(|row| {
                    row.spawn((label(3.4, ""), SettingsText(i)));
                });
            }
        });
}

/// Up/Down pick a row, Left/Right change it, Enter toggles or goes back, Esc
/// or B goes back to the title tiles.
pub(crate) fn settings_input(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    pads: Query<(Entity, &Gamepad)>,
    game: Res<Match>,
    mut screen: ResMut<MenuScreen>,
    mut menu: ResMut<SettingsMenu>,
    mut settings: ResMut<Settings>,
) {
    if game.phase != Phase::Menu || *screen != MenuScreen::Settings {
        return;
    }
    let taps = read_taps(keys.as_deref(), &pads);
    if !taps.pad_back.is_empty() {
        *screen = MenuScreen::Main;
        return;
    }
    let count = ROWS.len();
    if taps.up {
        menu.selected = (menu.selected + count - 1) % count;
    }
    if taps.down {
        menu.selected = (menu.selected + 1) % count;
    }
    let pick = taps.key_pick || !taps.pad_pick.is_empty();
    let dir = taps.right as i32 - taps.left as i32;
    let changed = match ROWS[menu.selected] {
        Row::Back if pick => {
            *screen = MenuScreen::Main;
            false
        }
        Row::Fullscreen if pick || dir != 0 => settings.step(menu.selected, 1),
        _ if dir != 0 => settings.step(menu.selected, dir),
        _ => false,
    };
    if changed {
        settings.save();
    }
}

#[allow(clippy::type_complexity)]
pub(crate) fn show_settings_screen(
    game: Res<Match>,
    screen: Res<MenuScreen>,
    menu: Res<SettingsMenu>,
    settings: Res<Settings>,
    mut root: Query<&mut Visibility, With<SettingsRoot>>,
    mut rows: Query<(&SettingsRow, &mut BackgroundColor)>,
    mut texts: Query<(&SettingsText, &mut Text)>,
) {
    let on = game.phase == Phase::Menu && *screen == MenuScreen::Settings;
    for mut v in &mut root {
        v.set_if_neq(shown(on));
    }
    if !on {
        return;
    }
    for (row, mut color) in &mut rows {
        color.0 = if row.0 == menu.selected {
            Color::srgb(0.55, 0.42, 0.9)
        } else {
            Color::srgba(0.3, 0.24, 0.55, 0.8)
        };
    }
    let percent = |v: f32| format!("< {:.0}% >", v * 100.0);
    for (text, mut words) in &mut texts {
        let line = match ROWS[text.0] {
            Row::Volume => format!("Volume   {}", percent(settings.volume)),
            Row::Music => format!("Music   {}", percent(settings.music)),
            Row::Fullscreen => format!(
                "Full screen   {}",
                if settings.fullscreen { "On" } else { "Off" }
            ),
            Row::Back => "Back".to_string(),
        };
        if words.0 != line {
            words.0 = line;
        }
    }
}

/// Puts changed settings into effect: the global volume for new sounds, the
/// music already playing, and the window mode.
fn apply_settings(
    settings: Res<Settings>,
    global: Option<ResMut<GlobalVolume>>,
    mut music: Query<&mut AudioSink, With<Music>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if !settings.is_changed() {
        return;
    }
    if let Some(mut global) = global {
        global.volume = Volume::Linear(settings.volume);
    }
    for mut sink in &mut music {
        sink.set_volume(Volume::Linear(settings.volume * settings.music));
    }
    let mode = if settings.fullscreen {
        WindowMode::BorderlessFullscreen(MonitorSelection::Current)
    } else {
        WindowMode::Windowed
    };
    for mut window in &mut windows {
        if window.mode != mode {
            window.mode = mode;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_stay_in_range_and_toggle() {
        let mut s = Settings::default();
        assert!(s.step(0, -1));
        assert_eq!(s.volume, 0.9);
        for _ in 0..20 {
            s.step(0, 1);
        }
        assert_eq!(s.volume, 1.0);
        assert!(!s.step(0, 1), "already at the top");
        for _ in 0..20 {
            s.step(1, -1);
        }
        assert_eq!(s.music, 0.0);
        assert!(s.step(2, 1));
        assert!(s.fullscreen);
        assert!(!s.step(3, 1), "Back has nothing to change");
    }

    #[test]
    fn settings_survive_a_restart() {
        let file = std::env::temp_dir()
            .join(format!("tt-settings-{}", std::process::id()))
            .join("settings.ron");
        let mut s = Settings::load(file.clone());
        s.music = 0.3;
        s.fullscreen = true;
        s.save();
        let again = Settings::load(file.clone());
        assert_eq!((again.music, again.fullscreen), (0.3, true));
        let _ = std::fs::remove_dir_all(file.parent().unwrap());
    }
}
