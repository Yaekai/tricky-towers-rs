//! Text drawn with the original's letters. The original's font only exists in
//! the player's install as a picture of its letters plus measurements, so when
//! that is staged (`font_main` pointing at a letter sheet, see
//! `docs/art-layout.md`), every UI `Text` is shown as a row of letter images
//! instead. Systems keep writing plain `Text`; a mirror node draws it.

use bevy::prelude::*;

use crate::art::{Art, Letters};


const SDF_SHADER: Handle<Shader> = bevy::asset::uuid_handle!("4d1e7a29-c603-4b85-9f2e-61b8d0a3e5c7");

const SDF_WGSL: &str = r#"// One letter from the original's distance-field font (TextMeshPro), sharp at any size:
// the field is read in the letter's rectangle and cut at `edge`, softened by one screen pixel.
#import bevy_ui::ui_vertex_output::UiVertexOutput

struct SdfLetter {
    color: vec4<f32>,
    // xy: the rectangle's top left, zw: its size, as shares of the atlas.
    rect: vec4<f32>,
    // x: where the edge sits in the field.
    edge: vec4<f32>,
    // The face's colour at the letter's bottom when its alpha is above 0 (a TextMeshPro face
    // texture running top to bottom); otherwise `color` all over.
    low: vec4<f32>,
};

@group(1) @binding(0) var<uniform> material: SdfLetter;
@group(1) @binding(1) var field: texture_2d<f32>;
@group(1) @binding(2) var field_sampler: sampler;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let uv = material.rect.xy + in.uv * material.rect.zw;
    let d = textureSample(field, field_sampler, uv).a;
    let w = max(fwidth(d) * 0.7, 0.002);
    let a = smoothstep(material.edge.x - w, material.edge.x + w, d);
    var c = material.color;
    if material.low.a > 0.0 {
        c = mix(material.color, material.low, in.uv.y);
    }
    return vec4(c.rgb, c.a * a);
}
"#;

/// One letter drawn from the distance field.
#[derive(Asset, TypePath, bevy::render::render_resource::AsBindGroup, Clone)]
pub struct SdfLetter {
    #[uniform(0)]
    pub look: SdfLook,
    #[texture(1)]
    #[sampler(2)]
    pub field: Handle<Image>,
}

#[derive(bevy::render::render_resource::ShaderType, Clone, Copy, Debug)]
pub struct SdfLook {
    pub color: Vec4,
    pub rect: Vec4,
    pub edge: Vec4,
    pub low: Vec4,
}

impl bevy::ui_render::prelude::UiMaterial for SdfLetter {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        bevy::shader::ShaderRef::Handle(SDF_SHADER)
    }
}

/// The field's edge for the letters' face and for their outline, from the outlined Enzo
/// materials (EnzoBlackMilkyWhiteOutline26: face dilate 0.35, outline width 0.35, scale ratio
/// 0.833): TextMeshPro grows the letters by dilate x ratio / 2 and centres the outline on that
/// edge, so the face ends at 0.5 and the outline at 0.5 - 2 x 0.146 = 0.208.
const FACE_EDGE: f32 = 0.5;
const OUTLINE_EDGE: f32 = 0.208;

/// Which staged letter sheet draws the text: the original's display letters (Enzo,
/// titles, buttons, hints) and its body letters (Hans Sans Bold, list rows).
const FONT: &str = "font_main";
const BODY_FONT: &str = "font_body";

/// The display letters in use: the original swaps EnzoBlack for another font in Russian and
/// Japanese (Meiryo Bold), Chinese (Hans Sans Bold, our body letters), Korean (Hans Sans KR)
/// and Arabic (Myriad) (InitFontsCommand). The body letters stay Hans Sans Bold everywhere.
static DISPLAY: std::sync::RwLock<&str> = std::sync::RwLock::new(FONT);

/// The picked language's file code ("en", "jp"...).
static LANGUAGE: std::sync::RwLock<&str> = std::sync::RwLock::new("en");

pub(crate) fn language() -> &'static str {
    LANGUAGE.read().map_or("en", |code| *code)
}

fn display_font(code: &str) -> &'static str {
    match code {
        "ru" | "jp" => "font_meiryo",
        "ch" | "tw" => BODY_FONT,
        "ko" => "font_hans_kr",
        "ar" => "font_myriad_letters",
        _ => FONT,
    }
}

/// The picked language's display letters, or EnzoBlack's when that font isn't staged.
fn display_sheet(art: &Art) -> Option<&Letters> {
    let name = DISPLAY.read().map_or(FONT, |n| *n);
    art.letters.get(name).or_else(|| art.letters.get(FONT))
}

/// How a text looks when drawn with the original's letters, like its TextMeshPro
/// material: the body letters or the display ones, a face colour that replaces the
/// `TextColor` (which stays readable on the plain-shape screens), and an outline.
#[derive(Component, Clone, Copy, PartialEq, Debug, Default)]
pub struct TextLook {
    pub body: bool,
    pub face: Option<Color>,
    pub outline: Option<Color>,
    /// The face's colour at the bottom of each letter, for the materials whose face
    /// texture is a top-to-bottom gradient (gph_gradientTextureFont/Font02).
    pub low: Option<Color>,
    /// Where the outline's outer edge sits in the field, from the material's face dilate
    /// and outline width (0.5 - (dilate + width) x 0.833 / 2); 0 for the outlined Enzo
    /// materials' usual 0.35 / 0.35 (OUTLINE_EDGE).
    pub outline_edge: f32,
}

impl TextLook {
    /// EnzoBlackGold: its face texture runs from yellow at the top to orange at the bottom.
    pub const GOLD: Self = Self {
        body: false,
        face: Some(Color::srgb(253.0 / 255.0, 1.0, 102.0 / 255.0)),
        outline: Some(Color::srgb(0.169, 0.037, 0.138)),
        low: Some(Color::srgb(1.0, 137.0 / 255.0, 22.0 / 255.0)),
        // Dilate and outline 0.25 (EnzoBlackGold46).
        outline_edge: 0.5 - 0.5 * 0.8333 / 2.0,
    };
    /// EnzoBlackRed: red at the top to pink at the bottom (BACK).
    pub const PINK: Self = Self {
        body: false,
        face: Some(Color::srgb(239.0 / 255.0, 43.0 / 255.0, 97.0 / 255.0)),
        outline: Some(Color::srgb(0.145, 0.051, 0.157)),
        low: Some(Color::srgb(1.0, 41.0 / 255.0, 182.0 / 255.0)),
        // Dilate and outline 0.25 (EnzoBlackRed46).
        outline_edge: 0.5 - 0.5 * 0.8333 / 2.0,
    };
    /// List rows and buttons: near-black body letters.
    pub const BODY: Self = Self {
        body: true,
        face: Some(Color::srgb(0.07, 0.055, 0.12)),
        outline: None,
        low: None,
        outline_edge: 0.0,
    };
    /// Screen titles: milky white display letters with a dark outline
    /// (the original's EnzoBlackMilkyWhiteOutline26). Its face (182, 197, 254) is lit by the
    /// material's bevel (BEVEL_ON, diffuse 0.5): measured on Max's title clip it shows as
    /// (208, 224, 253), which is what is drawn here.
    pub const TITLE: Self = Self {
        body: false,
        face: Some(Color::srgb(0.816, 0.878, 0.992)),
        outline: Some(Color::srgb(0.145, 0.051, 0.157)),
        low: None,
        outline_edge: 0.0,
    };
    /// Display letters in their own colour with the dark outline (button hints, tiles).
    pub const OUTLINED: Self = Self {
        body: false,
        face: None,
        outline: Some(Color::srgb(0.145, 0.051, 0.157)),
        low: None,
        outline_edge: 0.0,
    };
}

pub struct LettersPlugin;

impl Plugin for LettersPlugin {
    fn build(&self, app: &mut App) {
        if let Some(mut shaders) = app.world_mut().get_resource_mut::<Assets<Shader>>() {
            let _ = shaders.insert(SDF_SHADER.id(), Shader::from_wgsl(SDF_WGSL, "sdf_letter.wgsl"));
            app.add_plugins(bevy::ui_render::prelude::UiMaterialPlugin::<SdfLetter>::default());
        }
        app.init_resource::<Translations>().add_systems(
            PostUpdate,
            (
                load_translations,
                mirror_texts,
                sync_mirrors,
                tidy_mirrors,
                follow_nodes,
                redraw_when_fields_load,
                draw_letters,
            )
                .chain()
                .before(bevy::ui::UiSystems::Prepare),
        );
    }
}

/// On a hidden `Text`: the node that draws it with the original's letters.
#[derive(Component)]
struct MirroredBy(Entity);

/// On a twin: the text it draws.
#[derive(Component)]
struct Mirrors(Entity);

/// A text's character spacing in the font's units (TextMeshPro's characterSpacing: added to
/// every letter's advance before scaling; most of the original's menu texts use -1).
#[derive(Component, Clone, Copy, PartialEq, Debug, Default)]
pub struct LetterSpacing(pub f32);

/// TextMeshPro's auto-size: the text shrinks, down to `min`, until its widest line fits
/// `width` (both in the text's own font-size unit).
#[derive(Component, Clone, Copy)]
pub struct AutoSize {
    pub min: f32,
    pub width: f32,
}

/// The size `words` draws at once `fit` has shrunk it.
fn fit_size(sheet: &Letters, words: &str, size: f32, spacing: f32, fit: &AutoSize) -> f32 {
    let k = size / sheet.size.max(1.0);
    let widest = words
        .split('\n')
        .map(|line| {
            line.chars()
                .map(|ch| {
                    let advance = sheet
                        .glyphs
                        .get(&ch)
                        .or_else(|| sheet.glyphs.get(&ch.to_ascii_uppercase()))
                        .map_or(sheet.size * 0.3, |g| g.advance);
                    (advance + spacing) * k
                })
                .sum::<f32>()
        })
        .fold(0.0, f32::max);
    if widest > fit.width { (size * fit.width / widest).max(fit.min) } else { size }
}

/// How many lines `text` draws in the picked language, and their line height, at `size`
/// shrunk by `fit` (same unit as `size`).
pub fn translated_lines(art: &Art, text: &str, size: f32, fit: &AutoSize) -> (usize, f32) {
    let words = CURRENT.read().map(|w| w.clone()).unwrap_or_default();
    let translations = Translations { code: String::new(), words, patterns: Vec::new() };
    let Some(sheet) = display_sheet(art) else {
        return (text.split('\n').count(), size);
    };
    let words = translations.apply(text, Some(sheet));
    let size = fit_size(sheet, &words, size, 0.0, fit);
    (words.split('\n').count(), sheet.line_height * size / sheet.size.max(1.0))
}

/// Text to draw with the letter sheet.
#[derive(Component, Clone, PartialEq)]
struct LetterText {
    spacing: f32,
    text: String,
    size: f32,
    /// Whether `size` is a share of the view height rather than pixels.
    view_height: bool,
    color: Color,
    centered: bool,
    body: bool,
    outline: Option<Color>,
    low: Option<Color>,
    outline_edge: f32,
}

/// The language files' names, in `settings::LANGUAGES` order (Finnish is `fl`).
const LANGUAGE_FILES: [&str; 19] = [
    "en", "de", "fr", "es", "pt", "it", "ch", "tw", "jp", "ko", "ru", "pl", "ar", "tr", "nl", "sv",
    "no", "fl", "ro",
];

/// The picked language's wording for our English texts: each English line of the original's
/// `en.txt`, in capitals with its button glyphs (`[continue]`...) and line breaks taken out,
/// mapped to the same key's line in the picked language's file (staged `language/<code>.txt`).
#[derive(Resource, Default)]
pub struct Translations {
    code: String,
    words: std::collections::HashMap<String, String>,
    /// Lines with one placeholder (`First to {targetScore} points`): the English line's
    /// capitals before and after it, and the picked language's line with its placeholder.
    patterns: Vec<(String, String, String)>,
}

impl Translations {
    /// `text` in the picked language, kept in capitals if it was, or `text` itself when there
    /// is no translation or the letters can't draw it (the language's font isn't staged).
    fn apply(&self, text: &str, sheet: Option<&Letters>) -> String {
        let key = plain(text);
        // A line with a value in it: the value goes into the picked language's line.
        let filled = || {
            self.patterns.iter().find_map(|(before, after, picked)| {
                let middle = key.strip_prefix(before.as_str())?.strip_suffix(after.as_str())?;
                if middle.is_empty() {
                    return None;
                }
                // A value that is itself a line ("{cupType} Winner!") is translated too.
                let middle = self.words.get(middle).cloned().unwrap_or_else(|| middle.to_string());
                let start = picked.find('{')?;
                let end = picked[start..].find('}')? + start;
                Some(format!("{}{}{}", &picked[..start], middle, &picked[end + 1..]))
            })
        };
        let Some(found) = self.words.get(&key).cloned().or_else(filled) else {
            return text.to_string();
        };
        let found = if text == text.to_uppercase() { found.to_uppercase() } else { found };
        // Leading empty lines leave room for a button picture drawn over them ("\nTO JOIN").
        let found = format!("{}{found}", "\n".repeat(text.len() - text.trim_start_matches('\n').len()));
        let drawable = sheet.is_none_or(|sheet| {
            found.chars().filter(|c| !c.is_whitespace()).all(|c| {
                sheet.glyphs.contains_key(&c) || c.to_uppercase().any(|u| sheet.glyphs.contains_key(&u))
            })
        });
        if drawable { found } else { text.to_string() }
    }
}

/// The word the original puts for `[controller]` in each language (LanguageUtil).
fn controller_word(code: &str) -> &'static str {
    match code {
        "es" => "mando",
        "fl" => "ohjain",
        "fr" => "manette de jeu",
        "no" => "kontroller",
        "pl" => "kontroler",
        "pt" => "comando",
        "sv" => "handkontroll",
        "tr" => "kontrol cihazı",
        "ch" | "tw" => "控制器",
        "jp" => "コントローラー",
        "ko" => "컨트롤러",
        // Reversed like every Arabic line (LanguageUtil).
        "ar" => "مكحت ةدحو",
        _ => "controller",
    }
}

/// Arabic lines as the original draws them (LanguageManager._ReverseText): the letters in
/// reverse order, with `[...]` and `{...}` kept readable; no joining of the letters.
fn reverse_text(text: &str) -> String {
    let mut out: Vec<char> = Vec::new();
    let mut chars = text.chars().peekable();
    // Reverse each bracketed part first, so the whole-line reverse turns it back.
    while let Some(c) = chars.next() {
        let close = match c {
            '[' => ']',
            '{' => '}',
            _ => {
                out.push(c);
                continue;
            }
        };
        let mut part = vec![c];
        for d in chars.by_ref() {
            part.push(d);
            if d == close {
                break;
            }
        }
        out.extend(part.into_iter().rev());
    }
    out.into_iter().rev().collect()
}

/// The text before and after a line's only button glyph (`[...]`), if it has exactly one.
fn around_glyph(line: &str) -> Option<(String, String)> {
    let open = line.find('[')?;
    let close = line[open..].find(']')? + open;
    if line[close + 1..].contains('[') {
        return None;
    }
    Some((line[..open].to_string(), line[close + 1..].to_string()))
}

/// The current translations for code that measures text outside the ECS (a copy of
/// `Translations::words`).
static CURRENT: std::sync::LazyLock<std::sync::RwLock<std::collections::HashMap<String, String>>> =
    std::sync::LazyLock::new(Default::default);

/// How wide `line` draws in the picked language (see `line_width`).
pub fn translated_width(art: &Art, line: &str, size: f32) -> f32 {
    let words = CURRENT.read().map(|w| w.clone()).unwrap_or_default();
    let translations = Translations { code: String::new(), words, patterns: Vec::new() };
    line_width(art, &translations.apply(line, display_sheet(art)), size)
}

/// A line as the translations are keyed: capitals, no button glyphs, one line.
fn plain(text: &str) -> String {
    clean(text).split_whitespace().collect::<Vec<_>>().join(" ").to_uppercase()
}

/// A line without its button glyphs, its written line breaks (`\n`) made real ones and the
/// spaces around them tidied.
fn clean(text: &str) -> String {
    let mut out = String::new();
    let mut skipping = false;
    for c in text.replace("\\n", "\n").chars() {
        match c {
            '[' => skipping = true,
            ']' => skipping = false,
            _ if !skipping => out.push(c),
            _ => {}
        }
    }
    out.lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Every lookup of one of the original's language files, by key.
fn read_language(code: &str) -> std::collections::HashMap<String, String> {
    let text = std::fs::read_to_string(crate::art::extracted_dir().join(format!("language/{code}.txt")))
        .unwrap_or_default();
    let mut words = std::collections::HashMap::new();
    for block in text.split("<lookup>").skip(1) {
        let between = |open: &str, close: &str| {
            let start = block.find(open)? + open.len();
            let end = block[start..].find(close)? + start;
            Some(block[start..end].to_string())
        };
        if let (Some(key), Some(value)) = (between("<key>", "</key>"), between("<translation>", "</translation>")) {
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

/// Loads the picked language when Settings changes it, and has every text redrawn.
fn load_translations(
    settings: Option<Res<crate::settings::Settings>>,
    mut translations: ResMut<Translations>,
    mut texts: Query<&mut Text, With<MirroredBy>>,
    mut drawn: Query<&mut LetterText>,
) {
    let code = settings
        .as_ref()
        .and_then(|s| crate::settings::LANGUAGES.iter().position(|&l| l == s.language))
        .map_or("en", |i| LANGUAGE_FILES[i]);
    if translations.code == code {
        return;
    }
    translations.code = code.to_string();
    translations.words.clear();
    translations.patterns.clear();
    if code != "en" {
        let english = read_language("en");
        let picked = read_language(code);
        // [controller] is the local word for a controller (LanguageUtil), not a button.
        let word = controller_word(code);
        for (key, english) in &english {
            let english = &english.replace("[controller]", "controller");
            if let Some(picked) = picked
                .get(key)
                .filter(|p| !p.trim().is_empty())
                .map(|p| if code == "ar" { reverse_text(&p.replace("\\n", "\n")) } else { p.clone() })
                .map(|p| p.replace("[controller]", word))
                .as_ref()
            {
                translations.words.insert(plain(english), clean(picked));
                let english_line = plain(english);
                if let (Some(start), 1) = (english_line.find('{'), english_line.matches('{').count())
                    && let Some(end) = english_line[start..].find('}').map(|e| e + start)
                    && picked.matches('{').count() == 1
                {
                    translations.patterns.push((
                        english_line[..start].to_string(),
                        english_line[end + 1..].to_string(),
                        clean(picked),
                    ));
                }
                // A line around one button glyph ("Press [continue] to start") also gives its
                // two halves, for screens that draw the glyph between two texts.
                if let (Some((eb, ea)), Some((pb, pa))) = (around_glyph(english), around_glyph(picked)) {
                    for (e, p) in [(eb, pb), (ea, pa)] {
                        if !plain(&e).is_empty() {
                            translations.words.entry(plain(&e)).or_insert_with(|| clean(&p));
                        }
                    }
                }
            }
        }
    }
    if let Ok(mut words) = CURRENT.write() {
        *words = translations.words.clone();
    }
    if let Ok(mut font) = DISPLAY.write() {
        *font = display_font(code);
    }
    if let Ok(mut language) = LANGUAGE.write() {
        *language = code;
    }
    for mut text in &mut texts {
        text.set_changed();
    }
    // Texts that read the same (numbers) are drawn again in the new letters too.
    for mut text in &mut drawn {
        text.set_changed();
    }
}

fn letter_text(
    text: &Text,
    font: &TextFont,
    color: Option<&TextColor>,
    layout: Option<&TextLayout>,
    look: Option<&TextLook>,
    spacing: Option<&LetterSpacing>,
    fit: Option<&AutoSize>,
    translations: &Translations,
    sheet: Option<&Letters>,
) -> LetterText {
    let look = look.copied().unwrap_or_default();
    let (mut size, view_height) = match font.font_size {
        FontSize::Px(px) => (px, false),
        FontSize::Vh(vh) => (vh, true),
        _ => (20.0, false),
    };
    let spacing = spacing.map_or(0.0, |s| s.0);
    let words = translations.apply(&text.0, sheet);
    if let (Some(fit), Some(sheet)) = (fit, sheet) {
        size = fit_size(sheet, &words, size, spacing, fit);
    }
    LetterText {
        spacing,
        text: words,
        size,
        view_height,
        color: look.face.unwrap_or(color.map_or(Color::WHITE, |c| c.0)),
        centered: layout.is_some_and(|l| l.justify == Justify::Center),
        body: look.body,
        outline: look.outline,
        low: look.low,
        outline_edge: look.outline_edge,
    }
}

/// Gives every new UI text a letter-drawn twin in the same place and hides it.
#[allow(clippy::type_complexity)]
fn mirror_texts(
    mut commands: Commands,
    art: Res<Art>,
    translations: Res<Translations>,
    mut texts: Query<
        (
            Entity,
            &Text,
            &TextFont,
            Option<&TextColor>,
            Option<&TextLayout>,
            Option<&TextLook>,
            &mut Node,
            Option<&ChildOf>,
            Option<&GlobalZIndex>,
            Option<&LetterSpacing>,
            Option<&AutoSize>,
        ),
        Without<MirroredBy>,
    >,
    children: Query<&Children>,
) {
    if !art.letters.contains_key(FONT) {
        return;
    }
    for (entity, text, font, color, layout, look, mut node, parent, z, spacing, fit) in &mut texts {
        let mirror = commands
            .spawn((
                node.clone(),
                letter_text(text, font, color, layout, look, spacing, fit, &translations, display_sheet(&art)),
                Mirrors(entity),
            ))
            .id();
        // A text's own stacking goes with it (HUD items sit over the lanes at their own z).
        if let Some(z) = z {
            commands.entity(mirror).insert(*z);
        }
        if let Some(parent) = parent {
            let index = children
                .get(parent.parent())
                .ok()
                .and_then(|c| c.iter().position(|e| e == entity))
                .map_or(0, |i| i + 1);
            commands.entity(parent.parent()).insert_child(index, mirror);
        }
        node.display = Display::None;
        commands.entity(entity).insert(MirroredBy(mirror));
    }
}

/// Copies text changes to the twin.
#[allow(clippy::type_complexity)]
fn sync_mirrors(
    art: Res<Art>,
    translations: Res<Translations>,
    texts: Query<
        (
            &MirroredBy,
            &Text,
            &TextFont,
            Option<&TextColor>,
            Option<&TextLayout>,
            Option<&TextLook>,
            Option<&LetterSpacing>,
            Option<&AutoSize>,
        ),
        Or<(
            Changed<Text>,
            Changed<TextColor>,
            Changed<TextFont>,
            Changed<TextLook>,
            Changed<LetterSpacing>,
        )>,
    >,
    mut mirrors: Query<&mut LetterText>,
) {
    for (mirror, text, font, color, layout, look, spacing, fit) in &texts {
        if let Ok(mut letters) = mirrors.get_mut(mirror.0) {
            let new = letter_text(text, font, color, layout, look, spacing, fit, &translations, display_sheet(&art));
            if *letters != new {
                *letters = new;
            }
        }
    }
}

/// Twins show and hide with their text, and go when it does: HUD labels have no
/// parent, so a twin left behind kept showing an old 1ST over the next match.
fn tidy_mirrors(
    mut commands: Commands,
    mut twins: Query<(Entity, &Mirrors, &mut Visibility)>,
    texts: Query<&Visibility, (With<Text>, With<MirroredBy>, Without<Mirrors>)>,
) {
    for (twin, text, mut visibility) in &mut twins {
        match texts.get(text.0) {
            Ok(shown) => {
                visibility.set_if_neq(*shown);
            }
            Err(_) => {
                commands.entity(twin).despawn();
                if let Ok(mut text) = commands.get_entity(text.0) {
                    text.remove::<MirroredBy>();
                }
            }
        }
    }
}

/// Moves each twin with its text: HUD labels (place, counts) are placed every frame
/// after the twin was made, and a twin left behind showed at its first spot.
fn follow_nodes(
    texts: Query<(&MirroredBy, &Node), Changed<Node>>,
    mut twins: Query<&mut Node, Without<MirroredBy>>,
) {
    for (mirror, node) in &texts {
        if let Ok(mut twin) = twins.get_mut(mirror.0) {
            twin.left = node.left;
            twin.top = node.top;
            twin.right = node.right;
            twin.bottom = node.bottom;
        }
    }
}

/// Lays the letters out: one row per line, each letter cut from the sheet.
/// Texts drawn before the distance fields had loaded used the plain letter pictures; once the
/// fields are in, every text is drawn again from them.
/// How wide `line` draws in the display letters at `size` (pixels, or the same unit
/// the size is in), as `spawn_line` lays it out; 0 without the letter sheet.
pub fn line_width(art: &Art, line: &str, size: f32) -> f32 {
    let Some(sheet) = display_sheet(art) else {
        return 0.0;
    };
    let k = size / sheet.size.max(1.0);
    line.chars()
        .map(|ch| {
            sheet
                .glyphs
                .get(&ch)
                .or_else(|| sheet.glyphs.get(&ch.to_ascii_uppercase()))
                .map_or(sheet.size * 0.3, |g| g.advance)
                * k
        })
        .sum()
}

fn redraw_when_fields_load(
    art: Res<Art>,
    images: Option<Res<Assets<Image>>>,
    mut texts: Query<&mut LetterText>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }
    let Some(images) = images else { return };
    let mut fields = art.letters.values().filter_map(|l| l.sdf_handle.as_ref()).peekable();
    if fields.peek().is_none() {
        return;
    }
    if fields.all(|h| images.get(h).is_some()) {
        *done = true;
        for mut text in &mut texts {
            text.set_changed();
        }
    }
}

fn draw_letters(
    mut commands: Commands,
    art: Res<Art>,
    images: Option<Res<Assets<Image>>>,
    mut sdf: Option<ResMut<Assets<SdfLetter>>>,
    texts: Query<(Entity, &LetterText), Changed<LetterText>>,
) {
    let Some(main) = display_sheet(&art) else {
        return;
    };
    for (entity, text) in &texts {
        let sheet = if text.body {
            art.letters.get(BODY_FONT).unwrap_or(main)
        } else {
            main
        };
        // The distance field, once loaded, draws the letters sharp at any size.
        let field = sheet.sdf_handle.as_ref().and_then(|h| {
            let size = images.as_ref()?.get(h)?.size_f32();
            Some((h.clone(), size))
        });
        let mut node = commands.entity(entity);
        node.despawn_children();
        node.with_children(|lines| {
            let column = Node {
                flex_direction: FlexDirection::Column,
                align_items: if text.centered {
                    AlignItems::Center
                } else {
                    AlignItems::FlexStart
                },
                ..default()
            };
            lines.spawn(column).with_children(|lines| {
                for line in text.text.split('\n') {
                    spawn_line(lines, sheet, text, line, field.clone().zip(sdf.as_deref_mut()));
                }
            });
        });
    }
}

fn spawn_line(
    lines: &mut ChildSpawnerCommands,
    sheet: &Letters,
    text: &LetterText,
    line: &str,
    mut field: Option<((Handle<Image>, Vec2), &mut Assets<SdfLetter>)>,
) {
    let k = text.size / sheet.size.max(1.0);
    let len = |v: f32| {
        if text.view_height {
            Val::Vh(v)
        } else {
            Val::Px(v)
        }
    };
    let height = sheet.line_height * k;
    lines
        .spawn(Node {
            height: len(height),
            ..default()
        })
        .with_children(|row| {
            for ch in line.chars() {
                let Some(glyph) = sheet
                    .glyphs
                    .get(&ch)
                    .or_else(|| sheet.glyphs.get(&ch.to_ascii_uppercase()))
                else {
                    // Unknown letters (and spaces without a glyph) leave a gap.
                    row.spawn(Node {
                        width: len(sheet.size * 0.3 * k),
                        ..default()
                    });
                    continue;
                };
                let (x, y, w, h) = glyph.rect;
                let (ox, oy) = glyph.offset;
                row.spawn(Node {
                    width: len((glyph.advance + text.spacing) * k),
                    height: len(height),
                    ..default()
                })
                .with_children(|cell| {
                    if w <= 0.0 || h <= 0.0 {
                        return;
                    }
                    let has_field = field.is_some();
                    // A letter from the field: its rectangle as shares of the atlas.
                    let mut from_field = |rect: (f32, f32, f32, f32), color: Color, edge: f32, low: Option<Color>| {
                        let ((handle, size), sdf) = field.as_mut()?;
                        let look = SdfLook {
                            color: Vec4::from_array(color.to_linear().to_f32_array()),
                            rect: Vec4::new(rect.0 / size.x, rect.1 / size.y, rect.2 / size.x, rect.3 / size.y),
                            edge: Vec4::new(edge, 0.0, 0.0, 0.0),
                            low: low.map_or(Vec4::ZERO, |c| Vec4::from_array(c.to_linear().to_f32_array())),
                        };
                        Some(bevy::ui_render::prelude::MaterialNode(sdf.add(SdfLetter {
                            look,
                            field: handle.clone(),
                        })))
                    };
                    if let Some(color) = text.outline
                        && (sheet.outline_handle.is_some() || has_field)
                    {
                        let p = sheet.outline_pad;
                        let place = Node {
                                position_type: PositionType::Absolute,
                                left: len((ox - p) * k),
                                top: len((sheet.ascender - oy - p) * k),
                                width: len((w + 2.0 * p) * k),
                                height: len((h + 2.0 * p) * k),
                                ..default()
                            };
                        match from_field((x - p, y - p, w + 2.0 * p, h + 2.0 * p), color, if text.outline_edge > 0.0 { text.outline_edge } else { OUTLINE_EDGE }, None) {
                            Some(material) => {
                                cell.spawn((material, place));
                            }
                            None => {
                                if let Some(outline) = &sheet.outline_handle {
                                    cell.spawn((
                                        ImageNode {
                                            image: outline.clone(),
                                            rect: Some(Rect::new(x - p, y - p, x + w + p, y + h + p)),
                                            color,
                                            ..default()
                                        },
                                        place,
                                    ));
                                }
                            }
                        }
                    }
                    let place = Node {
                        position_type: PositionType::Absolute,
                        left: len(ox * k),
                        top: len((sheet.ascender - oy) * k),
                        width: len(w * k),
                        height: len(h * k),
                        ..default()
                    };
                    match from_field((x, y, w, h), text.color, FACE_EDGE, text.low) {
                        Some(material) => {
                            cell.spawn((material, place));
                        }
                        None => {
                            cell.spawn((
                                ImageNode {
                                    image: sheet.handle.clone(),
                                    rect: Some(Rect::new(x, y, x + w, y + h)),
                                    color: text.color,
                                    ..default()
                                },
                                place,
                            ));
                        }
                    }
                });
            }
        });
}

#[cfg(test)]
mod tests {
    #[test]
    fn arabic_lines_reverse_with_their_brackets_kept() {
        assert_eq!(super::reverse_text("ab [cd] {ef}"), "{ef} [cd] ba");
    }

    #[test]
    fn translations_are_keyed_without_button_glyphs() {
        assert_eq!(super::plain("Continue [continue]"), "CONTINUE");
        assert_eq!(super::plain("Connect\n[controller]"), "CONNECT");
        assert_eq!(
            super::around_glyph("Press [continue] to start"),
            Some(("Press ".to_string(), " to start".to_string()))
        );
        let translations = super::Translations {
            code: "de".into(),
            words: [("SELECT MODE".to_string(), "Modus auswählen".to_string())].into(),
            patterns: Vec::new(),
        };
        assert_eq!(translations.apply("SELECT MODE", None), "MODUS AUSWÄHLEN");
        assert_eq!(translations.apply("Select Mode", None), "Modus auswählen");
        assert_eq!(translations.apply("RACE", None), "RACE");
        let translations = super::Translations {
            code: "de".into(),
            words: Default::default(),
            patterns: vec![("FIRST TO ".into(), " POINTS".into(), "Wer zuerst {targetScore} Punkte hat".into())],
        };
        assert_eq!(translations.apply("First to 6 points", None), "Wer zuerst 6 Punkte hat");
    }
}
