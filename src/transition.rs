use bevy::prelude::*;

use crate::door::{Door, DoorLeaf, DoorOpen};
use crate::enemy::maybe_spawn_enemy_after_door;
use crate::game_state::{GameState, HallwayProgress, RunStats};
use crate::hallway::{spawn_hallway_segment, HallwayEntity};
use crate::player::{Player, PlayerLook, PLAYER_EYE_HEIGHT};

#[derive(Resource, Default)]
pub struct RoomTransition {
    pub active: bool,
    pub timer: f32,
    pub phase: TransitionPhase,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum TransitionPhase {
    #[default]
    Idle,
    FadeOut,
    Swap,
    FadeIn,
}

#[derive(Component)]
pub struct FadeOverlay;

#[derive(Resource, Default)]
pub struct WrongDoorFlash {
    pub remaining: f32,
}

pub struct TransitionPlugin;

impl Plugin for TransitionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RoomTransition>()
            .init_resource::<WrongDoorFlash>()
            .add_systems(
                Update,
                (
                    tick_wrong_door_flash,
                    update_fade_overlay,
                    animate_doors,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                tick_room_transition
                    .in_set(crate::game_state::GameplaySystems::Door)
                    .after(crate::hallway::interact_with_doors)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

pub fn start_room_transition(transition: &mut RoomTransition) {
    transition.active = true;
    transition.timer = 0.0;
    transition.phase = TransitionPhase::FadeOut;
}

pub fn start_wrong_door_flash(flash: &mut WrongDoorFlash) {
    flash.remaining = 0.35;
}

fn tick_wrong_door_flash(time: Res<Time>, mut flash: ResMut<WrongDoorFlash>) {
    if flash.remaining > 0.0 {
        flash.remaining = (flash.remaining - time.delta_secs()).max(0.0);
    }
}

fn tick_room_transition(
    time: Res<Time>,
    mut transition: ResMut<RoomTransition>,
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
    if !transition.active {
        return;
    }

    transition.timer += time.delta_secs();

    match transition.phase {
        TransitionPhase::FadeOut if transition.timer >= 0.35 => {
            transition.phase = TransitionPhase::Swap;
            transition.timer = 0.0;

            stats.doors_cleared += 1;
            progress.door_number += 1;
            progress.room_seed = progress
                .room_seed
                .wrapping_mul(1664525)
                .wrapping_add(1013904223);

            for entity in &hallway {
                commands.entity(entity).despawn();
            }

            if let Ok((mut player_tf, mut look)) = player.single_mut() {
                player_tf.translation = Vec3::new(0.0, PLAYER_EYE_HEIGHT, 2.0);
                player_tf.rotation = Quat::IDENTITY;
                look.yaw = 0.0;
                look.pitch = 0.0;
                look.yaw_velocity = 0.0;
                look.pitch_velocity = 0.0;
            }
            if let Ok(mut camera_tf) = cameras.single_mut() {
                camera_tf.rotation = Quat::IDENTITY;
            }

            spawn_hallway_segment(
                &mut commands,
                &mut meshes,
                &mut materials,
                progress.door_number,
                progress.room_seed,
            );
            maybe_spawn_enemy_after_door(
                &mut commands,
                &mut meshes,
                &mut materials,
                &enemies,
                progress.door_number,
            );
        }
        TransitionPhase::Swap => {
            transition.phase = TransitionPhase::FadeIn;
            transition.timer = 0.0;
        }
        TransitionPhase::FadeIn if transition.timer >= 0.35 => {
            transition.active = false;
            transition.phase = TransitionPhase::Idle;
            transition.timer = 0.0;
        }
        _ => {}
    }
}

fn update_fade_overlay(
    mut commands: Commands,
    transition: Res<RoomTransition>,
    flash: Res<WrongDoorFlash>,
    mut overlay_queries: ParamSet<(
        Query<Entity, With<FadeOverlay>>,
        Query<&mut BackgroundColor, With<FadeOverlay>>,
    )>,
) {
    let fade_strength = if transition.active {
        match transition.phase {
            TransitionPhase::FadeOut => (transition.timer / 0.35).clamp(0.0, 1.0),
            TransitionPhase::Swap => 1.0,
            TransitionPhase::FadeIn => (1.0 - transition.timer / 0.35).clamp(0.0, 1.0),
            TransitionPhase::Idle => 0.0,
        }
    } else {
        0.0
    };

    let flash_strength = (flash.remaining / 0.35).clamp(0.0, 1.0) * 0.55;
    let alpha = fade_strength.max(flash_strength);

    if alpha <= 0.001 {
        for entity in overlay_queries.p0().iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    let color = if flash_strength > fade_strength {
        Color::srgba(0.35, 0.02, 0.02, alpha)
    } else {
        Color::srgba(0.02, 0.0, 0.0, alpha)
    };

    if overlay_queries.p0().is_empty() {
        commands.spawn((
            FadeOverlay,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(color),
        ));
    } else if let Ok(mut bg) = overlay_queries.p1().single_mut() {
        *bg = BackgroundColor(color);
    }
}

fn animate_doors(
    time: Res<Time>,
    player: Query<&Transform, With<Player>>,
    mut leaves: Query<(&Door, &mut DoorOpen, &mut Transform), With<DoorLeaf>>,
) {
    let Ok(player_tf) = player.single() else {
        return;
    };

    for (door, mut open, mut leaf_tf) in &mut leaves {
        let near = crate::door::player_at_door(player_tf, door.placement);
        let target_open = if near { 0.75 } else { 0.0 };
        let blend = 1.0 - (-8.0 * time.delta_secs()).exp();
        open.amount += (target_open - open.amount) * blend;

        let base_y = match door.placement {
            crate::door::DoorPlacement::Ahead => std::f32::consts::PI,
            crate::door::DoorPlacement::Left => -std::f32::consts::FRAC_PI_2,
            crate::door::DoorPlacement::Right => std::f32::consts::FRAC_PI_2,
        };
        leaf_tf.rotation =
            Quat::from_rotation_y(base_y) * Quat::from_rotation_y(open.amount * 1.15);
    }
}
