use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::game_state::{GameState, HallwayProgress, RunStats};
use crate::hallway::spawn_hallway_segment;
use crate::player::spawn_player;

#[derive(Component)]
pub struct LobbyEntity;

#[derive(Component)]
pub struct LobbyCamera;

pub struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Lobby), setup_lobby)
            .add_systems(OnExit(GameState::Lobby), cleanup_lobby)
            .add_systems(
                Update,
                (lobby_start_input, release_cursor_in_lobby)
                    .run_if(in_state(GameState::Lobby)),
            );
    }
}

fn setup_lobby(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let carpet = materials.add(StandardMaterial {
        base_color: Color::srgb(0.35, 0.12, 0.12),
        perceptual_roughness: 0.95,
        ..default()
    });
    let wall = materials.add(StandardMaterial {
        base_color: Color::srgb(0.72, 0.65, 0.52),
        perceptual_roughness: 0.9,
        ..default()
    });
    let trim = materials.add(StandardMaterial {
        base_color: Color::srgb(0.25, 0.2, 0.15),
        ..default()
    });
    let ceiling_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.08, 0.07, 0.06),
        ..default()
    });

    let floor_mesh = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(1.0)));
    let wall_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));

    commands.spawn((
        LobbyEntity,
        Mesh3d(floor_mesh.clone()),
        MeshMaterial3d(carpet),
        Transform::from_scale(Vec3::new(14.0, 1.0, 14.0)),
    ));

    for (pos, scale, is_ceiling) in [
        (Vec3::new(0.0, 2.0, -7.0), Vec3::new(14.0, 4.0, 0.3), false),
        (Vec3::new(-7.0, 2.0, 0.0), Vec3::new(0.3, 4.0, 14.0), false),
        (Vec3::new(7.0, 2.0, 0.0), Vec3::new(0.3, 4.0, 14.0), false),
        (Vec3::new(0.0, 4.0, 0.0), Vec3::new(14.0, 0.2, 14.0), true),
    ] {
        let mat = if is_ceiling { &ceiling_mat } else { &wall };
        commands.spawn((
            LobbyEntity,
            Mesh3d(wall_mesh.clone()),
            MeshMaterial3d(mat.clone()),
            Transform::from_translation(pos).with_scale(scale),
        ));
    }

    commands.spawn((
        LobbyEntity,
        Mesh3d(wall_mesh.clone()),
        MeshMaterial3d(trim.clone()),
        Transform::from_xyz(0.0, 1.0, 6.5).with_scale(Vec3::new(4.0, 2.5, 0.2)),
    ));

    commands.spawn((
        LobbyEntity,
        PointLight {
            color: Color::srgb(1.0, 0.85, 0.65),
            intensity: 800_000.0,
            range: 20.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 3.2, 0.0),
    ));

    commands.spawn((
        LobbyEntity,
        DirectionalLight {
            illuminance: 120.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.2, 0.4, 0.0)),
    ));

    commands.spawn((
        LobbyCamera,
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, -4.0).looking_at(Vec3::new(0.0, 1.2, 2.0), Vec3::Y),
    ));
}

fn cleanup_lobby(
    mut commands: Commands,
    lobby: Query<Entity, With<LobbyEntity>>,
    cameras: Query<Entity, With<LobbyCamera>>,
) {
    for entity in lobby.iter().chain(cameras.iter()) {
        commands.entity(entity).despawn();
    }
}

fn lobby_start_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut progress: ResMut<HallwayProgress>,
    mut stats: ResMut<RunStats>,
) {
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }

    progress.door_number = 1;
    stats.doors_cleared = 0;

    spawn_player(&mut commands);
    spawn_hallway_segment(&mut commands, &mut meshes, &mut materials, progress.door_number);

    next_state.set(GameState::Playing);
}

fn release_cursor_in_lobby(mut window_query: Query<&mut CursorOptions, With<PrimaryWindow>>) {
    let Ok(mut cursor) = window_query.single_mut() else {
        return;
    };
    cursor.grab_mode = CursorGrabMode::None;
    cursor.visible = true;
}
