//! Options Panel Component
//! 
//! Configuration options for boot mode, partition scheme, and file system.

use egui::{Ui, RichText, ComboBox, Checkbox, CollapsingHeader};
use crate::core::app_state::{SharedAppState, BootMode, PartitionScheme, FileSystem};
use crate::ui::i18n::t;
use crate::ui::styles::INFO_COLOR;

/// Options panel component
pub struct OptionsPanelComponent;

impl OptionsPanelComponent {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, ui: &mut Ui, state: &mut SharedAppState) {
        let mut state_guard = state.lock().unwrap();

        ui.heading(RichText::new(t("options.title")).size(20.0));
        ui.separator();

        // Boot Mode selection
        ui.horizontal(|ui| {
            ui.label(RichText::new(t("options.boot_mode")).strong());

            ComboBox::from_id_source("boot_mode")
                .selected_text(match state_guard.config.boot_mode {
                    BootMode::BiosLegacy => t("boot.bios"),
                    BootMode::Uefi => t("boot.uefi"),
                    BootMode::UefiSecureBoot => t("boot.uefi_secure"),
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut state_guard.config.boot_mode, 
                        BootMode::BiosLegacy, t("boot.bios"));
                    ui.selectable_value(&mut state_guard.config.boot_mode, 
                        BootMode::Uefi, t("boot.uefi"));
                    ui.selectable_value(&mut state_guard.config.boot_mode, 
                        BootMode::UefiSecureBoot, t("boot.uefi_secure"));
                });
        });

        ui.add_space(8.0);

        // Partition Scheme selection
        ui.horizontal(|ui| {
            ui.label(RichText::new(t("options.partition")).strong());

            ComboBox::from_id_source("partition_scheme")
                .selected_text(match state_guard.config.partition_scheme {
                    PartitionScheme::Mbr => t("partition.mbr"),
                    PartitionScheme::Gpt => t("partition.gpt"),
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut state_guard.config.partition_scheme, 
                        PartitionScheme::Mbr, t("partition.mbr"));
                    ui.selectable_value(&mut state_guard.config.partition_scheme, 
                        PartitionScheme::Gpt, t("partition.gpt"));
                });
        });

        ui.add_space(8.0);

        // File System selection
        ui.horizontal(|ui| {
            ui.label(RichText::new(t("options.file_system")).strong());

            ComboBox::from_id_source("file_system")
                .selected_text(match state_guard.config.file_system {
                    FileSystem::Fat32 => t("fs.fat32"),
                    FileSystem::Ntfs => t("fs.ntfs"),
                    FileSystem::ExFat => t("fs.exfat"),
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut state_guard.config.file_system, 
                        FileSystem::Fat32, t("fs.fat32"));
                    ui.selectable_value(&mut state_guard.config.file_system, 
                        FileSystem::Ntfs, t("fs.ntfs"));
                    ui.selectable_value(&mut state_guard.config.file_system, 
                        FileSystem::ExFat, t("fs.exfat"));
                });
        });

        ui.add_space(8.0);

        // Advanced options
        CollapsingHeader::new(t("options.advanced"))
            .default_open(false)
            .show(ui, |ui| {
                ui.checkbox(&mut state_guard.config.quick_format, t("options.quick_format"));
                ui.checkbox(&mut state_guard.config.verify_after_write, t("options.verify"));
                ui.checkbox(&mut state_guard.config.bad_block_check, t("options.bad_blocks"));
            });

        // Compatibility warning
        if state_guard.config.boot_mode == BootMode::UefiSecureBoot 
            && state_guard.config.partition_scheme == PartitionScheme::Mbr {
            ui.add_space(8.0);
            ui.label(
                RichText::new("⚠ UEFI Secure Boot with MBR is not recommended. Consider using GPT.")
                    .color(INFO_COLOR)
            );
        }
    }
}
