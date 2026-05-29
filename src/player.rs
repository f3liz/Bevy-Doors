use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::game_state::GameState;
use crate::transition::RoomTransition;

use crate::hallway::HALL_WIDTH;

pub const PLAYER_EYE_HEIGHT: f32 = 1.6;
pub const MOVE_SPEED: f32 = 5.5;
pub const MOUSE_SENSITIVITY: f32 = 0.002;
const MOVE_SMOOTHING: f32 = 14.0;
const LOOK_SMOOTHING: f32 = 22.0;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerCamera;

#[derive(Component, Default)]
pub struct PlayerLook {
    pub yaw: f32,
    pub pitch: f32,
    pub yaw_velocity: f32,
    pub pitch_velocity: f32,
}

#[derive(Component, Default)]
pub struct PlayerMotion {
    pub velocity: Vec3,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            player_controls
                .in_set(crate::game_state::GameplaySystems::Player)
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(Update, cursor_control);
    }
}

pub fn spawn_player(commands: &mut Commands) {
    commands
        .spawn((
            Player,
            PlayerLook::default(),
            PlayerMotion::default(),
            Transform::from_xyz(0.0, PLAYER_EYE_HEIGHT, 2.0),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                PlayerCamera,
                Camera3d::default(),
                IsDefaultUiCamera,
                Transform::IDENTITY,
            ));
        });
}

fn player_controls(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    transition: Res<RoomTransition>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut player_query: Query<
        (&mut Transform, &mut PlayerLook, &mut PlayerMotion),
        (With<Player>, Without<PlayerCamera>),
    >,
    mut camera_query: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
) {
    let Ok((mut transform, mut look, mut motion)) = player_query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    let blocked = transition.active;

    if !blocked {
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
        let target_velocity = if direction.length_squared() > 0.0 {
            direction.normalize() * MOVE_SPEED
        } else {
            Vec3::ZERO
        };

        let blend = 1.0 - (-MOVE_SMOOTHING * dt).exp();
        motion.velocity = motion.velocity.lerp(target_velocity, blend);
    } else {
        motion.velocity = Vec3::ZERO;
    }

    transform.translation += motion.velocity * dt;

    let half_w = HALL_WIDTH * 0.5 - 0.4;
    transform.translation.x = transform.translation.x.clamp(-half_w, half_w);
    transform.translation.y = PLAYER_EYE_HEIGHT;

    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    if !blocked {
        let mut delta = Vec2::ZERO;
        for event in mouse_motion.read() {
            delta += event.delta;
        }

        if delta != Vec2::ZERO {
            look.yaw_velocity -= delta.x * MOUSE_SENSITIVITY * 60.0;
            look.pitch_velocity -= delta.y * MOUSE_SENSITIVITY * 60.0;
        }
    }

    let look_blend = 1.0 - (-LOOK_SMOOTHING * dt).exp();
    look.yaw += look.yaw_velocity * look_blend;
    look.pitch += look.pitch_velocity * look_blend;
    look.pitch = look.pitch.clamp(-1.4, 1.4);
    look.yaw_velocity *= 1.0 - look_blend;
    look.pitch_velocity *= 1.0 - look_blend;

    transform.rotation = Quat::from_rotation_y(look.yaw);
    camera_transform.rotation = Quat::from_rotation_x(look.pitch);
}

fn cursor_control(
    state: Res<State<GameState>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut window_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let Ok(mut cursor) = window_query.single_mut() else {
        return;
    };

    match state.get() {
        GameState::Playing => {
            if keys.just_pressed(KeyCode::Escape) {
                cursor.grab_mode = CursorGrabMode::None;
                cursor.visible = true;
            } else {
                cursor.grab_mode = CursorGrabMode::Locked;
                cursor.visible = false;
            }
        }
        GameState::Lobby | GameState::GameOver => {
            cursor.grab_mode = CursorGrabMode::None;
            cursor.visible = true;
        }
    }
}
