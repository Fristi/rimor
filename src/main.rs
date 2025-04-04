mod pathfinding;

use crate::pathfinding::PathfindingResult;
use eframe::egui::{Context, Sense, StrokeKind};
use eframe::{egui, Frame};
use pathfinding::Graph;
use std::sync::{Arc, Mutex};

#[cfg(target_arch = "wasm32")]
fn main() {

    use wasm_bindgen::JsCast;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();
    let document = web_sys::window()
        .expect("No window")
        .document()
        .expect("No document");

    let canvas = document
        .get_element_by_id("the_canvas_id")
        .expect("Failed to find the_canvas_id")
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .expect("the_canvas_id was not a HtmlCanvasElement");

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(MyApp::build()))),
            )
            .await
            .expect("failed to start app");
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eframe::run_native(
        "pathfinding",
        eframe::NativeOptions::default(),
        Box::new(|ctx| Ok(Box::new(MyApp::build())))
    )
    .expect("failed to initialise app")
}

const WIDGET_SPACING: f32 = 10.0;

#[derive(Debug, PartialEq)]
enum PathfindingStrategy {
    BestFirstSearch,
    DepthFirstSearch
}

pub struct MyApp {
    stroke: egui::Stroke,
    rounding: egui::CornerRadius,
    graph: Arc<Mutex<Graph>>,
    timesteps: u32,
    max_milliseconds: usize,
    recovery_rate: u32,
    strategy: PathfindingStrategy,
    path: Arc<Mutex<PathfindingResult>>,
    start: Option<(usize, usize)>
}

impl MyApp {
    pub fn build() -> Self {
        MyApp {
            stroke: egui::Stroke::new(1.0, egui::Color32::DARK_GRAY),
            rounding: egui::CornerRadius::default(),
            graph: Arc::new(Mutex::new(Graph::new(10))),
            timesteps: 10,
            max_milliseconds: 1000,
            recovery_rate: 1,
            strategy: PathfindingStrategy::BestFirstSearch,
            path: Arc::new(Mutex::new(PathfindingResult::empty())),
            start: None
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn upload_file(&self) {
        let graph = Arc::clone(&self.graph);
        let path = Arc::clone(&self.path);

        let future = async move {
            let file = rfd::AsyncFileDialog::new()
                .add_filter("text", &["txt"])
                .pick_file()
                .await;

            let mut path_ = path.lock().expect("Failed to obtain mutex for path");
            let mut graph_ = graph.lock().expect("Failed to obtain mutex for graph");

            if let Some(file) = file {
                let bytes = file.read().await;
                *graph_ = Graph::from_bytes(bytes);
                *path_ = PathfindingResult::empty();
            }
        };

        wasm_bindgen_futures::spawn_local(future);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn upload_file(&self) {
        let file = rfd::FileDialog::new()
            .add_filter("text", &["txt"])
            .pick_file();

        let path = Arc::clone(&self.path);
        let graph = Arc::clone(&self.graph);

        if let Some(file) = file {

            let mut path_ = path.lock().expect("Failed to obtain mutex for path");
            let mut graph_ = graph.lock().expect("Failed to obtain mutex for graph");

            let file_path = file.as_path();
            *graph_ = Graph::from_file(file_path);
            *path_ = PathfindingResult::empty();
        }
    }
}

impl eframe::App for MyApp {

    fn update(&mut self, ctx: &Context, frame: &mut Frame) {

        let path = Arc::clone(&self.path);
        let graph = Arc::clone(&self.graph);

        egui::SidePanel::right("my_left_panel").show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                    ui.add_space(WIDGET_SPACING);
                    ui.label(format!("Score: {}", path.lock().expect("Failed to obtain mutex for path").score()));

                    ui.add_space(WIDGET_SPACING);
                    ui.label("SETTINGS");
                    ui.add_space(WIDGET_SPACING);

                    egui::ComboBox::from_label("Strategy")
                        .selected_text(format!("{:?}", self.strategy))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.strategy, PathfindingStrategy::BestFirstSearch, "Breadth first search");
                            ui.selectable_value(&mut self.strategy, PathfindingStrategy::DepthFirstSearch, "Depth first search");
                        });

                    ui.add(
                        egui::Slider::new(&mut self.timesteps, 2..=300)
                            .text("Timesteps")
                            .integer(),
                    );

                    ui.add(
                        egui::Slider::new(&mut self.max_milliseconds, 10..=2000)
                            .text("Max duration in milliseconds")
                            .integer(),
                    );

                    ui.add(
                        egui::Slider::new(&mut self.recovery_rate, 1..=100)
                            .text("Recovery rate per timestep")
                            .integer(),
                    );

                    if ui.button("Open grid file…").clicked() {
                        self.upload_file()
                    }

                    ui.add_space(WIDGET_SPACING);

                    if ui.button("Find Path").clicked() {

                        let origin = match self.start {
                            Some((x, y)) => (x, y),
                            None => (0, 0)
                        };

                        let found_path = match self.strategy {
                            PathfindingStrategy::BestFirstSearch => graph.lock().expect("Failed to obtain mutex for graph").path_planning_bfs(origin, self.timesteps, self.recovery_rate),
                            PathfindingStrategy::DepthFirstSearch => graph.lock().expect("Failed to obtain mutex for graph").path_planning_dfs(origin, self.timesteps, self.recovery_rate)
                        };

                        let mut path_ = path.lock().expect("Failed to obtain mutex for path");

                        *path_ = found_path;
                    }

                },)
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            let panel_size = ui.available_size();
            let graph_size = graph.lock().expect("Failed to obtain mutex for graph").size();
            let rect_size = egui::Vec2::new(
                (panel_size.x - 20.0) / graph_size as f32,
                (panel_size.y - 20.0) / graph_size as f32,
            );

            let (_, painter) = ui.allocate_painter(panel_size, Sense::hover());

            // Use a single loop to accumulate draw calls
            let mut shapes = Vec::new();

            for y in 0..graph_size {
                for x in 0..graph_size {
                    let x_coord = x as f32 * rect_size.x + 10.0;
                    let y_coord = y as f32 * rect_size.y + 10.0;
                    let pos = egui::pos2(x_coord, y_coord);
                    let rect = egui::Rect::from_min_size(pos, rect_size);

                    let stroke = if let Some((xs, ys)) = self.start {
                        if (x, y) == (xs, ys) {
                            egui::Stroke::new(2.0, egui::Color32::RED)
                        } else {
                            self.stroke
                        }
                    } else {
                        self.stroke
                    };

                    // Batch drawing instead of individual draw calls
                    shapes.push(egui::epaint::Shape::rect_stroke(
                        rect,
                        self.rounding,
                        stroke,
                        StrokeKind::Inside
                    ));
                }
            }

            // Execute batched drawing
            painter.extend(shapes);

            // Only draw text for visible regions (dynamic culling)
            let visible_rect = ui.clip_rect();
            let path_len = path.lock().expect("Failed to obtain mutex for path").path.len();

            for y in 0..graph_size {
                for x in 0..graph_size {
                    let x_coord = x as f32 * rect_size.x + 10.0;
                    let y_coord = y as f32 * rect_size.y + 10.0;
                    let pos = egui::pos2(x_coord, y_coord);
                    let rect = egui::Rect::from_min_size(pos, rect_size);
                    let idx = path.lock().expect("Failed to obtain mutex for path").occurs_at((x, y));

                    if let Some(idx) = idx {
                        let (r, g, b) = percentage_to_rgb(idx as f32 / path_len as f32 * 100.0);
                        painter.rect_filled(rect, self.rounding, egui::Color32::from_rgba_premultiplied(r, g, b, 100));

                        ui.painter().text(
                            rect.min,
                            egui::Align2::LEFT_TOP,
                            format!("{}", idx + 1),
                            egui::FontId::proportional(8.0), // Reduce font size
                            egui::Color32::DARK_GRAY,
                        );

                    }

                    if visible_rect.intersects(rect) {
                        let graph = graph.lock().expect("Failed to obtain mutex for graph");
                        let node = graph.get_node_at((x, y));
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("{}", node.score),
                            egui::FontId::proportional(10.0), // Reduce font size
                            egui::Color32::BLACK,
                        );


                    }

                    ui.allocate_ui_at_rect(rect, |ui| {
                        let (_, res) = ui.allocate_exact_size(rect_size, Sense::click());
                        if res.clicked() {
                            self.start = Some((x, y));
                        }
                    });


                }
            }
        });

    }
}

fn percentage_to_rgb(percent: f32) -> (u8, u8, u8) {
    let percent = percent.clamp(0.0, 100.0);
    let range = 100.0 / 3.0;

    if percent < range {
        // Red to Blue
        let t = percent / range;
        (
            (255.0 * (1.0 - t)) as u8, // Red decreases
            0,
            (255.0 * t) as u8 // Blue increases
        )
    } else if percent < 2.0 * range {
        // Blue to Green
        let t = (percent - range) / range;
        (
            0,
            (255.0 * t) as u8, // Green increases
            (255.0 * (1.0 - t)) as u8 // Blue decreases
        )
    } else {
        // Green to Red
        let t = (percent - 2.0 * range) / range;
        (
            (255.0 * t) as u8, // Red increases
            (255.0 * (1.0 - t)) as u8, // Green decreases
            0
        )
    }
}
