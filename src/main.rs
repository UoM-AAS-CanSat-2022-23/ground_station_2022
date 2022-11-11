use std::sync::mpsc::channel;
use std::thread::{self, JoinHandle};

use anyhow::Result;
use eframe::egui;
use ground_station::app::{GroundStationGui, GroundStationGuiBuilder};
use ground_station::reader::TelemetryReader;

fn main() -> Result<()> {
    // initialise the logger
    env_logger::init();

    // create a channel for communicating between the reader thread and the main thread
    let (tx, rx) = channel();
    let mut reader = TelemetryReader::new(tx);
    let _handle: JoinHandle<Result<()>> = thread::Builder::new()
        .name("reader".to_string())
        .spawn(move || reader.run())?;

    // run GUI
    let options = eframe::NativeOptions::default();
    let my_app: GroundStationGui = GroundStationGuiBuilder::default().rx(rx).build()?;

    eframe::run_native(
        "MCP Ground Station",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            let ppp = cc.egui_ctx.pixels_per_point();
            log::info!("ppp = {ppp}");
            cc.egui_ctx.set_pixels_per_point(ppp * 0.9);
            Box::new(my_app)
        }),
    );
}