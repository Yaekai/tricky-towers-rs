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
        app.add_systems(
            PostUpdate,
            (mirror_texts, sync_mirrors, tidy_mirrors, follow_nodes, redraw_when_fields_load, draw_letters)
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

fn letter_text(
    text: &Text,
    font: &TextFont,
    color: Option<&TextColor>,
    layout: Option<&TextLayout>,
    look: Option<&TextLook>,
    spacing: Option<&LetterSpacing>,
) -> LetterText {
    let look = look.copied().unwrap_or_default();
    let (size, view_height) = match font.font_size {
        FontSize::Px(px) => (px, false),
        FontSize::Vh(vh) => (vh, true),
        _ => (20.0, false),
    };
    LetterText {
        spacing: spacing.map_or(0.0, |s| s.0),
        text: text.0.clone(),
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
        ),
        Without<MirroredBy>,
    >,
    children: Query<&Children>,
) {
    if !art.letters.contains_key(FONT) {
        return;
    }
    for (entity, text, font, color, layout, look, mut node, parent, z, spacing) in &mut texts {
        let mirror = commands
            .spawn((
                node.clone(),
                letter_text(text, font, color, layout, look, spacing),
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
    texts: Query<
        (
            &MirroredBy,
            &Text,
            &TextFont,
            Option<&TextColor>,
            Option<&TextLayout>,
            Option<&TextLook>,
            Option<&LetterSpacing>,
        ),
        Or<(
            Changed<Text>,
            Changed<TextColor>,
            Changed<TextFont>,
            Changed<TextLook>,
        )>,
    >,
    mut mirrors: Query<&mut LetterText>,
) {
    for (mirror, text, font, color, layout, look, spacing) in &texts {
        if let Ok(mut letters) = mirrors.get_mut(mirror.0) {
            let new = letter_text(text, font, color, layout, look, spacing);
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
    let Some(sheet) = art.letters.get(FONT) else {
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
    let Some(main) = art.letters.get(FONT) else {
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
