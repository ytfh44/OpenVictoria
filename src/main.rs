use eframe::{egui, App, Frame};
use egui::{Pos2};

// 导入我们自己的库
use openvictoria::{World, HexMapFactory};
use openvictoria::s::{InputSystem, RenderSystem};

// The main game application
struct MyApp {
    ecs_world: World,
    end_turn_clicked: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        let mut world = World::new();
        
        // 创建地图，设置大小和原点
        let origin = Pos2::new(300.0, 300.0);
        let hex_size = 30.0;
        HexMapFactory::create_map(&mut world, 8, 8, hex_size, origin);
        
        Self {
            ecs_world: world,
            end_turn_clicked: false,
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Set up the central panel for the game
        egui::CentralPanel::default().show(ctx, |ui| {
            // Game area
            let (response, _painter) = ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());

            // Process input first
            InputSystem::update(&mut self.ecs_world, ui, &response);
            
            // Handle end turn button
            if self.end_turn_clicked {
                self.end_turn_clicked = false;
                InputSystem::end_turn(&mut self.ecs_world);
            }
            
            // Render the game
            RenderSystem::render(&self.ecs_world, ui);
            
            // Draw UI controls in a side panel
            egui::SidePanel::right("controls").show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("控制面板");
                    ui.add_space(10.0);
                    self.end_turn_clicked = RenderSystem::draw_end_turn_button(ui);
                    
                    ui.add_space(20.0);
                    ui.label("点击单位选择它");
                    ui.label("绿色格子表示移动范围");
                    ui.label("红色格子表示攻击范围");
                });
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "OpenVictoria 六边形策略游戏",
        options,
        Box::new(|_cc| Box::new(MyApp::default()))
    )
}
