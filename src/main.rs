mod app;

fn main() {
    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "Rust drawer",
        options,
        Box::new(|cc| Ok(Box::new(app::RustyApp::new(cc)))),
    );
}
