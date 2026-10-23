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
    /// Brick-pack bricks: the entry of the bevel map this colour pattern is lit by
    /// (World/Bricks/Default: red/green the bevel's slope, blue the outline and cracks).
    #[serde(default)]
    pub bevel: Option<String>,
    /// With `bevel`: the highlight and shadow colours `(r, g, b, a)`, 0 to 1, and the
    /// light's angle in degrees (the brick material's `_HighlightColor`, `_ShadowColor`
    /// and `_Angle`).
    #[serde(default)]
    pub light: Option<Pair<(f32, f32, f32, f32)>>,
    #[serde(default)]
    pub light_angle: Option<f32>,
    /// Galaxy bricks: the outline colour (the brick material's `_BorderColor`), 0 to 1.
    #[serde(default)]
    pub line: Option<(f32, f32, f32)>,
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
    /// Screen layers only: opacity 0 to 1 (1 if left out), with or without a `tint`.
    #[serde(default)]
    pub opacity: Option<f32>,
    /// Screen layers only: stretched across the view's width (the reflected band).
    #[serde(default)]
    pub stretch: Option<bool>,
    /// Screen layers only: the Survival reflected band, tinted by the mode's overlay
    /// colour, which a curse wave moves to `weather.survival_wave_band`.
    #[serde(default)]
    pub overlay: Option<bool>,
    /// Screen layers only: part of Survival's starry sky (the moon and stars), which
    /// curse waves leave alone; `true` for a star that twinkles and now and then twirls.
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

/// DEBUG-TOOL: names looked up this run, written by `--log-assets`.
pub static USED: std::sync::Mutex<Option<std::collections::HashSet<String>>> =
    std::sync::Mutex::new(None);

pub(crate) fn note_used(name: &str) {
    if let Ok(mut used) = USED.lock()
        && let Some(set) = used.as_mut()
        && !set.contains(name)
    {
        set.insert(name.to_string());
    }
}

/// A name-to-handle map that notes every name the game asks for (DEBUG-TOOL, for the asset
/// audit); `raw()` reads without noting, for the loader's own passes.
pub struct Lookup<T>(HashMap<String, T>);

impl<T> Default for Lookup<T> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl<T> std::ops::Deref for Lookup<T> {
    type Target = HashMap<String, T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Lookup<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Lookup<T> {
    pub fn get(&self, name: &str) -> Option<&T> {
        note_used(name);
        self.0.get(name)
    }
    pub fn contains_key(&self, name: &str) -> bool {
        note_used(name);
        self.0.contains_key(name)
    }
    pub fn raw(&self) -> &HashMap<String, T> {
        &self.0
    }
}

impl<T> std::ops::Index<&str> for Lookup<T> {
    type Output = T;
    fn index(&self, name: &str) -> &T {
        note_used(name);
        &self.0[name]
    }
}

/// Images and sounds found in the extracted folder, by logical name.
#[derive(Resource, Default)]
pub struct Art {
    pub images: Lookup<Handle<Image>>,
    pub sounds: Lookup<Handle<AudioSource>>,
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
    /// The original's distance field for the same rectangles (in the alpha), so the letters can
    /// be drawn sharp at any size; the face's edge sits at 0.5 of it, the outline's at 0.325.
    #[serde(default)]
    pub sdf: Option<String>,
    #[serde(skip)]
    pub sdf_handle: Option<Handle<Image>>,
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
            .init_resource::<MatchEnd>()
            .add_systems(PreStartup, load_art)
            .add_systems(
                Update,
                (
                    drop_failed_art,
                    recolor_masks,
                    light_bricks,
                    play_animations,
                    apply_font,
                    (match_end_stingers, play_music, duck_music).chain(),
                    play_sounds,
                    menu_cursor_sound,
                    auto_rotate_sounds,
                    stone_and_ice_sounds,
                    ice_hit_sounds,
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
                    letters.sdf_handle = letters
                        .sdf
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
            let handle = art.images.raw().get(name)?.clone();
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

/// The bricks' outline and crack colour, fixed in the original's lit brick shaders (default,
/// Christmas, Halloween): (19, 13, 40).
/// Bricks whose material has no light: Candy's (World/Bricks/Default with _BorderColor,
/// disassembled: the pattern lerped to the border colour by the outline, plus the flash).
pub(crate) fn unlit_brick(name: &str) -> bool {
    name.starts_with("brick_candy_")
}

pub(crate) const BRICK_LINE: (f32, f32, f32) = (0.074_509_81, 0.050_980_39, 0.156_862_75);

/// The light's direction in the original's lit brick shaders, up and to the left. The
/// material's `_Angle` isn't used by them.
const BRICK_LIGHT: (f32, f32) = (-0.447_213_6, 0.894_427_2);

/// Lights brick-pack bricks once their pattern and bevel map are loaded, as the original's
/// lit brick shader does (World/Bricks/Default, read from its disassembly): the bevel map's red
/// and green are the slope, lit from up and to the left; the colour takes the highlight by the
/// slope towards the light times its alpha, the shadow likewise away from it, then the outline
/// colour by 1 - (1 - blue) x alpha of the map. Alpha is the larger of the pattern's alpha and
/// the map's blue. The baked picture is for what never turns (menus, the next-brick card); bricks
/// in play are drawn by `brick_material::LitBrick` from the unlit pattern, kept as `<name>_raw`,
/// so their light stays fixed in the world like the original's.
fn light_bricks(
    mut art: ResMut<Art>,
    mut images: ResMut<Assets<Image>>,
    mut done: Local<std::collections::HashSet<String>>,
) {
    type Ready = (String, Handle<Image>, Handle<Image>, [Srgba; 2], (f32, f32, f32));
    let ready: Vec<Ready> = art
        .entries
        .iter()
        .filter(|(name, _)| !done.contains(*name))
        .filter_map(|(name, entry)| {
            let ((hr, hg, hb, ha), (sr, sg, sb, sa)) = entry.light.as_ref()?.get()?;
            let pattern = art.images.raw().get(name)?.clone();
            let bevel = art.images.raw().get(entry.bevel.as_ref()?)?.clone();
            images.get(&pattern)?;
            images.get(&bevel)?;
            // Candy's World/Bricks/Default (the variant with _BorderColor) draws the pattern and
            // its outline only: no highlight or shadow.
            let shade = if unlit_brick(name) { 0.0 } else { 1.0 };
            let light = [Srgba::new(hr, hg, hb, ha * shade), Srgba::new(sr, sg, sb, sa * shade)];
            let line = entry.line.unwrap_or(BRICK_LINE);
            Some((name.clone(), pattern, bevel, light, line))
        })
        .collect();
    for (name, pattern, bevel, [high, shadow], line) in ready {
        done.insert(name.clone());
        let (Some(pattern), Some(mut out)) =
            (images.get(&pattern).cloned(), images.get(&bevel).cloned())
        else {
            continue;
        };
        let size = out.size();
        // The pattern is stretched over the brick like the original's UVs.
        let scale = pattern.size().as_vec2() / size.as_vec2();
        for y in 0..size.y {
            for x in 0..size.x {
                let (Ok(map), Ok(base)) = (
                    out.get_color_at(x, y),
                    pattern.get_color_at((x as f32 * scale.x) as u32, (y as f32 * scale.y) as u32),
                ) else {
                    continue;
                };
                let (map, base) = (map.to_srgba(), base.to_srgba());
                let slope = (map.red * 2.0 - 1.0) * BRICK_LIGHT.0 + (map.green * 2.0 - 1.0) * BRICK_LIGHT.1;
                let (to, by) = if slope > 0.0 {
                    (high, slope)
                } else {
                    (shadow, -slope)
                };
                let outline = 1.0 - (1.0 - map.blue) * map.alpha;
                let lit = |c: f32, t: f32, line: f32| {
                    let c = c + (t - c) * by.min(1.0) * to.alpha;
                    c + (line - c) * outline
                };
                let color = Color::srgba(
                    lit(base.red, to.red, line.0).clamp(0.0, 1.0),
                    lit(base.green, to.green, line.1).clamp(0.0, 1.0),
                    lit(base.blue, to.blue, line.2).clamp(0.0, 1.0),
                    base.alpha.max(map.blue),
                );
                let _ = out.set_color_at(x, y, color);
            }
        }
        let lit = images.add(out);
        if let Some(raw) = art.images.insert(name.clone(), lit) {
            art.images.insert(format!("{name}_raw"), raw);
        }
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

/// A looping ambience (the original's AMBIENT channel): the menu's, or the mode's
/// for the whole match.
#[derive(Component)]
pub(crate) struct Ambience(&'static str);

/// A stinger on the music channel (MusicPlayEffect with no type): the music volume, not
/// paused with the sound effects.
#[derive(Component)]
pub(crate) struct Stinger;

/// How a match's music ends (the original's effect triggers and bask controllers): the
/// roof coming stops the music for SFX_FINISH_TOWER (ROOF_SPAWNED); the roof set down
/// (ROOF_PLACED), a Puzzle battle's winner, or a trial or Endless run's end plays the win
/// stinger and the win loop 1.88 s later; a lost trial plays the lose stinger and its loop
/// 3.3 s later (SinglePlayerBaskController).
#[derive(Resource, Default)]
pub(crate) struct MatchEnd {
    round: u64,
    finish_tower: bool,
    /// Seconds (Time's elapsed) the stinger started, and whether it is the win one.
    stinger: Option<(f32, bool)>,
}

fn match_end_stingers(
    mut commands: Commands,
    time: Res<Time>,
    art: Res<Art>,
    game: Res<Match>,
    config: Res<crate::config::GameConfig>,
    towers: Query<(&Tower, &crate::gameplay::TowerState)>,
    mut roofs: MessageReader<crate::gameplay::RoofPlaced>,
    mut end: ResMut<MatchEnd>,
    settings: Option<Res<crate::settings::Settings>>,
) {
    if end.round != game.round {
        *end = MatchEnd {
            round: game.round,
            ..default()
        };
    }
    let mut play = |name: &str| {
        if let Some(handle) = art.sounds.get(name) {
            commands.spawn((AudioPlayer(handle.clone()), PlaybackSettings::DESPAWN));
        }
    };
    let roofing = towers
        .iter()
        .any(|(_, s)| s.roof == crate::gameplay::RoofStage::Steering);
    if roofing && !end.finish_tower {
        end.finish_tower = true;
        play("sfx_finish_tower");
    }
    let now = time.elapsed_secs();
    let roofed = roofs.read().count() > 0;
    if end.stinger.is_some() || game.phase != Phase::Over {
        return;
    }
    let single = config.trial.is_some() || game.mode.is_endless();
    let win = if roofed {
        Some(true)
    } else if single && !end.finish_tower {
        Some(towers.iter().any(|(t, s)| t.is_first_player() && s.status == crate::gameplay::TowerStatus::Won))
    } else if game.mode.rules() == ModeKind::Puzzle && game.winner.is_some() {
        Some(true)
    } else {
        None
    };
    if let Some(win) = win {
        // STINGER_WIN / STINGER_LOSE are MusicPlayEffects: the music volume.
        if let Some(handle) = art.sounds.get(if win { "sfx_win" } else { "sfx_lose" }) {
            let volume = settings.as_ref().map_or(1.0, |s| s.music);
            commands.spawn((
                Stinger,
                AudioPlayer(handle.clone()),
                PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(volume)),
            ));
        }
        end.stinger = Some((now, win));
    }
}

/// Loops the music for the menu or the current mode. A match started from Select
/// Mode, the Endless screen or the Trials list opens with its mode's start stinger
/// (`<MODE>_SFX_START_STINGER`, once, at the music volume).
#[allow(clippy::too_many_arguments)]
fn play_music(
    mut commands: Commands,
    art: Res<Art>,
    game: Res<Match>,
    config: Res<crate::config::GameConfig>,
    playing: Query<(Entity, &Music)>,
    ambience: Query<(Entity, &Ambience)>,
    towers: Query<&Tower>,
    settings: Option<Res<crate::settings::Settings>>,
    screen: Option<Res<crate::menu::MenuScreen>>,
    popup: Option<Res<crate::popup::Popup>>,
    cup: Option<Res<crate::cups::Cup>>,
    mut last: Local<(u64, Option<crate::menu::MenuScreen>)>,
    end: Res<MatchEnd>,
    time: Res<Time>,
) {
    use crate::menu::MenuScreen;
    // The cup winner popup stops the ambience and rolls a drum until its trophy is in,
    // then plays the win music (TournamentWinnerPopup).
    let cup_win = popup.as_ref().and_then(|p| p.winner_since()).map(|since| {
        let trophy_in = cup.as_ref().map_or(0.0, |c| crate::cups_menu::winner_trophy_in(c, &config));
        if since < trophy_in { "sfx_drum_roll" } else { "music_win_game_loop" }
    });
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
                Stinger,
                AudioPlayer(handle.clone()),
                PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(volume)),
            ));
        }
    }
    // The explanation before a match and the pause menu keep whatever is playing.
    if game.phase == Phase::Menu
        && matches!(
            screen.as_deref(),
            Some(MenuScreen::Explain | MenuScreen::Pause)
        )
    {
        return;
    }
    // Each mode's ambience loops through the whole match, the menu's in the menus.
    let race_pro = config.difficulty == Some(crate::config::Difficulty::Pro);
    let sounds: &[&'static str] = match (game.phase, game.mode.look()) {
        _ if cup_win.is_some() => &[],
        (Phase::Menu, _) => &["sfx_amb_ambience_menu"],
        (_, ModeKind::Race) if race_pro => &["sfx_amb_ambience_storm_v4"],
        (_, ModeKind::Race) => &["sfx_amb_ambience_water", "sfx_amb_ambience_breeze"],
        (_, ModeKind::Survival) => &["sfx_amb_ambience_spooky"],
        (_, _) => &["sfx_amb_ambience_wind"],
    };
    for (entity, playing) in &ambience {
        if !sounds.contains(&playing.0) {
            commands.entity(entity).despawn();
        }
    }
    for &name in sounds {
        if !ambience.iter().any(|(_, a)| a.0 == name)
            && let Some(handle) = art.sounds.get(name)
        {
            commands.spawn((
                Ambience(name),
                AudioPlayer(handle.clone()),
                PlaybackSettings::LOOP,
            ));
        }
    }
    // A battle's music starts with play, after its intro's 3-2-1.
    // Once a battle is over, the results track plays (win or lose, for the first
    // player), when it is staged; otherwise the battle music goes on.
    // After the stinger, the win or lose loop; nothing between (or while the roof comes).
    let _ = &towers;
    let results = match end.stinger {
        Some((since, win)) => {
            let (name, wait) = if win { ("music_win_game_loop", 1.8818) } else { ("music_lose_game_loop", 3.3) };
            Some((time.elapsed_secs() - since >= wait).then_some(name))
        }
        None if end.finish_tower && game.phase != Phase::Menu => Some(None),
        None => None,
    };
    let battle = match (game.phase, game.mode.look()) {
        // Joining an online game stops the music (the join handlers' MusicStopEffect).
        (Phase::Menu, _) if matches!(screen.as_deref(), Some(MenuScreen::Lobby | MenuScreen::Invite)) => None,
        (Phase::Menu, _) => Some("music_menu"),
        (Phase::Intro, _) => None,
        (_, ModeKind::Race) if race_pro && art.sounds.contains_key("music_racepro_v3") => {
            Some("music_racepro_v3")
        }
        (_, ModeKind::Race) => Some("music_race"),
        (_, ModeKind::Survival) => Some("music_survival"),
        (_, ModeKind::Puzzle) => Some("music_puzzle"),
        (_, _) => unreachable!("Endless modes look like their battle mode"),
    };
    let want = match results {
        _ if cup_win.is_some() => cup_win,
        Some(after) => after,
        None => battle,
    };
    if want.is_some_and(|want| playing.iter().any(|(_, m)| m.0 == want)) {
        return;
    }
    for (entity, _) in &playing {
        commands.entity(entity).despawn();
    }
    let Some(want) = want else {
        return;
    };
    if let Some(handle) = art.sounds.get(want) {
        commands.spawn((
            Music(want.into()),
            AudioPlayer(handle.clone()),
            PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::Linear(volume)),
        ));
    }
}

/// The count of the race hold countdown nearest its end (the leader's, whose CountDownEffect
/// is the one heard), or 0.
fn lead_countdown(
    config: &crate::config::GameConfig,
    towers: &Query<(&Tower, &TowerState, Option<&crate::gameplay::TowerSkin>)>,
) -> u32 {
    towers
        .iter()
        .filter_map(|(_, s, _)| s.count_left(config))
        .fold(None, |best: Option<f32>, t| Some(best.map_or(t, |b| b.min(t))))
        .map_or(0, |t| t.ceil().max(1.0) as u32)
}

/// The music's share of its volume: CountDownEffect turns it down to 0.6, 0.3 and 0.05 on the
/// 3, 2 and 1 (in 0.15 s), GameModePlayController brings it back over 0.5 s once no tower
/// counts, and the pause menu holds it at 0.05.
fn duck_music(
    time: Res<Time>,
    config: Res<crate::config::GameConfig>,
    game: Res<Match>,
    screen: Option<Res<crate::menu::MenuScreen>>,
    settings: Option<Res<crate::settings::Settings>>,
    towers: Query<(&Tower, &TowerState, Option<&crate::gameplay::TowerSkin>)>,
    mut sinks: Query<&mut bevy::audio::AudioSink, With<Music>>,
    mut share: Local<Option<f32>>,
) {
    use bevy::audio::AudioSinkPlayback;
    let paused = screen.as_deref() == Some(&crate::menu::MenuScreen::Pause);
    let target = if paused {
        0.05
    } else if game.phase == Phase::Playing {
        match lead_countdown(&config, &towers) {
            3 => 0.6,
            2 => 0.3,
            1 => 0.05,
            _ => 1.0,
        }
    } else {
        1.0
    };
    let now = share.get_or_insert(1.0);
    let rate = if target < *now { 1.0 / 0.15 } else { 1.0 / 0.5 };
    let step = rate * time.delta_secs();
    *now = if target < *now { (*now - step).max(target) } else { (*now + step).min(target) };
    let volume = settings.as_ref().map_or(1.0, |s| s.music) * *now;
    for mut sink in &mut sinks {
        if (sink.volume().to_linear() - volume).abs() > 1e-3 {
            sink.set_volume(bevy::audio::Volume::Linear(volume));
        }
    }
}

/// What your tower looked like last frame, to notice changes worth a sound.
#[derive(Default)]
struct Heard {
    active: Option<Entity>,
    roll: u32,
    placed: u32,
    dropped: u32,
    lives: Option<u32>,
    lines: u32,
    casts: u64,
    countdown: u32,
    phase: Option<Phase>,
}

/// The menus' cursor noise: one of the original's four select sounds, at random,
/// each time the cursor is pushed.
fn menu_cursor_sound(
    mut commands: Commands,
    art: Res<Art>,
    game: Res<Match>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    pads: Query<(Entity, &Gamepad)>,
    mut roll: Local<usize>,
) {
    if game.phase != Phase::Menu {
        return;
    }
    let taps = crate::select::read_taps(keys.as_deref(), &pads);
    if !(taps.up || taps.down || taps.left || taps.right) {
        return;
    }
    *roll = (*roll * 7 + 3) % 4;
    if let Some(handle) = art.sounds.get(&format!("sfx_menu_select_0{}", *roll + 1)) {
        commands.spawn((AudioPlayer(handle.clone()), PlaybackSettings::DESPAWN));
    }
}

#[allow(clippy::too_many_arguments)]
fn play_sounds(
    mut commands: Commands,
    art: Res<Art>,
    config: Res<crate::config::GameConfig>,
    game: Res<Match>,
    log: Res<SpellLog>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    bindings: Option<Res<crate::keybinding::KeyBindings>>,
    towers: Query<(&Tower, &TowerState, Option<&crate::gameplay::TowerSkin>)>,
    active: Query<(), With<ActiveBrick>>,
    bricks: Query<&crate::gameplay::Brick>,
    mut heard: Local<Heard>,
) {
    if art.sounds.is_empty() {
        return;
    }
    let mut play_with = |name: &str, volume: f32, speed: f32| {
        if let Some(handle) = art.sounds.get(name) {
            commands.spawn((
                AudioPlayer(handle.clone()),
                PlaybackSettings::DESPAWN
                    .with_volume(bevy::audio::Volume::Linear(volume))
                    .with_speed(speed),
            ));
        }
    };
    let Some((_, you, skin)) = towers.iter().find(|(t, _, _)| t.is_first_player()) else {
        return;
    };
    // BRICK_SPAWNED: SFX_BRICK_SPAWN at half volume for each new brick.
    // ponytail: the original starts it 0.15 s late; played at once here.
    if you.active.is_some() && you.active != heard.active {
        play_with("sfx_brick_spawn", 0.5, 1.0);
    }
    heard.active = you.active;
    // BrickRotatedAudioEffect turns at a random pitch, 0.8 to 1.2.
    heard.roll = heard.roll.wrapping_mul(1_103_515_245).wrapping_add(12_345);
    let rotate_pitch = 0.8 + 0.4 * ((heard.roll >> 16) % 1000) as f32 / 1000.0;
    // CountDownEffect: SFX_COUNT for the leading count's 3, 2, 1, each at a higher pitch; a
    // Puzzle battle's laser touch runs one too (without the music ducking).
    let laser = towers
        .iter()
        .filter_map(|(_, s, _)| s.laser_count)
        .filter(|&t| t < config.puzzle.count_seconds)
        .map(|t| (config.puzzle.count_seconds - t).ceil().max(1.0) as u32)
        .min();
    let countdown = if game.phase == Phase::Playing {
        match (lead_countdown(&config, &towers), laser) {
            (0, Some(n)) => n,
            (n, Some(m)) => n.min(m),
            (n, None) => n,
        }
    } else {
        0
    };
    if countdown != heard.countdown && (1..=3).contains(&countdown) {
        play_with("sfx_count_v3", 1.0, 1.0 + (3 - countdown) as f32 / 6.0);
    }
    heard.countdown = countdown;
    let mut play = |name: &str| match name {
        // BrickControl plays SFX_BRICK_MOVE at 0.15.
        "sfx_move" => play_with(name, 0.15, 1.0),
        "sfx_rotate" => play_with(name, 1.0, rotate_pitch),
        // A dark cast's zap (SFX_CAST_, looping at 0.8 while the beam is up).
        "sfx_zap_enemy" => play_with(name, 1.0, 0.8),
        _ => play_with(name, 1.0, 1.0),
    };
    let light_pick = heard.roll >> 20 & 1;
    if game.phase == Phase::Playing {
        if you.bricks_placed > heard.placed {
            // BrickPlacedAudioEffect: a Large brick lands heavy, an iced one with one of four
            // ice hits; any other plays the landing sound, plus its pack's own (one of three).
            let placed = you.last_placed.and_then(|e| bricks.get(e).ok());
            let variant = you.bricks_placed % 3 + 1;
            if placed.is_some_and(|b| b.scale > 1.0) {
                play("sfx_brick_heavy");
            } else if placed.is_some_and(|b| b.ice) {
                play(&format!("sfx_ice_brick_hit_0{}", you.bricks_placed % 4 + 1));
            } else {
                play("sfx_land");
                let pack = skin.map(|s| s.pack).unwrap_or_default();
                let pack_sound = format!("sfx_brick_placed_{}_{variant}", pack.key());
                if pack != crate::config::BrickPack::Default {
                    play(&pack_sound);
                }
            }
        }
        // A lost brick: Puzzle's sand (BRICK_LOST_PUZZLE, one of four), a lost life's
        // BRICK_LOST elsewhere (LIFE_LOST); Race only splashes.
        if you.bricks_dropped > heard.dropped {
            match game.mode.rules() {
                ModeKind::Puzzle => play(&format!("sfx_brick_drop_sand_0{}", you.bricks_dropped % 4 + 1)),
                ModeKind::Race => {}
                _ if you.lives < heard.lives => play("sfx_drop_lost"),
                _ => {}
            }
        }
        if you.lines_reached > heard.lines {
            play("sfx_line_collect");
        }
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
        use crate::config::SpellKind;
        if cast.spell.is_dark() {
            // WizardCastUpwardEffect's SFX_CAST_DARK_MAGIC and the zap.
            play("sfx_cast_dark");
            play("sfx_zap_enemy");
        } else {
            // One of SFX_CAST_LIGHT_MAGIC_1 and _2 at random.
            play(["sfx_cast_light", "sfx_cast_lightning_level_02"][light_pick as usize]);
        }
        // Each spell's own sound (AbstractInitSpellsCommand).
        // ponytail: played with the cast, not when the spell lands; AUTO_BUILD's loop not done.
        let own = match cast.spell {
            SpellKind::AutoRotate => "sfx_auto_rotate_start",
            SpellKind::Minibase => "sfx_spawn_minibase",
            SpellKind::ExtraLife => "sfx_spawn_life_v2",
            SpellKind::Bubble => "sfx_bubble_pop",
            SpellKind::Slow => "sfx_spawn_balloon",
            SpellKind::Fast => "sfx_fast_bricks",
            _ => "",
        };
        play(own);
    }
    heard.placed = you.bricks_placed;
    heard.dropped = you.bricks_dropped;
    heard.lives = you.lives;
    heard.lines = you.lines_reached;
    heard.casts = log.1;
    heard.phase = Some(game.phase);
}

/// A brick turning to stone plays one of SFX_PETRIFY_1..5; a brick iced plays
/// SFX_SPAWN_ICE_BRICK (PetrifyBrickModifier, IceBrickModifier).
fn stone_and_ice_sounds(
    mut commands: Commands,
    art: Res<Art>,
    stones: Query<Entity, Added<crate::spells::Stone>>,
    bricks: Query<(Entity, &crate::gameplay::Brick), Changed<crate::gameplay::Brick>>,
    mut iced: Local<Vec<Entity>>,
) {
    let mut play = |name: &str| {
        if let Some(handle) = art.sounds.get(name) {
            commands.spawn((AudioPlayer(handle.clone()), PlaybackSettings::DESPAWN));
        }
    };
    for stone in &stones {
        play(&format!("sfx_petrify_brick_0{}", stone.to_bits() % 5 + 1));
    }
    for (entity, brick) in &bricks {
        if brick.ice && !iced.contains(&entity) {
            iced.push(entity);
            play("sfx_spawn_icebrick");
        }
    }
    if iced.len() > 256 {
        iced.drain(..128);
    }
}

/// Ice bricks knock on everything they hit (IceBrickModifier's CollisionSoundBehaviour): one
/// of SFX_BRICK_PLACED_ICE_1..4 (staged as sfx_ice_brick_hit_01..04) on each new contact
/// faster than `ice_hit_speed` cells a second.
fn ice_hit_sounds(
    mut commands: Commands,
    art: Res<Art>,
    config: Res<crate::config::GameConfig>,
    iced: Query<(Entity, &crate::gameplay::Brick), (Changed<crate::gameplay::Brick>, Without<avian2d::prelude::CollisionEventsEnabled>)>,
    bricks: Query<&crate::gameplay::Brick>,
    velocities: Query<&avian2d::prelude::LinearVelocity>,
    mut starts: MessageReader<avian2d::prelude::CollisionStart>,
    mut roll: Local<u32>,
) {
    for (entity, brick) in &iced {
        if brick.ice {
            commands.entity(entity).try_insert(avian2d::prelude::CollisionEventsEnabled);
        }
    }
    let speed = |body: Option<Entity>| body.and_then(|b| velocities.get(b).ok()).map_or(Vec2::ZERO, |v| v.0);
    for start in starts.read() {
        let ice = [start.body1, start.body2]
            .iter()
            .flatten()
            .any(|b| bricks.get(*b).is_ok_and(|brick| brick.ice));
        if !ice || (speed(start.body1) - speed(start.body2)).length() <= config.cells(config.ice_hit_speed) {
            continue;
        }
        *roll = (*roll * 7 + 3) % 4;
        if let Some(sound) = art.sounds.get(&format!("sfx_ice_brick_hit_0{}", *roll + 1)) {
            commands.spawn((AudioPlayer(sound.clone()), PlaybackSettings::DESPAWN));
        }
    }
}

/// Auto Rotate's turns after the first play one of SFX_AUTO_ROTATE_1..3, 0.3 s after the
/// wind-back starts, at 0.95 to 1.05 pitch (AutoRotateBrickModifier, RandomAudioEffect).
fn auto_rotate_sounds(
    mut commands: Commands,
    art: Res<Art>,
    bricks: Query<(Entity, &ActiveBrick)>,
    mut heard: Local<Vec<(Entity, u32)>>,
) {
    heard.retain(|(e, _)| bricks.contains(*e));
    for (entity, brick) in &bricks {
        let Some(cycle) = brick.mods.auto_rotate else { continue };
        let turns = ((brick.auto_rotate_timer - 0.3) / cycle.max(0.01)).floor().max(0.0) as u32;
        let was = heard.iter().find(|h| h.0 == entity).map_or(0, |h| h.1);
        if turns > was {
            let roll = (brick.auto_rotate_timer * 1000.0) as u32;
            if let Some(handle) = art.sounds.get(&format!("sfx_auto_rotate_0{}", roll % 3 + 1)) {
                let pitch = 0.95 + (roll % 101) as f32 / 1000.0;
                commands.spawn((AudioPlayer(handle.clone()), PlaybackSettings::DESPAWN.with_speed(pitch)));
            }
        }
        heard.retain(|h| h.0 != entity);
        heard.push((entity, turns));
    }
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

    #[test]
    fn lit_brick_entry_reads() {
        let e = read_entry(r#"(file: "bricks/T.png", bevel: "brick_T_bevel", light: ((0.7106, 0.9034, 0.9118, 0.903), (0.2279, 0.2974, 0.5, 0.903)), light_angle: 280.0)"#);
        assert_eq!(e.bevel.as_deref(), Some("brick_T_bevel"));
        assert!(e.light.unwrap().get().is_some());
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
