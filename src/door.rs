use bevy::prelude::*;
use bevy::sprite::Text2d;

use crate::hallway::{HALL_LENGTH, HALL_WIDTH, SIDE_DOOR_Z};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DoorPlacement {
    Ahead,
    Left,
    Right,
}

#[derive(Clone, Debug)]
pub struct DoorSpawn {
    pub number: u32,
    pub placement: DoorPlacement,
}

#[derive(Clone, Debug)]
pub struct RoomLayout {
    pub target_number: u32,
    pub doors: Vec<DoorSpawn>,
}

#[derive(Component, Clone, Copy)]
pub struct Door {
    pub number: u32,
    pub placement: DoorPlacement,
}

#[derive(Component)]
pub struct DoorLeaf;

#[derive(Component, Default)]
pub struct DoorOpen {
    pub amount: f32,
}

#[derive(Component)]
pub struct DoorSign;

#[derive(Component)]
pub struct DoorFrame;

impl RoomLayout {
    pub fn generate(target_number: u32, seed: u32) -> Self {
        let roll = hash_u32(seed, target_number) % 100;
        let door_count = if roll < 20 {
            1
        } else if roll < 55 {
            2
        } else {
            3
        };

        let placements = pick_placements(door_count, seed.wrapping_add(target_number));
        let mut doors = Vec::new();

        doors.push(DoorSpawn {
            number: target_number,
            placement: placements[0],
        });

        for i in 1..door_count {
            let wrong = wrong_door_number(target_number, seed.wrapping_add(i as u32));
            doors.push(DoorSpawn {
                number: wrong,
                placement: placements[i],
            });
        }

        Self {
            target_number,
            doors,
        }
    }
}

pub fn spawn_door(
    commands: &mut Commands,
    _meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    door: &DoorSpawn,
    _target_number: u32,
    box_mesh: &Handle<Mesh>,
    door_wood: &Handle<StandardMaterial>,
    sign_wood: &Handle<StandardMaterial>,
) {
    let half_w = HALL_WIDTH * 0.5;

const WALL_HALF: f32 = 0.125;
const DOOR_HALF_DEPTH: f32 = 0.06;

    let (frame_pos, leaf_scale, sign_local, text_local, hinge_rotation) = match door.placement {
        DoorPlacement::Ahead => (
            Vec3::new(0.0, 1.4, HALL_LENGTH - 0.2),
            Vec3::new(1.6, 2.8, 0.15),
            Vec3::new(0.0, 1.15, -0.28),
            Vec3::new(0.0, 1.15, -0.36),
            Quat::from_rotation_y(std::f32::consts::PI),
        ),
        DoorPlacement::Left => (
            Vec3::new(-half_w + WALL_HALF + DOOR_HALF_DEPTH, 1.4, SIDE_DOOR_Z),
            Vec3::new(0.12, 2.8, 1.6),
            Vec3::new(0.28, 1.15, 0.0),
            Vec3::new(0.36, 1.15, 0.0),
            Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2),
        ),
        DoorPlacement::Right => (
            Vec3::new(half_w - WALL_HALF - DOOR_HALF_DEPTH, 1.4, SIDE_DOOR_Z),
            Vec3::new(0.12, 2.8, 1.6),
            Vec3::new(-0.28, 1.15, 0.0),
            Vec3::new(-0.36, 1.15, 0.0),
            Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
        ),
    };

    let _ = materials;
    let frame = commands
        .spawn((
            crate::hallway::HallwayEntity,
            DoorFrame,
            Door {
                number: door.number,
                placement: door.placement,
            },
            Transform::from_translation(frame_pos),
            Visibility::default(),
        ))
        .id();

    commands.entity(frame).with_children(|parent| {
        parent.spawn((
            DoorLeaf,
            DoorOpen::default(),
            Door {
                number: door.number,
                placement: door.placement,
            },
            Mesh3d(box_mesh.clone()),
            MeshMaterial3d(door_wood.clone()),
            Transform::from_xyz(0.0, 0.0, 0.0).with_scale(leaf_scale),
        ));
        parent.spawn((
            DoorSign,
            Mesh3d(box_mesh.clone()),
            MeshMaterial3d(sign_wood.clone()),
            Transform::from_translation(sign_local)
                .with_rotation(hinge_rotation)
                .with_scale(Vec3::new(0.85, 0.42, 0.06)),
        ));
        parent.spawn((
            Text2d::new(format!("{}", door.number)),
            TextFont {
                font_size: if door.number >= 10 { 52.0 } else { 64.0 },
                ..default()
            },
            TextColor(Color::srgb(0.88, 0.82, 0.68)),
            Transform::from_translation(text_local)
                .with_rotation(hinge_rotation)
                .with_scale(Vec3::splat(0.0045)),
        ));
    });
}

pub fn player_at_door(position: Vec3, placement: DoorPlacement) -> bool {
    let half_w = HALL_WIDTH * 0.5;
    match placement {
        DoorPlacement::Ahead => {
            position.z >= HALL_LENGTH - 4.0 && position.x.abs() < 1.8
        }
        DoorPlacement::Left => {
            position.x <= -half_w + 1.8 && (position.z - SIDE_DOOR_Z).abs() < 2.2
        }
        DoorPlacement::Right => {
            position.x >= half_w - 1.8 && (position.z - SIDE_DOOR_Z).abs() < 2.2
        }
    }
}

fn hash_u32(a: u32, b: u32) -> u32 {
    let mut x = a.wrapping_mul(374761393).wrapping_add(b.wrapping_mul(668265263));
    x = (x ^ (x >> 13)).wrapping_mul(1274126177);
    x ^ (x >> 16)
}

fn pick_placements(count: usize, seed: u32) -> Vec<DoorPlacement> {
    let mut available = vec![
        DoorPlacement::Ahead,
        DoorPlacement::Left,
        DoorPlacement::Right,
    ];
    let mut picked = Vec::new();
    for i in 0..count {
        if available.is_empty() {
            break;
        }
        let idx = hash_u32(seed, i as u32) as usize % available.len();
        picked.push(available.remove(idx));
    }
    picked
}

fn wrong_door_number(target: u32, salt: u32) -> u32 {
    let h = hash_u32(target, salt);
    let offset = (h % 18) + 1;
    let mut wrong = if h % 2 == 0 {
        target.saturating_add(offset)
    } else {
        target.saturating_sub(offset).max(1)
    };
    if wrong == target {
        wrong = target + offset + 1;
    }
    wrong
}
