use bevy::prelude::*;

use crate::game_state::{GameState, HallwayProgress, RunStats};

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct DoorLabel;

#[derive(Component)]
pub struct MenuRoot;

#[derive(Component)]
pub struct MenuTitle;

#[derive(Component)]
pub struct MenuSubtitle;

#[derive(Component)]
pub struct OverlayCamera;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui)
            .add_systems(OnEnter(GameState::GameOver), spawn_overlay_camera)
            .add_systems(OnExit(GameState::GameOver), despawn_overlay_camera)
            .add_systems(Update, sync_ui)
            .add_systems(Update, game_over_input.run_if(in_state(GameState::GameOver)));
    }
}

pub fn setup_ui(mut commands: Commands) {
    commands
        .spawn((
            HudRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
            Visibility::Hidden,
        ))
        .with_children(|parent| {
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
            Visibility::Visible,
        ))
        .with_children(|parent| {
            parent.spawn((
                MenuTitle,
                Text::new("Bevy Doors"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.82, 0.65)),
            ));
            parent.spawn((
                MenuSubtitle,
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

fn sync_ui(
    state: Res<State<GameState>>,
    stats: Res<RunStats>,
    progress: Res<HallwayProgress>,
    mut hud: Query<&mut Visibility, (With<HudRoot>, Without<MenuRoot>)>,
    mut menu: Query<&mut Visibility, (With<MenuRoot>, Without<HudRoot>)>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<DoorLabel>>,
        Query<&mut Text, With<MenuTitle>>,
        Query<&mut Text, With<MenuSubtitle>>,
    )>,
) {
    let Ok(mut hud_vis) = hud.single_mut() else {
        return;
    };
    let Ok(mut menu_vis) = menu.single_mut() else {
        return;
    };

    match state.get() {
        GameState::Playing => {
            *hud_vis = Visibility::Visible;
            *menu_vis = Visibility::Hidden;
            if let Ok(mut text) = text_queries.p0().single_mut() {
                **text = format!("Door {}", progress.door_number);
            }
        }
        GameState::Lobby => {
            *hud_vis = Visibility::Hidden;
            *menu_vis = Visibility::Visible;
            if let Ok(mut text) = text_queries.p1().single_mut() {
                **text = "Bevy Doors".to_string();
            }
            if let Ok(mut text) = text_queries.p2().single_mut() {
                **text = "Press SPACE to enter the hotel".to_string();
            }
        }
        GameState::GameOver => {
            *hud_vis = Visibility::Hidden;
            *menu_vis = Visibility::Visible;
            if let Ok(mut text) = text_queries.p1().single_mut() {
                **text = "You died".to_string();
            }
            if let Ok(mut text) = text_queries.p2().single_mut() {
                **text = format!(
                    "Doors cleared: {}  —  Press SPACE for lobby",
                    stats.doors_cleared
                );
            }
        }
    }
}

fn spawn_overlay_camera(mut commands: Commands) {
    commands.spawn((
        OverlayCamera,
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn despawn_overlay_camera(mut commands: Commands, cameras: Query<Entity, With<OverlayCamera>>) {
    for entity in &cameras {
        commands.entity(entity).despawn();
    }
}

fn game_over_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Lobby);
    }
}
