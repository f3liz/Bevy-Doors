use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::game_state::GameState;

pub const PLAYER_EYE_HEIGHT: f32 = 1.6;
pub const MOVE_SPEED: f32 = 5.5;
pub const MOUSE_SENSITIVITY: f32 = 0.002;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerCamera;

#[derive(Component, Default)]
pub struct PlayerLook {
    pub pitch: f32,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                player_movement,
                player_look,
                cursor_grab_on_playing,
                escape_releases_cursor,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

pub fn spawn_player(commands: &mut Commands) {
    commands
        .spawn((
            Player,
            PlayerLook::default(),
            Transform::from_xyz(0.0, PLAYER_EYE_HEIGHT, 2.0),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                PlayerCamera,
                Camera3d::default(),
                Transform::IDENTITY,
            ));
        });
}

fn player_movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };

    let mut direction = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        direction += transform.forward().as_vec3();
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        direction -= transform.forward().as_vec3();
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        direction -= transform.right().as_vec3();
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        direction += transform.right().as_vec3();
    }

    direction.y = 0.0;
    if direction.length_squared() > 0.0 {
        direction = direction.normalize();
        transform.translation += direction * MOVE_SPEED * time.delta_secs();
    }
}

fn player_look(
    mut mouse_motion: MessageReader<bevy::input::mouse::MouseMotion>,
    mut player_query: Query<(&mut Transform, &mut PlayerLook), With<Player>>,
    mut camera_query: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
) {
    let Ok((mut player_transform, mut look)) = player_query.single_mut() else {
        return;
    };
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    let mut delta = Vec2::ZERO;
    for event in mouse_motion.read() {
        delta += event.delta;
    }
    if delta == Vec2::ZERO {
        return;
    }

    look.pitch -= delta.y * MOUSE_SENSITIVITY;
    look.pitch = look.pitch.clamp(-1.4, 1.4);

    player_transform.rotate_y(-delta.x * MOUSE_SENSITIVITY);
    camera_transform.rotation = Quat::from_rotation_x(look.pitch);
}

fn cursor_grab_on_playing(
    mut window_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let Ok(mut cursor) = window_query.single_mut() else {
        return;
    };
    cursor.grab_mode = CursorGrabMode::Locked;
    cursor.visible = false;
}

fn escape_releases_cursor(
    keys: Res<ButtonInput<KeyCode>>,
    mut window_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    let Ok(mut cursor) = window_query.single_mut() else {
        return;
    };
    cursor.grab_mode = CursorGrabMode::None;
    cursor.visible = true;
}
