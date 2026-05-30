use bevy::prelude::*;

use crate::door::{self, Door, DoorPlacement, RoomLayout};
use crate::game_state::{GameState, HallwayProgress};
use crate::player::{Player, PlayerMotion};
use crate::transition::{
    start_room_transition, start_wrong_door_flash, DoorContactCooldown, RoomTransition,
    WrongDoorFlash,
};

pub const HALL_WIDTH: f32 = 8.0;
pub const HALL_LENGTH: f32 = 24.0;
pub const HALL_HEIGHT: f32 = 4.0;
pub const SIDE_DOOR_Z: f32 = 16.0;
const WALL_THICKNESS: f32 = 0.25;
const SIDE_DOOR_GAP_HALF: f32 = 1.05;

#[derive(Component)]
pub struct HallwayEntity;

pub struct HallwayPlugin;

impl Plugin for HallwayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                block_wrong_doors,
                check_doors_on_contact,
            )
                .chain()
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
    room_seed: u32,
) {
    let layout = RoomLayout::generate(door_number, room_seed);

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
    let sign_wood = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.85, 0.7),
        emissive: LinearRgba::from(Color::srgb(0.35, 0.3, 0.15)),
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

    let left_gaps: Vec<f32> = layout
        .doors
        .iter()
        .filter(|d| d.placement == DoorPlacement::Left)
        .map(|d| door_gap_center(d.placement))
        .collect();
    let right_gaps: Vec<f32> = layout
        .doors
        .iter()
        .filter(|d| d.placement == DoorPlacement::Right)
        .map(|d| door_gap_center(d.placement))
        .collect();

    spawn_wall_along_z(
        commands,
        &box_mesh,
        &wallpaper,
        -half_w,
        &left_gaps,
    );
    spawn_wall_along_z(
        commands,
        &box_mesh,
        &wallpaper,
        half_w,
        &right_gaps,
    );

    commands.spawn((
        HallwayEntity,
        Mesh3d(box_mesh.clone()),
        MeshMaterial3d(wallpaper.clone()),
        Transform::from_xyz(0.0, HALL_HEIGHT * 0.5, 0.0).with_scale(Vec3::new(HALL_WIDTH, HALL_HEIGHT, WALL_THICKNESS)),
    ));

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

    for door in &layout.doors {
        door::spawn_door(
            commands,
            meshes,
            materials,
            door,
            layout.target_number,
            &box_mesh,
            &door_wood,
            &sign_wood,
        );
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

    let _ = layout;
}

fn door_gap_center(placement: DoorPlacement) -> f32 {
    match placement {
        DoorPlacement::Left | DoorPlacement::Right => SIDE_DOOR_Z,
        DoorPlacement::Ahead => HALL_LENGTH - 0.2,
    }
}

fn spawn_wall_along_z(
    commands: &mut Commands,
    box_mesh: &Handle<Mesh>,
    wallpaper: &Handle<StandardMaterial>,
    x: f32,
    gaps: &[f32],
) {
    let mut segments = vec![(0.0_f32, HALL_LENGTH)];

    for &gap_center in gaps {
        let gap_start = (gap_center - SIDE_DOOR_GAP_HALF).max(0.0);
        let gap_end = (gap_center + SIDE_DOOR_GAP_HALF).min(HALL_LENGTH);
        let mut next = Vec::new();
        for (start, end) in segments {
            if gap_end <= start || gap_start >= end {
                next.push((start, end));
                continue;
            }
            if start < gap_start {
                next.push((start, gap_start));
            }
            if gap_end < end {
                next.push((gap_end, end));
            }
        }
        segments = next;
    }

    for (start, end) in segments {
        let length = end - start;
        if length < 0.05 {
            continue;
        }
        commands.spawn((
            HallwayEntity,
            Mesh3d(box_mesh.clone()),
            MeshMaterial3d(wallpaper.clone()),
            Transform::from_xyz(x, HALL_HEIGHT * 0.5, start + length * 0.5)
                .with_scale(Vec3::new(WALL_THICKNESS, HALL_HEIGHT, length)),
        ));
    }
}

fn block_wrong_doors(
    mut player: Query<(&mut Transform, &mut PlayerMotion), With<Player>>,
    doors: Query<&Door, With<door::DoorFrame>>,
    progress: Res<HallwayProgress>,
    transition: Res<RoomTransition>,
) {
    if transition.active {
        return;
    }

    let Ok((mut transform, mut motion)) = player.single_mut() else {
        return;
    };

    let half_w = HALL_WIDTH * 0.5;

    for door in &doors {
        if door.number == progress.door_number {
            continue;
        }
        if !door::player_at_door(transform.translation, door.placement) {
            continue;
        }

        match door.placement {
            DoorPlacement::Ahead => {
                transform.translation.z = transform.translation.z.min(HALL_LENGTH - 4.0);
                if motion.velocity.z > 0.0 {
                    motion.velocity.z = 0.0;
                }
            }
            DoorPlacement::Left => {
                let limit = -half_w + 1.8;
                transform.translation.x = transform.translation.x.max(limit);
                if motion.velocity.x < 0.0 {
                    motion.velocity.x = 0.0;
                }
            }
            DoorPlacement::Right => {
                let limit = half_w - 1.8;
                transform.translation.x = transform.translation.x.min(limit);
                if motion.velocity.x > 0.0 {
                    motion.velocity.x = 0.0;
                }
            }
        }
        return;
    }
}

pub fn check_doors_on_contact(
    time: Res<Time>,
    player: Query<&Transform, With<Player>>,
    doors: Query<&Door, With<door::DoorFrame>>,
    progress: Res<HallwayProgress>,
    mut transition: ResMut<RoomTransition>,
    mut flash: ResMut<WrongDoorFlash>,
    mut cooldown: ResMut<DoorContactCooldown>,
) {
    if transition.active {
        return;
    }

    cooldown.remaining = (cooldown.remaining - time.delta_secs()).max(0.0);

    let Ok(player_tf) = player.single() else {
        return;
    };

    for door in &doors {
        if !door::player_at_door(player_tf.translation, door.placement) {
            continue;
        }

        if door.number == progress.door_number {
            start_room_transition(&mut transition);
        } else if cooldown.remaining <= 0.0 {
            start_wrong_door_flash(&mut flash);
            cooldown.remaining = 0.8;
        }
        return;
    }
}
