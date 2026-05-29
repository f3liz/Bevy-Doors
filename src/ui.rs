use bevy::prelude::*;

use crate::game_state::{GameState, HallwayProgress, RunStats};

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct DoorLabel;

#[derive(Component)]
pub struct MenuRoot;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui)
            .add_systems(OnEnter(GameState::Lobby), show_lobby_menu)
            .add_systems(OnEnter(GameState::GameOver), show_game_over_menu)
            .add_systems(OnEnter(GameState::Playing), show_hud_hide_menu)
            .add_systems(
                Update,
                (update_door_label, game_over_input).run_if(in_state(GameState::Playing).or(in_state(GameState::GameOver))),
            )
            .add_systems(
                Update,
                game_over_input.run_if(in_state(GameState::GameOver)),
            );
    }
}

fn setup_ui(mut commands: Commands) {
    commands.spawn((
        HudRoot,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::all(Val::Px(16.0)),
            ..default()
        },
        Visibility::Hidden,
    )).with_children(|parent| {
        parent.spawn((
            DoorLabel,
            Text::new("Door 1"),
            TextFont {
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::srgb(0.95, 0.88, 0.7)),
        ));
    });

    commands
        .spawn((
            MenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Bevy Doors"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.82, 0.65)),
            ));
            parent.spawn((
                Text::new("Press SPACE to enter the hotel"),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.8, 0.75)),
            ));
            parent.spawn((
                Text::new("WASD / Arrows to move  •  Mouse to look  •  Esc releases cursor"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.65, 0.62, 0.58)),
            ));
        });
}

fn show_lobby_menu(
    mut hud: Query<&mut Visibility, (With<HudRoot>, Without<MenuRoot>)>,
    mut menu: Query<&mut Visibility, (With<MenuRoot>, Without<HudRoot>)>,
    mut menu_text: Query<&mut Text, With<MenuRoot>>,
) {
    if let Ok(mut vis) = hud.single_mut() {
        *vis = Visibility::Hidden;
    }
    if let Ok(mut vis) = menu.single_mut() {
        *vis = Visibility::Visible;
    }
    for mut text in &mut menu_text {
        **text = "Bevy Doors".to_string();
    }
}

fn show_game_over_menu(
    mut hud: Query<&mut Visibility, (With<HudRoot>, Without<MenuRoot>)>,
    mut menu: Query<&mut Visibility, (With<MenuRoot>, Without<HudRoot>)>,
    stats: Res<RunStats>,
    mut menu_children: Query<&mut Text, Without<DoorLabel>>,
) {
    if let Ok(mut vis) = hud.single_mut() {
        *vis = Visibility::Hidden;
    }
    if let Ok(mut vis) = menu.single_mut() {
        *vis = Visibility::Visible;
    }
    let mut lines = menu_children.iter_mut().collect::<Vec<_>>();
    if lines.len() >= 2 {
        **lines[0] = "You died".to_string();
        **lines[1] = format!(
            "Doors cleared: {}  —  Press SPACE for lobby",
            stats.doors_cleared
        );
    }
}

fn show_hud_hide_menu(
    mut hud: Query<&mut Visibility, (With<HudRoot>, Without<MenuRoot>)>,
    mut menu: Query<&mut Visibility, (With<MenuRoot>, Without<HudRoot>)>,
) {
    if let Ok(mut vis) = hud.single_mut() {
        *vis = Visibility::Visible;
    }
    if let Ok(mut vis) = menu.single_mut() {
        *vis = Visibility::Hidden;
    }
}

fn update_door_label(
    progress: Res<HallwayProgress>,
    mut label: Query<&mut Text, With<DoorLabel>>,
) {
    let Ok(mut text) = label.single_mut() else {
        return;
    };
    **text = format!("Door {}", progress.door_number);
}

fn game_over_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Lobby);
    }
}
