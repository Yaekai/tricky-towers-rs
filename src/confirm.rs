//! The title's Exit Game asks first, like the original's ConfirmPopup ("Exit game",
//! "Are you sure you want to exit the game?", CONFIRM / CANCEL; MainMenuController). It opens
//! as every BasePopup does: the dark overlay fades in over 0.2 s, then the panel rises from a
//! screen below over 0.33 s (out-back); closing runs that backwards (in-back, then the fade).
//! CANCEL is picked to start with; CONFIRM quits at once.

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::art::Art;
use crate::popup::{BAR_SLICE, px, sliced};

pub struct ConfirmPlugin;

impl Plugin for ConfirmPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExitConfirm>()
            .add_systems(Startup, spawn_confirm)
            .add_systems(
                Update,
                (
                    confirm_input
                        .in_set(crate::menu::MenuSystems)
                        .before(crate::menu::main_menu_input),
                    show_confirm,
                ),
            );
    }
}

/// The popup: open, its picked button (0 CONFIRM, 1 CANCEL), when it opened and when it
/// began to close.
#[derive(Resource, Debug, Default)]
pub struct ExitConfirm {
    pub open: bool,
    pub selected: usize,
    at: f32,
    closing: Option<f32>,
}

impl ExitConfirm {
    pub fn show(&mut self, now: f32) {
        *self = Self {
            open: true,
            selected: 1,
            at: now,
            closing: None,
        };
    }
}

/// BasePopup's fade and move times.
const FADE: f32 = 0.2;
const MOVE: f32 = 0.33;
/// ConfirmPopup's Background (780 x 450, centred) and the list under it: PopupListItem rows
/// 60 px apart from the panel's bottom edge.
const PANEL: (f32, f32) = (780.0, 450.0);
const ROW: f32 = 60.0;
const ITEM: f32 = 30.0;
const BAR: (f32, f32) = (240.0, 28.0);
const ITEMS: [&str; 2] = ["Confirm", "Cancel"];
/// The panel's 9-slice borders (gph_popupBackground_0: left 238, right 240, top 38, bottom 55).
const PANEL_SLICE: [f32; 4] = [238.0, 240.0, 38.0, 55.0];

#[derive(Component)]
struct ConfirmRoot;
#[derive(Component)]
struct ConfirmPanel;
#[derive(Component)]
struct ConfirmBar(usize);
#[derive(Component)]
struct ConfirmWords(usize);

fn text(size: f32, value: &str, look: crate::letters::TextLook, justify: Justify) -> impl Bundle {
    (
        Text::new(value.to_uppercase()),
        TextFont {
            font_size: FontSize::Vh(size / 10.8),
            ..default()
        },
        TextColor(Color::WHITE),
        TextLayout::justify(justify),
        look,
    )
}

fn spawn_confirm(mut commands: Commands, art: Option<Res<Art>>) {
    let image = |name: &str| art.as_ref().and_then(|a| a.image(name));
    let bar = image("menu_item_selected");
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            // ponytail: ConfirmPopup darkens with two layers (gph_brick_guide at 0.9 and the
            // Modal (8, 5, 24) at 0.847); one dark layer covering about as much stands in.
            BackgroundColor(Color::NONE),
            GlobalZIndex(30),
            Visibility::Hidden,
            ConfirmRoot,
        ))
        .with_children(|root| {
            let mut panel = root.spawn((
                Node {
                    width: px(PANEL.0),
                    height: px(PANEL.1),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.87, 0.9, 1.0)),
                ConfirmPanel,
            ));
            if let Some(back) = image("popup_confirm_background") {
                panel.insert((sliced(back, PANEL_SLICE), BackgroundColor(Color::NONE)));
            }
            panel.with_children(|panel| {
                // ErrorTitle: EnzoBlackRed60 at 60 px, its 80 px box's top 43.84 px over the panel.
                panel
                    .spawn(Node {
                        position_type: PositionType::Absolute,
                        left: px((PANEL.0 - 740.35) / 2.0),
                        width: px(740.35),
                        top: px(-43.84),
                        height: px(80.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    })
                    .with_child(text(
                        60.0,
                        "Exit game",
                        crate::letters::TextLook::PINK_60,
                        Justify::Center,
                    ));
                // Description: EnzoBlackBlue at 24.55 px, centred in a 650 x 300 box 70 px down.
                panel
                    .spawn(Node {
                        position_type: PositionType::Absolute,
                        left: px((PANEL.0 - 650.0) / 2.0),
                        width: px(650.0),
                        top: px(70.0),
                        height: px(300.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_child(text(
                        24.55,
                        "Are you sure you want to exit the game?",
                        crate::letters::TextLook {
                            body: false,
                            face: Some(Color::srgb(0.263, 0.286, 0.380)),
                            outline: None,
                            low: None,
                            outline_edge: 0.0,
                            bevel: false,
                        },
                        Justify::Center,
                    ));
                for (i, item) in ITEMS.iter().enumerate() {
                    let mut row = panel.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: px((PANEL.0 - BAR.0) / 2.0),
                            width: px(BAR.0),
                            top: px(PANEL.1 + ROW * i as f32 + (ITEM - BAR.1) / 2.0),
                            height: px(BAR.1),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                        ConfirmBar(i),
                    ));
                    if let Some(bar) = &bar {
                        let mut node = sliced(bar.clone(), BAR_SLICE);
                        node.color = Color::NONE;
                        row.insert(node);
                    }
                    row.with_child((
                        text(36.0, item, crate::letters::TextLook::PINK, Justify::Center),
                        ConfirmWords(i),
                        crate::letters::LetterSpacing(-1.0),
                    ));
                }
            });
        });
}

/// The button under a cursor (top-left origin) in a window `window` big.
fn button_at(cursor: Vec2, window: Vec2) -> Option<usize> {
    let x = cursor.x / window.x * 1920.0;
    let y = cursor.y / window.y * 1080.0;
    let top = 540.0 + PANEL.1 / 2.0;
    ((x - 960.0).abs() <= BAR.0 / 2.0)
        .then(|| {
            (0..ITEMS.len()).find(|&i| (y - (top + ROW * i as f32 + ITEM / 2.0)).abs() <= ROW / 2.0)
        })
        .flatten()
}

#[allow(clippy::too_many_arguments)]
fn confirm_input(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    gamepads: Query<&Gamepad>,
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
    mut confirm: ResMut<ExitConfirm>,
    mut exit: MessageWriter<AppExit>,
    mut last_cursor: Local<Option<Vec2>>,
) {
    if !confirm.open || confirm.closing.is_some() {
        return;
    }
    let (mut up, mut down, mut pick) = (false, false, false);
    if let Some(keys) = keys {
        up |= keys.just_pressed(KeyCode::ArrowUp);
        down |= keys.just_pressed(KeyCode::ArrowDown);
        pick |= keys.any_just_pressed([KeyCode::Enter, KeyCode::Space, KeyCode::NumpadEnter]);
    }
    for pad in &gamepads {
        up |= pad.just_pressed(GamepadButton::DPadUp);
        down |= pad.just_pressed(GamepadButton::DPadDown);
        pick |= pad.just_pressed(crate::input::continue_back(pad).0);
    }
    if let Ok(window) = windows.single() {
        let cursor = window.cursor_position();
        let over = cursor.and_then(|c| button_at(c, window.size()));
        if let Some(i) = over
            && cursor != *last_cursor
        {
            confirm.selected = i;
        }
        *last_cursor = cursor;
        if let Some(i) = over
            && mouse
                .as_deref()
                .is_some_and(|m| m.just_pressed(MouseButton::Left))
        {
            confirm.selected = i;
            pick = true;
        }
    }
    if up || down {
        confirm.selected = 1 - confirm.selected.min(1);
    }
    if pick {
        if confirm.selected == 0 {
            exit.write(AppExit::Success);
        } else {
            confirm.closing = Some(time.elapsed_secs());
        }
    }
}

fn back_out(t: f32) -> f32 {
    EaseFunction::BackOut.sample_clamped(t)
}

#[allow(clippy::type_complexity)]
fn show_confirm(
    time: Res<Time>,
    mut confirm: ResMut<ExitConfirm>,
    mut root: Query<(&mut Visibility, &mut BackgroundColor), With<ConfirmRoot>>,
    mut panel: Query<&mut UiTransform, With<ConfirmPanel>>,
    mut bars: Query<(&ConfirmBar, Option<&mut ImageNode>)>,
    mut words: Query<
        (
            &ConfirmWords,
            &mut crate::letters::TextLook,
            &mut TextFont,
            &mut UiTransform,
        ),
        Without<ConfirmPanel>,
    >,
) {
    let now = time.elapsed_secs();
    // How far in it is: the overlay's alpha and the panel's rise, 0 to 1 each.
    let (fade, rise) = match confirm.closing {
        None => {
            let t = now - confirm.at;
            ((t / FADE).clamp(0.0, 1.0), back_out((t - FADE) / MOVE))
        }
        Some(at) => {
            let t = now - at;
            let rise = 1.0 - EaseFunction::BackIn.sample_clamped(t / MOVE);
            let fade = 1.0 - ((t - MOVE) / FADE).clamp(0.0, 1.0);
            if t >= MOVE + FADE {
                confirm.open = false;
                confirm.closing = None;
            }
            (fade, rise)
        }
    };
    for (mut v, mut colour) in &mut root {
        v.set_if_neq(crate::select::shown(confirm.open));
        let want = Color::srgba(8.0 / 255.0, 5.0 / 255.0, 24.0 / 255.0, 0.97 * fade);
        if colour.0 != want {
            colour.0 = want;
        }
    }
    if !confirm.open {
        return;
    }
    for mut at in &mut panel {
        let want = Val2::new(Val::Px(0.0), Val::Vh((1.0 - rise) * 100.0));
        if at.translation != want {
            at.translation = want;
        }
    }
    for (bar, image) in &mut bars {
        if let Some(mut image) = image {
            let want = if bar.0 == confirm.selected {
                Color::WHITE
            } else {
                Color::NONE
            };
            if image.color != want {
                image.color = want;
            }
        }
    }
    // PopupListItem: the picked one in EnzoBlackGold46 at 46 px, the other in EnzoBlackRed46 at
    // 36 px, 4 px up.
    for (item, mut look, mut font, mut lift) in &mut words {
        let (want_look, size, up) = if item.0 == confirm.selected {
            (crate::letters::TextLook::GOLD, 46.0, 0.0)
        } else {
            (crate::letters::TextLook::PINK, 36.0, 4.0)
        };
        look.set_if_neq(want_look);
        let want_size = FontSize::Vh(size / 10.8);
        if font.font_size != want_size {
            font.font_size = want_size;
        }
        let want_lift = Val2::new(Val::Px(0.0), Val::Vh(-up / 10.8));
        if lift.translation != want_lift {
            lift.translation = want_lift;
        }
    }
}
