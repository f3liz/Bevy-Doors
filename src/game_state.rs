use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum GameState {
    #[default]
    Lobby,
    Playing,
    GameOver,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameplaySystems {
    Player,
    Door,
    Enemy,
    Jumpscare,
    Animation,
}

#[derive(Resource, Default)]
pub struct RunStats {
    pub doors_cleared: u32,
}

pub const MAX_LIVES: u32 = 3;
pub const HIT_COOLDOWN: f32 = 1.25;

#[derive(Resource)]
pub struct PlayerHealth {
    pub lives: u32,
    pub hit_cooldown: f32,
}

impl Default for PlayerHealth {
    fn default() -> Self {
        Self {
            lives: MAX_LIVES,
            hit_cooldown: 0.0,
        }
    }
}

impl PlayerHealth {
    pub fn reset(&mut self) {
        self.lives = MAX_LIVES;
        self.hit_cooldown = 0.0;
    }
}

#[derive(Resource)]
pub struct HallwayProgress {
    pub door_number: u32,
    pub room_seed: u32,
}

impl Default for HallwayProgress {
    fn default() -> Self {
        Self {
            door_number: 1,
            room_seed: 0xC0FFEE,
        }
    }
}
