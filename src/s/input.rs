use crate::e::entity::{Entity, World};
use crate::e::factory::HexMapFactory;
use crate::c::*;
use eframe::egui;
use std::collections::{HashMap, HashSet};

// System for handling mouse input and UI interactions
pub struct InputSystem;

impl InputSystem {
    pub fn update(world: &mut World, _ui: &egui::Ui, response: &egui::Response) {
        // Get the game state and map settings
        let game_state_entity = match HexMapFactory::get_game_state_entity(world) {
            Some(entity) => entity,
            None => return,
        };
        
        let map_settings_entity = match HexMapFactory::get_map_settings_entity(world) {
            Some(entity) => entity,
            None => return,
        };
        
        let (hex_size, origin) = {
            if let Some(settings) = world.get_component::<MapSettings>(map_settings_entity) {
                (settings.hex_size, settings.origin)
            } else {
                return;
            }
        };
        
        let current_turn = if let Some(game_state) = world.get_component::<GameState>(game_state_entity) {
            game_state.current_turn
        } else {
            return;
        };
        
        // Check if hovering over any tile
        let hovering_entities: Vec<Entity> = world.query::<Hovering>()
            .into_iter()
            .map(|(entity, _)| entity)
            .collect();
        
        // 清除所有悬停状态 - 只移除Hovering组件而不是删除实体
        for entity in hovering_entities {
            world.remove_component::<Hovering>(entity);
        }
        
        // Check for hovering
        if let Some(mouse_pos) = response.hover_pos() {
            let hex_coord = HexCoord::from_pixel(mouse_pos, hex_size, origin);
            let hex_entities = HexMapFactory::get_hex_entity_map(world);
            
            if let Some(&entity) = hex_entities.get(&hex_coord) {
                // Add hovering component
                world.add_component(entity, Hovering);
                
                // Update game state
                if let Some(game_state) = world.get_component_mut::<GameState>(game_state_entity) {
                    game_state.hover_entity = Some(entity);
                }
                
                // Handle click
                if response.clicked() {
                    let selected_entity = if let Some(game_state) = world.get_component::<GameState>(game_state_entity) {
                        game_state.selected_entity
                    } else {
                        None
                    };
                    
                    if let Some(selected) = selected_entity {
                        if selected == entity {
                            // Deselect if clicking on already selected tile
                            Self::deselect_current(world);
                        } else {
                            // Check if in movement or attack range
                            let in_movement_range = world.get_component::<InMovementRange>(entity).is_some();
                            let in_attack_range = world.get_component::<InAttackRange>(entity).is_some();
                            
                            if in_movement_range {
                                // Move unit
                                Self::move_unit(world, selected, entity);
                            } else if in_attack_range {
                                // Attack
                                Self::attack_unit(world, selected, entity);
                            } else {
                                // Try to select new tile
                                Self::select_tile(world, entity, current_turn);
                            }
                        }
                    } else {
                        // Try to select new tile
                        Self::select_tile(world, entity, current_turn);
                    }
                }
            }
        }
    }
    
    // Deselect the currently selected tile
    fn deselect_current(world: &mut World) {
        // Get game state
        let game_state_entity = match HexMapFactory::get_game_state_entity(world) {
            Some(entity) => entity,
            None => return,
        };
        
        // Clear selection in game state
        if let Some(game_state) = world.get_component_mut::<GameState>(game_state_entity) {
            game_state.selected_entity = None;
        }
        
        // Collect all entities with components to remove
        let movement_range_entities: Vec<Entity> = world.query::<InMovementRange>()
            .into_iter()
            .map(|(entity, _)| entity)
            .collect();
        
        let attack_range_entities: Vec<Entity> = world.query::<InAttackRange>()
            .into_iter()
            .map(|(entity, _)| entity)
            .collect();
        
        let selected_entities: Vec<Entity> = world.query::<Selected>()
            .into_iter()
            .map(|(entity, _)| entity)
            .collect();
        
        // Remove components (not implemented in our simple ECS)
        for _entity in movement_range_entities {
            // world.remove_component::<InMovementRange>(entity);
        }
        
        for _entity in attack_range_entities {
            // world.remove_component::<InAttackRange>(entity);
        }
        
        for _entity in selected_entities {
            // world.remove_component::<Selected>(entity);
        }
    }
    
    // Try to select a tile
    fn select_tile(world: &mut World, entity: Entity, current_turn: u8) {
        // Check if entity has a unit of the current team
        let team_matches = if let Some(team) = world.get_component::<Team>(entity) {
            team.team_id == current_turn
        } else {
            false
        };
        
        let has_movement = if let Some(unit_state) = world.get_component::<UnitState>(entity) {
            unit_state.movement_left > 0 && !unit_state.has_acted
        } else {
            false
        };
        
        if team_matches && has_movement {
            // Add Selected component
            world.add_component(entity, Selected);
            
            // Update game state
            let game_state_entity = HexMapFactory::get_game_state_entity(world).unwrap();
            if let Some(game_state) = world.get_component_mut::<GameState>(game_state_entity) {
                game_state.selected_entity = Some(entity);
            }
            
            // Calculate movement and attack ranges
            Self::calculate_ranges(world, entity);
        }
    }
    
    // Calculate movement and attack ranges for the selected entity
    fn calculate_ranges(world: &mut World, entity: Entity) {
        // First get the position and movement points
        let (coord, movement_points) = if let (Some(position), Some(unit_state)) = (
            world.get_component::<Position>(entity),
            world.get_component::<UnitState>(entity)
        ) {
            (position.coord, unit_state.movement_left)
        } else {
            return;
        };
        
        // Calculate attack range
        let attack_range = if let Some(unit_stats) = world.get_component::<UnitStats>(entity) {
            let range = unit_stats.range;
            
            // For simplicity, we'll just use distance for attack range
            Self::calculate_attack_range(world, &coord, range)
        } else {
            HashSet::new()
        };
        
        // Calculate movement range
        let movement_range = Self::calculate_movement_range(world, &coord, movement_points);
        
        // Add components for visualization
        let hex_entities = HexMapFactory::get_hex_entity_map(world);
        
        for coord in movement_range {
            if let Some(&tile_entity) = hex_entities.get(&coord) {
                world.add_component(tile_entity, InMovementRange);
            }
        }
        
        for coord in attack_range {
            if let Some(&tile_entity) = hex_entities.get(&coord) {
                // Only add attack range if there's an enemy unit
                let is_enemy = if let Some(team) = world.get_component::<Team>(tile_entity) {
                    let attacker_team = world.get_component::<Team>(entity).unwrap();
                    team.team_id != attacker_team.team_id
                } else {
                    false
                };
                
                if is_enemy {
                    world.add_component(tile_entity, InAttackRange);
                }
            }
        }
    }
    
    // Calculate all hexes within attack range
    fn calculate_attack_range(world: &World, start: &HexCoord, range: i32) -> HashSet<HexCoord> {
        let mut attack_hexes = HashSet::new();
        let hex_entities = HexMapFactory::get_hex_entity_map(world);
        
        // For each hex on the map
        for (coord, _) in &hex_entities {
            // If it's within range distance
            if coord.distance(start) <= range && coord != start {
                attack_hexes.insert(*coord);
            }
        }
        
        attack_hexes
    }
    
    // Calculate all hexes reachable with given movement points
    fn calculate_movement_range(world: &World, start: &HexCoord, movement_points: i32) -> HashSet<HexCoord> {
        let mut visited = HashMap::new();
        let mut to_visit = vec![(*start, movement_points)];
        let mut range = HashSet::new();
        let hex_entities = HexMapFactory::get_hex_entity_map(world);
        
        let current_turn = {
            let game_state_entity = HexMapFactory::get_game_state_entity(world).unwrap();
            let game_state = world.get_component::<GameState>(game_state_entity).unwrap();
            game_state.current_turn
        };
        
        while let Some((current, remaining_movement)) = to_visit.pop() {
            // Skip if we've already found a better path to this hex
            if let Some(&prev_movement) = visited.get(&current) {
                if prev_movement >= remaining_movement {
                    continue;
                }
            }
            
            visited.insert(current, remaining_movement);
            
            if current != *start {
                range.insert(current);
            }
            
            // Check each neighbor
            for neighbor in current.neighbors().iter() {
                if let Some(&entity) = hex_entities.get(neighbor) {
                    // Get terrain cost
                    let cost = if let Some(terrain) = world.get_component::<Terrain>(entity) {
                        terrain.terrain_type.movement_cost()
                    } else {
                        continue;
                    };
                    
                    // Skip if this neighbor has a unit of opposing team
                    let has_enemy = if let Some(team) = world.get_component::<Team>(entity) {
                        team.team_id != current_turn
                    } else {
                        false
                    };
                    
                    // Skip if exceeds movement or has enemy
                    if cost > remaining_movement || has_enemy {
                        continue;
                    }
                    
                    to_visit.push((*neighbor, remaining_movement - cost));
                }
            }
        }
        
        range
    }
    
    // Move a unit from one hex to another
    fn move_unit(world: &mut World, from_entity: Entity, to_entity: Entity) {
        // First check if the move is valid
        if world.get_component::<InMovementRange>(to_entity).is_none() {
            return;
        }
        
        // Get current position of the unit
        let (_from_coord, _to_coord) = if let (Some(from_pos), Some(to_pos)) = (
            world.get_component::<Position>(from_entity),
            world.get_component::<Position>(to_entity)
        ) {
            (from_pos.coord, to_pos.coord)
        } else {
            return;
        };
        
        // Calculate movement cost
        let movement_cost = if let Some(terrain) = world.get_component::<Terrain>(to_entity) {
            terrain.terrain_type.movement_cost()
        } else {
            return;
        };
        
        // Update unit's movement points
        if let Some(unit_state) = world.get_component_mut::<UnitState>(from_entity) {
            if unit_state.movement_left < movement_cost {
                return;
            }
            unit_state.movement_left -= movement_cost;
        } else {
            return;
        }
        
        // Move components from from_entity to to_entity
        if let Some(unit_stats) = world.get_component::<UnitStats>(from_entity).cloned() {
            world.add_component(to_entity, unit_stats);
        }
        
        if let Some(unit_state) = world.get_component::<UnitState>(from_entity).cloned() {
            world.add_component(to_entity, unit_state);
        }
        
        if let Some(team) = world.get_component::<Team>(from_entity).cloned() {
            world.add_component(to_entity, team);
        }
        
        // Remove components from original entity
        // In a real ECS, we'd use world.remove_component<T>(entity)
        // Here we'll leave them and handle it in the render system
        
        // Deselect if no more movement
        let should_deselect = if let Some(unit_state) = world.get_component::<UnitState>(to_entity) {
            unit_state.movement_left <= 0
        } else {
            true
        };
        
        if should_deselect {
            Self::deselect_current(world);
        } else {
            // Update selection and recalculate ranges
            let game_state_entity = HexMapFactory::get_game_state_entity(world).unwrap();
            if let Some(game_state) = world.get_component_mut::<GameState>(game_state_entity) {
                game_state.selected_entity = Some(to_entity);
            }
            
            // Collect all entities with components to remove
            let movement_range_entities: Vec<Entity> = world.query::<InMovementRange>()
                .into_iter()
                .map(|(entity, _)| entity)
                .collect();
            
            let attack_range_entities: Vec<Entity> = world.query::<InAttackRange>()
                .into_iter()
                .map(|(entity, _)| entity)
                .collect();
            
            // Remove components
            for _ in movement_range_entities {
                // world.remove_component::<InMovementRange>(entity);
            }
            
            for _ in attack_range_entities {
                // world.remove_component::<InAttackRange>(entity);
            }
            
            // Recalculate ranges
            Self::calculate_ranges(world, to_entity);
        }
    }
    
    // Attack from one hex to another
    fn attack_unit(world: &mut World, attacker_entity: Entity, defender_entity: Entity) {
        // Check if attack is valid
        if world.get_component::<InAttackRange>(defender_entity).is_none() {
            return;
        }
        
        // Get unit stats
        let (attacker_attack, defender_defense) = if let (Some(attacker_stats), Some(defender_stats)) = (
            world.get_component::<UnitStats>(attacker_entity),
            world.get_component::<UnitStats>(defender_entity)
        ) {
            (attacker_stats.attack, defender_stats.defense)
        } else {
            return;
        };
        
        // Calculate damage
        let damage = std::cmp::max(1, attacker_attack - defender_defense / 2);
        
        // Apply damage to defender
        let defender_destroyed = if let Some(defender_state) = world.get_component_mut::<UnitState>(defender_entity) {
            defender_state.health -= damage;
            defender_state.health <= 0
        } else {
            false
        };
        
        // Mark attacker as has acted
        if let Some(attacker_state) = world.get_component_mut::<UnitState>(attacker_entity) {
            attacker_state.movement_left = 0;
            attacker_state.has_acted = true;
        }
        
        // Check if game is over if defender was destroyed
        if defender_destroyed {
            // In a real ECS, we would remove components
            // For now, we'll just set health to 0 and handle in render
            
            let (game_over, player_won) = HexMapFactory::check_game_over(world);
            
            if game_over {
                let game_state_entity = HexMapFactory::get_game_state_entity(world).unwrap();
                if let Some(game_state) = world.get_component_mut::<GameState>(game_state_entity) {
                    game_state.game_over = true;
                    game_state.player_won = player_won;
                }
            }
        }
        
        // Deselect current unit
        Self::deselect_current(world);
    }
    
    // End the current turn
    pub fn end_turn(world: &mut World) {
        // Get game state
        let game_state_entity = match HexMapFactory::get_game_state_entity(world) {
            Some(entity) => entity,
            None => return,
        };
        
        // Get the current turn and update it
        let current_turn = {
            if let Some(game_state) = world.get_component::<GameState>(game_state_entity) {
                game_state.current_turn
            } else {
                return;
            }
        };
        
        let next_turn = 1 - current_turn;
        
        // Update game state
        if let Some(game_state) = world.get_component_mut::<GameState>(game_state_entity) {
            game_state.current_turn = next_turn;
            game_state.turn_number += 1;
        }
        
        // Reset movement for new team
        HexMapFactory::reset_team_units_for_new_turn(world, next_turn);
        
        // Deselect current selection
        Self::deselect_current(world);
    }
} 