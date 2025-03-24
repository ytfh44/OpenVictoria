use crate::e::entity::Entity;
use std::any::Any;

// General marker components

// Selected component (marker)
#[derive(Debug, Clone)]
pub struct Selected;

// Hovering component (marker)
#[derive(Debug, Clone)]
pub struct Hovering;

// In movement range component (marker)
#[derive(Debug, Clone)]
pub struct InMovementRange;

// In attack range component (marker)
#[derive(Debug, Clone)]
pub struct InAttackRange;

// Team component
#[derive(Debug, Clone)]
pub struct Team {
    pub team_id: u8, // 0 for player, 1 for enemy
}

// Game state component (singleton)
#[derive(Debug, Clone)]
pub struct GameState {
    pub selected_entity: Option<Entity>,
    pub hover_entity: Option<Entity>,
    pub current_turn: u8, // 0 for player, 1 for enemy
    pub turn_number: i32,
    pub game_over: bool,
    pub player_won: bool,
}

// Map settings component (singleton)
#[derive(Debug, Clone)]
pub struct MapSettings {
    pub map_width: i32,
    pub map_height: i32,
    pub hex_size: f32,
    pub origin: eframe::egui::Pos2,
}

// 一个特征，用于存储和管理组件
pub trait ComponentVec {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
} 