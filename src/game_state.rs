use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum GameState {
    #[default]
    Lobby,
    Playing,
    GameOver,
}

#[derive(Resource, Default)]
pub struct RunStats {
    pub doors_cleared: u32,
}

#[derive(Resource)]
pub struct HallwayProgress {
    pub door_number: u32,
}

impl Default for HallwayProgress {
    fn default() -> Self {
        Self { door_number: 1 }
    }
}
