use std::collections::HashMap;
use crate::e::entity::{Entity, World};
use crate::c::*;
use eframe::egui::Pos2;

// Factory for creating and managing hex map entities
pub struct HexMapFactory;

impl HexMapFactory {
    // Create a new hex map with specified dimensions
    pub fn create_map(world: &mut World, width: i32, height: i32, hex_size: f32, origin: Pos2) {
        // Create the game state entity
        let game_state_entity = world.create_entity();
        world.add_component(game_state_entity, GameState {
            selected_entity: None,
            hover_entity: None,
            current_turn: 0, // Player starts
            turn_number: 1,
            game_over: false,
            player_won: false,
        });
        
        // Create map settings entity
        let map_settings_entity = world.create_entity();
        world.add_component(map_settings_entity, MapSettings {
            map_width: width,
            map_height: height,
            hex_size,
            origin,
        });
        
        // Create a HashMap to store hex coordinates to entity mapping
        let mut hex_entity_map = HashMap::new();
        
        // Generate hex tiles
        for q in 0..width {
            for r in 0..height {
                let coord = HexCoord { q, r };
                let entity = world.create_entity();
                
                // Position component
                world.add_component(entity, Position { coord });
                
                // Random terrain (simplified for this example)
                let terrain_type = match (q + r) % 4 {
                    0 => TerrainType::Plain,
                    1 => TerrainType::Forest,
                    2 => TerrainType::Mountain,
                    _ => TerrainType::Water,
                };
                
                world.add_component(entity, Terrain { terrain_type });
                
                // Add to mapping
                hex_entity_map.insert(coord, entity);
            }
        }
        
        // Add the hex entity map to the map settings entity
        world.add_component(map_settings_entity, HexEntityMap { map: hex_entity_map });
        
        // Add player units
        Self::add_player_unit(world, HexCoord { q: 1, r: 1 }, UnitType::Infantry);
        Self::add_player_unit(world, HexCoord { q: 2, r: 2 }, UnitType::Archer);
        Self::add_player_unit(world, HexCoord { q: 3, r: 1 }, UnitType::Cavalry);
        
        // Add enemy units
        Self::add_enemy_unit(world, HexCoord { q: width - 2, r: height - 2 }, UnitType::Infantry);
        Self::add_enemy_unit(world, HexCoord { q: width - 3, r: height - 3 }, UnitType::Archer);
        Self::add_enemy_unit(world, HexCoord { q: width - 4, r: height - 2 }, UnitType::Cavalry);
    }
    
    // Add a player unit at the specified coordinate
    fn add_player_unit(world: &mut World, coord: HexCoord, unit_type: UnitType) {
        let hex_entities = Self::get_hex_entity_map(world);
        
        if let Some(&entity) = hex_entities.get(&coord) {
            // Add unit components based on type
            match unit_type {
                UnitType::Infantry => {
                    world.add_component(entity, UnitStats {
                        unit_type,
                        attack: 3,
                        defense: 2,
                        movement: 2,
                        range: 1,
                        max_health: 10,
                    });
                },
                UnitType::Archer => {
                    world.add_component(entity, UnitStats {
                        unit_type,
                        attack: 4,
                        defense: 1,
                        movement: 2,
                        range: 2,
                        max_health: 8,
                    });
                },
                UnitType::Cavalry => {
                    world.add_component(entity, UnitStats {
                        unit_type,
                        attack: 5,
                        defense: 1,
                        movement: 4,
                        range: 1,
                        max_health: 12,
                    });
                },
            }
            
            // Add unit state
            let movement = world.get_component::<UnitStats>(entity).unwrap().movement;
            world.add_component(entity, UnitState {
                health: world.get_component::<UnitStats>(entity).unwrap().max_health,
                movement_left: movement,
                has_acted: false,
            });
            
            // Add team component
            world.add_component(entity, Team { team_id: 0 }); // Player team
        }
    }
    
    // Add an enemy unit at the specified coordinate
    fn add_enemy_unit(world: &mut World, coord: HexCoord, unit_type: UnitType) {
        let hex_entities = Self::get_hex_entity_map(world);
        
        if let Some(&entity) = hex_entities.get(&coord) {
            // Add unit components based on type
            match unit_type {
                UnitType::Infantry => {
                    world.add_component(entity, UnitStats {
                        unit_type,
                        attack: 3,
                        defense: 2,
                        movement: 2,
                        range: 1,
                        max_health: 10,
                    });
                },
                UnitType::Archer => {
                    world.add_component(entity, UnitStats {
                        unit_type,
                        attack: 4,
                        defense: 1,
                        movement: 2,
                        range: 2,
                        max_health: 8,
                    });
                },
                UnitType::Cavalry => {
                    world.add_component(entity, UnitStats {
                        unit_type,
                        attack: 5,
                        defense: 1,
                        movement: 4,
                        range: 1,
                        max_health: 12,
                    });
                },
            }
            
            // Add unit state
            let movement = world.get_component::<UnitStats>(entity).unwrap().movement;
            world.add_component(entity, UnitState {
                health: world.get_component::<UnitStats>(entity).unwrap().max_health,
                movement_left: movement,
                has_acted: false,
            });
            
            // Add team component
            world.add_component(entity, Team { team_id: 1 }); // Enemy team
        }
    }
    
    // Get the hex entity map from the world
    pub fn get_hex_entity_map(world: &World) -> HashMap<HexCoord, Entity> {
        // Find the map settings entity
        if let Some(map_settings_entity) = Self::get_map_settings_entity(world) {
            // Get the hex entity map component
            if let Some(hex_entity_map) = world.get_component::<HexEntityMap>(map_settings_entity) {
                return hex_entity_map.map.clone();
            }
        }
        
        HashMap::new()
    }
    
    // Get the game state entity
    pub fn get_game_state_entity(world: &World) -> Option<Entity> {
        // Query for entities with a GameState component
        for (entity, _) in world.query::<GameState>() {
            return Some(entity);
        }
        
        None
    }
    
    // Get the map settings entity
    pub fn get_map_settings_entity(world: &World) -> Option<Entity> {
        // Query for entities with a MapSettings component
        for (entity, _) in world.query::<MapSettings>() {
            return Some(entity);
        }
        
        None
    }
    
    // Reset team units for a new turn
    pub fn reset_team_units_for_new_turn(world: &mut World, team_id: u8) {
        // We need to avoid mutably borrowing world multiple times
        // First find all entities with the team component
        let mut entity_ids = Vec::new();
        
        for (entity, team_comp) in world.query::<Team>() {
            if team_comp.team_id == team_id {
                entity_ids.push(entity);
            }
        }
        
        // Then update each entity
        for entity in entity_ids {
            if let Some(unit_stats) = world.get_component::<UnitStats>(entity) {
                let movement = unit_stats.movement;
                
                // Update unit state
                if let Some(unit_state) = world.get_component_mut::<UnitState>(entity) {
                    unit_state.movement_left = movement;
                    unit_state.has_acted = false;
                }
            }
        }
    }
    
    // Check if the game is over (one team has no units left)
    pub fn check_game_over(world: &World) -> (bool, bool) {
        let mut player_units = 0;
        let mut enemy_units = 0;
        
        // Count units for each team with health > 0
        for (entity, team) in world.query::<Team>() {
            if let Some(unit_state) = world.get_component::<UnitState>(entity) {
                if unit_state.health > 0 {
                    if team.team_id == 0 {
                        player_units += 1;
                    } else {
                        enemy_units += 1;
                    }
                }
            }
        }
        
        // Game is over if one team has no units
        if player_units == 0 {
            (true, false) // Enemy won
        } else if enemy_units == 0 {
            (true, true) // Player won
        } else {
            (false, false) // Game continues
        }
    }
} 