use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
    Paused,
    LevelUp,
    GameOver,
}

impl GameState {
    pub fn is_menu(&self) -> bool {
        matches!(self, GameState::Paused | GameState::GameOver | GameState::Loading | GameState::LevelUp)
    }
}
