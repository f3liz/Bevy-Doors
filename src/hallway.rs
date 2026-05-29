use bevy::prelude::*;

use crate::enemy::maybe_spawn_enemy_after_door;
use crate::game_state::{GameState, HallwayProgress, RunStats};
use crate::player::{Player, PLAYER_EYE_HEIGHT};

pub const HALL_WIDTH: f32 = 8.0;
pub const HALL_LENGTH: f32 = 24.0;
pub const HALL_HEIGHT: f32 = 4.0;
pub const DOOR_TRIGGER_Z: f32 = HALL_LENGTH - 4.0;

#[derive(Component)]
pub struct HallwayEntity;

pub struct HallwayPlugin;

impl Plugin for HallwayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (advance_through_door, clamp_player_in_hall).run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnExit(GameState::Playing), cleanup_playing_world);
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
        emissive: LinearRgba::from(Color::srgb(0.4, 0.35, 0.2)),
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

    let door_x = if door_number % 2 == 0 { -1.6 } else { 1.6 };
    commands.spawn((
        HallwayEntity,
        Mesh3d(box_mesh.clone()),
        MeshMaterial3d(door_wood.clone()),
        Transform::from_xyz(door_x, 1.4, HALL_LENGTH - 0.2).with_scale(Vec3::new(1.4, 2.8, 0.15)),
    ));

    commands.spawn((
        HallwayEntity,
        Mesh3d(box_mesh.clone()),
        MeshMaterial3d(sign.clone()),
        Transform::from_xyz(0.0, 2.2, HALL_LENGTH - 0.35).with_scale(Vec3::new(1.2, 0.5, 0.08)),
    ));

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

fn advance_through_door(
    mut commands: Commands,
    mut player: Query<&mut Transform, With<Player>>,
    mut progress: ResMut<HallwayProgress>,
    mut stats: ResMut<RunStats>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    hallway: Query<Entity, With<HallwayEntity>>,
    enemies: Query<Entity, With<crate::enemy::Enemy>>,
) {
    let Ok(mut player_tf) = player.single_mut() else {
        return;
    };

    if player_tf.translation.z < DOOR_TRIGGER_Z {
        return;
    }

    stats.doors_cleared += 1;
    progress.door_number += 1;

    for entity in &hallway {
        commands.entity(entity).despawn();
    }

    player_tf.translation = Vec3::new(
        player_tf.translation.x.clamp(-3.0, 3.0),
        PLAYER_EYE_HEIGHT,
        2.0,
    );

    spawn_hallway_segment(
        &mut commands,
        &mut meshes,
        &mut materials,
        progress.door_number,
    );
    maybe_spawn_enemy_after_door(
        &mut commands,
        &mut meshes,
        &mut materials,
        &enemies,
        progress.door_number,
    );
}

fn clamp_player_in_hall(mut player: Query<&mut Transform, With<Player>>) {
    let Ok(mut tf) = player.single_mut() else {
        return;
    };
    let half_w = HALL_WIDTH * 0.5 - 0.4;
    tf.translation.x = tf.translation.x.clamp(-half_w, half_w);
    tf.translation.y = PLAYER_EYE_HEIGHT;
}
