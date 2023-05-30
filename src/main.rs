mod modules;
pub use crate::modules::rpsgame;
fn main() {
    let app = modules::rpsgame::MyApp::default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
