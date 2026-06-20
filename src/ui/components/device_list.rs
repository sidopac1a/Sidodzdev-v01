//! Device List Component
//! 
//! Displays and manages USB device selection.

use egui::{Ui, RichText, Color32, ScrollArea, SelectableLabel, Button, Vec2};
use crate::core::app_state::{SharedAppState, Language};
use crate::disk::device::UsbDevice;
use crate::ui::i18n::t;
use crate::ui::styles::{SUCCESS_COLOR, WARNING_COLOR, ERROR_COLOR, INFO_COLOR};

/// Device list component
pub struct DeviceListComponent;

impl DeviceListComponent {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, ui: &mut Ui, state: &mut SharedAppState) {
        let mut state_guard = state.lock().unwrap();

        ui.heading(RichText::new(t("device.title")).size(20.0));
        ui.separator();

        // Refresh button
        ui.horizontal(|ui| {
            if ui.button(t("device.refresh")).clicked() {
                // Trigger device refresh via event bus
                // This would send a RefreshDevices command
            }
        });

        ui.add_space(8.0);

        // Device list
        if state_guard.devices.is_empty() {
            ui.label(RichText::new(t("device.none")).color(WARNING_COLOR));
        } else {
            ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for device in &state_guard.devices.clone() {
                        let is_selected = state_guard.config.selected_device.as_ref()
                            == Some(&device.device_id);

                        let device_text = format_device_info(&device);

                        if ui.selectable_label(is_selected, device_text).clicked() {
                            state_guard.config.selected_device = Some(device.device_id.clone());
                            state_guard.add_log(
                                crate::core::app_state::LogLevel::Info,
                                format!("Selected device: {}", device.friendly_name)
                            );
                        }

                        ui.separator();
                    }
                });
        }

        // Warning message
        if state_guard.config.selected_device.is_some() {
            ui.add_space(8.0);
            ui.label(
                RichText::new(t("device.warning"))
                    .color(ERROR_COLOR)
                    .strong()
            );
        }

        // Selected device details
        if let Some(selected_id) = &state_guard.config.selected_device {
            if let Some(device) = state_guard.devices.iter()
                .find(|d| d.device_id == *selected_id) {
                ui.add_space(8.0);
                ui.group(|ui| {
                    ui.label(RichText::new(t("device.name")).strong());
                    ui.label(&device.friendly_name);
                    ui.label(RichText::new(t("device.capacity")).strong());
                    ui.label(device.capacity_display());
                    ui.label(RichText::new(t("device.interface")).strong());
                    ui.label(&device.interface_type);
                    ui.label(RichText::new(t("device.id")).strong());
                    ui.label(&device.device_id);
                });
            }
        }
    }
}

/// Format device information for display
fn format_device_info(device: &UsbDevice) -> String {
    format!(
        "{} - {} ({} - {})",
        device.friendly_name,
        device.capacity_display(),
        device.interface_type,
        device.vid_pid
    )
}
