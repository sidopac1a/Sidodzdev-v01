//! UI Module
//! 
//! Main UI module using egui framework for cross-platform GUI.

pub mod app;
pub mod styles;
pub mod i18n;
pub mod components;

use eframe::NativeOptions;
use egui::ViewportBuilder;
use tracing::info;

use crate::core::app_state::SharedAppState;
use crate::core::events::EventBus;
use crate::core::worker::Worker;

/// Run the application
pub fn run() -> anyhow::Result<()> {
    info!("Initializing Sidozdev UI");

    let options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_min_inner_size([700.0, 500.0])
            .with_title("Sidozdev - USB Bootable Media Creator"),
        ..Default::default()
    };

    let state = crate::core::app_state::create_shared_state();
    let event_bus = std::sync::Arc::new(std::sync::Mutex::new(EventBus::new()));

    // Start worker thread
    let mut worker = Worker::new()?;
    worker.start(state.clone(), event_bus.clone());

    eframe::run_native(
        "Sidozdev",
        options,
        Box::new(|cc| {
            let style = styles::configure_style(&cc.egui_ctx);
            cc.egui_ctx.set_style(style);

            Box::new(app::SidozdevApp::new(state, event_bus, worker))
        }),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run application: {}", e))?;

    Ok(())
}
