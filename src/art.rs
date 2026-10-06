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

/// Folder holding the extracted files, as an absolute path so the game finds it
/// whatever folder it is started from. `TT_EXTRACTED` wins; otherwise the first
/// `extracted/` holding a `manifest.ron` in the working folder or next to the
/// executable or any folder above it (so `target/debug/game.exe` finds the one
/// beside `Cargo.toml`).
pub fn extracted_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("TT_EXTRACTED") {
        return absolute(PathBuf::from(dir));
    }
    let mut candidates = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("extracted"));
    }
    if let Ok(exe) = std::env::current_exe() {
        candidates.extend(exe.ancestors().skip(1).map(|d| d.join("extracted")));
    }
    candidates
        .iter()
        .find(|d| d.join("manifest.ron").is_file())
        .cloned()
        .unwrap_or_else(|| absolute(PathBuf::from("extracted")))
}

fn absolute(path: PathBuf) -> PathBuf {
    std::path::absolute(&path).unwrap_or(path)
}

/// Registers the `extracted://` asset source. Must run before `DefaultPlugins`.
pub fn register_source(app: &mut App) {
    let dir = extracted_dir();
    app.register_asset_source(
        SOURCE,
        AssetSourceBuilder::platform_default(&dir.to_string_lossy(), None),
    );
    // Art packs in mods load from `mods://<mod>/<file>`.
    let mods = absolute(crate::mods::mods_dir());
    app.register_asset_source(
        crate::mods::SOURCE,
        AssetSourceBuilder::platform_default(&mods.to_string_lossy(), None),
    );
}

/// One entry of `extracted/manifest.ron`.
#[derive(Deserialize, Clone, Debug)]
pub struct ArtEntry {
    /// Path inside the extracted folder. Left out for entries that are only
    /// numbers, such as `sky`.
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub pixels_per_unit: Option<f32>,
    /// Pivot in 0..1 from the bottom left; the centre if left out.
    #[serde(default)]
    pub pivot: Option<(f32, f32)>,
    /// Background layers only: where the pivot sits, in cells from the middle of
    /// the platform's top. A layer with a position is drawn once per tower lane;
    /// one without fills the whole view.
    #[serde(default)]
    pub position: Option<(f32, f32)>,
    /// Background layers only: draw order, back to front (the layer number if left out).
    #[serde(default)]
    pub z: Option<f32>,
    /// Background layers only: 1 moves with the tower (default), 0 stays on screen.
    #[serde(default)]
    pub parallax: Option<f32>,
    /// Extra size factor on top of `pixels_per_unit` (1 if left out).
    #[serde(default)]
    pub scale: Option<f32>,
    /// Background layers only: pin the layer to the screen instead of a lane. The
    /// point of the screen, 0 to 1 from its bottom left, that `position` counts from.
    #[serde(default)]
    pub screen: Option<(f32, f32)>,
    /// Screen layers only: how many units tall the screen is, for `position` and
    /// `pixels_per_unit` (10 if left out).
    #[serde(default)]
    pub view_height: Option<f32>,
    /// Colours, 0 to 255: the `sky` gradient top to bottom, or the tints the
    /// `spell_line` pieces cycle through.
    #[serde(default)]
    pub colors: Option<Vec<(f32, f32, f32)>>,
    /// How many pieces make up the `spell_line` (16 if left out).
    #[serde(default)]
    pub count: Option<u32>,
    /// For two-colour mask sprites the original recolours: the colours, 0 to 255,
    /// for the red areas and the yellow areas. Shades in between blend. Written as
    /// a list `[(r, g, b), (r, g, b)]` or a pair `((r, g, b), (r, g, b))`.
    #[serde(default)]
    pub mask: Option<Pair<(f32, f32, f32)>>,
    /// Mask sprites only: the blue outline's colour at the top and at the bottom,
    /// `(r, g, b, a)` 0 to 255. Transparent if left out, like the original's ruins.
    #[serde(default)]
    pub outline: Option<Pair<(f32, f32, f32, f32)>>,
    /// Screen layers only: slides sideways at a speed picked from this range, in
    /// screen units a second, wrapping round the view (Race Pro's clouds: 0.15-0.25).
    #[serde(default)]
    pub drift: Option<(f32, f32)>,
    /// Screen layers only: a brighter picture shown over this one as the lightning
    /// flashes (Race Pro's `_DARK` islands lit by the flash).
    #[serde(default)]
    pub lit: Option<String>,
    /// Screen layers only: a tint (r, g, b) 0 to 255 (Survival's large clouds).
    #[serde(default)]
    pub tint: Option<(f32, f32, f32)>,
    /// Screen layers only: a star that twinkles and now and then twirls.
    #[serde(default)]
    pub star: Option<bool>,
    /// An animation instead of a file: `(seconds, image name)` pairs in order, each
    /// image shown from its time until the next.
    #[serde(default)]
    pub frames: Option<Vec<(f32, String)>>,
    /// Animation length in seconds (the last frame's time plus the step before it,
    /// if left out).
    #[serde(default)]
    pub length: Option<f32>,
    /// Whether the animation repeats (true if left out); otherwise it holds the last frame.
    #[serde(default)]
    pub looping: Option<bool>,
    /// A character built from parts (`wizard<n>`): the part names, back to front.
    /// Each part is the entry `<this name>_<part>`, its clips `<this name>_<part>_<clip>`.
    #[serde(default)]
    pub parts: Option<Vec<String>>,
}

/// Two values, written as a two-item list or a pair.
#[derive(Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum Pair<T> {
    List(Vec<T>),
    Tuple((T, T)),
}

impl<T: Clone> Pair<T> {
    /// The two values, or `None` for a list that doesn't hold exactly two.
    pub fn get(&self) -> Option<(T, T)> {
        match self {
            Pair::List(v) if v.len() == 2 => Some((v[0].clone(), v[1].clone())),
            Pair::List(_) => None,
            Pair::Tuple(t) => Some(t.clone()),
        }
    }
}

impl ArtEntry {
    /// `colors` as Bevy colours.
    pub fn colors(&self) -> Vec<Color> {
        self.colors
            .iter()
            .flatten()
            .map(|&(r, g, b)| Color::srgb_u8(r as u8, g as u8, b as u8))
            .collect()
    }
}

/// Images and sounds found in the extracted folder, by logical name.
#[derive(Resource, Default)]
pub struct Art {
    pub images: HashMap<String, Handle<Image>>,
    pub sounds: HashMap<String, Handle<AudioSource>>,
    pub fonts: HashMap<String, Handle<Font>>,
    /// Fonts the original only has as a picture of its letters (`font_*` entries
    /// pointing at a `.ron` letter sheet).
    pub letters: HashMap<String, Letters>,
    pub entries: HashMap<String, ArtEntry>,
}

impl Art {
    /// An image by name; for an animation, its first frame.
    pub fn image(&self, name: &str) -> Option<Handle<Image>> {
        self.images
            .get(name)
            .cloned()
            .or_else(|| self.frame(name, 0.0))
    }

    /// Whether `name` is a staged animation.
    pub fn is_animation(&self, name: &str) -> bool {
        self.entries
            .get(name)
            .is_some_and(|e| e.frames.as_ref().is_some_and(|f| !f.is_empty()))
    }

    /// The frame of animation `name` to show `t` seconds after it started.
    pub fn frame(&self, name: &str, t: f32) -> Option<Handle<Image>> {
        let image = self.frame_name(name, t)?;
        self.images.get(image).cloned()
    }

    /// Seconds animation `name` lasts, if it is one.
    pub fn length(&self, name: &str) -> Option<f32> {
        let entry = self.entries.get(name)?;
        let frames = entry.frames.as_ref().filter(|f| !f.is_empty())?;
        let last = frames.last()?.0;
        let step = match frames.len() {
            1 => 0.1,
            n => (last - frames[n - 2].0).max(0.01),
        };
        Some(entry.length.unwrap_or(last + step).max(0.01))
    }

    /// The image name of the frame of animation `name` at `t` seconds.
    pub fn frame_name(&self, name: &str, t: f32) -> Option<&str> {
        let entry = self.entries.get(name)?;
        let frames = entry.frames.as_ref().filter(|f| !f.is_empty())?;
        let length = self.length(name)?;
        let t = if entry.looping.unwrap_or(true) {
            t.rem_euclid(length)
        } else {
            t
        };
        let (_, image) = frames
            .iter()
            .rev()
            .find(|(at, _)| *at <= t)
            .unwrap_or(&frames[0]);
        Some(image)
    }

    pub fn has_any(&self) -> bool {
        !self.images.is_empty() || !self.sounds.is_empty()
    }
}

/// A font drawn from a picture of its letters, as the original stores it. Every
/// number is in the picture's pixels.
#[derive(Deserialize, Clone, Debug)]
pub struct Letters {
    /// The letter picture, inside the extracted folder.
    pub image: String,
    /// The text size the numbers below are measured at.
    pub size: f32,
    /// Distance from one line's top to the next.
    pub line_height: f32,
    /// Distance from a line's top down to its base line.
    pub ascender: f32,
    pub glyphs: HashMap<char, Glyph>,
    /// The same letters grown into an outline, at the same glyph rectangles
    /// (padded by `outline_pad` pixels on every side).
    #[serde(default)]
    pub outline: Option<String>,
    #[serde(default)]
    pub outline_pad: f32,
    /// The loaded picture.
    #[serde(skip)]
    pub handle: Handle<Image>,
    #[serde(skip)]
    pub outline_handle: Option<Handle<Image>>,
}

/// One letter in a [`Letters`] picture.
#[derive(Deserialize, Clone, Copy, Debug)]
pub struct Glyph {
    /// `(x, y, width, height)` in the picture, from its top left.
    pub rect: (f32, f32, f32, f32),
    /// `(x, y)`: from the pen on the base line to the letter's top left, y up.
    pub offset: (f32, f32),
    /// How far the pen moves on after this letter.
    pub advance: f32,
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
            .add_systems(
                Update,
                (
                    drop_failed_art,
                    recolor_masks,
                    play_animations,
                    apply_font,
                    play_music,
                    play_sounds,
                ),
            );
    }
}

/// No sound at all: set by the offscreen mode, so a scripted run is silent.
#[derive(Resource, Default)]
pub struct Silent;

/// Loads one manifest's files, found in `dir` and read through `prefix`.
fn load_entries(
    art: &mut Art,
    assets: &AssetServer,
    silent: bool,
    dir: &Path,
    prefix: &str,
    entries: &HashMap<String, ArtEntry>,
) {
    for (name, entry) in entries {
        if entry.file.is_empty() {
            continue;
        }
        if !dir.join(&entry.file).is_file() {
            warn!("{name}: {} is missing", entry.file);
            continue;
        }
        let path = format!("{prefix}{}", entry.file);
        // The file type decides what it is; the name prefix covers odd extensions.
        let ext = Path::new(&entry.file)
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        // Text files (the original's UI wording, `text_<lang>`, and the credits list) are read where used.
        if ext == "txt" || ext == "json" {
            continue;
        }
        let is_font = matches!(ext.as_str(), "ttf" | "otf");
        let is_sound = matches!(ext.as_str(), "ogg" | "wav" | "mp3" | "flac")
            || name.starts_with("sfx_")
            || name.starts_with("music_");
        if ext == "ron" && name.starts_with("font_") {
            let sheet = std::fs::read_to_string(dir.join(&entry.file))
                .map_err(|e| e.to_string())
                .and_then(|text| ron::from_str::<Letters>(&text).map_err(|e| e.to_string()));
            match sheet {
                Ok(mut letters) if dir.join(&letters.image).is_file() => {
                    letters.handle = assets.load(format!("{prefix}{}", letters.image));
                    letters.outline_handle = letters
                        .outline
                        .as_ref()
                        .filter(|file| dir.join(file).is_file())
                        .map(|file| assets.load(format!("{prefix}{file}")));
                    art.letters.insert(name.clone(), letters);
                }
                Ok(letters) => warn!("{name}: {} is missing", letters.image),
                Err(err) => warn!("{name}: invalid letter sheet: {err}"),
            }
        } else if is_font {
            art.fonts.insert(name.clone(), assets.load(path));
        } else if is_sound {
            if silent {
                continue;
            }
            art.sounds.insert(name.clone(), assets.load(path));
        } else {
            art.images.insert(name.clone(), assets.load(path));
        }
    }
}

fn load_art(mut art: ResMut<Art>, assets: Res<AssetServer>, silent: Option<Res<Silent>>) {
    let dir = extracted_dir();
    let mut entries = read_manifest(&dir);
    load_entries(
        &mut art,
        &assets,
        silent.is_some(),
        &dir,
        &format!("{SOURCE}://"),
        &entries,
    );
    // Art packs from mods replace entries of the same name, in mod order.
    for m in crate::mods::list_mods(&crate::mods::mods_dir()) {
        let Some(pack) = &m.art else { continue };
        let pack_entries = read_manifest(pack);
        // An entry that becomes an animation or a set of numbers drops the old file.
        for (name, entry) in &pack_entries {
            if entry.file.is_empty() {
                art.images.remove(name);
                art.sounds.remove(name);
            }
        }
        let prefix = format!("{}://{}/", crate::mods::SOURCE, m.name);
        load_entries(
            &mut art,
            &assets,
            silent.is_some(),
            pack,
            &prefix,
            &pack_entries,
        );
        info!("mod {}: {} art entries", m.name, pack_entries.len());
        entries.extend(pack_entries);
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

/// Forgets images and sounds that failed to load, so whatever used them falls
/// back to plain shapes instead of drawing nothing.
fn drop_failed_art(mut art: ResMut<Art>, assets: Res<AssetServer>) {
    let failed = |id: bevy::asset::UntypedAssetId| {
        matches!(
            assets.get_load_state(id),
            Some(bevy::asset::LoadState::Failed(_))
        )
    };
    let bad_images: Vec<String> = art
        .images
        .iter()
        .filter(|(_, h)| failed(h.id().untyped()))
        .map(|(n, _)| n.clone())
        .collect();
    let bad_sounds: Vec<String> = art
        .sounds
        .iter()
        .filter(|(_, h)| failed(h.id().untyped()))
        .map(|(n, _)| n.clone())
        .collect();
    if bad_images.is_empty() && bad_sounds.is_empty() {
        return;
    }
    for name in bad_images {
        warn!("{name}: failed to load; drawing plain shapes instead");
        art.images.remove(&name);
    }
    for name in bad_sounds {
        warn!("{name}: failed to load");
        art.sounds.remove(&name);
    }
}

/// Recolours mask sprites once loaded, like the original's mask shader: red areas
/// take the first `mask` colour, yellow areas the second, and blue outlines the
/// `outline` colours graded from top to bottom (transparent if left out).
fn recolor_masks(
    mut art: ResMut<Art>,
    mut images: ResMut<Assets<Image>>,
    mut done: Local<std::collections::HashSet<String>>,
) {
    type Ready = (String, [Color; 2], [Color; 2], Handle<Image>);
    let ready: Vec<Ready> = art
        .entries
        .iter()
        .filter(|(name, _)| !done.contains(*name))
        .filter_map(|(name, entry)| {
            let (a, b) = entry.mask.as_ref()?.get()?;
            let handle = art.images.get(name)?.clone();
            images.get(&handle)?;
            let color = |(r, g, b): (f32, f32, f32)| Color::srgb_u8(r as u8, g as u8, b as u8);
            let rgba = |(r, g, b, a): (f32, f32, f32, f32)| {
                Color::srgba_u8(r as u8, g as u8, b as u8, a as u8)
            };
            let (top, bottom) = entry
                .outline
                .as_ref()
                .and_then(Pair::get)
                .map_or((Color::NONE, Color::NONE), |(t, b)| (rgba(t), rgba(b)));
            Some((name.clone(), [color(a), color(b)], [top, bottom], handle))
        })
        .collect();
    for (name, [red, yellow], [top, bottom], handle) in ready {
        done.insert(name.clone());
        let Some(mut image) = images.get(&handle).cloned() else {
            continue;
        };
        let size = image.size();
        for y in 0..size.y {
            let edge = top
                .mix(&bottom, y as f32 / (size.y.max(2) - 1) as f32)
                .to_srgba();
            for x in 0..size.x {
                let Ok(pixel) = image.get_color_at(x, y) else {
                    continue;
                };
                let p = pixel.to_srgba();
                let body = p.red.max(p.green);
                let line = p.blue;
                if body + line <= 0.0 {
                    continue;
                }
                let t = if body > 0.0 {
                    (p.green / body).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let fill = red.mix(&yellow, t).to_srgba();
                // Body and outline share the pixel by how much of each the mask has.
                let total = body + line;
                let mix = |f: f32, e: f32| (f * body + e * line) / total;
                let out = Color::srgba(
                    mix(fill.red * body, edge.red),
                    mix(fill.green * body, edge.green),
                    mix(fill.blue * body, edge.blue),
                    p.alpha * (body + line * edge.alpha) / total,
                );
                let _ = image.set_color_at(x, y, out);
            }
        }
        let recolored = images.add(image);
        art.images.insert(name, recolored);
    }
}

/// Plays animation `name` (a manifest entry with `frames`) on this entity's
/// `Sprite` or `ImageNode`.
#[derive(Component, Debug, Clone)]
pub struct Animated {
    pub name: String,
    pub time: f32,
}

impl Animated {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            time: 0.0,
        }
    }
}

fn play_animations(
    time: Res<Time>,
    art: Res<Art>,
    mut sprites: Query<(&mut Animated, Option<&mut Sprite>, Option<&mut ImageNode>)>,
) {
    for (mut anim, sprite, node) in &mut sprites {
        anim.time += time.delta_secs();
        let Some(frame) = art.frame(&anim.name, anim.time) else {
            continue;
        };
        if let Some(mut sprite) = sprite
            && sprite.image != frame
        {
            sprite.image = frame.clone();
        }
        if let Some(mut node) = node
            && node.image != frame
        {
            node.image = frame;
        }
    }
}

/// Sets every text to the original's font (`font_main`) when one is staged.
fn apply_font(art: Res<Art>, mut texts: Query<&mut TextFont>) {
    let Some(font) = art.fonts.get("font_main") else {
        return;
    };
    let all = art.is_changed();
    for mut text in &mut texts {
        let current = matches!(&text.font, FontSource::Handle(h) if h == font);
        if (all || text.is_added()) && !current {
            text.font = font.clone().into();
        }
    }
}

#[derive(Component)]
pub(crate) struct Music(String);

/// Loops the music for the menu or the current mode. A match started from Select
/// Mode, the Endless screen or the Trials list opens with its mode's start stinger
/// (`<MODE>_SFX_START_STINGER`, once, at the music volume).
#[allow(clippy::too_many_arguments)]
fn play_music(
    mut commands: Commands,
    art: Res<Art>,
    game: Res<Match>,
    playing: Query<(Entity, &Music)>,
    settings: Option<Res<crate::settings::Settings>>,
    screen: Option<Res<crate::menu::MenuScreen>>,
    mut last: Local<(u64, Option<crate::menu::MenuScreen>)>,
) {
    use crate::menu::MenuScreen;
    let volume = settings.as_ref().map_or(1.0, |s| s.music);
    if game.phase == Phase::Menu {
        last.1 = screen.as_deref().copied();
    }
    if game.round != last.0 {
        last.0 = game.round;
        let from_select = matches!(
            last.1,
            Some(MenuScreen::Modes | MenuScreen::Endless | MenuScreen::Trials)
        );
        let stinger = match game.mode.look() {
            ModeKind::Survival => "music_track_02_spooky_stinger",
            ModeKind::Puzzle => "music_track_03_puzzle_stinger",
            _ => "music_track_01_race_stinger",
        };
        if from_select && let Some(handle) = art.sounds.get(stinger) {
            commands.spawn((
                AudioPlayer(handle.clone()),
                PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(volume)),
            ));
        }
    }
    // The explanation before a match keeps whatever is playing.
    if game.phase == Phase::Menu && screen.as_deref() == Some(&MenuScreen::Explain) {
        return;
    }
    let want = match (game.phase, game.mode.look()) {
        (Phase::Menu, _) => "music_menu",
        (_, ModeKind::Race) => "music_race",
        (_, ModeKind::Survival) => "music_survival",
        (_, ModeKind::Puzzle) => "music_puzzle",
        (_, _) => unreachable!("Endless modes look like their battle mode"),
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
            PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::Linear(volume)),
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
    bindings: Option<Res<crate::keybinding::KeyBindings>>,
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
            use crate::keybinding::Action;
            let key = |action| {
                bindings
                    .as_ref()
                    .map_or(Action::default_key(action), |b| b.key(action))
            };
            let any = |actions: &[Action]| actions.iter().any(|a| keys.just_pressed(key(*a)));
            if any(&[Action::NudgeLeft, Action::NudgeRight]) {
                play("sfx_nudge");
            } else if any(&[Action::MoveLeft, Action::MoveRight]) {
                play("sfx_move");
            }
            if any(&[Action::TurnRight, Action::TurnLeft]) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_reads_layer_fields() {
        let dir = std::env::temp_dir().join(format!("tt_manifest_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("manifest.ron"),
            r#"{
                "background_0": (file: "bg/sky.png"),
                "background_1": (file: "bg/hill.png", pixels_per_unit: 50.0, pivot: (0.5, 1.0),
                    position: (0, -1), z: 3, parallax: 0.5, scale: 2.0),
            }"#,
        )
        .unwrap();
        let entries = read_manifest(&dir);
        std::fs::remove_dir_all(&dir).ok();
        assert!(entries["background_0"].position.is_none());
        let hill = &entries["background_1"];
        assert_eq!(hill.position, Some((0.0, -1.0)));
        assert_eq!(hill.z, Some(3.0));
        assert_eq!(hill.parallax, Some(0.5));
        assert_eq!(hill.scale, Some(2.0));
    }

    #[test]
    fn animation_frames_follow_their_times() {
        let handle = |n: u128| {
            Handle::<Image>::Uuid(bevy::asset::uuid::Uuid::from_u128(n), Default::default())
        };
        let mut art = Art::default();
        for (n, name) in ["a", "b", "c"].iter().enumerate() {
            art.images.insert(name.to_string(), handle(n as u128 + 1));
        }
        let entry = |looping| ArtEntry {
            frames: Some(vec![
                (0.0, "a".into()),
                (0.2, "b".into()),
                (0.43, "c".into()),
            ]),
            looping,
            ..read_entry("(file: \"\")")
        };
        art.entries.insert("wave".into(), entry(None));
        art.entries.insert("once".into(), entry(Some(false)));
        assert_eq!(art.frame("wave", 0.1), Some(handle(1)));
        assert_eq!(art.frame("wave", 0.3), Some(handle(2)));
        assert_eq!(art.frame("wave", 0.5), Some(handle(3)));
        // Length is 0.43 + 0.23, so 0.7 wraps to the first frame.
        assert_eq!(art.frame("wave", 0.7), Some(handle(1)));
        assert_eq!(art.frame("once", 5.0), Some(handle(3)));
        assert_eq!(art.image("wave"), Some(handle(1)));
    }

    fn read_entry(text: &str) -> ArtEntry {
        ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(text)
            .unwrap()
    }

    #[test]
    fn mask_reads_as_list_or_pair() {
        let list = read_entry(r#"(file: "a.png", mask: [(1, 2, 3), (4, 5, 6)])"#);
        let pair = read_entry(r#"(file: "a.png", mask: ((1, 2, 3), (4, 5, 6)))"#);
        let want = Some(((1.0, 2.0, 3.0), (4.0, 5.0, 6.0)));
        assert_eq!(list.mask.unwrap().get(), want);
        assert_eq!(pair.mask.unwrap().get(), want);
        let outline = read_entry(r#"(file: "a.png", outline: [(1, 2, 3, 0), (4, 5, 6, 255)])"#);
        assert!(outline.outline.unwrap().get().is_some());
    }
}
