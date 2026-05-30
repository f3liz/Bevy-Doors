use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow, WindowCloseRequested};

use crate::game_state::GameState;
use crate::transition::RoomTransition;

use crate::hallway::HALL_WIDTH;

pub const PLAYER_EYE_HEIGHT: f32 = 1.6;
pub const MOVE_SPEED: f32 = 5.5;
pub const MOUSE_SENSITIVITY: f32 = 0.0018;
/// Default yaw so the player looks down the hallway (+Z), not at the back wall.
pub const DEFAULT_LOOK_YAW: f32 = std::f32::consts::PI;
const MOVE_SMOOTHING: f32 = 10.0;
const LOOK_SMOOTHING: f32 = 28.0;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerCamera;

#[derive(Component, Default)]
pub struct PlayerLook {
    pub yaw: f32,
    pub pitch: f32,
    pub target_yaw: f32,
    pub target_pitch: f32,
}

#[derive(Component, Default)]
pub struct PlayerMotion {
    pub velocity: Vec3,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, lock_cursor)
            .add_systems(PostUpdate, quit_on_escape)
            .add_systems(
                Update,
                player_controls
                    .in_set(crate::game_state::GameplaySystems::Player)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

pub fn reset_player_into_room(
    transform: &mut Transform,
    look: &mut PlayerLook,
    camera: Option<&mut Transform>,
) {
    look.yaw = DEFAULT_LOOK_YAW;
    look.pitch = 0.0;
    look.target_yaw = DEFAULT_LOOK_YAW;
    look.target_pitch = 0.0;
    transform.rotation = Quat::from_rotation_y(DEFAULT_LOOK_YAW);
    if let Some(cam) = camera {
        cam.rotation = Quat::IDENTITY;
    }
}

pub fn spawn_player(commands: &mut Commands) {
    let mut look = PlayerLook::default();
    look.yaw = DEFAULT_LOOK_YAW;
    look.target_yaw = DEFAULT_LOOK_YAW;

    commands
        .spawn((
            Player,
            look,
            PlayerMotion::default(),
            Transform::from_xyz(0.0, PLAYER_EYE_HEIGHT, 2.0)
                .with_rotation(Quat::from_rotation_y(DEFAULT_LOOK_YAW)),
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

fn quit_on_escape(
    keys: Res<ButtonInput<KeyCode>>,
    mut cursor: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut close: MessageWriter<WindowCloseRequested>,
    windows: Query<Entity, With<PrimaryWindow>>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }

    if let Ok(mut cursor) = cursor.single_mut() {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    }

    if let Ok(window) = windows.single() {
        close.write(WindowCloseRequested { window });
    }
}

fn lock_cursor(
    state: Res<State<GameState>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut window_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        return;
    }

    let Ok(mut cursor) = window_query.single_mut() else {
        return;
    };

    match state.get() {
        GameState::Playing => {
            cursor.grab_mode = CursorGrabMode::Locked;
            cursor.visible = false;
        }
        GameState::Lobby | GameState::GameOver => {
            cursor.grab_mode = CursorGrabMode::None;
            cursor.visible = true;
        }
    }
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

    let dt = time.delta_secs().min(1.0 / 30.0);
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

        let move_blend = 1.0 - (-MOVE_SMOOTHING * dt).exp();
        motion.velocity = motion.velocity.lerp(target_velocity, move_blend);

        for event in mouse_motion.read() {
            look.target_yaw -= event.delta.x * MOUSE_SENSITIVITY;
            look.target_pitch -= event.delta.y * MOUSE_SENSITIVITY;
        }
        look.target_pitch = look.target_pitch.clamp(-1.4, 1.4);
    } else {
        motion.velocity = Vec3::ZERO;
    }

    transform.translation += motion.velocity * dt;

    let half_w = HALL_WIDTH * 0.5 - 0.4;
    transform.translation.x = transform.translation.x.clamp(-half_w, half_w);
    transform.translation.y = PLAYER_EYE_HEIGHT;

    let look_blend = 1.0 - (-LOOK_SMOOTHING * dt).exp();
    look.yaw = look.yaw + (look.target_yaw - look.yaw) * look_blend;
    look.pitch = look.pitch + (look.target_pitch - look.pitch) * look_blend;

    transform.rotation = Quat::from_rotation_y(look.yaw);

    if let Ok(mut camera_transform) = camera_query.single_mut() {
        camera_transform.rotation = Quat::from_rotation_x(look.pitch);
    }
}
