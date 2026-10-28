mod app;
mod editor;
mod execution;
mod settings;
mod workspace;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("VYRON IDE")
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([900.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "VYRON IDE",
        options,
        Box::new(|cc| Ok(Box::new(app::VyronApp::new(cc)))),
    )
}
