use eframe::{epi, egui};
use eframe::egui::emath::{vec2, Vec2, Rot2, pos2, Pos2};
use eframe::egui::epaint::{Mesh, Color32, Vertex, TextureId, Shape};

use bots::{World, WorldConfig};

use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::{sync_channel, SyncSender, Receiver};
use std::f32::consts::PI;
use std::time::{Duration, Instant};

struct BotPosition {
    x: i32,
    y: i32,
    heading: u32,
    turret_offset: u32,
}

struct WorldUpdate {
    bots: Vec<BotPosition>,
}

fn bots_main_loop(tx: SyncSender<WorldUpdate>, redraw: Arc<dyn epi::RepaintSignal>) {
    let mut world = World::new(WorldConfig {
        cpus_per_tick: 1,
        ..WorldConfig::default()
    });

    world.add_bot(Path::new("testbot.bc"));
    world.add_bot(Path::new("testbot.bc"));

    world.place_bots();

    send_world_update(&tx, &world);

    std::thread::sleep(std::time::Duration::from_secs(5));

    let mut tp1 = Instant::now();
    let mut tp2 = tp1.clone();
    let mut accumulated: isize = 0;
    let target: isize = 1000 / 60;
    loop {
        tp2 = Instant::now();
        let elapsed = tp2 - tp1;
        tp1 = tp2.clone();

        accumulated -= elapsed.as_millis() as isize;
        accumulated += target;

        if accumulated > 0 {
            std::thread::sleep(Duration::from_millis(accumulated as u64));
        }

        world.tick();
        send_world_update(&tx, &world);
        redraw.as_ref().request_repaint();
    }
}

fn send_world_update(tx: &SyncSender<WorldUpdate>, world: &World) {
    let mut bot_positions: Vec<BotPosition> = Vec::new();
    for bot in &world.bots {
        let tank = bot.tank_mut();
        bot_positions.push(BotPosition {
            x: tank.x,
            y: tank.y,
            heading: tank.heading,
            turret_offset: tank.turret_offset
        });
    }
    tx.send(WorldUpdate {
        bots: bot_positions
    }).unwrap();
}

struct App {
    body_tex: TextureId,
    turret_tex: TextureId,
    rx: Option<Receiver<WorldUpdate>>,
    botpos: Option<WorldUpdate>
}

impl Default for App {
    fn default() -> Self {
        Self {
            body_tex: TextureId::default(),
            turret_tex: TextureId::default(),
            rx: None,
            botpos: None,
        }
    }
}

impl epi::App for App {
    fn name(&self) -> &str { "Bots" }

    fn setup(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame, storage: Option<&dyn epi::Storage>) {
        let body_img = image::open("tankbody.png").unwrap().to_rgba8();
        let turret_img = image::open("tankturret.png").unwrap().to_rgba8();

        let body_dimensions = body_img.dimensions();
        let mut body_pixels: Vec<Color32> = Vec::new();
        body_pixels.reserve(body_dimensions.0 as usize * body_dimensions.1 as usize);
        for pixel in body_img.pixels() {
            body_pixels.push(Color32::from_rgba_premultiplied(pixel[0], pixel[1], pixel[2], pixel[3]));
        }
        self.body_tex = frame.tex_allocator().alloc_srgba_premultiplied(
            (body_dimensions.0 as usize, body_dimensions.1 as usize),
            &body_pixels
        );

        let turret_dimensions = turret_img.dimensions();
        let mut turret_pixels: Vec<Color32> = Vec::new();
        turret_pixels.reserve(turret_dimensions.0 as usize * turret_dimensions.1 as usize);
        for pixel in turret_img.pixels() {
            turret_pixels.push(Color32::from_rgba_premultiplied(pixel[0], pixel[1], pixel[2], pixel[3]));
        }
        self.turret_tex = frame.tex_allocator().alloc_srgba_premultiplied(
            (turret_dimensions.0 as usize, turret_dimensions.1 as usize),
            &turret_pixels
        );

        let (tx, rx) = sync_channel(8);
        self.rx = Some(rx);
        let trigger_redraw = frame.repaint_signal().clone();

        std::thread::spawn(move || {
            bots_main_loop(tx, trigger_redraw);
        });

        println!("ready");
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame) {
        match self.rx.as_ref().unwrap().try_recv() {
            Ok(update) => {
                self.botpos = Some(update);
            },
            _ => {},
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            match &self.botpos {
                Some(update) => {
                    for tank in &update.bots {
                        self.render_tank(
                            ui.painter(),
                            self.map_world_coord(tank.x),
                            self.map_world_coord(tank.y),
                            -1f32 * (tank.heading as f32 * PI / 512f32),
                            -1f32 * (tank.turret_offset as f32 * PI / 512f32),
                        );
                    }
                },
                _ => {},
            }
        });
    }
}

impl App {
    fn render_tank(&self, painter: &egui::Painter, x: f32, y: f32, tank_angle: f32, turret_angle: f32) {
        let position = pos2(x, y);
        let rotation_body = Rot2::from_angle(tank_angle) * 0.1f32;
        let rotation_turret = Rot2::from_angle(tank_angle + turret_angle) * 0.1f32;

        let mut mesh = Mesh::with_texture(self.body_tex);
        Self::add_tank_vertices(&mut mesh, rotation_body, position);
        painter.add(Shape::Mesh(mesh));

        let mut mesh = Mesh::with_texture(self.turret_tex);
        Self::add_tank_vertices(&mut mesh, rotation_turret, position);
        painter.add(Shape::Mesh(mesh));
    }

    fn add_tank_vertices(mesh: &mut Mesh, rot: Rot2, pos: Pos2) {
        mesh.vertices.push(Vertex {
            pos: pos + (rot * vec2(-480f32, -205f32)),
            uv: egui::pos2(0f32, 0f32),
            color: Color32::WHITE
        });
        mesh.vertices.push(Vertex {
            pos: pos + (rot * vec2(480f32, -205f32)),
            uv: egui::pos2(1f32, 0f32),
            color: Color32::WHITE
        });
        mesh.vertices.push(Vertex {
            pos: pos + (rot * vec2(-480f32, 335f32)),
            uv: egui::pos2(0f32, 1f32),
            color: Color32::WHITE
        });
        mesh.add_triangle(0, 1, 2);

        mesh.vertices.push(Vertex {
            pos: pos + (rot * vec2(480f32, 335f32)),
            uv: egui::pos2(1f32, 1f32),
            color: Color32::WHITE
        });
        mesh.add_triangle(1, 2, 3);
    }

    fn map_world_coord(&self, x: i32) -> f32 {
        x as f32 / 8f32 + 500f32
    }
}

fn main() {
    let app = App::default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
