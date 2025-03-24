use eframe::egui::{Color32, Pos2};
use std::collections::HashMap;
use crate::e::entity::Entity;

// 单位类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitType {
    Infantry,
    Archer,
    Cavalry,
}

// Hexagonal coordinate system (using axial coordinates)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HexCoord {
    pub q: i32, // column
    pub r: i32, // row
}

impl HexCoord {
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }
    
    // Get third coordinate (for cube coordinates)
    pub fn s(&self) -> i32 {
        -self.q - self.r
    }
    
    // Convert to cube coordinates (x, y, z)
    pub fn to_cube(&self) -> (i32, i32, i32) {
        (self.q, self.r, self.s())
    }
    
    // Get all 6 neighbors of this hex
    pub fn neighbors(&self) -> [HexCoord; 6] {
        [
            HexCoord::new(self.q + 1, self.r), 
            HexCoord::new(self.q + 1, self.r - 1),
            HexCoord::new(self.q, self.r - 1), 
            HexCoord::new(self.q - 1, self.r),
            HexCoord::new(self.q - 1, self.r + 1), 
            HexCoord::new(self.q, self.r + 1),
        ]
    }
    
    // Calculate distance between two hex coordinates
    pub fn distance(&self, other: &HexCoord) -> i32 {
        let (x1, y1, z1) = self.to_cube();
        let (x2, y2, z2) = other.to_cube();
        
        ((x1 - x2).abs() + (y1 - y2).abs() + (z1 - z2).abs()) / 2
    }
    
    // Convert pixel position to hex coordinate
    pub fn from_pixel(pos: Pos2, hex_size: f32, origin: Pos2) -> Self {
        let x = pos.x - origin.x;
        let y = pos.y - origin.y;
        
        // Use formulas that don't require SQRT_3 constant
        let q_float = (x * (2.0/3.0)) / hex_size;
        let r_float = ((-x / 3.0) + ((y * (3.0_f32.sqrt())) / 3.0)) / hex_size;
        
        // Round to the nearest hex
        let mut q = q_float.round() as i32;
        let mut r = r_float.round() as i32;
        let s = (-q - r) as f32;
        
        let q_diff = (q as f32 - q_float).abs();
        let r_diff = (r as f32 - r_float).abs();
        let s_diff = (s - (-q_float - r_float)).abs();
        
        if q_diff > r_diff && q_diff > s_diff {
            q = -r - s as i32;
        } else if r_diff > s_diff {
            r = -q - s as i32;
        }
        
        Self { q, r }
    }
    
    // Convert hex coordinate to pixel position
    pub fn to_pixel(&self, hex_size: f32, origin: Pos2) -> Pos2 {
        let x = hex_size * (3.0/2.0) * self.q as f32;
        let y = hex_size * (3.0_f32.sqrt()) * (self.r as f32 + self.q as f32 / 2.0);
        
        Pos2::new(origin.x + x, origin.y + y)
    }
}

// Terrain types for hex tiles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainType {
    Plain,
    Forest, 
    Mountain,
    Water,
}

impl TerrainType {
    // Get color for this terrain type
    pub fn color(&self) -> Color32 {
        match self {
            TerrainType::Plain => Color32::from_rgb(124, 252, 0),    // Light green
            TerrainType::Forest => Color32::from_rgb(34, 139, 34),    // Forest green
            TerrainType::Mountain => Color32::from_rgb(128, 128, 128), // Gray
            TerrainType::Water => Color32::from_rgb(65, 105, 225),    // Royal blue
        }
    }
    
    // Get movement cost for this terrain
    pub fn movement_cost(&self) -> i32 {
        match self {
            TerrainType::Plain => 1,
            TerrainType::Forest => 2,
            TerrainType::Mountain => 3,
            TerrainType::Water => 5,
        }
    }
    
    // Get terrain name as string
    pub fn name(&self) -> &'static str {
        match self {
            TerrainType::Plain => "Plain",
            TerrainType::Forest => "Forest",
            TerrainType::Mountain => "Mountain",
            TerrainType::Water => "Water",
        }
    }
}

// ===== HEX MAP COMPONENTS =====

// Position component representing the hex coordinate
#[derive(Clone, Debug)]
pub struct Position {
    pub coord: HexCoord,
}

// Terrain component
#[derive(Clone, Debug)]
pub struct Terrain {
    pub terrain_type: TerrainType,
}

// Unit stats component
#[derive(Clone, Debug)]
pub struct UnitStats {
    pub unit_type: UnitType,
    pub max_health: i32,
    pub attack: i32,
    pub defense: i32,
    pub movement: i32,
    pub range: i32,
}

// Current state of a unit (health, movement left, etc.)
#[derive(Clone, Debug)]
pub struct UnitState {
    pub health: i32,
    pub movement_left: i32,
    pub has_acted: bool,
}

// Hex entity map component - stores mapping between HexCoord and Entity
#[derive(Clone, Debug)]
pub struct HexEntityMap {
    pub map: HashMap<HexCoord, Entity>,
} 