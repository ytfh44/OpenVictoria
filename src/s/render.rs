use crate::e::entity::World;
use crate::e::factory::HexMapFactory;
use crate::c::*;
use eframe::egui::{self, Color32, Pos2, Stroke};

// System for rendering the hex map and game UI
pub struct RenderSystem;

impl RenderSystem {
    pub fn render(world: &World, ui: &mut egui::Ui) {
        // Get the game state and map settings
        let game_state_entity = match HexMapFactory::get_game_state_entity(world) {
            Some(entity) => entity,
            None => return,
        };
        
        let map_settings_entity = match HexMapFactory::get_map_settings_entity(world) {
            Some(entity) => entity,
            None => return,
        };
        
        // Get the hex_size and origin from map settings
        let (hex_size, origin) = {
            if let Some(settings) = world.get_component::<MapSettings>(map_settings_entity) {
                (settings.hex_size, settings.origin)
            } else {
                return;
            }
        };
        
        // Get game state
        let (game_over, player_won, current_turn) = {
            if let Some(game_state) = world.get_component::<GameState>(game_state_entity) {
                (game_state.game_over, game_state.player_won, game_state.current_turn)
            } else {
                return;
            }
        };
        
        // Get the hex grid entities
        let hex_entities = HexMapFactory::get_hex_entity_map(world);
        let selected_entity = {
            if let Some(game_state) = world.get_component::<GameState>(game_state_entity) {
                game_state.selected_entity
            } else {
                None
            }
        };
        
        // Draw each hex tile
        for (hex_coord, &entity) in &hex_entities {
            // Get components for this hex
            let terrain = world.get_component::<Terrain>(entity);
            let unit_stats = world.get_component::<UnitStats>(entity);
            let unit_state = world.get_component::<UnitState>(entity);
            let team = world.get_component::<Team>(entity);
            let hovering = world.get_component::<Hovering>(entity).is_some();
            let selected = world.get_component::<Selected>(entity).is_some();
            let in_movement_range = world.get_component::<InMovementRange>(entity).is_some();
            let in_attack_range = world.get_component::<InAttackRange>(entity).is_some();
            
            // Calculate pixel position
            let pixel_pos = hex_coord.to_pixel(hex_size, origin);
            
            // Draw hex base
            Self::draw_hex(
                ui,
                pixel_pos,
                hex_size,
                terrain.map_or(Color32::GRAY, |t| t.terrain_type.color()),
                Stroke::new(1.0, Color32::BLACK),
            );
            
            // Draw movement range
            if in_movement_range {
                Self::draw_hex(
                    ui,
                    pixel_pos,
                    hex_size * 0.9,
                    Color32::from_rgba_premultiplied(0, 255, 0, 100),
                    Stroke::NONE,
                );
            }
            
            // Draw attack range
            if in_attack_range {
                Self::draw_hex(
                    ui,
                    pixel_pos,
                    hex_size * 0.9,
                    Color32::from_rgba_premultiplied(255, 0, 0, 100),
                    Stroke::NONE,
                );
            }
            
            // Draw unit if present
            if let (Some(stats), Some(state), Some(team_info)) = (unit_stats, unit_state, team) {
                // Only draw if health > 0
                if state.health > 0 {
                    let unit_color = if team_info.team_id == 0 {
                        Color32::BLUE
                    } else {
                        Color32::RED
                    };
                    
                    Self::draw_unit(
                        ui,
                        pixel_pos,
                        hex_size * 0.6,
                        unit_color,
                        state.health,
                        stats.max_health,
                        state.movement_left,
                        state.has_acted,
                    );
                }
            }
            
            // Draw selection or hover highlight
            if selected || Some(entity) == selected_entity {
                Self::draw_hex(
                    ui,
                    pixel_pos,
                    hex_size * 0.8,
                    Color32::from_rgba_premultiplied(255, 255, 0, 100),
                    Stroke::new(2.0, Color32::YELLOW),
                );
            } else if hovering {
                Self::draw_hex(
                    ui,
                    pixel_pos,
                    hex_size * 0.8,
                    Color32::from_rgba_premultiplied(255, 255, 255, 100),
                    Stroke::new(2.0, Color32::WHITE),
                );
            }
            
            // Draw coordinates for debugging
            ui.painter().text(
                Pos2::new(pixel_pos.x, pixel_pos.y),
                egui::Align2::CENTER_CENTER,
                format!("{},{}", hex_coord.q, hex_coord.r),
                egui::FontId::proportional(10.0),
                Color32::BLACK,
            );
        }
        
        // Draw game over message if game is over
        if game_over {
            let screen_rect = ui.max_rect();
            let text = if player_won {
                "Player Won!"
            } else {
                "Enemy Won!"
            };
            
            ui.painter().rect_filled(
                screen_rect,
                0.0,
                Color32::from_rgba_premultiplied(0, 0, 0, 150),
            );
            
            ui.painter().text(
                screen_rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::proportional(32.0),
                if player_won { Color32::GREEN } else { Color32::RED },
            );
        }
        
        // Draw current turn indicator
        let turn_text = if current_turn == 0 {
            "Player's Turn"
        } else {
            "Enemy's Turn"
        };
        
        ui.painter().text(
            Pos2::new(ui.max_rect().right() - 100.0, ui.max_rect().top() + 20.0),
            egui::Align2::RIGHT_TOP,
            turn_text,
            egui::FontId::proportional(16.0),
            if current_turn == 0 { Color32::BLUE } else { Color32::RED },
        );
    }
    
    // Draw a hexagon at the given position
    fn draw_hex(ui: &mut egui::Ui, center: Pos2, size: f32, fill_color: Color32, stroke: Stroke) {
        let points = (0..6).map(|i| {
            let angle = std::f32::consts::PI / 3.0 * i as f32;
            Pos2::new(
                center.x + size * angle.cos(),
                center.y + size * angle.sin(),
            )
        }).collect::<Vec<Pos2>>();
        
        ui.painter().add(egui::Shape::convex_polygon(
            points,
            fill_color,
            stroke,
        ));
    }
    
    // Draw a unit with health bar
    fn draw_unit(ui: &mut egui::Ui, center: Pos2, size: f32, color: Color32, health: i32, max_health: i32, movement_left: i32, has_acted: bool) {
        // Draw unit circle
        ui.painter().circle_filled(center, size, color);
        ui.painter().circle_stroke(center, size, Stroke::new(1.0, Color32::BLACK));
        
        // If unit has acted, draw an X
        if has_acted {
            let size_mult = size * 0.6;
            ui.painter().line_segment(
                [
                    Pos2::new(center.x - size_mult, center.y - size_mult),
                    Pos2::new(center.x + size_mult, center.y + size_mult),
                ],
                Stroke::new(3.0, Color32::BLACK),
            );
            ui.painter().line_segment(
                [
                    Pos2::new(center.x + size_mult, center.y - size_mult),
                    Pos2::new(center.x - size_mult, center.y + size_mult),
                ],
                Stroke::new(3.0, Color32::BLACK),
            );
        }
        
        // Draw health bar
        let health_ratio = health as f32 / max_health as f32;
        let health_bar_width = size * 1.5;
        let health_bar_height = size * 0.2;
        let health_bar_y = center.y + size + health_bar_height;
        
        // Background
        ui.painter().rect_filled(
            egui::Rect::from_min_max(
                Pos2::new(center.x - health_bar_width / 2.0, health_bar_y),
                Pos2::new(center.x + health_bar_width / 2.0, health_bar_y + health_bar_height),
            ),
            0.0,
            Color32::DARK_GRAY,
        );
        
        // Health fill
        ui.painter().rect_filled(
            egui::Rect::from_min_max(
                Pos2::new(center.x - health_bar_width / 2.0, health_bar_y),
                Pos2::new(
                    center.x - health_bar_width / 2.0 + health_bar_width * health_ratio,
                    health_bar_y + health_bar_height,
                ),
            ),
            0.0,
            if health_ratio > 0.6 {
                Color32::GREEN
            } else if health_ratio > 0.3 {
                Color32::YELLOW
            } else {
                Color32::RED
            },
        );
        
        // Health text
        ui.painter().text(
            Pos2::new(center.x, health_bar_y + health_bar_height / 2.0),
            egui::Align2::CENTER_CENTER,
            format!("{}/{}", health, max_health),
            egui::FontId::proportional(10.0),
            Color32::WHITE,
        );
        
        // Draw movement indicator
        if movement_left > 0 {
            ui.painter().text(
                Pos2::new(center.x, center.y),
                egui::Align2::CENTER_CENTER,
                format!("{}MP", movement_left),
                egui::FontId::proportional(10.0),
                Color32::WHITE,
            );
        }
    }
    
    // Draw end turn button
    pub fn draw_end_turn_button(ui: &mut egui::Ui) -> bool {
        let button = egui::Button::new("End Turn")
            .min_size(egui::Vec2::new(100.0, 30.0))
            .rounding(5.0);
        
        ui.add_sized([100.0, 30.0], button).clicked()
    }
} 