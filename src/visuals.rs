//! Placeholder visuals: coloured squares, a camera that follows the tower, and
//! a text HUD. No original game art is used; this is the layer an asset mod (or
//! a loader for your own installed copy's assets) would replace.

use bevy::prelude::*;

use crate::config::{GameConfig, ModeConfig};
use crate::gameplay::{BrickBag, Brick, Phase, Platform, Round};

pub struct VisualsPlugin;

impl Plugin for VisualsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0.12, 0.14, 0.22)))
            .add_systems(Startup, setup_scene)
            .add_systems(Update, (dress_bricks, dress_platform, follow_tower, update_hud));
    }
}

#[derive(Component)]
struct Hud;

fn setup_scene(mut commands: Commands, config: Res<GameConfig>) {
    commands.spawn((Camera2d, Transform::from_xyz(0.0, config.cells(6.0), 0.0)));

    if let ModeConfig::Race { finish_height, .. } = config.mode {
        commands.spawn((
            Sprite::from_color(
                Color::srgba(1.0, 1.0, 1.0, 0.5),
                Vec2::new(config.cells(config.platform_width + 8.0), 2.0),
            ),
            Transform::from_xyz(0.0, config.cells(finish_height), -1.0),
        ));
    }

    commands.spawn((
        Hud,
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(20.0),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

/// Gives every new brick one sprite per cell.
fn dress_bricks(mut commands: Commands, config: Res<GameConfig>, bricks: Query<(Entity, &Brick), Added<Brick>>) {
    let c = config.cell_size;
    for (entity, brick) in &bricks {
        commands.entity(entity).insert(Visibility::default()).with_children(|parent| {
            for &cell in &brick.cells {
                parent.spawn((
                    Sprite::from_color(brick.color, Vec2::splat(c * 0.94)),
                    Transform::from_translation((cell * c).extend(0.0)),
                ));
            }
        });
    }
}

fn dress_platform(mut commands: Commands, config: Res<GameConfig>, platforms: Query<Entity, Added<Platform>>) {
    for entity in &platforms {
        commands.entity(entity).insert(Sprite::from_color(
            Color::srgb(0.55, 0.50, 0.45),
            Vec2::new(config.cells(config.platform_width), config.cell_size),
        ));
    }
}

/// Keeps the top of the tower in view.
fn follow_tower(
    time: Res<Time>,
    config: Res<GameConfig>,
    round: Option<Res<Round>>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
) {
    let (Some(round), Ok(mut transform)) = (round, camera.single_mut()) else {
        return;
    };
    let target = config.cells(round.tower_height.max(6.0) + 2.0);
    let t = 1.0 - (-3.0 * time.delta_secs()).exp();
    transform.translation.y += (target - transform.translation.y) * t;
}

fn update_hud(
    config: Res<GameConfig>,
    round: Option<Res<Round>>,
    bag: Option<ResMut<BrickBag>>,
    mut hud: Query<&mut Text, With<Hud>>,
) {
    let (Some(round), Some(mut bag), Ok(mut text)) = (round, bag, hud.single_mut()) else {
        return;
    };
    let next = &config.pieces[bag.peek()].name;
    let goal = match config.mode {
        ModeConfig::Race {
            finish_height,
            hold_seconds,
        } => {
            let hold = if round.hold_timer > 0.0 {
                format!("  hold {:.1}/{hold_seconds:.0}s", round.hold_timer)
            } else {
                String::new()
            };
            format!("RACE  height {:.1} / {finish_height:.0}{hold}", round.tower_height)
        }
        ModeConfig::Survival { bricks_to_place, .. } => format!(
            "SURVIVAL  bricks {} / {bricks_to_place}  lives {}",
            round.bricks_placed,
            round.lives.unwrap_or(0)
        ),
    };
    let status = match round.phase {
        Phase::Playing => String::new(),
        Phase::Won => "\nYOU WIN! Press R to play again".into(),
        Phase::Lost => "\nTOWER FELL. Press R to try again".into(),
    };
    text.0 = format!(
        "{goal}\nplaced {}  dropped {}  next {next}\nLeft/Right move  Up/Space rotate  Down drop faster  R restart{status}",
        round.bricks_placed, round.bricks_dropped
    );
}
