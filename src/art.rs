//! Original art and sound, loaded at runtime from the player's own copy of the
//! game. Nothing from the original is in this repo: a local extract step copies
//! the files into the gitignored `extracted/` folder and writes
//! `extracted/manifest.ron` (see `docs/art-layout.md`). Without it the game
//! draws plain shapes and stays silent.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use bevy::asset::io::AssetSourceBuilder;
use bevy::prelude::*;
use serde::Deserialize;

use crate::config::ModeKind;
use crate::gameplay::{ActiveBrick, Match, Phase, Tower, TowerState};
use crate::spells::SpellLog;

/// Asset source name for the extracted folder: `extracted://bricks/I.png`.
pub const SOURCE: &str = "extracted";

/// Folder holding the extracted files; `TT_EXTRACTED` overrides it.
pub fn extracted_dir() -> PathBuf {
    std::env::var_os("TT_EXTRACTED")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("extracted"))
}

/// Registers the `extracted://` asset source. Must run before `DefaultPlugins`.
pub fn register_source(app: &mut App) {
    let dir = extracted_dir();
    app.register_asset_source(
        SOURCE,
        AssetSourceBuilder::platform_default(&dir.to_string_lossy(), None),
    );
}

/// One entry of `extracted/manifest.ron`.
#[derive(Deserialize, Clone, Debug)]
pub struct ArtEntry {
    /// Path inside the extracted folder.
    pub file: String,
    #[serde(default)]
    pub pixels_per_unit: Option<f32>,
    /// Pivot in 0..1 from the bottom left; the centre if left out.
    #[serde(default)]
    pub pivot: Option<(f32, f32)>,
}

/// Images and sounds found in the extracted folder, by logical name.
#[derive(Resource, Default)]
pub struct Art {
    pub images: HashMap<String, Handle<Image>>,
    pub sounds: HashMap<String, Handle<AudioSource>>,
    pub entries: HashMap<String, ArtEntry>,
}

impl Art {
    pub fn image(&self, name: &str) -> Option<Handle<Image>> {
        self.images.get(name).cloned()
    }

    pub fn has_any(&self) -> bool {
        !self.images.is_empty() || !self.sounds.is_empty()
    }
}

/// Reads the manifest. Missing or broken manifests leave the game on plain shapes.
pub fn read_manifest(dir: &Path) -> HashMap<String, ArtEntry> {
    let path = dir.join("manifest.ron");
    let Ok(text) = std::fs::read_to_string(&path) else {
        info!("no {}; using plain shapes", path.display());
        return HashMap::new();
    };
    // Optional fields can be written plainly: `pivot: (0.5, 1.0)`.
    let options =
        ron::Options::default().with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME);
    match options.from_str(&text) {
        Ok(entries) => entries,
        Err(err) => {
            error!("invalid {}: {err}; using plain shapes", path.display());
            HashMap::new()
        }
    }
}

pub struct ArtPlugin;

impl Plugin for ArtPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Art>()
            .add_systems(PreStartup, load_art)
            .add_systems(Update, (play_music, play_sounds));
    }
}

fn load_art(mut art: ResMut<Art>, assets: Res<AssetServer>) {
    let dir = extracted_dir();
    let entries = read_manifest(&dir);
    for (name, entry) in &entries {
        if !dir.join(&entry.file).is_file() {
            warn!("{name}: {} is missing", entry.file);
            continue;
        }
        let path = format!("{SOURCE}://{}", entry.file);
        let is_sound = name.starts_with("sfx_") || name.starts_with("music_");
        if is_sound {
            art.sounds.insert(name.clone(), assets.load(path));
        } else {
            art.images.insert(name.clone(), assets.load(path));
        }
    }
    art.entries = entries;
    if art.has_any() {
        info!(
            "loaded {} images and {} sounds from {}",
            art.images.len(),
            art.sounds.len(),
            dir.display()
        );
    }
}

#[derive(Component)]
struct Music(String);

/// Loops the music for the menu or the current mode.
fn play_music(
    mut commands: Commands,
    art: Res<Art>,
    game: Res<Match>,
    playing: Query<(Entity, &Music)>,
) {
    let want = match (game.phase, game.mode) {
        (Phase::Menu, _) => "music_menu",
        (_, ModeKind::Race) => "music_race",
        (_, ModeKind::Survival) => "music_survival",
        (_, ModeKind::Puzzle) => "music_puzzle",
    };
    if playing.iter().any(|(_, m)| m.0 == want) {
        return;
    }
    for (entity, _) in &playing {
        commands.entity(entity).despawn();
    }
    if let Some(handle) = art.sounds.get(want) {
        commands.spawn((
            Music(want.into()),
            AudioPlayer(handle.clone()),
            PlaybackSettings::LOOP,
        ));
    }
}

/// What your tower looked like last frame, to notice changes worth a sound.
#[derive(Default)]
struct Heard {
    placed: u32,
    dropped: u32,
    lines: u32,
    casts: u64,
    countdown: u32,
    phase: Option<Phase>,
}

#[allow(clippy::too_many_arguments)]
fn play_sounds(
    mut commands: Commands,
    art: Res<Art>,
    game: Res<Match>,
    log: Res<SpellLog>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    towers: Query<(&Tower, &TowerState)>,
    active: Query<(), With<ActiveBrick>>,
    mut heard: Local<Heard>,
) {
    if art.sounds.is_empty() {
        return;
    }
    let mut play = |name: &str| {
        if let Some(handle) = art.sounds.get(name) {
            commands.spawn((AudioPlayer(handle.clone()), PlaybackSettings::DESPAWN));
        }
    };
    let Some((_, you)) = towers.iter().find(|(t, _)| t.is_first_player()) else {
        return;
    };
    if game.phase == Phase::Playing {
        if you.bricks_placed > heard.placed {
            play("sfx_land");
        }
        if you.bricks_dropped > heard.dropped {
            play("sfx_drop_lost");
        }
        if you.lines_reached > heard.lines {
            play("sfx_line_collect");
        }
        let countdown = you.hold_timer.ceil() as u32;
        if countdown > heard.countdown {
            play("sfx_countdown");
        }
        heard.countdown = countdown;
        let steering = you.active.is_some_and(|e| active.contains(e));
        if let Some(keys) = keys.filter(|_| steering) {
            let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
            let sideways = keys.any_just_pressed([
                KeyCode::ArrowLeft,
                KeyCode::ArrowRight,
                KeyCode::KeyA,
                KeyCode::KeyD,
            ]);
            if sideways {
                play(if shift { "sfx_nudge" } else { "sfx_move" });
            }
            if keys.any_just_pressed([
                KeyCode::ArrowUp,
                KeyCode::KeyW,
                KeyCode::Space,
                KeyCode::KeyZ,
            ]) {
                play("sfx_rotate");
            }
        }
    }
    for cast in log
        .0
        .iter()
        .rev()
        .take((log.1 - heard.casts.min(log.1)) as usize)
    {
        play(if cast.spell.is_dark() {
            "sfx_cast_dark"
        } else {
            "sfx_cast_light"
        });
    }
    if game.phase == Phase::Over && heard.phase == Some(Phase::Playing) {
        play(if game.winner == Some(0) {
            "sfx_win"
        } else {
            "sfx_lose"
        });
    }
    heard.placed = you.bricks_placed;
    heard.dropped = you.bricks_dropped;
    heard.lines = you.lines_reached;
    heard.casts = log.1;
    heard.phase = Some(game.phase);
}
