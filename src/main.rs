mod door;
mod enemy;
mod game_state;
mod hallway;
mod jumpscare;
mod lobby;
mod player;
mod transition;
mod ui;

use bevy::prelude::*;
use bevy::window::{PresentMode, WindowResolution};

use game_state::{GameState, GameplaySystems, HallwayProgress, PlayerHealth, RunStats};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Bevy Doors".into(),
                    resolution: WindowResolution::new(1280, 720),
                    present_mode: PresentMode::AutoVsync,
                    ..default()
                }),
                ..default()
            }),
        )
        .init_state::<GameState>()
        .init_resource::<RunStats>()
        .init_resource::<PlayerHealth>()
        .init_resource::<HallwayProgress>()
        .configure_sets(
            Update,
            (
                GameplaySystems::Player,
                GameplaySystems::Door,
                GameplaySystems::Enemy,
                GameplaySystems::Jumpscare,
                GameplaySystems::Animation,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        )
        .add_plugins((
            player::PlayerPlugin,
            lobby::LobbyPlugin,
            hallway::HallwayPlugin,
            transition::TransitionPlugin,
            enemy::EnemyPlugin,
            jumpscare::JumpscarePlugin,
            ui::UiPlugin,
        ))
        .add_systems(Startup, setup_scene.after(ui::setup_ui))
        .run();
}

fn setup_scene(mut commands: Commands) {
    commands.spawn((
        AmbientLight {
            color: Color::srgb(0.35, 0.32, 0.28),
            brightness: 12.0,
            ..default()
        },
    ));
}
