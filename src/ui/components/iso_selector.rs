//! ISO Selector Component
//! 
//! Handles ISO file selection and information display.

use egui::{Ui, RichText, Button, ScrollArea, Color32, TextEdit};
use rfd::FileDialog;
use crate::core::app_state::{SharedAppState, LogLevel};
use crate::ui::i18n::t;
use crate::ui::styles::{SUCCESS_COLOR, INFO_COLOR, WARNING_COLOR};

/// ISO selector component
pub struct IsoSelectorComponent;

impl IsoSelectorComponent {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, ui: &mut Ui, state: &mut SharedAppState) {
        let mut state_guard = state.lock().unwrap();

        ui.heading(RichText::new(t("iso.title")).size(20.0));
        ui.separator();

        // ISO selection row
        ui.horizontal(|ui| {
            // Display current ISO path
            let iso_path = state_guard.config.iso_path.as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| t("iso.select"));

            ui.label(RichText::new(iso_path).monospace());

            if ui.button(t("iso.browse")).clicked() && !state_guard.is_running {
                // Open file dialog
                if let Some(path) = FileDialog::new()
                    .add_filter("ISO/IMG Files", &["iso", "img", "bin"])
                    .add_filter("All Files", &["*"])
                    .pick_file() {

                    state_guard.config.iso_path = Some(path.clone());
                    state_guard.add_log(LogLevel::Info, 
                        format!("Selected ISO: {}", path.display()));

                    // Trigger ISO validation
                    // This would send a SelectIso command to the worker
                }
            }
        });

        ui.add_space(8.0);

        // ISO Information display
        if let Some(iso_info) = &state_guard.iso_info {
            ui.group(|ui| {
                ui.label(RichText::new(t("iso.info")).strong().size(16.0));
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label(RichText::new(t("iso.label")).strong());
                    ui.label(&iso_info.label);
                });

                ui.horizontal(|ui| {
                    ui.label(RichText::new(t("iso.size")).strong());
                    ui.label(crate::iso::parser::format_size(iso_info.size));
                });

                ui.horizontal(|ui| {
                    ui.label(RichText::new(t("iso.type")).strong());
                    ui.label(iso_info.iso_type.to_string());
                });

                ui.horizontal(|ui| {
                    ui.label(RichText::new(t("iso.bootable")).strong());
                    if iso_info.is_bootable {
                        ui.label(RichText::new("✓ Yes").color(SUCCESS_COLOR));
                    } else {
                        ui.label(RichText::new("✗ No").color(WARNING_COLOR));
                    }
                });

                if !iso_info.boot_modes.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(t("iso.boot_modes")).strong());
                        ui.label(iso_info.boot_modes.join(", "));
                    });
                }

                ui.horizontal(|ui| {
                    ui.label(RichText::new(t("iso.architecture")).strong());
                    ui.label(&iso_info.architecture);
                });

                if iso_info.is_hybrid {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Hybrid ISO:").strong());
                        ui.label(RichText::new("Yes").color(INFO_COLOR));
                    });
                }

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Estimated Write Time:").strong());
                    let minutes = iso_info.estimated_write_time / 60;
                    let seconds = iso_info.estimated_write_time % 60;
                    ui.label(format!("~{:02}:{:02} (at 10 MB/s)", minutes, seconds));
                });
            });

            // Hash verification buttons
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(t("iso.hash")).strong());

                if ui.button("MD5").clicked() && !state_guard.is_running {
                    // Trigger MD5 verification
                }
                if ui.button("SHA1").clicked() && !state_guard.is_running {
                    // Trigger SHA1 verification
                }
                if ui.button("SHA256").clicked() && !state_guard.is_running {
                    // Trigger SHA256 verification
                }
            });

            // Display calculated hashes if available
            if let Some(md5) = &iso_info.md5_hash {
                ui.horizontal(|ui| {
                    ui.label("MD5:");
                    ui.label(RichText::new(md5).monospace().size(12.0));
                });
            }
            if let Some(sha1) = &iso_info.sha1_hash {
                ui.horizontal(|ui| {
                    ui.label("SHA1:");
                    ui.label(RichText::new(sha1).monospace().size(12.0));
                });
            }
            if let Some(sha256) = &iso_info.sha256_hash {
                ui.horizontal(|ui| {
                    ui.label("SHA256:");
                    ui.label(RichText::new(sha256).monospace().size(12.0));
                });
            }
        }
    }
}
