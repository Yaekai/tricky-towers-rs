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
    // An outline drawn with its face in one pass (alpha above 0): the face's colour, cut at
    // edge.w; edge.y and edge.z place the face's top and height in the node for `low`.
    face: vec4<f32>,
    // TextMeshPro's GLOW_ON (alpha above 0): its colour, and in the field its centre (the
    // dilated edge less the offset), inner and outer reach and power.
    glow: vec4<f32>,
    glow_shape: vec4<f32>,
    // BEVEL_ON (x above 0): y the material's weight plus its bevel offset, z the bevel's
    // width (outline width + bevel width).
    bevel: vec4<f32>,
};

// The bevel every BEVEL_ON Enzo material shares (MilkyWhite*, White): bevel 0.4, clamp
// 0.351, roundness 0, raised (ShaderFlags 1), gradient scale 6, light angle 3.14, diffuse
// 0.5, ambient 0, no specular.
const BEVEL: f32 = 0.4;
const BEVEL_CLAMP: f32 = 0.351;
const GRADIENT_SCALE: f32 = 6.0;
const LIGHT_ANGLE: f32 = 3.14;
const DIFFUSE: f32 = 0.5;
const AMBIENT: f32 = 0.0;

// TextMeshPro's GetSurfaceNormal on four field samples half a texel either side.
fn bevel_height(h: f32) -> f32 {
    let t = clamp((h + material.bevel.y - 0.5) / material.bevel.z + 0.5, 0.0, 1.0);
    let raised = 1.0 - abs(t * 2.0 - 1.0);
    return min(raised, 1.0 - BEVEL_CLAMP);
}

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
    if material.face.a > 0.0 {
        var f = material.face;
        if material.low.a > 0.0 {
            let v = clamp((in.uv.y - material.edge.y) / max(material.edge.z, 0.001), 0.0, 1.0);
            f = mix(material.face, material.low, v);
        }
        let af = smoothstep(material.edge.w - w, material.edge.w + w, d);
        c = vec4(mix(c.rgb, f.rgb, af), mix(c.a, f.a, af));
    } else if material.low.a > 0.0 {
        c = mix(material.color, material.low, in.uv.y);
    }
    // Premultiplied: a glow adds its colour inside the letter and lies over what is behind
    // outside it (GetGlowColor: from the edge out over `outer` and in over `inner`, one pixel
    // more, to the power).
    var rgb = c.rgb * c.a * a;
    var alpha = c.a * a;
    if material.bevel.x > 0.0 {
        let texel = 0.5 / vec2<f32>(textureDimensions(field));
        let left = bevel_height(textureSample(field, field_sampler, uv - vec2(texel.x, 0.0)).a);
        let right = bevel_height(textureSample(field, field_sampler, uv + vec2(texel.x, 0.0)).a);
        // The field's rows run down here and up in the original's atlas.
        let up = bevel_height(textureSample(field, field_sampler, uv - vec2(0.0, texel.y)).a);
        let down = bevel_height(textureSample(field, field_sampler, uv + vec2(0.0, texel.y)).a);
        let k = BEVEL * material.bevel.z * GRADIENT_SCALE * -2.0;
        let n = normalize(vec3(-(right - left) * k, (up - down) * k, 1.0));
        let light = normalize(vec3(sin(LIGHT_ANGLE), cos(LIGHT_ANGLE), -1.0));
        rgb = rgb * (1.0 - dot(n, light) * DIFFUSE) * mix(AMBIENT, 1.0, n.z * n.z);
    }
    if material.glow.a > 0.0 {
        let g = material.glow_shape.x - d;
        let t = select(material.glow_shape.y, material.glow_shape.z, g >= 0.0);
        let v = clamp(abs(g) / (t + fwidth(d)), 0.0, 1.0);
        let k = clamp(material.glow.a * (1.0 - pow(v, material.glow_shape.w)) * 2.0, 0.0, 1.0);
        rgb = rgb + material.glow.rgb * k;
        alpha = max(alpha, k);
    }
    return vec4(rgb, alpha);
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
    pub face: Vec4,
    pub glow: Vec4,
    pub glow_shape: Vec4,
    pub bevel: Vec4,
}

impl bevy::ui_render::prelude::UiMaterial for SdfLetter {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        bevy::shader::ShaderRef::Handle(SDF_SHADER)
    }

    fn specialize(
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _key: bevy::ui_render::prelude::UiMaterialKey<Self>,
    ) {
        if let Some(target) = descriptor.fragment.as_mut().and_then(|f| f.targets.first_mut()).and_then(|t| t.as_mut()) {
            target.blend = Some(bevy::render::render_resource::BlendState::PREMULTIPLIED_ALPHA_BLENDING);
        }
    }
}

/// A TextMeshPro material's glow (GLOW_ON), in its own units: colour, face dilate, glow
/// offset, inner and outer reach, power, and the material's scale ratios A and B.
#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub struct TextGlow {
    pub color: Color,
    pub dilate: f32,
    pub offset: f32,
    pub inner: f32,
    pub outer: f32,
    pub power: f32,
    pub ratio_a: f32,
    pub ratio_b: f32,
}

impl TextGlow {
    /// EnzoBlackMainMenuImage (the title's "?"): a grey glow, dilate 0.13, offset 0.01, inner
    /// 0.317, outer 0.197, power 0.64.
    pub const MAIN_MENU_IMAGE: Self = Self {
        color: Color::srgb(0.44705883, 0.41960785, 0.41960785),
        dilate: 0.13,
        offset: 0.01,
        inner: 0.317,
        outer: 0.197,
        power: 0.64,
        ratio_a: 0.8333333,
        ratio_b: 0.72499996,
    };

    /// The shader's colour and shape: field units are half the material's, scaled by its ratios.
    fn shader(&self) -> (Vec4, Vec4) {
        (
            Vec4::from_array(self.color.to_linear().to_f32_array()),
            Vec4::new(
                0.5 - 0.5 * self.dilate * self.ratio_a - 0.5 * self.offset * self.ratio_b,
                0.5 * self.inner,
                0.5 * self.outer * self.ratio_b,
                self.power,
            ),
        )
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
    /// The material's BEVEL_ON lighting (every MilkyWhite and White Enzo material shares it).
    pub bevel: bool,
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
        bevel: false,
    };
    /// EnzoBlackRed: red at the top to pink at the bottom (BACK).
    pub const PINK: Self = Self {
        body: false,
        face: Some(Color::srgb(239.0 / 255.0, 43.0 / 255.0, 97.0 / 255.0)),
        outline: Some(Color::srgb(0.145, 0.051, 0.157)),
        low: Some(Color::srgb(1.0, 41.0 / 255.0, 182.0 / 255.0)),
        // Dilate and outline 0.25 (EnzoBlackRed46).
        outline_edge: 0.5 - 0.5 * 0.8333 / 2.0,
        bevel: false,
    };
    /// EnzoBlackRed60: the red gradient, dilate and outline 0.2 (the trials' LOCKED).
    pub const PINK_60: Self = Self { outline_edge: 0.5 - 0.4 * 0.8333 / 2.0, ..Self::PINK };
    /// EnzoBlackSilver46: gph_gradientTextureFont05, pale blue at the top to lavender at the
    /// bottom, dilate and outline 0.25.
    pub const SILVER: Self = Self {
        face: Some(Color::srgb(208.0 / 255.0, 222.0 / 255.0, 241.0 / 255.0)),
        low: Some(Color::srgb(138.0 / 255.0, 135.0 / 255.0, 222.0 / 255.0)),
        ..Self::PINK
    };
    /// EnzoBlackGold75: the gold gradient with a thinner outline (dilate and outline 0.15).
    pub const GOLD_75: Self = Self { outline_edge: 0.5 - 0.3 * 0.8333 / 2.0, ..Self::GOLD };
    /// EnzoBlackRed75: the red gradient, dilate and outline 0.15.
    pub const PINK_75: Self = Self { outline_edge: 0.5 - 0.3 * 0.8333 / 2.0, ..Self::PINK };
    /// EnzoBlackWhite32: face (234, 238, 255), dark outline, dilate and outline 0.3.
    pub const WHITE_32: Self = Self {
        body: false,
        face: Some(Color::srgb(0.918, 0.933, 1.0)),
        outline: Some(Color::srgb(0.169, 0.035, 0.137)),
        low: None,
        outline_edge: 0.5 - 0.6 * 0.8333 / 2.0,
        bevel: false,
    };
    /// EnzoBlackBlackThickOutline: a dark face in a pale blue outline (182, 197, 254),
    /// dilate and outline 0.3.
    pub const DARK_THICK: Self = Self {
        body: false,
        face: Some(Color::srgb(0.145, 0.051, 0.157)),
        outline: Some(Color::srgb(0.714, 0.773, 0.996)),
        low: None,
        outline_edge: 0.5 - 0.6 * 0.8333 / 2.0,
        bevel: false,
    };
    /// List rows and buttons: near-black body letters.
    pub const BODY: Self = Self {
        body: true,
        face: Some(Color::srgb(0.07, 0.055, 0.12)),
        outline: None,
        low: None,
        outline_edge: 0.0,
        bevel: false,
    };
    /// Screen titles: milky white display letters with a dark outline
    /// (the original's EnzoBlackMilkyWhiteOutline26): its face (182, 197, 254) lit by the
    /// material's bevel, which brightens the flat face to the (208, 224, 253) of Max's title clip.
    pub const TITLE: Self = Self {
        body: false,
        face: Some(Color::srgb(0.7137255, 0.77254903, 0.99607843)),
        outline: Some(Color::srgb(0.145, 0.051, 0.157)),
        low: None,
        outline_edge: 0.0,
        bevel: true,
    };
    /// Display letters in their own colour with the dark outline (button hints, tiles).
    pub const OUTLINED: Self = Self {
        body: false,
        face: None,
        outline: Some(Color::srgb(0.145, 0.051, 0.157)),
        low: None,
        outline_edge: 0.0,
        bevel: false,
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
                follow_transforms,
                redraw_when_fields_load,
                draw_letters,
            )
                .chain()
                .before(bevy::ui::UiSystems::Prepare)
                // UiSystems::Stack isn't chained with the others: letters drawn after it had
                // no place in the stack for a frame and showed under everything (the count
                // flashed behind its card).
                .before(bevy::ui::UiSystems::Stack),
        );
    }
}

/// On a hidden `Text`: the node that draws it with the original's letters.
#[derive(Component)]
pub(crate) struct MirroredBy(pub(crate) Entity);

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
    /// Lines lined up on their right ends (TextMeshPro's Right alignment).
    right: bool,
    body: bool,
    outline: Option<Color>,
    low: Option<Color>,
    outline_edge: f32,
    glow: Option<TextGlow>,
    bevel: bool,
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
    glow: Option<&TextGlow>,
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
    // A face or outline colour from the look still fades with the TextColor's alpha.
    let fade = color.map_or(1.0, |c| c.0.alpha());
    let faded = |c: Color| c.with_alpha(c.alpha() * fade);
    LetterText {
        spacing,
        text: words,
        size,
        view_height,
        color: look.face.map(faded).unwrap_or(color.map_or(Color::WHITE, |c| c.0)),
        centered: layout.is_some_and(|l| l.justify == Justify::Center),
        right: layout.is_some_and(|l| l.justify == Justify::Right),
        body: look.body,
        outline: look.outline.map(faded),
        low: look.low.map(faded),
        outline_edge: look.outline_edge,
        glow: glow.copied(),
        bevel: look.bevel,
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
            Option<&TextGlow>,
        ),
        Without<MirroredBy>,
    >,
    children: Query<&Children>,
) {
    if !art.letters.contains_key(FONT) {
        return;
    }
    for (entity, text, font, color, layout, look, mut node, parent, z, spacing, fit, glow) in &mut texts {
        let mirror = commands
            .spawn((
                node.clone(),
                letter_text(text, font, color, layout, look, spacing, fit, glow, &translations, display_sheet(&art)),
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
            Option<&TextGlow>,
        ),
        Or<(
            Changed<Text>,
            Changed<TextColor>,
            Changed<TextFont>,
            Changed<TextLook>,
            Changed<LetterSpacing>,
            Changed<AutoSize>,
            Changed<TextGlow>,
        )>,
    >,
    mut mirrors: Query<&mut LetterText>,
) {
    for (mirror, text, font, color, layout, look, spacing, fit, glow) in &texts {
        if let Ok(mut letters) = mirrors.get_mut(mirror.0) {
            let new = letter_text(text, font, color, layout, look, spacing, fit, glow, &translations, display_sheet(&art));
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

/// Turns and scales each twin with its text (the HUD labels' pops).
fn follow_transforms(
    texts: Query<(&MirroredBy, &UiTransform), Changed<UiTransform>>,
    mut twins: Query<&mut UiTransform, Without<MirroredBy>>,
) {
    for (mirror, transform) in &texts {
        if let Ok(mut twin) = twins.get_mut(mirror.0)
            && *twin != *transform
        {
            *twin = *transform;
        }
    }
}

/// Moves each twin with its text: HUD labels (place, counts) are placed every frame
/// after the twin was made, and a twin left behind showed at its first spot.
fn follow_nodes(
    mut texts: Query<(&MirroredBy, &mut Node), Changed<Node>>,
    mut twins: Query<&mut Node, Without<MirroredBy>>,
) {
    for (mirror, mut node) in &mut texts {
        // A text whose whole Node was replaced (the HUD rebuilds its items) would show
        // itself again beside its twin.
        if node.display != Display::None {
            node.display = Display::None;
        }
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

/// Letter materials already made, by their look and field.
#[derive(Default)]
struct SdfCache(std::collections::HashMap<Vec<u32>, Handle<SdfLetter>>);

fn draw_letters(
    mut commands: Commands,
    art: Res<Art>,
    images: Option<Res<Assets<Image>>>,
    mut sdf: Option<ResMut<Assets<SdfLetter>>>,
    texts: Query<(Entity, &LetterText), Changed<LetterText>>,
    mut cache: Local<SdfCache>,
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
                } else if text.right {
                    AlignItems::FlexEnd
                } else {
                    AlignItems::FlexStart
                },
                ..default()
            };
            lines.spawn(column).with_children(|lines| {
                for line in text.text.split('\n') {
                    spawn_line(
                        lines,
                        sheet,
                        text,
                        line,
                        field.clone().zip(sdf.as_deref_mut().map(|s| (s, &mut *cache))),
                    );
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
    mut field: Option<((Handle<Image>, Vec2), (&mut Assets<SdfLetter>, &mut SdfCache))>,
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
                    let mut from_field = |glyph_rect: (f32, f32, f32, f32),
                                          pad: f32,
                                          color: Color,
                                          edge: f32,
                                          low: Option<Color>,
                                          with_face: bool| {
                        let ((handle, size), (sdf, cache)) = field.as_mut()?;
                        let linear = |c: Color| Vec4::from_array(c.to_linear().to_f32_array());
                        let handle_hash = {
                            use std::hash::{Hash, Hasher};
                            let mut h = std::collections::hash_map::DefaultHasher::new();
                            handle.id().hash(&mut h);
                            h.finish() as u32
                        };
                        // The look and cache key for one glyph's rectangle in this style.
                        let make = |(gx, gy, gw, gh): (f32, f32, f32, f32)| {
                            let rect = (gx - pad, gy - pad, gw + 2.0 * pad, gh + 2.0 * pad);
                            let full = rect.3.max(0.001);
                            let face = if with_face { (linear(text.color), pad / full, gh / full) } else { (Vec4::ZERO, 0.0, 1.0) };
                            let low = low.map_or(Vec4::ZERO, linear);
                            // The glow goes on the node that draws the face.
                            let (glow, glow_shape) = text
                                .glow
                                .filter(|_| with_face || text.outline.is_none())
                                .map_or((Vec4::ZERO, Vec4::ZERO), |g| g.shader());
                            // BEVEL_ON on the node that draws the face: the material's weight (its
                            // dilate, taken as equal to its outline width as in most of them)
                            // plus the bevel offset 0.077, and the outline width plus 0.311.
                            let bevel = if text.bevel && (with_face || text.outline.is_none()) {
                                let edge = if text.outline_edge > 0.0 { text.outline_edge } else { OUTLINE_EDGE };
                                let width = (0.5 - edge) / 0.8333;
                                Vec4::new(1.0, width * 0.8333 * 0.5 + 0.077, (width + 0.311).max(0.01), 0.0)
                            } else {
                                Vec4::ZERO
                            };
                            let look = SdfLook {
                                color: linear(color),
                                rect: Vec4::new(rect.0 / size.x, rect.1 / size.y, rect.2 / size.x, rect.3 / size.y),
                                edge: Vec4::new(edge, face.1, face.2, FACE_EDGE),
                                low,
                                face: face.0,
                                glow,
                                glow_shape,
                                bevel,
                            };
                            let key: Vec<u32> = [look.color, look.rect, look.edge, look.low, look.face, look.glow, look.glow_shape, look.bevel]
                                .iter()
                                .flat_map(|v| v.to_array())
                                .map(f32::to_bits)
                                .chain([handle_hash])
                                .collect();
                            (key, look)
                        };
                        // A letter drawn before reuses its material: a new one isn't ready to draw
                        // for a frame, so a changing count showed only some of its layers.
                        let (key, look) = make(glyph_rect);
                        if !cache.0.contains_key(&key) && ch.is_ascii_digit() {
                            // First digit in this style: make 0-9 now so the next count doesn't blink.
                            for d in sheet.glyphs.iter().filter(|(c, _)| c.is_ascii_digit()).map(|(_, g)| g.rect) {
                                if d.2 > 0.0 && d.3 > 0.0 {
                                    let (dk, dl) = make(d);
                                    cache.0.entry(dk).or_insert_with(|| sdf.add(SdfLetter { look: dl, field: handle.clone() }));
                                }
                            }
                        }
                        let material = cache
                            .0
                            .entry(key)
                            .or_insert_with(|| sdf.add(SdfLetter { look, field: handle.clone() }))
                            .clone();
                        Some(bevy::ui_render::prelude::MaterialNode(material))
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
                        // The face goes in the same node: two nodes rounded to pixels apart left
                        // the outline thin on the top left and thick on the bottom right.
                        let face = (text.color.alpha() > 0.0).then_some(());
                        match from_field(
                            (x, y, w, h),
                            p,
                            color,
                            if text.outline_edge > 0.0 { text.outline_edge } else { OUTLINE_EDGE },
                            face.and(text.low),
                            face.is_some(),
                        ) {
                            Some(material) => {
                                cell.spawn((material, place));
                                if face.is_some() {
                                    return;
                                }
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
                    match from_field((x, y, w, h), 0.0, text.color, FACE_EDGE, text.low, false) {
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
