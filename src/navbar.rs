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

    /// Drawn with the original's letters: CONTINUE in EnzoBlackGold, BACK in EnzoBlackRed,
    /// both gradient faces.
    fn look(self) -> crate::letters::TextLook {
        match self {
            NavItem::Continue => crate::letters::TextLook::GOLD,
            NavItem::Back => crate::letters::TextLook::PINK,
        }
    }

    fn color(self) -> Color {
        match self {
            NavItem::Continue => CONTINUE_COLOR,
            NavItem::Back => BACK_COLOR,
        }
    }

    /// The key that does it: keyboard, then pad.
    /// Its glyph art (the original's TextMeshPro key sprites): keyboard, then pad.
    fn glyphs(self) -> (&'static str, &'static str) {
        match self {
            NavItem::Continue => ("menu_key_space", "menu_pad_a"),
            NavItem::Back => ("menu_key_esc", "menu_pad_b"),
        }
    }

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
        // The pause screen shows its controls instead of hints.
        MenuScreen::Start | MenuScreen::Pause | MenuScreen::Explain => &[],
        MenuScreen::Main => &[NavItem::Continue],
        _ => &[NavItem::Back, NavItem::Continue],
    }
}

#[derive(Component)]
struct NavBar;

/// The text on an item's key cap.
#[derive(Component)]
struct NavKey(NavItem);

/// An item's key glyph, when the original's are staged (instead of the cap).
#[derive(Component)]
struct NavGlyph(NavItem);

/// An item's words, which change with the screen (SELECT, START GAME...).
#[derive(Component)]
struct NavWords(NavItem);

/// What the go-on hint says on a screen, as the original words it: on Select Mode NEXT
/// until the last column, then START GAME (Max's clips).
fn continue_words(screen: MenuScreen, column: usize, on_list: bool) -> &'static str {
    match screen {
        MenuScreen::Single if on_list => "CONTINUE",
        MenuScreen::Settings | MenuScreen::Single => "SELECT",
        MenuScreen::Trials | MenuScreen::Endless => "START GAME",
        MenuScreen::Modes if column >= 2 => "START GAME",
        MenuScreen::Modes => "NEXT",
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
                        // MenuItemYellow / MenuItemRed space their words -1.
                        crate::letters::LetterSpacing(-1.0),
                        item.look(),
                        TextFont {
                            font_size: FontSize::Vh(vh(TEXT_SIZE)),
                            ..default()
                        },
                        TextColor(item.color()),
                    ));
                    if let Some(glyph) = art.as_deref().and_then(|a| a.image(item.glyphs().0)) {
                        row.spawn((
                            ImageNode::new(glyph),
                            Node {
                                height: Val::Vh(vh(ITEM.1)),
                                ..default()
                            },
                            NavGlyph(item),
                        ));
                        return;
                    }
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
    pads: Query<&Gamepad>,
    mut bar: Query<&mut Visibility, With<NavBar>>,
    mut rows: Query<(&NavItem, &mut NavSlot, &mut Node, &mut Visibility), Without<NavBar>>,
    mut keys: Query<(&NavKey, &mut Text), Without<NavWords>>,
    mut words: Query<(&NavWords, &mut Text), Without<NavKey>>,
    mut glyphs: Query<(&NavGlyph, &mut ImageNode)>,
    art: Option<Res<Art>>,
    stats: Option<Res<crate::stats::StatsMenu>>,
    credits: Option<Res<crate::credits::CreditsMenu>>,
    lobby: Option<Res<crate::select::Lobby>>,
    single: Option<Res<crate::single::SingleMenu>>,
) {
    let column = lobby.as_ref().map_or(0, |l| l.column);
    // Single Player says CONTINUE once a list button is picked, SELECT before.
    let on_list = single.as_ref().is_some_and(|m| m.step >= 2);
    // Select Players offers CONTINUE only once two players have joined (canStartGame).
    let few = *screen == MenuScreen::Players && !lobby.as_ref().is_some_and(|l| l.can_start());
    // The Stats popup has only its BACK; the credits roll has none.
    let list = if credits.is_some_and(|c| c.open) {
        &[]
    } else if stats.is_some_and(|s| s.open) {
        &[NavItem::Back]
    } else if few {
        &[NavItem::Back]
    } else {
        items(*screen)
    };
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
            NavItem::Continue => continue_words(*screen, column, on_list),
            other => other.words(),
        };
        if text.0 != want {
            text.0 = want.to_string();
        }
    }
    let pad = !pads.is_empty();
    let playstation = pads.iter().next().is_some_and(crate::input::is_playstation);
    for (glyph, mut image) in &mut glyphs {
        let (keyboard, button) = glyph.0.glyphs();
        let button = match (playstation, glyph.0) {
            (true, NavItem::Continue) => "menu_pad_cross",
            (true, NavItem::Back) => "menu_pad_circle",
            _ => button,
        };
        if let Some(want) = art
            .as_deref()
            .and_then(|a| a.image(if pad { button } else { keyboard }))
            && image.image != want
        {
            image.image = want;
        }
    }
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
