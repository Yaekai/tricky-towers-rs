//! The Help screen behind the title's "?" tile (ExplanationOverviewPopup): over the modal
//! tile, seven pages in a row that slide sideways: the controls, then the three battles
//! with their difficulties and the three trials (InitExplanationsCommand's overview
//! explanations), page dots along the top and arrows either side.
//! Layout from ModeExplanationOverviewPopup, ModeExplanationItem, ExplanationOverlayPopup and
//! Pagination, in 1920 x 1080 pixels from the screen centre (y up).

use bevy::prelude::*;

use crate::art::Art;
use crate::config::{Difficulty, ModeKind};
use crate::gameplay::{Match, Phase};
use crate::letters::{LetterSpacing, TextLook};
use crate::menu::MenuScreen;
use crate::select::{SCREEN, place, read_taps, shown};

pub struct HelpPlugin;

impl Plugin for HelpPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HelpMenu>()
            .add_systems(Startup, spawn_help)
            .add_systems(
                Update,
                (help_input, show_help)
                    .chain()
                    .after(crate::menu::MenuSystems),
            );
    }
}

/// The page shown, the one it slid from and which way, and the slide's time.
#[derive(Resource, Default)]
pub struct HelpMenu {
    pub page: usize,
    from: Option<(usize, f32)>,
    time: f32,
}

/// One page: (title key, line key, picture, the mode whose difficulties it shows).
const PAGES: [(&str, &str, &str, Option<ModeKind>); 6] = [
    (
        "EXPLANATION_TITLE_MULTIPLAYER_RACE",
        "EXPLANATION_MULTIPLAYER_RACE",
        "menu_mode_race",
        Some(ModeKind::Race),
    ),
    (
        "EXPLANATION_TITLE_MULTIPLAYER_SURVIVAL",
        "EXPLANATION_MULTIPLAYER_SURVIVAL",
        "menu_mode_survivalsp",
        Some(ModeKind::Survival),
    ),
    (
        "EXPLANATION_TITLE_MULTIPLAYER_PUZZLE",
        "EXPLANATION_MULTIPLAYER_PUZZLE",
        "menu_mode_puzzle",
        Some(ModeKind::Puzzle),
    ),
    (
        "EXPLANATION_TITLE_SINGLE_PLAYER_RACE",
        "EXPLANATION_SINGLE_PLAYER_RACE",
        "menu_explanation_mode_race_sp",
        None,
    ),
    (
        "EXPLANATION_TITLE_SINGLE_PLAYER_SURVIVAL",
        "EXPLANATION_SINGLE_PLAYER_SURVIVAL",
        "menu_mode_survivalsp",
        None,
    ),
    (
        "EXPLANATION_TITLE_SINGLE_PLAYER_PUZZLE",
        "EXPLANATION_SINGLE_PLAYER_PUZZLE",
        "menu_mode_puzzle",
        None,
    ),
];
const PAGE_COUNT: usize = PAGES.len() + 1;

/// Pages slide a screen wide over this long (out-quad, as the scroll panels).
const SLIDE_TIME: f32 = 0.3;
/// The dots: 20 px apart, 60 px under the top (ExplanationPagination).
const DOT: f32 = 20.0;
const DOTS_Y: f32 = SCREEN.1 / 2.0 - 60.0;
/// The arrows, 300 px in from each side.
const ARROW: (f32, f32) = (94.0, 55.0);
const ARROW_X: f32 = SCREEN.0 / 2.0 - 300.0;
/// ModeImageContainer's middle; the mode pictures at their prefab sizes.
const PICTURE_Y: f32 = 81.0;
/// The difficulty columns: picture, its size, the column's x.
const COLUMNS: [(Difficulty, &str, (f32, f32), f32); 3] = [
    (
        Difficulty::Easy,
        "menu_difficulty_easy",
        (155.0, 115.0),
        -340.0,
    ),
    (
        Difficulty::Normal,
        "menu_difficulty_normal",
        (246.0, 131.0),
        0.0,
    ),
    (
        Difficulty::Pro,
        "menu_difficulty_pro",
        (278.0, 141.0),
        390.0,
    ),
];
/// The light and dark slots of each column, 55 px boxes with a 50 px icon.
const SLOT_X: [[f32; 2]; 3] = [[-375.0, -305.0], [-35.0, 35.0], [355.0, 425.0]];

/// The mode pictures' prefab sizes (ModeImage/Race, SurvivalExplanations, Puzzle,
/// RaceSinglePlayer).
pub(crate) fn picture_size(name: &str) -> (f32, f32) {
    match name {
        "menu_mode_survivalsp" => (482.0, 403.0),
        "menu_mode_puzzle" => (505.0, 392.0),
        "menu_explanation_mode_race_sp" => (497.0, 416.0),
        _ => (487.0, 414.0),
    }
}

#[derive(Component)]
struct HelpRoot;

#[derive(Component)]
struct HelpPage(usize);

#[derive(Component)]
struct HelpDot(usize);

fn words(size: f32, text: String, look: TextLook, spacing: f32) -> impl Bundle {
    (
        Text::new(text.to_uppercase()),
        look,
        TextFont { font_size: FontSize::Vh(size / 10.8), ..default() },
        TextColor(Color::WHITE),
        TextLayout::justify(Justify::Center),
        LetterSpacing(spacing),
    )
}

fn spawn_help(
    mut commands: Commands,
    art: Option<Res<Art>>,
    keys: Option<Res<crate::keybinding::KeyBindings>>,
) {
    let art = art.as_deref();
    let image = |name: &str| art.and_then(|a| a.image(name));
    let english = crate::credits::english();
    let say = |key: &str| english.get(key).cloned().unwrap_or_default();
    let defaults = crate::keybinding::KeyBindings::default();
    let full = || Node {
        position_type: PositionType::Absolute,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..default()
    };
    let mut root = commands.spawn((
        full(),
        BackgroundColor(Color::srgb(0.12, 0.1, 0.2)),
        GlobalZIndex(12),
        Visibility::Hidden,
        HelpRoot,
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
    root.with_children(|root| {
        // The controls page: the keyboard panel in the middle.
        root.spawn((full(), HelpPage(0), Visibility::Hidden, UiTransform::default()))
            .with_children(|page| {
                crate::menu::spawn_controls_panel(
                    page,
                    art,
                    keys.as_deref().unwrap_or(&defaults),
                    Some((SCREEN.0 / 2.0, SCREEN.1 / 2.0)),
                );
            });
        for (i, &(title, line, picture, mode)) in PAGES.iter().enumerate() {
            root.spawn((full(), HelpPage(i + 1), Visibility::Hidden, UiTransform::default()))
                .with_children(|page| {
                    page.spawn(place((1600.0, 100.0), (0.0, 410.0)))
                        .with_children(|t| {
                            t.spawn(words(75.0, say(title), TextLook::GOLD_75, -2.0));
                        });
                    page.spawn(place((1600.0, 50.0), (0.0, 333.5)))
                        .with_children(|t| {
                            t.spawn(words(26.0, say(line), TextLook::WHITE_32, -1.0));
                        });
                    let mut pic = page.spawn(place(picture_size(picture), (0.0, PICTURE_Y)));
                    if let Some(art) = image(picture) {
                        pic.insert(ImageNode::new(art));
                    }
                    // SurvivalExplanations' brick count over its picture.
                    if picture == "menu_mode_survivalsp" {
                        page.spawn(place((50.0, 60.0), (-24.0, PICTURE_Y + 110.0)))
                            .with_children(|t| {
                                t.spawn(words(46.0, "66".into(), TextLook::GOLD, 0.0));
                            });
                    }
                    let Some(mode) = mode else { return };
                    for (c, &(difficulty, art_name, size, x)) in COLUMNS.iter().enumerate() {
                        let mut pic = page.spawn(place(size, (x, -235.0)));
                        if let Some(art) = image(art_name) {
                            pic.insert(ImageNode::new(art));
                        }
                        let key = match difficulty {
                            Difficulty::Easy => "DIFFICULTY_EASY",
                            Difficulty::Normal => "DIFFICULTY_NORMAL",
                            Difficulty::Pro => "DIFFICULTY_PRO",
                        };
                        let width = if c == 0 { 275.0 } else { 373.6 };
                        page.spawn(place((width, 50.0), (x, -349.0)))
                            .with_children(|t| {
                                t.spawn(words(46.0, say(key), TextLook::DARK_THICK, 0.0));
                            });
                        if difficulty == Difficulty::Pro {
                            let key = title.trim_start_matches("EXPLANATION_TITLE_");
                            page.spawn(place((373.0, 50.0), (x, -391.0)))
                                .with_children(|t| {
                                    t.spawn(words(
                                        26.0,
                                        say(&format!("EXPLANATION_{key}_PRO_SUBTITLE")),
                                        TextLook::PINK_75,
                                        -1.0,
                                    ));
                                });
                        }
                        let icons = crate::explain::slots(mode, difficulty);
                        for (s, back) in ["spell_slot_light", "spell_slot_dark"].iter().enumerate()
                        {
                            let mut slot = page.spawn(place((55.0, 55.0), (SLOT_X[c][s], -450.0)));
                            if let Some(art) = image(back) {
                                slot.insert(ImageNode::new(art));
                            }
                            if let Some(icon) = icons[s].and_then(image) {
                                slot.with_child((
                                    ImageNode::new(icon),
                                    Node {
                                        width: Val::Percent(50.0 / 55.0 * 100.0),
                                        height: Val::Percent(50.0 / 55.0 * 100.0),
                                        ..default()
                                    },
                                ));
                            }
                        }
                    }
                });
        }
        // The page dots: every page's off dot, the current page's on dot over it.
        let on = image("page_dot_on").or_else(|| image("menu_pagination_active"));
        let off = image("page_dot_off");
        for i in 0..PAGE_COUNT {
            let x = (i as f32 - (PAGE_COUNT - 1) as f32 / 2.0) * DOT;
            let mut dot = root.spawn(place((DOT, DOT), (x, DOTS_Y)));
            match &off {
                Some(art) => dot.insert(ImageNode::new(art.clone())),
                None => dot.insert(BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.4))),
            };
            let mut lit = root.spawn((place((DOT, DOT), (x, DOTS_Y)), HelpDot(i)));
            match &on {
                Some(art) => lit.insert(ImageNode::new(art.clone())),
                None => lit.insert(BackgroundColor(Color::WHITE)),
            };
        }
        for right in [false, true] {
            let x = if right { ARROW_X } else { -ARROW_X };
            let mut arrow = root.spawn(place(ARROW, (x, 0.0)));
            if let Some(art) = image("menu_arrow") {
                // The staged arrow points up; these point out sideways.
                arrow.insert((
                    ImageNode::new(art),
                    UiTransform::from_rotation(Rot2::degrees(if right { 90.0 } else { -90.0 })),
                ));
            }
        }
    });
}

fn help_input(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    pads: Query<(Entity, &Gamepad)>,
    game: Res<Match>,
    mut screen: ResMut<MenuScreen>,
    mut help: ResMut<HelpMenu>,
) {
    if game.phase != Phase::Menu || *screen != MenuScreen::Help {
        return;
    }
    let taps = read_taps(keys.as_deref(), &pads);
    if !taps.pad_back.is_empty() {
        *screen = MenuScreen::Main;
        return;
    }
    let step = taps.right as i32 - taps.left as i32;
    if step != 0 {
        let from = help.page;
        help.page = (help.page as i32 + step).rem_euclid(PAGE_COUNT as i32) as usize;
        help.from = Some((from, step as f32));
        help.time = 0.0;
    }
}

fn show_help(
    time: Res<Time>,
    game: Res<Match>,
    screen: Res<MenuScreen>,
    mut help: ResMut<HelpMenu>,
    mut root: Query<&mut Visibility, With<HelpRoot>>,
    mut pages: Query<(&HelpPage, &mut Visibility, &mut UiTransform), Without<HelpRoot>>,
    mut dots: Query<(&HelpDot, &mut Visibility), (Without<HelpRoot>, Without<HelpPage>)>,
) {
    let on = game.phase == Phase::Menu && *screen == MenuScreen::Help;
    for mut v in &mut root {
        v.set_if_neq(shown(on));
    }
    if !on {
        // It opens on the controls each time.
        *help = HelpMenu::default();
        return;
    }
    help.time += time.delta_secs();
    let t = (help.time / SLIDE_TIME).min(1.0);
    let eased = 1.0 - (1.0 - t) * (1.0 - t);
    let from = help.from.filter(|_| t < 1.0);
    for (page, mut v, mut transform) in &mut pages {
        let x = if page.0 == help.page {
            from.map_or(0.0, |(_, dir)| dir * SCREEN.0 * (1.0 - eased))
        } else if from.is_some_and(|(f, _)| f == page.0) {
            -from.map_or(0.0, |(_, dir)| dir) * SCREEN.0 * eased
        } else {
            v.set_if_neq(Visibility::Hidden);
            continue;
        };
        v.set_if_neq(Visibility::Inherited);
        transform.translation = Val2::new(Val::Vw(x / SCREEN.0 * 100.0), Val::Px(0.0));
    }
    for (dot, mut v) in &mut dots {
        v.set_if_neq(shown(dot.0 == help.page));
    }
}
