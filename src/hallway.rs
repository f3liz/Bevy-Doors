use bevy::prelude::*;

use crate::enemy::maybe_spawn_enemy_after_door;
use crate::game_state::{DoorPlacement, GameState, HallwayProgress, RunStats};
use crate::player::{Player, PlayerLook, PLAYER_EYE_HEIGHT, HALL_WIDTH};

pub const HALL_LENGTH: f32 = 24.0;
pub const HALL_HEIGHT: f32 = 4.0;
pub const SIDE_DOOR_Z: f32 = 16.0;
pub const AHEAD_DOOR_TRIGGER_Z: f32 = HALL_LENGTH - 4.0;

#[derive(Component)]
pub struct HallwayEntity;

pub struct HallwayPlugin;

impl Plugin for HallwayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            advance_through_door
                .in_set(crate::game_state::GameplaySystems::Door)
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            OnExit(GameState::Playing),
            cleanup_playing_world.after(crate::jumpscare::cleanup_jumpscare),
        );
    }
}

fn cleanup_playing_world(
    mut commands: Commands,
    hallway: Query<Entity, With<HallwayEntity>>,
    player: Query<Entity, With<Player>>,
    enemies: Query<Entity, With<crate::enemy::Enemy>>,
    cameras: Query<Entity, With<crate::player::PlayerCamera>>,
) {
    for entity in &hallway {
        commands.entity(entity).despawn();
    }
    for entity in &enemies {
        commands.entity(entity).despawn();
    }
    for entity in &cameras {
        commands.entity(entity).despawn();
    }
    for entity in &player {
        commands.entity(entity).despawn();
    }
}

pub fn spawn_hallway_segment(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    door_number: u32,
    placement: DoorPlacement,
) {
    let carpet = materials.add(StandardMaterial {
        base_color: Color::srgb(0.42, 0.14, 0.14),
        perceptual_roughness: 0.95,
        ..default()
    });
    let wallpaper = materials.add(StandardMaterial {
        base_color: Color::srgb(0.68, 0.58, 0.45),
        perceptual_roughness: 0.92,
        ..default()
    });
    let trim = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.16, 0.12),
        ..default()
    });
    let ceiling = materials.add(StandardMaterial {
        base_color: Color::srgb(0.06, 0.05, 0.05),
        ..default()
    });
    let door_wood = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.22, 0.14),
        ..default()
    });
    let sign = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.85, 0.7),
        emissive: LinearRgba::from(Color::srgb(
            0.35 + (door_number as f32 * 0.02).min(0.2),
            0.3,
            0.15,
        )),
        ..default()
    });
    let guide = materials.add(StandardMaterial {
        base_color: Color::srgb(0.55, 0.45, 0.25),
        emissive: LinearRgba::from(Color::srgb(0.15, 0.12, 0.05)),
        ..default()
    });

    let box_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let plane = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(1.0)));

    commands.spawn((
        HallwayEntity,
        Mesh3d(plane.clone()),
        MeshMaterial3d(carpet),
        Transform::from_xyz(0.0, 0.0, HALL_LENGTH * 0.5).with_scale(Vec3::new(HALL_WIDTH, 1.0, HALL_LENGTH)),
    ));

    let half_w = HALL_WIDTH * 0.5;
    for (x, z, sx, sz) in [
        (-half_w, HALL_LENGTH * 0.5, 0.25, HALL_LENGTH),
        (half_w, HALL_LENGTH * 0.5, 0.25, HALL_LENGTH),
        (0.0, 0.0, HALL_WIDTH, 0.25),
    ] {
        commands.spawn((
            HallwayEntity,
            Mesh3d(box_mesh.clone()),
            MeshMaterial3d(wallpaper.clone()),
            Transform::from_xyz(x, HALL_HEIGHT * 0.5, z).with_scale(Vec3::new(sx, HALL_HEIGHT, sz)),
        ));
    }

    commands.spawn((
        HallwayEntity,
        Mesh3d(box_mesh.clone()),
        MeshMaterial3d(ceiling.clone()),
        Transform::from_xyz(0.0, HALL_HEIGHT, HALL_LENGTH * 0.5)
            .with_scale(Vec3::new(HALL_WIDTH, 0.15, HALL_LENGTH)),
    ));

    for x in [-2.8_f32, 2.8] {
        commands.spawn((
            HallwayEntity,
            Mesh3d(box_mesh.clone()),
            MeshMaterial3d(trim.clone()),
            Transform::from_xyz(x, 0.15, HALL_LENGTH - 0.15)
                .with_scale(Vec3::new(0.2, 0.3, HALL_LENGTH)),
        ));
    }

    match placement {
        DoorPlacement::Ahead => {
            commands.spawn((
                HallwayEntity,
                Mesh3d(box_mesh.clone()),
                MeshMaterial3d(door_wood.clone()),
                Transform::from_xyz(0.0, 1.4, HALL_LENGTH - 0.2).with_scale(Vec3::new(1.6, 2.8, 0.15)),
            ));
            commands.spawn((
                HallwayEntity,
                Mesh3d(box_mesh.clone()),
                MeshMaterial3d(sign.clone()),
                Transform::from_xyz(0.0, 2.2, HALL_LENGTH - 0.35).with_scale(Vec3::new(1.2, 0.5, 0.08)),
            ));
            commands.spawn((
                HallwayEntity,
                Mesh3d(box_mesh.clone()),
                MeshMaterial3d(guide.clone()),
                Transform::from_xyz(0.0, 0.02, HALL_LENGTH * 0.5).with_scale(Vec3::new(0.35, 0.02, HALL_LENGTH - 2.0)),
            ));
        }
        DoorPlacement::Left => {
            let wall_x = -half_w + 0.12;
            commands.spawn((
                HallwayEntity,
                Mesh3d(box_mesh.clone()),
                MeshMaterial3d(door_wood.clone()),
                Transform::from_xyz(wall_x, 1.4, SIDE_DOOR_Z).with_scale(Vec3::new(0.12, 2.8, 1.6)),
            ));
            commands.spawn((
                HallwayEntity,
                Mesh3d(box_mesh.clone()),
                MeshMaterial3d(sign.clone()),
                Transform::from_xyz(wall_x + 0.18, 2.2, SIDE_DOOR_Z).with_scale(Vec3::new(0.08, 0.5, 1.0)),
            ));
            commands.spawn((
                HallwayEntity,
                Mesh3d(box_mesh.clone()),
                MeshMaterial3d(guide.clone()),
                Transform::from_xyz(-1.2, 0.02, SIDE_DOOR_Z).with_scale(Vec3::new(half_w - 0.8, 0.02, 0.35)),
            ));
            commands.spawn((
                HallwayEntity,
                PointLight {
                    color: Color::srgb(1.0, 0.75, 0.45),
                    intensity: 420_000.0,
                    range: 10.0,
                    ..default()
                },
                Transform::from_xyz(-2.5, 3.2, SIDE_DOOR_Z),
            ));
        }
        DoorPlacement::Right => {
            let wall_x = half_w - 0.12;
            commands.spawn((
                HallwayEntity,
                Mesh3d(box_mesh.clone()),
                MeshMaterial3d(door_wood.clone()),
                Transform::from_xyz(wall_x, 1.4, SIDE_DOOR_Z).with_scale(Vec3::new(0.12, 2.8, 1.6)),
            ));
            commands.spawn((
                HallwayEntity,
                Mesh3d(box_mesh.clone()),
                MeshMaterial3d(sign.clone()),
                Transform::from_xyz(wall_x - 0.18, 2.2, SIDE_DOOR_Z).with_scale(Vec3::new(0.08, 0.5, 1.0)),
            ));
            commands.spawn((
                HallwayEntity,
                Mesh3d(box_mesh.clone()),
                MeshMaterial3d(guide.clone()),
                Transform::from_xyz(1.2, 0.02, SIDE_DOOR_Z).with_scale(Vec3::new(half_w - 0.8, 0.02, 0.35)),
            ));
            commands.spawn((
                HallwayEntity,
                PointLight {
                    color: Color::srgb(1.0, 0.75, 0.45),
                    intensity: 420_000.0,
                    range: 10.0,
                    ..default()
                },
                Transform::from_xyz(2.5, 3.2, SIDE_DOOR_Z),
            ));
        }
    }

    for i in 0..3 {
        commands.spawn((
            HallwayEntity,
            PointLight {
                color: Color::srgb(1.0, 0.75, 0.45),
                intensity: 350_000.0,
                range: 14.0,
                ..default()
            },
            Transform::from_xyz(0.0, 3.4, 4.0 + i as f32 * 7.0),
        ));
    }
}

fn player_reached_door(player: &Transform, placement: DoorPlacement) -> bool {
    let half_w = HALL_WIDTH * 0.5;
    match placement {
        DoorPlacement::Ahead => {
            player.translation.z >= AHEAD_DOOR_TRIGGER_Z && player.translation.x.abs() < 1.8
        }
        DoorPlacement::Left => {
            player.translation.x <= -half_w + 1.8
                && (player.translation.z - SIDE_DOOR_Z).abs() < 2.2
        }
        DoorPlacement::Right => {
            player.translation.x >= half_w - 1.8
                && (player.translation.z - SIDE_DOOR_Z).abs() < 2.2
        }
    }
}

fn advance_through_door(
    mut commands: Commands,
    mut player: Query<(&mut Transform, &mut PlayerLook), With<Player>>,
    mut cameras: Query<
        &mut Transform,
        (With<crate::player::PlayerCamera>, Without<Player>),
    >,
    mut progress: ResMut<HallwayProgress>,
    mut stats: ResMut<RunStats>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    hallway: Query<Entity, With<HallwayEntity>>,
    enemies: Query<Entity, With<crate::enemy::Enemy>>,
) {
    let Ok((mut player_tf, mut look)) = player.single_mut() else {
        return;
    };

    if !player_reached_door(&player_tf, progress.current_placement) {
        return;
    }

    stats.doors_cleared += 1;
    progress.door_number += 1;
    progress.current_placement = DoorPlacement::for_door_number(progress.door_number);

    for entity in &hallway {
        commands.entity(entity).despawn();
    }

    player_tf.translation = Vec3::new(0.0, PLAYER_EYE_HEIGHT, 2.0);
    player_tf.rotation = Quat::IDENTITY;
    look.pitch = 0.0;
    if let Ok(mut camera_tf) = cameras.single_mut() {
        camera_tf.rotation = Quat::IDENTITY;
    }

    spawn_hallway_segment(
        &mut commands,
        &mut meshes,
        &mut materials,
        progress.door_number,
        progress.current_placement,
    );
    maybe_spawn_enemy_after_door(
        &mut commands,
        &mut meshes,
        &mut materials,
        &enemies,
        progress.door_number,
    );
}
