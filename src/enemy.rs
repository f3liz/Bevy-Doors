use bevy::prelude::*;

use crate::game_state::{GameState, PlayerHealth, HIT_COOLDOWN};
use crate::hallway::HALL_LENGTH;
use crate::player::Player;

pub const ENEMY_SPEED: f32 = 1.5;
pub const KILL_DISTANCE: f32 = 1.1;
pub const MAX_ENEMIES: usize = 2;
const TUNG_HEIGHT: f32 = 0.95;

#[derive(Component)]
pub struct Enemy;

#[derive(Resource)]
pub(crate) struct TungModelAssets {
    body_mesh: Handle<Mesh>,
    box_mesh: Handle<Mesh>,
    eye_mesh: Handle<Mesh>,
    pupil_mesh: Handle<Mesh>,
    wood: Handle<StandardMaterial>,
    wood_dark: Handle<StandardMaterial>,
    bat_wood: Handle<StandardMaterial>,
    mouth: Handle<StandardMaterial>,
    eye_white: Handle<StandardMaterial>,
    pupil: Handle<StandardMaterial>,
    foot: Handle<StandardMaterial>,
}

impl FromWorld for TungModelAssets {
    fn from_world(world: &mut World) -> Self {
        let body_mesh = {
            let mut meshes = world.resource_mut::<Assets<Mesh>>();
            meshes.add(Capsule3d::new(0.28, 0.95))
        };
        let box_mesh = {
            let mut meshes = world.resource_mut::<Assets<Mesh>>();
            meshes.add(Cuboid::new(1.0, 1.0, 1.0))
        };
        let eye_mesh = {
            let mut meshes = world.resource_mut::<Assets<Mesh>>();
            meshes.add(Sphere::new(0.2))
        };
        let pupil_mesh = {
            let mut meshes = world.resource_mut::<Assets<Mesh>>();
            meshes.add(Sphere::new(0.085))
        };

        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        Self {
            body_mesh,
            box_mesh,
            eye_mesh,
            pupil_mesh,
            wood: materials.add(StandardMaterial {
                base_color: Color::srgb(0.52, 0.33, 0.15),
                perceptual_roughness: 0.88,
                ..default()
            }),
            wood_dark: materials.add(StandardMaterial {
                base_color: Color::srgb(0.12, 0.07, 0.03),
                perceptual_roughness: 0.95,
                ..default()
            }),
            bat_wood: materials.add(StandardMaterial {
                base_color: Color::srgb(0.68, 0.48, 0.24),
                perceptual_roughness: 0.75,
                ..default()
            }),
            mouth: materials.add(StandardMaterial {
                base_color: Color::srgb(0.08, 0.04, 0.03),
                perceptual_roughness: 0.95,
                ..default()
            }),
            eye_white: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 1.0, 0.98),
                emissive: LinearRgba::from(Color::srgb(0.55, 0.55, 0.5)),
                ..default()
            }),
            pupil: materials.add(StandardMaterial {
                base_color: Color::srgb(0.02, 0.02, 0.02),
                emissive: LinearRgba::from(Color::srgb(0.05, 0.0, 0.0)),
                ..default()
            }),
            foot: materials.add(StandardMaterial {
                base_color: Color::srgb(0.72, 0.56, 0.42),
                perceptual_roughness: 0.9,
                ..default()
            }),
        }
    }
}

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TungModelAssets>().add_systems(
            Update,
            enemy_update
                .in_set(crate::game_state::GameplaySystems::Enemy)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

pub fn spawn_enemy_at(
    commands: &mut Commands,
    assets: &TungModelAssets,
    position: Vec3,
) {
    let mut spawn_pos = position;
    spawn_pos.y = TUNG_HEIGHT;

    commands
        .spawn((
            Enemy,
            Transform::from_translation(spawn_pos),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(assets.body_mesh.clone()),
                MeshMaterial3d(assets.wood.clone()),
                Transform::from_xyz(0.0, 0.18, 0.0),
            ));

            parent.spawn((
                Mesh3d(assets.box_mesh.clone()),
                MeshMaterial3d(assets.wood_dark.clone()),
                Transform::from_xyz(0.0, 0.12, -0.3).with_scale(Vec3::new(0.42, 0.07, 0.06)),
            ));

            for x in [-0.14_f32, 0.14] {
                parent.spawn((
                    Mesh3d(assets.eye_mesh.clone()),
                    MeshMaterial3d(assets.eye_white.clone()),
                    Transform::from_xyz(x, 0.5, -0.34),
                ));
                parent.spawn((
                    Mesh3d(assets.pupil_mesh.clone()),
                    MeshMaterial3d(assets.pupil.clone()),
                    Transform::from_xyz(x, 0.5, -0.46),
                ));
            }

            for (x, y, z, rot_z) in [
                (-0.1_f32, 0.28, -0.39, 0.42),
                (0.0, 0.25, -0.4, 0.0),
                (0.1, 0.28, -0.39, -0.42),
            ] {
                parent.spawn((
                    Mesh3d(assets.box_mesh.clone()),
                    MeshMaterial3d(assets.mouth.clone()),
                    Transform::from_xyz(x, y, z)
                        .with_rotation(Quat::from_rotation_z(rot_z))
                        .with_scale(Vec3::new(0.09, 0.035, 0.035)),
                ));
            }

            for x in [-0.2_f32, 0.2] {
                parent.spawn((
                    Mesh3d(assets.box_mesh.clone()),
                    MeshMaterial3d(assets.wood.clone()),
                    Transform::from_xyz(x, -0.48, 0.0).with_scale(Vec3::new(0.1, 0.38, 0.1)),
                ));
                parent.spawn((
                    Mesh3d(assets.box_mesh.clone()),
                    MeshMaterial3d(assets.foot.clone()),
                    Transform::from_xyz(x, -0.82, 0.08).with_scale(Vec3::new(0.3, 0.1, 0.42)),
                ));
            }

            parent.spawn((
                Mesh3d(assets.box_mesh.clone()),
                MeshMaterial3d(assets.bat_wood.clone()),
                Transform::from_xyz(0.36, 0.34, 0.08)
                    .with_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, 0.35, 0.15))
                    .with_scale(Vec3::new(0.07, 0.07, 0.55)),
            ));
            parent.spawn((
                Mesh3d(assets.box_mesh.clone()),
                MeshMaterial3d(assets.bat_wood.clone()),
                Transform::from_xyz(0.52, 0.52, 0.1)
                    .with_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, 0.35, 0.15))
                    .with_scale(Vec3::new(0.15, 0.15, 0.38)),
            ));
        });
}

pub fn maybe_spawn_enemy_after_door(
    commands: &mut Commands,
    assets: &TungModelAssets,
    enemies: &Query<Entity, With<Enemy>>,
    door_number: u32,
) {
    if door_number < 3 || enemies.iter().count() >= MAX_ENEMIES {
        return;
    }

    if door_number % 2 != 0 {
        return;
    }

    let x = ((door_number.wrapping_mul(17) % 50) as f32 / 10.0) - 2.5;
    let z = 6.0 + ((door_number.wrapping_mul(31) % 120) as f32 / 10.0);
    spawn_enemy_at(
        commands,
        assets,
        Vec3::new(x, TUNG_HEIGHT, z.min(HALL_LENGTH - 6.0)),
    );
}

fn enemy_update(
    time: Res<Time>,
    mut player: Query<&mut Transform, (With<Player>, Without<Enemy>)>,
    mut enemies: Query<&mut Transform, (With<Enemy>, Without<Player>)>,
    mut health: ResMut<PlayerHealth>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Ok(mut player_tf) = player.single_mut() else {
        return;
    };
    let player_pos = player_tf.translation;

    health.hit_cooldown = (health.hit_cooldown - time.delta_secs()).max(0.0);

    let mut damage_push = None;

    for mut enemy_tf in &mut enemies {
        let mut to_player = player_pos - enemy_tf.translation;
        to_player.y = 0.0;
        let dist = to_player.length();

        if dist < KILL_DISTANCE {
            if health.hit_cooldown <= 0.0 && damage_push.is_none() && dist > 0.01 {
                damage_push = Some(to_player / dist);
            }
        } else {
            to_player /= dist;
            enemy_tf.translation += to_player * ENEMY_SPEED * time.delta_secs();
        }

        enemy_tf.translation.y = TUNG_HEIGHT;
        let y = enemy_tf.translation.y;
        enemy_tf.look_at(Vec3::new(player_pos.x, y, player_pos.z), Vec3::Y);
    }

    if let Some(push) = damage_push {
        health.lives = health.lives.saturating_sub(1);
        health.hit_cooldown = HIT_COOLDOWN;
        player_tf.translation += Vec3::new(push.x * 1.5, 0.0, push.z * 1.5);

        if health.lives == 0 {
            next_state.set(GameState::GameOver);
        }
    }
}
