use bevy::prelude::*;

use crate::game_state::GameState;
use crate::player::PlayerCamera;

#[derive(Resource)]
pub struct JumpscareTimer {
    pub elapsed: f32,
    pub next_in: f32,
}

impl Default for JumpscareTimer {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            next_in: 22.0,
        }
    }
}

#[derive(Resource, Default)]
pub struct ActiveJumpscare {
    pub remaining: f32,
}

#[derive(Component)]
pub struct JumpscareOverlay;

#[derive(Component)]
pub struct JumpscareFace;

pub struct JumpscarePlugin;

impl Plugin for JumpscarePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<JumpscareTimer>()
            .init_resource::<ActiveJumpscare>()
            .add_systems(OnEnter(GameState::Playing), reset_jumpscare_timer)
            .add_systems(
                Update,
                jumpscare_system
                    .in_set(crate::game_state::GameplaySystems::Jumpscare)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), cleanup_jumpscare);
    }
}

fn reset_jumpscare_timer(mut timer: ResMut<JumpscareTimer>, mut active: ResMut<ActiveJumpscare>) {
    *timer = JumpscareTimer::default();
    *active = ActiveJumpscare::default();
}

fn jumpscare_system(
    time: Res<Time>,
    mut timer: ResMut<JumpscareTimer>,
    mut active: ResMut<ActiveJumpscare>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cameras: Query<Entity, With<PlayerCamera>>,
    overlay: Query<Entity, With<JumpscareOverlay>>,
    faces: Query<Entity, With<JumpscareFace>>,
) {
    if !overlay.is_empty() || !faces.is_empty() {
        if active.remaining <= 0.0 {
            return;
        }

        active.remaining -= time.delta_secs();
        if active.remaining > 0.0 {
            return;
        }

        for entity in &overlay {
            commands.entity(entity).despawn();
        }
        for entity in &faces {
            commands.entity(entity).despawn();
        }
        return;
    }

    timer.elapsed += time.delta_secs();
    if timer.elapsed < timer.next_in {
        return;
    }

    timer.elapsed = 0.0;
    timer.next_in = 18.0 + (timer.next_in % 12.0);
    active.remaining = 0.45;

    commands
        .spawn((
            JumpscareOverlay,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.55, 0.0, 0.0, 0.85)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("!"),
                TextFont {
                    font_size: 120.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });

    if let Ok(camera) = cameras.single() {
        let face_mesh = meshes.add(Cuboid::new(1.2, 1.2, 0.1));
        let face_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.05, 0.05, 0.05),
            emissive: LinearRgba::from(Color::srgb(0.8, 0.0, 0.0)),
            ..default()
        });

        commands.entity(camera).with_children(|parent| {
            parent.spawn((
                JumpscareFace,
                Mesh3d(face_mesh),
                MeshMaterial3d(face_mat),
                Transform::from_xyz(0.0, 0.0, -1.2),
            ));
        });
    }
}

pub fn cleanup_jumpscare(
    mut commands: Commands,
    overlay: Query<Entity, With<JumpscareOverlay>>,
    faces: Query<Entity, With<JumpscareFace>>,
    mut active: ResMut<ActiveJumpscare>,
) {
    for entity in &overlay {
        commands.entity(entity).despawn();
    }
    for entity in &faces {
        commands.entity(entity).despawn();
    }
    active.remaining = 0.0;
}
