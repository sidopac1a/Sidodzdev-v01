//! Log Panel Component
//! 
//! Detailed operation log display with filtering and export.

use egui::{Ui, RichText, ScrollArea, Color32, Button, TextEdit};
use crate::core::app_state::{SharedAppState, LogLevel};
use crate::ui::i18n::t;
use crate::ui::styles::{SUCCESS_COLOR, WARNING_COLOR, ERROR_COLOR, INFO_COLOR};

/// Log panel component
pub struct LogPanelComponent {
    filter_text: String,
    auto_scroll: bool,
}

impl LogPanelComponent {
    pub fn new() -> Self {
        Self {
            filter_text: String::new(),
            auto_scroll: true,
        }
    }

    pub fn render(&mut self, ui: &mut Ui, state: &mut SharedAppState) {
        let mut state_guard = state.lock().unwrap();

        ui.heading(RichText::new(t("logs.title")).size(20.0));
        ui.separator();

        // Log controls
        ui.horizontal(|ui| {
            if ui.button(t("logs.clear")).clicked() {
                state_guard.clear_logs();
            }

            if ui.button(t("logs.save")).clicked() {
                // Save logs to file
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Log Files", &["log"])
                    .add_filter("Text Files", &["txt"])
                    .set_file_name("sidozdev.log")
                    .save_file() {

                    let log_content = state_guard.logs.iter()
                        .map(|entry| format!("[{}] {:?} - {}", 
                            entry.timestamp, entry.level, entry.message))
                        .collect::<Vec<_>>()
                        .join("\n");

                    let _ = std::fs::write(&path, log_content);
                }
            }

            ui.checkbox(&mut self.auto_scroll, "Auto-scroll");
        });

        // Filter input
        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.text_edit_singleline(&mut self.filter_text);
        });

        ui.add_space(4.0);

        // Log display
        let filter_lower = self.filter_text.to_lowercase();
        let filtered_logs: Vec<_> = state_guard.logs.iter()
            .filter(|entry| {
                entry.message.to_lowercase().contains(&filter_lower) ||
                self.filter_text.is_empty()
            })
            .cloned()
            .collect();

        ScrollArea::vertical()
            .max_height(250.0)
            .auto_shrink([false; 2])
            .stick_to_bottom(self.auto_scroll)
            .show(ui, |ui| {
                for entry in &filtered_logs {
                    let (color, level_text) = match entry.level {
                        LogLevel::Info => (INFO_COLOR, t("logs.level.info")),
                        LogLevel::Warning => (WARNING_COLOR, t("logs.level.warning")),
                        LogLevel::Error => (ERROR_COLOR, t("logs.level.error")),
                        LogLevel::Success => (SUCCESS_COLOR, t("logs.level.success")),
                        LogLevel::Debug => (Color32::GRAY, "DEBUG".to_string()),
                    };

                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&entry.timestamp).monospace().size(11.0));
                        ui.label(RichText::new(level_text).color(color).strong().size(11.0));
                        ui.label(RichText::new(&entry.message).size(12.0));
                    });
                }
            });

        // Log count
        ui.add_space(4.0);
        ui.label(format!("Total logs: {} | Filtered: {}", 
            state_guard.logs.len(), filtered_logs.len()));
    }
}
