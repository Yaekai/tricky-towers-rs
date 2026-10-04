//! What the player sees of cups: the cup picker on Select Mode (C or the pad's
//! Select button cycles single match → each cup → single match), the standings
//! line while a cup runs, and the cup winner's trophy popping in at the end.
//! Layout and wording are placeholders until the original's are read (?).

use bevy::prelude::*;

use crate::art::Art;
use crate::config::GameConfig;
use crate::cups::Cup;
use crate::gameplay::{Match, Phase, Tower};
use crate::menu::MenuScreen;
use crate::select::{label, place, shown};

pub struct CupsMenuPlugin;

impl Plugin for CupsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_cup_ui)
            .add_systems(Update, (pick_cup, show_cup_ui).chain());
    }
}

#[derive(Component)]
struct CupPickLabel;
#[derive(Component)]
struct CupStandings;
#[derive(Component)]
struct CupTrophy;
#[derive(Component)]
struct CupTrophyText;

fn spawn_cup_ui(mut commands: Commands, art: Option<Res<Art>>) {
    // The picker line, under the Select Mode header.
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            GlobalZIndex(12),
            Visibility::Hidden,
            CupPickLabel,
        ))
        .with_children(|root| {
            root.spawn(place((900.0, 50.0), (0.0, 400.0)))
                .with_children(|line| {
                    line.spawn(label(2.8, ""));
                });
        });
    // The standings line at the top of a match while a cup runs.
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Vh(1.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        GlobalZIndex(19),
        Visibility::Hidden,
        CupStandings,
        children![label(2.6, "")],
    ));
    // The trophy for the cup winner, centred under WINNER!.
    let trophy_image = art.as_ref().and_then(|a| a.image("trophy"));
    let mut trophy = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(45.0),
            width: Val::Vh(20.0),
            height: Val::Vh(20.0),
            margin: UiRect::left(Val::Vh(-10.0)),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexEnd,
            align_items: AlignItems::Center,
            ..default()
        },
        UiTransform::default(),
        GlobalZIndex(20),
        Visibility::Hidden,
        CupTrophy,
    ));
    match trophy_image {
        Some(image) => trophy.insert(ImageNode::new(image)),
        None => trophy.insert(BackgroundColor(Color::srgb(0.95, 0.75, 0.2))),
    };
    trophy.with_children(|t| {
        t.spawn((
            label(2.4, ""),
            CupTrophyText,
            Node {
                margin: UiRect::bottom(Val::Vh(-4.0)),
                ..default()
            },
        ));
    });
}

/// C or the pad's Select button on Select Mode cycles the cup.
fn pick_cup(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    pads: Query<&Gamepad>,
    game: Res<Match>,
    screen: Res<MenuScreen>,
    config: Res<GameConfig>,
    mut cup: ResMut<Cup>,
) {
    if game.phase != Phase::Menu || *screen != MenuScreen::Modes {
        return;
    }
    let pressed = keys.is_some_and(|k| k.just_pressed(KeyCode::KeyC))
        || pads.iter().any(|p| p.just_pressed(GamepadButton::Select));
    if !pressed {
        return;
    }
    let count = config.cups.sizes.len();
    cup.pick = match cup.pick {
        None if count > 0 => Some(0),
        Some(i) if i + 1 < count => Some(i + 1),
        _ => None,
    };
}

fn player_name(towers: &Query<&Tower>, index: usize) -> String {
    match towers
        .iter()
        .find(|t| t.index == index)
        .and_then(|t| t.player)
    {
        Some(p) => format!("Player {}", p + 1),
        None => format!("Wizard {}", index + 1),
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn show_cup_ui(
    time: Res<Time>,
    game: Res<Match>,
    screen: Res<MenuScreen>,
    config: Res<GameConfig>,
    cup: Res<Cup>,
    towers: Query<&Tower>,
    mut roots: ParamSet<(
        Query<(&mut Visibility, &Children), With<CupPickLabel>>,
        Query<(&mut Visibility, &Children), With<CupStandings>>,
        Query<(&mut Visibility, &mut UiTransform, Option<&mut ImageNode>), With<CupTrophy>>,
    )>,
    lines: Query<&Children>,
    mut texts: Query<&mut Text>,
    trophy_text: Query<Entity, With<CupTrophyText>>,
    art: Option<Res<Art>>,
) {
    let set = |texts: &mut Query<&mut Text>, entity: Entity, words: String| {
        if let Ok(mut text) = texts.get_mut(entity)
            && text.0 != words
        {
            text.0 = words;
        }
    };
    // The picker.
    let picking = game.phase == Phase::Menu && *screen == MenuScreen::Modes;
    let pick_words = match cup.pick.and_then(|i| config.cups.sizes.get(i)) {
        Some(size) => format!(
            "{}: {} matches  (C / Select to change)",
            size.name, size.matches
        ),
        None => "Single match  (C / Select for a cup)".to_string(),
    };
    for (mut visibility, children) in &mut roots.p0() {
        visibility.set_if_neq(shown(picking));
        for line in children.iter() {
            for text in lines.get(line).into_iter().flat_map(|c| c.iter()) {
                set(&mut texts, text, pick_words.clone());
            }
        }
    }
    // The standings while a cup runs (and on its last result).
    let in_match = matches!(game.phase, Phase::Playing | Phase::Over);
    let cup_rules = cup.rules(&config.cups);
    let standing_words = cup_rules.map(|size| {
        let mut order: Vec<usize> = (0..cup.points.len()).collect();
        order.sort_by(|&a, &b| cup.points[b].cmp(&cup.points[a]).then(a.cmp(&b)));
        let scores: Vec<String> = order
            .iter()
            .map(|&t| format!("{} {}", player_name(&towers, t), cup.points[t]))
            .collect();
        let round = (cup.played + u32::from(game.phase == Phase::Playing)).min(size.matches);
        format!(
            "{}  match {round}/{}   {}",
            size.name,
            size.matches,
            scores.join("   ")
        )
    });
    for (mut visibility, children) in &mut roots.p1() {
        visibility.set_if_neq(shown(in_match && standing_words.is_some()));
        for text in children.iter() {
            set(&mut texts, text, standing_words.clone().unwrap_or_default());
        }
    }
    // The trophy once the cup is over.
    let winner = cup.standings.as_ref().and_then(|s| s.first().copied());
    let show_trophy = game.phase == Phase::Over && winner.is_some();
    // Each cup size has its own trophy picture.
    let picture = cup_rules.and_then(|size| art.as_ref().and_then(|a| a.image(&size.trophy)));
    for (mut visibility, mut transform, image) in &mut roots.p2() {
        if let (Some(mut node), Some(picture)) = (image, picture.clone())
            && node.image != picture
        {
            node.image = picture;
        }
        if !show_trophy {
            visibility.set_if_neq(Visibility::Hidden);
            transform.scale = Vec2::splat(config.moments.trophy_pop.0);
            continue;
        }
        visibility.set_if_neq(Visibility::Inherited);
        let (from, over) = config.moments.trophy_pop;
        let next = (transform.scale.x + (1.0 - from) * time.delta_secs() / over.max(0.01)).min(1.0);
        transform.scale = Vec2::splat(next);
    }
    if let (Some(winner), Some(size)) = (winner, cup_rules) {
        let xp = cup.xp.get(winner).copied().unwrap_or(0);
        let words = format!(
            "{} wins the {}!  +{xp} XP",
            player_name(&towers, winner),
            size.name
        );
        for entity in &trophy_text {
            set(&mut texts, entity, words.clone());
        }
    }
}
