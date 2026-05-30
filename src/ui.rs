use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::ui::widget::ImageNode;

use crate::game_state::{GameState, HallwayProgress, PlayerHealth, RunStats, MAX_LIVES};

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct DoorLabel;

#[derive(Component)]
pub struct HealthHearts;

#[derive(Component)]
pub struct HeartSlot(pub usize);

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

fn heart_pixel(fx: f32, fy: f32) -> bool {
    // Two top lobes (circles) plus a tapered bottom — classic heart icon shape.
    let in_left = (fx - 0.34).powi(2) + (fy - 0.34).powi(2) <= 0.118;
    let in_right = (fx - 0.66).powi(2) + (fy - 0.34).powi(2) <= 0.118;

    if fy >= 0.38 && fy <= 0.92 {
        let half_width = (0.92 - fy) * 0.62;
        if (fx - 0.5).abs() <= half_width {
            return true;
        }
    }

    in_left || in_right
}

fn create_heart_image() -> Image {
    let width = 48u32;
    let height = 44u32;
    let mut data = vec![0u8; (width * height * 4) as usize];

    for py in 0..height {
        for px in 0..width {
            let fx = (px as f32 + 0.5) / width as f32;
            let fy = (py as f32 + 0.5) / height as f32;
            let idx = ((py * width + px) * 4) as usize;
            if heart_pixel(fx, fy) {
                data[idx] = 255;
                data[idx + 1] = 255;
                data[idx + 2] = 255;
                data[idx + 3] = 255;
            }
        }
    }

    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::default(),
    )
}

pub fn setup_ui(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let heart_image = images.add(create_heart_image());

    commands
        .spawn((
            HudRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(16.0)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexStart,
                ..default()
            },
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                DoorLabel,
                Text::new("Door 1  |  Cleared: 0"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.88, 0.7)),
            ));

            parent
                .spawn((
                    HealthHearts,
                    Node {
                        column_gap: Val::Px(8.0),
                        ..default()
                    },
                ))
                .with_children(|hearts| {
                    for i in 0..MAX_LIVES as usize {
                        hearts.spawn((
                            HeartSlot(i),
                            Node {
                                width: Val::Px(30.0),
                                height: Val::Px(28.0),
                                ..default()
                            },
                            ImageNode {
                                image: heart_image.clone(),
                                color: Color::srgb(0.92, 0.15, 0.2),
                                ..default()
                            },
                        ));
                    }
                });
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
                Text::new("WASD to move  •  Mouse to look  •  Esc to quit"),
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
    health: Res<PlayerHealth>,
    mut hud: Query<&mut Visibility, (With<HudRoot>, Without<MenuRoot>)>,
    mut menu: Query<&mut Visibility, (With<MenuRoot>, Without<HudRoot>)>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<DoorLabel>>,
        Query<&mut Text, With<MenuTitle>>,
        Query<&mut Text, With<MenuSubtitle>>,
    )>,
    mut hearts: Query<(&HeartSlot, &mut ImageNode)>,
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
                **text = format!(
                    "Door {}  |  Cleared: {}",
                    progress.door_number, stats.doors_cleared
                );
            }
            for (slot, mut image) in &mut hearts {
                image.color = if slot.0 < health.lives as usize {
                    Color::srgb(0.92, 0.15, 0.2)
                } else {
                    Color::srgba(0.38, 0.34, 0.34, 0.45)
                };
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
