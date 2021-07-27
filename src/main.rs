use eframe::{epi, egui};
use eframe::egui::epaint::{Mesh, Color32, Vertex, TextureId, Shape};

struct App {
    body_tex: TextureId,
    turret_tex: TextureId
}

impl Default for App {
    fn default() -> Self {
        Self {
            body_tex: TextureId::default(),
            turret_tex: TextureId::default(),
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
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut mesh = Mesh::with_texture(self.body_tex);
            mesh.vertices.push(Vertex {
                pos: egui::pos2(0f32, 0f32),
                uv: egui::pos2(1f32, 1f32),
                color: Color32::WHITE
            });
            mesh.vertices.push(Vertex {
                pos: egui::pos2(960f32, 0f32),
                uv: egui::pos2(0f32, 1f32),
                color: Color32::RED
            });
            mesh.vertices.push(Vertex {
                pos: egui::pos2(0f32, 540f32),
                uv: egui::pos2(1f32, 0f32),
                color: Color32::YELLOW
            });
            mesh.add_triangle(0, 1, 2);

            mesh.vertices.push(Vertex {
                pos: egui::pos2(960f32, 540f32),
                uv: egui::pos2(0f32, 0f32),
                color: Color32::BLUE
            });
            mesh.add_triangle(1, 2, 3);
            ui.painter().add(Shape::Mesh(mesh));
        });
    }
}

fn main() {
    let app = App::default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
