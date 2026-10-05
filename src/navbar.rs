//! The button hints in the bottom-right corner of the menu screens, as the original
//! draws them (docs/research/gap-scan.md, section 8): a yellow "CONTINUE [key]" with a
//! red "BACK [key]" under it, 30 px in from the side and 50 px up from the bottom,
//! each item 400×50 and right aligned, more items stacking 60 px upward. The key is
//! the keyboard key on a key cap, or the pad button once a pad is connected.

use bevy::prelude::*;

use crate::art::Art;
use crate::gameplay::{Match, Phase};
use crate::menu::MenuScreen;
use crate::select::{SCREEN, place, shown};

pub struct NavBarPlugin;

impl Plugin for NavBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_nav_bar)
            .add_systems(Update, show_nav_bar);
    }
}

/// Layout from the readout, in 1920×1080 pixels from the screen centre (y up).
const ITEM: (f32, f32) = (400.0, 50.0);
const SIDE: f32 = 30.0;
const FOOT: f32 = 50.0;
const STEP: f32 = 60.0;
const TEXT_SIZE: f32 = 46.0;
/// The hint colours (?: "yellow" and "red" in the readout, exact values not read).
const CONTINUE_COLOR: Color = Color::srgb(1.0, 0.84, 0.2);
const BACK_COLOR: Color = Color::srgb(0.94, 0.27, 0.22);

/// One hint in the bar.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
enum NavItem {
    Continue,
    Back,
}

impl NavItem {
    fn words(self) -> &'static str {
        match self {
            NavItem::Continue => "CONTINUE",
            NavItem::Back => "BACK",
        }
    }

    fn color(self) -> Color {
        match self {
            NavItem::Continue => CONTINUE_COLOR,
            NavItem::Back => BACK_COLOR,
        }
    }

    /// The key that does it: keyboard, then pad.
    fn keys(self) -> (&'static str, &'static str) {
        match self {
            NavItem::Continue => ("Enter", "A"),
            NavItem::Back => ("Esc", "B"),
        }
    }
}

/// The hints a screen shows, bottom first. The start screen has its own prompt and
/// the title tiles have nothing to go back to.
fn items(screen: MenuScreen) -> &'static [NavItem] {
    match screen {
        MenuScreen::Start => &[],
        MenuScreen::Main => &[NavItem::Continue],
        _ => &[NavItem::Back, NavItem::Continue],
    }
}

#[derive(Component)]
struct NavBar;

/// The text on an item's key cap.
#[derive(Component)]
struct NavKey(NavItem);

/// An item's words, which change with the screen (SELECT, START GAME...).
#[derive(Component)]
struct NavWords(NavItem);

/// What the go-on hint says on a screen, as the original words it.
fn continue_words(screen: MenuScreen) -> &'static str {
    match screen {
        MenuScreen::Settings | MenuScreen::Single => "SELECT",
        MenuScreen::Trials | MenuScreen::Endless => "START GAME",
        _ => "CONTINUE",
    }
}

/// An item's place in the stack, bottom first.
#[derive(Component)]
struct NavSlot(usize);

fn vh(px: f32) -> f32 {
    px / SCREEN.1 * 100.0
}

fn spawn_nav_bar(mut commands: Commands, art: Option<Res<Art>>) {
    let cap = art.as_deref().and_then(|a| a.image("key_background"));
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            GlobalZIndex(13),
            Visibility::Hidden,
            Pickable::IGNORE,
            NavBar,
        ))
        .with_children(|bar| {
            for item in [NavItem::Continue, NavItem::Back] {
                bar.spawn((
                    Node {
                        justify_content: JustifyContent::FlexEnd,
                        column_gap: Val::Vh(vh(12.0)),
                        ..place(ITEM, (0.0, 0.0))
                    },
                    item,
                    // No slot yet, so the first show places it.
                    NavSlot(usize::MAX),
                ))
                .with_children(|row| {
                    row.spawn((
                        Text::new(item.words()),
                        NavWords(item),
                        crate::letters::TextLook::OUTLINED,
                        TextFont {
                            font_size: FontSize::Vh(vh(TEXT_SIZE)),
                            ..default()
                        },
                        TextColor(item.color()),
                    ));
                    let mut key = row.spawn(Node {
                        min_width: Val::Vh(vh(ITEM.1)),
                        height: Val::Vh(vh(ITEM.1)),
                        padding: UiRect::horizontal(Val::Vh(vh(8.0))),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    });
                    match &cap {
                        Some(image) => {
                            key.insert(ImageNode::new(image.clone()));
                        }
                        None => {
                            key.insert(BackgroundColor(Color::srgba(0.95, 0.95, 0.95, 0.9)));
                        }
                    }
                    key.with_child((
                        Text::new(item.keys().0),
                        TextFont {
                            // Small enough that "Enter" stays on its cap.
                            font_size: FontSize::Vh(vh(TEXT_SIZE * 0.4)),
                            ..default()
                        },
                        TextColor(Color::srgb(0.15, 0.1, 0.25)),
                        NavKey(item),
                    ));
                });
            }
        });
}

/// Shows the hints of the menu screen on show, stacked from the bottom, with the
/// pad's buttons once a pad is connected.
#[allow(clippy::type_complexity)]
fn show_nav_bar(
    game: Res<Match>,
    screen: Res<MenuScreen>,
    pads: Query<(), With<Gamepad>>,
    mut bar: Query<&mut Visibility, With<NavBar>>,
    mut rows: Query<(&NavItem, &mut NavSlot, &mut Node, &mut Visibility), Without<NavBar>>,
    mut keys: Query<(&NavKey, &mut Text), Without<NavWords>>,
    mut words: Query<(&NavWords, &mut Text), Without<NavKey>>,
) {
    let list = items(*screen);
    let on = game.phase == Phase::Menu && !list.is_empty();
    for mut visibility in &mut bar {
        visibility.set_if_neq(shown(on));
    }
    if !on {
        return;
    }
    for (item, mut slot, mut node, mut visibility) in &mut rows {
        let at = list.iter().position(|i| i == item);
        visibility.set_if_neq(shown(at.is_some()));
        let Some(at) = at else {
            continue;
        };
        if slot.0 != at {
            slot.0 = at;
            let x = SCREEN.0 / 2.0 - SIDE - ITEM.0 / 2.0;
            let y = -SCREEN.1 / 2.0 + FOOT + ITEM.1 / 2.0 + STEP * at as f32;
            let want = place(ITEM, (x, y));
            node.left = want.left;
            node.top = want.top;
        }
    }
    for (item, mut text) in &mut words {
        let want = match item.0 {
            NavItem::Continue => continue_words(*screen),
            other => other.words(),
        };
        if text.0 != want {
            text.0 = want.to_string();
        }
    }
    let pad = !pads.is_empty();
    for (key, mut text) in &mut keys {
        let (keyboard, button) = key.0.keys();
        let words = if pad { button } else { keyboard };
        if text.0 != words {
            text.0 = words.to_string();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_menu_but_the_start_shows_continue_and_back_below_it() {
        assert!(items(MenuScreen::Start).is_empty());
        assert_eq!(items(MenuScreen::Main), &[NavItem::Continue]);
        for screen in [
            MenuScreen::Players,
            MenuScreen::Modes,
            MenuScreen::Trials,
            MenuScreen::Settings,
        ] {
            assert_eq!(items(screen), &[NavItem::Back, NavItem::Continue]);
        }
    }
}
