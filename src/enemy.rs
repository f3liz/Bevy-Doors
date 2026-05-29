use bevy::prelude::*;

use crate::game_state::GameState;
use crate::hallway::HALL_LENGTH;
use crate::player::Player;

pub const ENEMY_SPEED: f32 = 2.8;
pub const KILL_DISTANCE: f32 = 1.1;
pub const MAX_ENEMIES: usize = 2;

#[derive(Component)]
pub struct Enemy;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (enemy_chase, enemy_kill_player).run_if(in_state(GameState::Playing)),
        );
    }
}

pub fn spawn_enemy_at(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
) {
    let mesh = meshes.add(Cuboid::new(0.9, 1.8, 0.9));
    let mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.08, 0.02, 0.02),
        emissive: LinearRgba::from(Color::srgb(0.25, 0.0, 0.0)),
        ..default()
    });

    commands.spawn((
        Enemy,
        Mesh3d(mesh),
        MeshMaterial3d(mat),
        Transform::from_translation(position),
    ));
}

pub fn maybe_spawn_enemy_after_door(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
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
        meshes,
        materials,
        Vec3::new(x, 0.9, z.min(HALL_LENGTH - 6.0)),
    );
}

pub fn despawn_enemies(mut commands: Commands, enemies: Query<Entity, With<Enemy>>) {
    for entity in &enemies {
        commands.entity(entity).despawn();
    }
}

fn enemy_chase(
    time: Res<Time>,
    player: Query<&Transform, With<Player>>,
    mut enemies: Query<&mut Transform, With<Enemy>>,
) {
    let Ok(player_tf) = player.single() else {
        return;
    };

    for mut enemy_tf in &mut enemies {
        let mut to_player = player_tf.translation - enemy_tf.translation;
        to_player.y = 0.0;
        let dist = to_player.length();
        if dist < 0.05 {
            continue;
        }
        to_player /= dist;
        enemy_tf.translation += to_player * ENEMY_SPEED * time.delta_secs();
        enemy_tf.translation.y = 0.9;
        enemy_tf.look_at(
            Vec3::new(
                player_tf.translation.x,
                enemy_tf.translation.y,
                player_tf.translation.z,
            ),
            Vec3::Y,
        );
    }
}

fn enemy_kill_player(
    player: Query<&Transform, With<Player>>,
    enemies: Query<&Transform, With<Enemy>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Ok(player_tf) = player.single() else {
        return;
    };

    for enemy_tf in &enemies {
        let flat = Vec2::new(
            player_tf.translation.x - enemy_tf.translation.x,
            player_tf.translation.z - enemy_tf.translation.z,
        );
        if flat.length() < KILL_DISTANCE {
            next_state.set(GameState::GameOver);
            return;
        }
    }
}
