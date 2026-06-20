//! Main Application UI
//! 
//! Central application controller integrating all UI components.

use egui::{CentralPanel, SidePanel, TopBottomPanel, Context, RichText, Color32, menu, Ui};
use std::sync::{Arc, Mutex};
use tracing::{info, debug, error};

use crate::core::app_state::{SharedAppState, Theme, Language, LogLevel, OperationPhase};
use crate::core::events::{EventBus, UiEvent, BackendCommand};
use crate::core::worker::Worker;
use crate::ui::styles::{self, apply_theme, SUCCESS_COLOR, WARNING_COLOR, ERROR_COLOR, INFO_COLOR};
use crate::ui::i18n::{init_translations, t};
use crate::ui::components::{
    device_list::DeviceListComponent,
    iso_selector::IsoSelectorComponent,
    options_panel::OptionsPanelComponent,
    progress_panel::ProgressPanelComponent,
    log_panel::LogPanelComponent,
};

/// Main application struct
pub struct SidozdevApp {
    state: SharedAppState,
    event_bus: Arc<Mutex<EventBus>>,
    worker: Worker,

    // Components
    device_list: DeviceListComponent,
    iso_selector: IsoSelectorComponent,
    options_panel: OptionsPanelComponent,
    progress_panel: ProgressPanelComponent,
    log_panel: LogPanelComponent,

    // UI State
    show_about: bool,
    show_settings: bool,
    show_error: bool,
    error_message: String,
}

impl SidozdevApp {
    pub fn new(
        state: SharedAppState,
        event_bus: Arc<Mutex<EventBus>>,
        worker: Worker,
    ) -> Self {
        // Initialize translations
        {
            let state_guard = state.lock().unwrap();
            init_translations(state_guard.language);
        }

        Self {
            state,
            event_bus,
            worker,
            device_list: DeviceListComponent::new(),
            iso_selector: IsoSelectorComponent::new(),
            options_panel: OptionsPanelComponent::new(),
            progress_panel: ProgressPanelComponent::new(),
            log_panel: LogPanelComponent::new(),
            show_about: false,
            show_settings: false,
            show_error: false,
            error_message: String::new(),
        }
    }

    /// Process events from the backend
    fn process_events(&mut self, ctx: &Context) {
        if let Ok(bus) = self.event_bus.lock() {
            while let Ok(event) = bus.ui_receiver.try_recv() {
                match event {
                    UiEvent::DevicesUpdated(devices) => {
                        let mut state = self.state.lock().unwrap();
                        state.devices = devices;
                        state.add_log(LogLevel::Success, 
                            format!("Devices updated: {} found", state.devices.len()));
                    }
                    UiEvent::IsoValidated(iso_info) => {
                        let mut state = self.state.lock().unwrap();
                        state.iso_info = Some(iso_info.clone());
                        state.add_log(LogLevel::Info, 
                            format!("ISO validated: {} ({} MB)", 
                                iso_info.label, 
                                iso_info.size / 1024 / 1024));
                    }
                    UiEvent::ProgressUpdated(progress) => {
                        let mut state = self.state.lock().unwrap();
                        state.progress = progress;
                    }
                    UiEvent::LogMessage(log_entry) => {
                        let mut state = self.state.lock().unwrap();
                        state.logs.push(log_entry);
                    }
                    UiEvent::OperationCompleted => {
                        let mut state = self.state.lock().unwrap();
                        state.is_running = false;
                        state.progress.phase = OperationPhase::Completed;
                        state.add_log(LogLevel::Success, "Operation completed successfully!");
                    }
                    UiEvent::OperationCancelled => {
                        let mut state = self.state.lock().unwrap();
                        state.is_running = false;
                        state.progress.phase = OperationPhase::Cancelled;
                        state.add_log(LogLevel::Warning, "Operation was cancelled");
                    }
                    UiEvent::Error(msg) => {
                        let mut state = self.state.lock().unwrap();
                        state.is_running = false;
                        state.progress.phase = OperationPhase::Error;
                        state.error_message = Some(msg.clone());
                        state.add_log(LogLevel::Error, msg.clone());
                        self.error_message = msg;
                        self.show_error = true;
                    }
                    UiEvent::DeviceRemoved(device_id) => {
                        let mut state = self.state.lock().unwrap();
                        state.devices.retain(|d| d.device_id != device_id);
                        if state.config.selected_device.as_ref() == Some(&device_id) {
                            state.config.selected_device = None;
                        }
                        state.add_log(LogLevel::Warning, 
                            format!("Device removed: {}", device_id));
                    }
                    UiEvent::DeviceAdded(device) => {
                        let mut state = self.state.lock().unwrap();
                        state.devices.push(device.clone());
                        state.add_log(LogLevel::Info, 
                            format!("Device added: {}", device.friendly_name));
                    }
                }
                ctx.request_repaint();
            }
        }
    }

    /// Send command to backend
    fn send_command(&self, command: BackendCommand) {
        if let Ok(bus) = self.event_bus.lock() {
            let _ = bus.backend_sender.send(command);
        }
    }
}

impl eframe::App for SidozdevApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Process events from backend
        self.process_events(ctx);

        // Apply theme
        let is_dark = {
            let state = self.state.lock().unwrap();
            match state.theme {
                Theme::Dark => true,
                Theme::Light => false,
                Theme::System => {
                    // Check system preference
                    ctx.style().visuals.dark_mode
                }
            }
        };
        apply_theme(ctx, is_dark);

        // Menu bar
        TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button(t("menu.file"), |ui| {
                    if ui.button(t("menu.settings")).clicked() {
                        self.show_settings = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button(t("menu.exit")).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button(t("menu.help"), |ui| {
                    if ui.button(t("menu.about")).clicked() {
                        self.show_about = true;
                        ui.close_menu();
                    }
                });
            });
        });

        // Left panel - Device and ISO
        SidePanel::left("left_panel")
            .resizable(true)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);

                // Device selection
                self.device_list.render(ui, &mut self.state);

                ui.add_space(16.0);
                ui.separator();

                // ISO selection
                self.iso_selector.render(ui, &mut self.state);

                ui.add_space(16.0);
            });

        // Right panel - Options and Progress
        SidePanel::right("right_panel")
            .resizable(true)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);

                // Options
                self.options_panel.render(ui, &mut self.state);

                ui.add_space(16.0);
                ui.separator();

                // Progress
                self.progress_panel.render(ui, &mut self.state);

                ui.add_space(16.0);
            });

        // Bottom panel - Logs
        TopBottomPanel::bottom("log_panel")
            .resizable(true)
            .default_height(200.0)
            .show(ctx, |ui| {
                self.log_panel.render(ui, &mut self.state);
            });

        // Central panel - Status and info
        CentralPanel::default().show(ctx, |ui| {
            ui.add_space(8.0);

            // Application title
            ui.heading(RichText::new(t("app.title")).size(24.0).strong());
            ui.label(RichText::new(t("app.subtitle")).color(Color32::GRAY));

            ui.add_space(16.0);

            // Status overview
            let state = self.state.lock().unwrap();

            ui.group(|ui| {
                ui.heading("Status Overview");
                ui.separator();

                // Device status
                ui.horizontal(|ui| {
                    ui.label("Device:");
                    if let Some(device) = &state.config.selected_device {
                        ui.label(RichText::new(device).color(SUCCESS_COLOR));
                    } else {
                        ui.label(RichText::new("Not selected").color(WARNING_COLOR));
                    }
                });

                // ISO status
                ui.horizontal(|ui| {
                    ui.label("ISO:");
                    if let Some(iso) = &state.config.iso_path {
                        ui.label(RichText::new(iso.to_string_lossy().to_string()).color(SUCCESS_COLOR));
                    } else {
                        ui.label(RichText::new("Not selected").color(WARNING_COLOR));
                    }
                });

                // Ready status
                let is_ready = state.is_ready();
                ui.horizontal(|ui| {
                    ui.label("Ready:");
                    if is_ready {
                        ui.label(RichText::new("✓ Yes").color(SUCCESS_COLOR).strong());
                    } else {
                        ui.label(RichText::new("✗ No").color(WARNING_COLOR));
                    }
                });

                // Running status
                ui.horizontal(|ui| {
                    ui.label("Running:");
                    if state.is_running {
                        ui.label(RichText::new("Yes").color(INFO_COLOR));
                    } else {
                        ui.label("No");
                    }
                });
            });

            ui.add_space(16.0);

            // Quick info
            ui.group(|ui| {
                ui.heading("Quick Information");
                ui.separator();

                ui.label("• Select a USB device from the left panel");
                ui.label("• Choose an ISO image file");
                ui.label("• Configure boot options in the right panel");
                ui.label("• Click Start to begin writing");
                ui.label("• All data on the USB device will be erased!");
            });
        });

        // About dialog
        if self.show_about {
            egui::Window::new(t("about.title"))
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.heading("Sidozdev");
                    ui.label(format!("{}: {}", t("about.version"), env!("CARGO_PKG_VERSION")));
                    ui.add_space(8.0);
                    ui.label(t("about.description"));
                    ui.add_space(8.0);
                    ui.label(t("about.copyright"));
                    ui.label(t("about.license"));
                    ui.add_space(16.0);

                    if ui.button("Close").clicked() {
                        self.show_about = false;
                    }
                });
        }

        // Settings dialog
        if self.show_settings {
            egui::Window::new(t("settings.title"))
                .collapsible(false)
                .resizable(true)
                .show(ctx, |ui| {
                    let mut state = self.state.lock().unwrap();

                    ui.heading(t("settings.language"));
                    ui.horizontal(|ui| {
                        if ui.selectable_label(
                            matches!(state.language, Language::English), 
                            "English"
                        ).clicked() {
                            state.language = Language::English;
                            init_translations(Language::English);
                        }
                        if ui.selectable_label(
                            matches!(state.language, Language::Arabic), 
                            "العربية"
                        ).clicked() {
                            state.language = Language::Arabic;
                            init_translations(Language::Arabic);
                        }
                    });

                    ui.add_space(16.0);

                    ui.heading(t("settings.theme"));
                    ui.horizontal(|ui| {
                        if ui.selectable_label(
                            matches!(state.theme, Theme::Light), 
                            t("settings.theme.light")
                        ).clicked() {
                            state.theme = Theme::Light;
                        }
                        if ui.selectable_label(
                            matches!(state.theme, Theme::Dark), 
                            t("settings.theme.dark")
                        ).clicked() {
                            state.theme = Theme::Dark;
                        }
                        if ui.selectable_label(
                            matches!(state.theme, Theme::System), 
                            t("settings.theme.system")
                        ).clicked() {
                            state.theme = Theme::System;
                        }
                    });

                    ui.add_space(16.0);

                    if ui.button("Close").clicked() {
                        self.show_settings = false;
                    }
                });
        }

        // Error dialog
        if self.show_error {
            egui::Window::new("Error")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(RichText::new(&self.error_message).color(ERROR_COLOR));
                    ui.add_space(16.0);
                    if ui.button("OK").clicked() {
                        self.show_error = false;
                    }
                });
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Clean up worker thread
        self.worker.request_cancel();
        info!("Application shutting down");
    }
}
