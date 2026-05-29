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
}

#[derive(Resource, Default)]
pub struct RunStats {
    pub doors_cleared: u32,
}

#[derive(Resource)]
pub struct HallwayProgress {
    pub door_number: u32,
    pub current_placement: DoorPlacement,
}

impl Default for HallwayProgress {
    fn default() -> Self {
        Self {
            door_number: 1,
            current_placement: DoorPlacement::for_door_number(1),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DoorPlacement {
    Ahead,
    Left,
    Right,
}

impl DoorPlacement {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ahead => "Ahead",
            Self::Left => "Left",
            Self::Right => "Right",
        }
    }

    pub fn for_door_number(door_number: u32) -> Self {
        match door_number % 3 {
            1 => Self::Ahead,
            2 => Self::Left,
            _ => Self::Right,
        }
    }
}
