use eframe::{egui, App, Frame};
use egui::{Pos2, Color32, RichText, Align, Layout};

// 导入我们自己的库
use openvictoria::{World, HexMapFactory, Locale};
use openvictoria::s::{InputSystem, RenderSystem};
use openvictoria::c::*;
use openvictoria::i18n::Language;

// 游戏界面状态
#[derive(PartialEq)]
enum GameScreen {
    MainMenu,
    Settings,
    Playing,
    GameOver,
}

// The main game application
struct MyApp {
    ecs_world: World,
    end_turn_clicked: bool,
    game_screen: GameScreen,
    map_size: i32,
    hex_size: f32,
    player_won: bool,
    show_help: bool,
    show_debug: bool,
    show_unit_info: bool,
    locale: Locale,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            ecs_world: World::new(),
            end_turn_clicked: false,
            game_screen: GameScreen::MainMenu,
            map_size: 8,
            hex_size: 30.0,
            player_won: false,
            show_help: false,
            show_debug: false,
            show_unit_info: true,
            locale: Locale::new(Language::Chinese), // 默认使用中文
        }
    }
}

impl MyApp {
    // 初始化游戏世界
    fn initialize_game(&mut self) {
        self.ecs_world = World::new();
        
        // 创建地图，设置大小和原点
        let origin = Pos2::new(300.0, 300.0);
        HexMapFactory::create_map(&mut self.ecs_world, self.map_size, self.map_size, self.hex_size, origin);
        
        self.game_screen = GameScreen::Playing;
        self.end_turn_clicked = false;
    }
    
    // 渲染主菜单
    fn render_main_menu(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                
                ui.heading(RichText::new(self.locale.get_message("app-title")).size(50.0).color(Color32::GOLD));
                ui.label(RichText::new(self.locale.get_message("app-subtitle")).size(20.0));
                
                ui.add_space(50.0);
                
                if ui.button(RichText::new(self.locale.get_message("start-game")).size(24.0)).clicked() {
                    self.initialize_game();
                }
                
                if ui.button(RichText::new(self.locale.get_message("game-settings")).size(24.0)).clicked() {
                    self.game_screen = GameScreen::Settings;
                }
                
                if ui.button(RichText::new(self.locale.get_message("exit")).size(24.0)).clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                
                ui.add_space(30.0);
                ui.label(self.locale.get_message("version-info"));
            });
        });
    }
    
    // 渲染设置界面
    fn render_settings(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.heading(self.locale.get_message("settings-title"));
                ui.add_space(20.0);
                
                ui.horizontal(|ui| {
                    ui.label(self.locale.get_message("map-size"));
                    ui.add(egui::Slider::new(&mut self.map_size, 4..=12).text(""));
                });
                
                ui.horizontal(|ui| {
                    ui.label(self.locale.get_message("hex-size"));
                    ui.add(egui::Slider::new(&mut self.hex_size, 20.0..=50.0).text(""));
                });
                
                ui.checkbox(&mut self.show_unit_info, &self.locale.get_message("show-unit-info"));
                ui.checkbox(&mut self.show_debug, &self.locale.get_message("show-debug"));
                
                // 语言选择
                ui.separator();
                ui.heading("Language / 语言");
                ui.horizontal(|ui| {
                    if ui.selectable_label(*self.locale.get_language() == Language::English, "English").clicked() {
                        self.locale.set_language(Language::English);
                    }
                    if ui.selectable_label(*self.locale.get_language() == Language::Chinese, "中文").clicked() {
                        self.locale.set_language(Language::Chinese);
                    }
                });
                
                ui.add_space(30.0);
                
                ui.horizontal(|ui| {
                    if ui.button(self.locale.get_message("back")).clicked() {
                        self.game_screen = GameScreen::MainMenu;
                    }
                    
                    if ui.button(self.locale.get_message("apply-and-start")).clicked() {
                        self.initialize_game();
                    }
                });
            });
        });
    }
    
    // 渲染游戏结束画面
    fn render_game_over(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                
                if self.player_won {
                    ui.heading(RichText::new(self.locale.get_message("victory-title")).size(50.0).color(Color32::GOLD));
                    ui.add_space(20.0);
                    ui.label(self.locale.get_message("victory-message"));
                } else {
                    ui.heading(RichText::new(self.locale.get_message("defeat-title")).size(50.0).color(Color32::RED));
                    ui.add_space(20.0);
                    ui.label(self.locale.get_message("defeat-message"));
                }
                
                ui.add_space(30.0);
                
                if ui.button(self.locale.get_message("return-to-menu")).clicked() {
                    self.game_screen = GameScreen::MainMenu;
                }
                
                if ui.button(self.locale.get_message("restart")).clicked() {
                    self.initialize_game();
                }
            });
        });
    }
    
    // 显示帮助面板
    fn render_help_window(&mut self, ctx: &egui::Context) {
        egui::Window::new(self.locale.get_message("help-title"))
            .open(&mut self.show_help)
            .show(ctx, |ui| {
                ui.label(self.locale.get_message("help-controls-header"));
                ui.label(self.locale.get_message("help-click-unit"));
                ui.label(self.locale.get_message("help-green-tiles"));
                ui.label(self.locale.get_message("help-red-tiles"));
                ui.label(self.locale.get_message("help-end-turn"));
                ui.add_space(10.0);
                ui.label(self.locale.get_message("help-unit-types-header"));
                ui.label(self.locale.get_message("help-infantry"));
                ui.label(self.locale.get_message("help-archer"));
                ui.label(self.locale.get_message("help-cavalry"));
                ui.add_space(10.0);
                ui.label(self.locale.get_message("help-terrain-header"));
                ui.label(self.locale.get_message("help-plains"));
                ui.label(self.locale.get_message("help-forest"));
                ui.label(self.locale.get_message("help-mountain"));
                ui.label(self.locale.get_message("help-water"));
            });
    }
    
    // 显示单位信息面板
    fn render_unit_info(&self, ctx: &egui::Context) {
        if let Some(game_state_entity) = HexMapFactory::get_game_state_entity(&self.ecs_world) {
            if let Some(game_state) = self.ecs_world.get_component::<GameState>(game_state_entity) {
                if let Some(hover_entity) = game_state.hover_entity {
                    if let Some(unit_stats) = self.ecs_world.get_component::<UnitStats>(hover_entity) {
                        if let Some(unit_state) = self.ecs_world.get_component::<UnitState>(hover_entity) {
                            if let Some(team) = self.ecs_world.get_component::<Team>(hover_entity) {
                                egui::Window::new(self.locale.get_message("unit-info-title"))
                                    .anchor(egui::Align2::RIGHT_TOP, [-10.0, 10.0])
                                    .show(ctx, |ui| {
                                        // 单位类型和所属方
                                        let unit_type_name = match unit_stats.unit_type {
                                            UnitType::Infantry => self.locale.get_message("unit-type-infantry"),
                                            UnitType::Archer => self.locale.get_message("unit-type-archer"),
                                            UnitType::Cavalry => self.locale.get_message("unit-type-cavalry"),
                                        };
                                        
                                        let team_name = if team.team_id == 0 { 
                                            self.locale.get_message("team-player")
                                        } else { 
                                            self.locale.get_message("team-enemy")
                                        };
                                        let team_color = if team.team_id == 0 { Color32::BLUE } else { Color32::RED };
                                        
                                        ui.horizontal(|ui| {
                                            ui.heading(unit_type_name);
                                            ui.label(RichText::new(team_name).color(team_color));
                                        });
                                        
                                        ui.separator();
                                        
                                        // 单位属性
                                        ui.horizontal(|ui| {
                                            ui.label(self.locale.get_message("health"));
                                            let health_percent = unit_state.health as f32 / unit_stats.max_health as f32;
                                            let health_color = if health_percent < 0.3 {
                                                Color32::RED
                                            } else if health_percent < 0.7 {
                                                Color32::YELLOW
                                            } else {
                                                Color32::GREEN
                                            };
                                            
                                            ui.label(RichText::new(format!("{}/{}", unit_state.health, unit_stats.max_health)).color(health_color));
                                        });
                                        
                                        ui.horizontal(|ui| {
                                            ui.label(self.locale.get_message("attack"));
                                            ui.label(format!("{}", unit_stats.attack));
                                        });
                                        
                                        ui.horizontal(|ui| {
                                            ui.label(self.locale.get_message("defense"));
                                            ui.label(format!("{}", unit_stats.defense));
                                        });
                                        
                                        ui.horizontal(|ui| {
                                            ui.label(self.locale.get_message("attack-range"));
                                            ui.label(format!("{}", unit_stats.range));
                                        });
                                        
                                        ui.horizontal(|ui| {
                                            ui.label(self.locale.get_message("movement-left"));
                                            ui.label(format!("{}/{}", unit_state.movement_left, unit_stats.movement));
                                        });
                                        
                                        // 单位状态
                                        ui.separator();
                                        if unit_state.has_acted {
                                            ui.label(RichText::new(self.locale.get_message("unit-acted")).color(Color32::GRAY));
                                        } else {
                                            ui.label(RichText::new(self.locale.get_message("unit-can-act")).color(Color32::GREEN));
                                        }
                                        
                                        // 获取地形信息
                                        if let Some(terrain) = self.ecs_world.get_component::<Terrain>(hover_entity) {
                                            let terrain_type_key = match terrain.terrain_type {
                                                TerrainType::Plain => "help-plains",
                                                TerrainType::Forest => "help-forest",
                                                TerrainType::Mountain => "help-mountain",
                                                TerrainType::Water => "help-water",
                                            };
                                            
                                            ui.separator();
                                            ui.label(format!("{} {}", self.locale.get_message("terrain"), 
                                                       terrain_type_key.trim_start_matches("help-").trim_start_matches("• ")));
                                            ui.label(format!("{} {}", self.locale.get_message("movement-cost"), 
                                                       terrain.terrain_type.movement_cost()));
                                        }
                                    });
                            }
                        }
                    }
                }
            }
        }
    }
    
    // 渲染游戏状态栏
    fn render_game_status(&self, ui: &mut egui::Ui) {
        if let Some(game_state_entity) = HexMapFactory::get_game_state_entity(&self.ecs_world) {
            if let Some(game_state) = self.ecs_world.get_component::<GameState>(game_state_entity) {
                // 当前回合信息
                ui.horizontal(|ui| {
                    let current_turn_text = if game_state.current_turn == 0 { 
                        RichText::new(self.locale.get_message("player-turn")).color(Color32::BLUE)
                    } else { 
                        RichText::new(self.locale.get_message("enemy-turn")).color(Color32::RED)
                    };
                    
                    ui.label(current_turn_text);
                    ui.label(self.locale.get_message_args("turn-number", &[("number", &game_state.turn_number.to_string())]));
                });
                
                ui.separator();
            }
        }
    }
    
    // 渲染调试信息
    fn render_debug_info(&self, ctx: &egui::Context) {
        if self.show_debug {
            egui::Window::new("调试信息")
                .anchor(egui::Align2::LEFT_BOTTOM, [10.0, -10.0])
                .show(ctx, |ui| {
                    if let Some(game_state_entity) = HexMapFactory::get_game_state_entity(&self.ecs_world) {
                        if let Some(game_state) = self.ecs_world.get_component::<GameState>(game_state_entity) {
                            ui.label(format!("游戏状态实体: {}", game_state_entity));
                            ui.label(format!("当前回合: {}", game_state.current_turn));
                            ui.label(format!("回合数: {}", game_state.turn_number));
                            
                            if let Some(selected) = game_state.selected_entity {
                                ui.label(format!("已选中实体: {}", selected));
                            } else {
                                ui.label("未选中实体");
                            }
                            
                            if let Some(hover) = game_state.hover_entity {
                                ui.label(format!("悬停实体: {}", hover));
                                
                                if let Some(pos) = self.ecs_world.get_component::<Position>(hover) {
                                    ui.label(format!("坐标: q={}, r={}", pos.coord.q, pos.coord.r));
                                }
                            }
                        }
                    }
                    
                    // 实体计数
                    let player_count = self.count_units(0);
                    let enemy_count = self.count_units(1);
                    ui.separator();
                    ui.label(format!("玩家单位数: {}", player_count));
                    ui.label(format!("敌人单位数: {}", enemy_count));
                });
        }
    }
    
    // 计算特定队伍单位数量
    fn count_units(&self, team_id: u8) -> usize {
        let mut count = 0;
        
        for (entity, team) in self.ecs_world.query::<Team>() {
            if team.team_id == team_id {
                if let Some(unit_state) = self.ecs_world.get_component::<UnitState>(entity) {
                    if unit_state.health > 0 {
                        count += 1;
                    }
                }
            }
        }
        
        count
    }
    
    // 检查游戏是否结束
    fn check_game_over(&mut self) {
        if let Some(game_state_entity) = HexMapFactory::get_game_state_entity(&self.ecs_world) {
            if let Some(game_state) = self.ecs_world.get_component::<GameState>(game_state_entity) {
                if game_state.game_over {
                    self.player_won = game_state.player_won;
                    self.game_screen = GameScreen::GameOver;
                }
            }
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // 根据当前游戏界面显示不同内容
        match self.game_screen {
            GameScreen::MainMenu => {
                self.render_main_menu(ctx);
            },
            GameScreen::Settings => {
                self.render_settings(ctx);
            },
            GameScreen::Playing => {
                // Set up the central panel for the game
                egui::CentralPanel::default().show(ctx, |ui| {
                    // 顶部状态栏
                    ui.horizontal(|ui| {
                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                            self.render_game_status(ui);
                        });
                        
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.button(self.locale.get_message("help")).clicked() {
                                self.show_help = !self.show_help;
                            }
                            
                            if ui.button(self.locale.get_message("menu")).clicked() {
                                self.game_screen = GameScreen::MainMenu;
                            }
                        });
                    });
                    
                    ui.separator();
                    
                    // Game area
                    let available_size = ui.available_size();
                    let (response, _painter) = ui.allocate_painter(available_size, egui::Sense::click_and_drag());

                    // Process input first
                    InputSystem::update(&mut self.ecs_world, ui, &response);
                    
                    // Handle end turn button
                    if self.end_turn_clicked {
                        self.end_turn_clicked = false;
                        InputSystem::end_turn(&mut self.ecs_world);
                    }
                    
                    // Render the game
                    RenderSystem::render(&self.ecs_world, ui);
                    
                    // 检查游戏是否结束
                    self.check_game_over();
                    
                    // Draw UI controls in a side panel
                    egui::SidePanel::right("controls").show_inside(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading(self.locale.get_message("control-panel"));
                            ui.add_space(10.0);
                            
                            // 回合信息
                            self.render_game_status(ui);
                            
                            // 结束回合按钮
                            if ui.add_sized([120.0, 30.0], egui::Button::new(self.locale.get_message("end-turn"))).clicked() {
                                self.end_turn_clicked = true;
                            }
                            
                            ui.add_space(20.0);
                            ui.label(self.locale.get_message("click-to-select"));
                            ui.label(self.locale.get_message("help-green-tiles"));
                            ui.label(self.locale.get_message("help-red-tiles"));
                            
                            ui.separator();
                            
                            // 玩家和敌人单位数量
                            let player_count = self.count_units(0);
                            let enemy_count = self.count_units(1);
                            
                            ui.heading(self.locale.get_message("unit-info-title"));
                            ui.horizontal(|ui| {
                                ui.label(self.locale.get_message("player-units"));
                                ui.label(RichText::new(format!("{}", player_count)).color(Color32::BLUE));
                            });
                            
                            ui.horizontal(|ui| {
                                ui.label(self.locale.get_message("enemy-units"));
                                ui.label(RichText::new(format!("{}", enemy_count)).color(Color32::RED));
                            });
                            
                            ui.separator();
                            
                            if ui.button(self.locale.get_message("menu")).clicked() {
                                self.game_screen = GameScreen::MainMenu;
                            }
                            
                            if ui.button(self.locale.get_message("game-settings")).clicked() {
                                self.game_screen = GameScreen::Settings;
                            }
                            
                            // 调试开关
                            ui.checkbox(&mut self.show_debug, &self.locale.get_message("show-debug"));
                            ui.checkbox(&mut self.show_unit_info, &self.locale.get_message("show-unit-info"));
                        });
                    });
                });
                
                // 渲染帮助窗口
                if self.show_help {
                    self.render_help_window(ctx);
                }
                
                // 渲染调试信息
                self.render_debug_info(ctx);
                
                // 渲染单位信息面板
                if self.show_unit_info {
                    self.render_unit_info(ctx);
                }
            },
            GameScreen::GameOver => {
                self.render_game_over(ctx);
            }
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "OpenVictoria",
        options,
        Box::new(|_cc| Box::new(MyApp::default()))
    )
}
