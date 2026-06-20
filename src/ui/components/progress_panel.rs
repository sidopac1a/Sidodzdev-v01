//! Progress Panel Component
//! 
//! Real-time progress display with speed, ETA, and status information.

use egui::{Ui, RichText, ProgressBar, Color32, Button, Vec2};
use crate::core::app_state::{SharedAppState, OperationPhase, LogLevel};
use crate::ui::i18n::t;
use crate::ui::styles::{SUCCESS_COLOR, WARNING_COLOR, ERROR_COLOR, INFO_COLOR};

/// Progress panel component
pub struct ProgressPanelComponent;

impl ProgressPanelComponent {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, ui: &mut Ui, state: &mut SharedAppState) {
        let mut state_guard = state.lock().unwrap();

        ui.heading(RichText::new(t("progress.title")).size(20.0));
        ui.separator();

        let progress = &state_guard.progress;

        // Progress bar
        let progress_pct = progress.percentage / 100.0;
        let progress_text = format!("{:.1}%", progress.percentage);

        let bar_color = match progress.phase {
            OperationPhase::Error => ERROR_COLOR,
            OperationPhase::Completed => SUCCESS_COLOR,
            OperationPhase::Cancelled => WARNING_COLOR,
            _ => INFO_COLOR,
        };

        ui.add(
            ProgressBar::new(progress_pct.clamp(0.0, 1.0))
                .text(progress_text)
                .desired_width(ui.available_width())
        );

        ui.add_space(8.0);

        // Status information
        ui.horizontal(|ui| {
            ui.label(RichText::new(t("progress.status")).strong());
            ui.label(&progress.current_operation);
        });

        // Speed and ETA
        if progress.speed_mbps > 0.0 && progress.phase == OperationPhase::Writing {
            ui.horizontal(|ui| {
                ui.label(RichText::new(t("progress.speed")).strong());
                ui.label(format!("{:.2} MB/s", progress.speed_mbps));
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new(t("progress.eta")).strong());
                let minutes = progress.eta_seconds / 60;
                let seconds = progress.eta_seconds % 60;
                ui.label(format!("{:02}:{:02}", minutes, seconds));
            });
        }

        // Bytes written
        if progress.total_bytes > 0 {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Written:").strong());
                let written_mb = progress.bytes_written as f64 / 1024.0 / 1024.0;
                let total_mb = progress.total_bytes as f64 / 1024.0 / 1024.0;
                ui.label(format!("{:.1} / {:.1} MB", written_mb, total_mb));
            });
        }

        ui.add_space(16.0);

        // Action buttons
        ui.horizontal(|ui| {
            let is_ready = state_guard.is_ready();
            let is_running = state_guard.is_running;

            // Start button
            let start_button = ui.add_sized(
                [120.0, 40.0],
                Button::new(RichText::new(t("progress.start")).size(16.0).strong())
                    .fill(if is_ready && !is_running { SUCCESS_COLOR } else { Color32::GRAY })
            );

            if start_button.clicked() && is_ready && !is_running {
                // Trigger start operation
                state_guard.is_running = true;
                state_guard.add_log(LogLevel::Info, "Starting write operation...");
            }

            ui.add_space(16.0);

            // Cancel button
            let cancel_button = ui.add_sized(
                [120.0, 40.0],
                Button::new(RichText::new(t("progress.cancel")).size(16.0).strong())
                    .fill(if is_running { ERROR_COLOR } else { Color32::GRAY })
            );

            if cancel_button.clicked() && is_running {
                state_guard.request_cancel();
            }
        });

        // Phase indicator
        ui.add_space(8.0);
        let phase_text = match progress.phase {
            OperationPhase::Idle => t("status.ready"),
            OperationPhase::ScanningDevices => t("status.scanning"),
            OperationPhase::ValidatingIso => t("status.validating"),
            OperationPhase::Formatting => t("status.formatting"),
            OperationPhase::Writing => t("status.writing"),
            OperationPhase::Verifying => t("status.verifying_write"),
            OperationPhase::Completed => t("status.completed"),
            OperationPhase::Cancelled => t("status.cancelled"),
            OperationPhase::Error => t("status.error"),
        };

        let phase_color = match progress.phase {
            OperationPhase::Completed => SUCCESS_COLOR,
            OperationPhase::Cancelled => WARNING_COLOR,
            OperationPhase::Error => ERROR_COLOR,
            _ => INFO_COLOR,
        };

        ui.label(RichText::new(phase_text).color(phase_color).strong().size(16.0));
    }
}
