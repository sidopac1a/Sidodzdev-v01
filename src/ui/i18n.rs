//! Internationalization
//! 
//! Multi-language support for Arabic and English.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

use crate::core::app_state::Language;

/// Translation strings
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Translations {
    strings: HashMap<String, String>,
}

impl Translations {
    pub fn new() -> Self {
        Self {
            strings: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> String {
        self.strings.get(key).cloned().unwrap_or_else(|| key.to_string())
    }

    pub fn load(language: Language) -> Self {
        match language {
            Language::Arabic => Self::load_arabic(),
            Language::English => Self::load_english(),
        }
    }

    fn load_english() -> Self {
        let mut strings = HashMap::new();

        strings.insert("app.title".to_string(), "Sidozdev - USB Bootable Media Creator".to_string());
        strings.insert("app.subtitle".to_string(), "Create bootable USB drives from ISO images".to_string());

        // Device section
        strings.insert("device.title".to_string(), "USB Device".to_string());
        strings.insert("device.select".to_string(), "Select USB Device".to_string());
        strings.insert("device.refresh".to_string(), "Refresh".to_string());
        strings.insert("device.none".to_string(), "No USB device detected".to_string());
        strings.insert("device.name".to_string(), "Name".to_string());
        strings.insert("device.capacity".to_string(), "Capacity".to_string());
        strings.insert("device.free".to_string(), "Free Space".to_string());
        strings.insert("device.interface".to_string(), "Interface".to_string());
        strings.insert("device.id".to_string(), "Device ID".to_string());
        strings.insert("device.warning".to_string(), "WARNING: All data on the selected device will be destroyed!".to_string());

        // ISO section
        strings.insert("iso.title".to_string(), "ISO Image".to_string());
        strings.insert("iso.select".to_string(), "Select ISO File".to_string());
        strings.insert("iso.browse".to_string(), "Browse...".to_string());
        strings.insert("iso.info".to_string(), "ISO Information".to_string());
        strings.insert("iso.label".to_string(), "Label".to_string());
        strings.insert("iso.size".to_string(), "Size".to_string());
        strings.insert("iso.type".to_string(), "Type".to_string());
        strings.insert("iso.bootable".to_string(), "Bootable".to_string());
        strings.insert("iso.boot_modes".to_string(), "Boot Modes".to_string());
        strings.insert("iso.architecture".to_string(), "Architecture".to_string());
        strings.insert("iso.hash".to_string(), "Hash".to_string());
        strings.insert("iso.verify".to_string(), "Verify ISO".to_string());
        strings.insert("iso.calculating".to_string(), "Calculating...".to_string());

        // Options section
        strings.insert("options.title".to_string(), "Options".to_string());
        strings.insert("options.boot_mode".to_string(), "Boot Mode".to_string());
        strings.insert("options.partition".to_string(), "Partition Scheme".to_string());
        strings.insert("options.file_system".to_string(), "File System".to_string());
        strings.insert("options.advanced".to_string(), "Advanced Options".to_string());
        strings.insert("options.quick_format".to_string(), "Quick Format".to_string());
        strings.insert("options.verify".to_string(), "Verify After Write".to_string());
        strings.insert("options.bad_blocks".to_string(), "Check Bad Blocks".to_string());

        // Boot modes
        strings.insert("boot.bios".to_string(), "BIOS (Legacy)".to_string());
        strings.insert("boot.uefi".to_string(), "UEFI".to_string());
        strings.insert("boot.uefi_secure".to_string(), "UEFI + Secure Boot".to_string());

        // Partition schemes
        strings.insert("partition.mbr".to_string(), "MBR".to_string());
        strings.insert("partition.gpt".to_string(), "GPT".to_string());

        // File systems
        strings.insert("fs.fat32".to_string(), "FAT32".to_string());
        strings.insert("fs.ntfs".to_string(), "NTFS".to_string());
        strings.insert("fs.exfat".to_string(), "exFAT".to_string());

        // Progress section
        strings.insert("progress.title".to_string(), "Progress".to_string());
        strings.insert("progress.start".to_string(), "Start".to_string());
        strings.insert("progress.cancel".to_string(), "Cancel".to_string());
        strings.insert("progress.status".to_string(), "Status".to_string());
        strings.insert("progress.percentage".to_string(), "Percentage".to_string());
        strings.insert("progress.speed".to_string(), "Speed".to_string());
        strings.insert("progress.eta".to_string(), "ETA".to_string());
        strings.insert("progress.elapsed".to_string(), "Elapsed".to_string());
        strings.insert("progress.writing".to_string(), "Writing...".to_string());
        strings.insert("progress.verifying".to_string(), "Verifying...".to_string());
        strings.insert("progress.completed".to_string(), "Completed!".to_string());
        strings.insert("progress.cancelled".to_string(), "Cancelled".to_string());
        strings.insert("progress.error".to_string(), "Error".to_string());

        // Logs section
        strings.insert("logs.title".to_string(), "Logs".to_string());
        strings.insert("logs.clear".to_string(), "Clear".to_string());
        strings.insert("logs.save".to_string(), "Save".to_string());
        strings.insert("logs.level.info".to_string(), "INFO".to_string());
        strings.insert("logs.level.warning".to_string(), "WARNING".to_string());
        strings.insert("logs.level.error".to_string(), "ERROR".to_string());
        strings.insert("logs.level.success".to_string(), "SUCCESS".to_string());

        // Status messages
        strings.insert("status.ready".to_string(), "Ready".to_string());
        strings.insert("status.scanning".to_string(), "Scanning devices...".to_string());
        strings.insert("status.validating".to_string(), "Validating ISO...".to_string());
        strings.insert("status.formatting".to_string(), "Formatting...".to_string());
        strings.insert("status.writing".to_string(), "Writing...".to_string());
        strings.insert("status.verifying_write".to_string(), "Verifying write...".to_string());
        strings.insert("status.completed".to_string(), "Operation completed successfully!".to_string());
        strings.insert("status.cancelled".to_string(), "Operation cancelled".to_string());
        strings.insert("status.error".to_string(), "An error occurred".to_string());

        // Error messages
        strings.insert("error.no_device".to_string(), "Please select a USB device".to_string());
        strings.insert("error.no_iso".to_string(), "Please select an ISO file".to_string());
        strings.insert("error.device_too_small".to_string(), "Device capacity is too small for the ISO".to_string());
        strings.insert("error.elevation_required".to_string(), "Administrator privileges required".to_string());
        strings.insert("error.write_failed".to_string(), "Write operation failed".to_string());
        strings.insert("error.verify_failed".to_string(), "Verification failed".to_string());

        // Menu
        strings.insert("menu.file".to_string(), "File".to_string());
        strings.insert("menu.settings".to_string(), "Settings".to_string());
        strings.insert("menu.help".to_string(), "Help".to_string());
        strings.insert("menu.about".to_string(), "About".to_string());
        strings.insert("menu.exit".to_string(), "Exit".to_string());

        // Settings
        strings.insert("settings.title".to_string(), "Settings".to_string());
        strings.insert("settings.language".to_string(), "Language".to_string());
        strings.insert("settings.theme".to_string(), "Theme".to_string());
        strings.insert("settings.theme.light".to_string(), "Light".to_string());
        strings.insert("settings.theme.dark".to_string(), "Dark".to_string());
        strings.insert("settings.theme.system".to_string(), "System".to_string());

        // About
        strings.insert("about.title".to_string(), "About Sidozdev".to_string());
        strings.insert("about.version".to_string(), "Version".to_string());
        strings.insert("about.description".to_string(), "A professional USB bootable media creation tool built with Rust.".to_string());
        strings.insert("about.copyright".to_string(), "Copyright (c) 2024 Sidozdev Team".to_string());
        strings.insert("about.license".to_string(), "Licensed under MIT License".to_string());

        Self { strings }
    }

    fn load_arabic() -> Self {
        let mut strings = HashMap::new();

        strings.insert("app.title".to_string(), "Sidozdev - منشئ وسائط USB قابلة للإقلاع".to_string());
        strings.insert("app.subtitle".to_string(), "إنشاء أقراص USB قابلة للإقلاع من ملفات ISO".to_string());

        // Device section
        strings.insert("device.title".to_string(), "جهاز USB".to_string());
        strings.insert("device.select".to_string(), "اختر جهاز USB".to_string());
        strings.insert("device.refresh".to_string(), "تحديث".to_string());
        strings.insert("device.none".to_string(), "لم يتم اكتشاف أي جهاز USB".to_string());
        strings.insert("device.name".to_string(), "الاسم".to_string());
        strings.insert("device.capacity".to_string(), "السعة".to_string());
        strings.insert("device.free".to_string(), "المساحة الحرة".to_string());
        strings.insert("device.interface".to_string(), "الواجهة".to_string());
        strings.insert("device.id".to_string(), "معرف الجهاز".to_string());
        strings.insert("device.warning".to_string(), "تحذير: سيتم حذف جميع البيانات على الجهاز المحدد!".to_string());

        // ISO section
        strings.insert("iso.title".to_string(), "صورة ISO".to_string());
        strings.insert("iso.select".to_string(), "اختر ملف ISO".to_string());
        strings.insert("iso.browse".to_string(), "استعراض...".to_string());
        strings.insert("iso.info".to_string(), "معلومات ISO".to_string());
        strings.insert("iso.label".to_string(), "التسمية".to_string());
        strings.insert("iso.size".to_string(), "الحجم".to_string());
        strings.insert("iso.type".to_string(), "النوع".to_string());
        strings.insert("iso.bootable".to_string(), "قابل للإقلاع".to_string());
        strings.insert("iso.boot_modes".to_string(), "أنظمة الإقلاع".to_string());
        strings.insert("iso.architecture".to_string(), "المعمارية".to_string());
        strings.insert("iso.hash".to_string(), "الهاش".to_string());
        strings.insert("iso.verify".to_string(), "التحقق من ISO".to_string());
        strings.insert("iso.calculating".to_string(), "جاري الحساب...".to_string());

        // Options section
        strings.insert("options.title".to_string(), "الخيارات".to_string());
        strings.insert("options.boot_mode".to_string(), "نظام الإقلاع".to_string());
        strings.insert("options.partition".to_string(), "نظام التقسيم".to_string());
        strings.insert("options.file_system".to_string(), "نظام الملفات".to_string());
        strings.insert("options.advanced".to_string(), "خيارات متقدمة".to_string());
        strings.insert("options.quick_format".to_string(), "تنسيق سريع".to_string());
        strings.insert("options.verify".to_string(), "التحقق بعد الكتابة".to_string());
        strings.insert("options.bad_blocks".to_string(), "فحص الباد سيكتور".to_string());

        // Boot modes
        strings.insert("boot.bios".to_string(), "BIOS (القديم)".to_string());
        strings.insert("boot.uefi".to_string(), "UEFI".to_string());
        strings.insert("boot.uefi_secure".to_string(), "UEFI + التمهيد الآمن".to_string());

        // Partition schemes
        strings.insert("partition.mbr".to_string(), "MBR".to_string());
        strings.insert("partition.gpt".to_string(), "GPT".to_string());

        // File systems
        strings.insert("fs.fat32".to_string(), "FAT32".to_string());
        strings.insert("fs.ntfs".to_string(), "NTFS".to_string());
        strings.insert("fs.exfat".to_string(), "exFAT".to_string());

        // Progress section
        strings.insert("progress.title".to_string(), "التقدم".to_string());
        strings.insert("progress.start".to_string(), "بدء".to_string());
        strings.insert("progress.cancel".to_string(), "إلغاء".to_string());
        strings.insert("progress.status".to_string(), "الحالة".to_string());
        strings.insert("progress.percentage".to_string(), "النسبة".to_string());
        strings.insert("progress.speed".to_string(), "السرعة".to_string());
        strings.insert("progress.eta".to_string(), "الوقت المتبقي".to_string());
        strings.insert("progress.elapsed".to_string(), "الوقت المنقضي".to_string());
        strings.insert("progress.writing".to_string(), "جاري الكتابة...".to_string());
        strings.insert("progress.verifying".to_string(), "جاري التحقق...".to_string());
        strings.insert("progress.completed".to_string(), "اكتملت العملية!".to_string());
        strings.insert("progress.cancelled".to_string(), "تم الإلغاء".to_string());
        strings.insert("progress.error".to_string(), "خطأ".to_string());

        // Logs section
        strings.insert("logs.title".to_string(), "السجلات".to_string());
        strings.insert("logs.clear".to_string(), "مسح".to_string());
        strings.insert("logs.save".to_string(), "حفظ".to_string());
        strings.insert("logs.level.info".to_string(), "معلومات".to_string());
        strings.insert("logs.level.warning".to_string(), "تحذير".to_string());
        strings.insert("logs.level.error".to_string(), "خطأ".to_string());
        strings.insert("logs.level.success".to_string(), "نجاح".to_string());

        // Status messages
        strings.insert("status.ready".to_string(), "جاهز".to_string());
        strings.insert("status.scanning".to_string(), "جاري فحص الأجهزة...".to_string());
        strings.insert("status.validating".to_string(), "جاري التحقق من ISO...".to_string());
        strings.insert("status.formatting".to_string(), "جاري التنسيق...".to_string());
        strings.insert("status.writing".to_string(), "جاري الكتابة...".to_string());
        strings.insert("status.verifying_write".to_string(), "جاري التحقق من الكتابة...".to_string());
        strings.insert("status.completed".to_string(), "اكتملت العملية بنجاح!".to_string());
        strings.insert("status.cancelled".to_string(), "تم إلغاء العملية".to_string());
        strings.insert("status.error".to_string(), "حدث خطأ".to_string());

        // Error messages
        strings.insert("error.no_device".to_string(), "الرجاء اختيار جهاز USB".to_string());
        strings.insert("error.no_iso".to_string(), "الرجاء اختيار ملف ISO".to_string());
        strings.insert("error.device_too_small".to_string(), "سعة الجهاز صغيرة جداً لملف ISO".to_string());
        strings.insert("error.elevation_required".to_string(), "مطلوب صلاحيات المسؤول".to_string());
        strings.insert("error.write_failed".to_string(), "فشلت عملية الكتابة".to_string());
        strings.insert("error.verify_failed".to_string(), "فشل التحقق".to_string());

        // Menu
        strings.insert("menu.file".to_string(), "ملف".to_string());
        strings.insert("menu.settings".to_string(), "إعدادات".to_string());
        strings.insert("menu.help".to_string(), "مساعدة".to_string());
        strings.insert("menu.about".to_string(), "حول".to_string());
        strings.insert("menu.exit".to_string(), "خروج".to_string());

        // Settings
        strings.insert("settings.title".to_string(), "الإعدادات".to_string());
        strings.insert("settings.language".to_string(), "اللغة".to_string());
        strings.insert("settings.theme".to_string(), "السمة".to_string());
        strings.insert("settings.theme.light".to_string(), "فاتح".to_string());
        strings.insert("settings.theme.dark".to_string(), "داكن".to_string());
        strings.insert("settings.theme.system".to_string(), "النظام".to_string());

        // About
        strings.insert("about.title".to_string(), "حول Sidozdev".to_string());
        strings.insert("about.version".to_string(), "الإصدار".to_string());
        strings.insert("about.description".to_string(), "أداة احترافية لإنشاء وسائط USB قابلة للإقلاع مبنية بـ Rust.".to_string());
        strings.insert("about.copyright".to_string(), "حقوق النشر (c) 2024 فريق Sidozdev".to_string());
        strings.insert("about.license".to_string(), "مرخصة بموجب رخصة MIT".to_string());

        Self { strings }
    }
}

impl Default for Translations {
    fn default() -> Self {
        Self::load_english()
    }
}

/// Global translations cache
static TRANSLATIONS: OnceLock<Translations> = OnceLock::new();

/// Initialize translations for a language
pub fn init_translations(language: Language) {
    let _ = TRANSLATIONS.set(Translations::load(language));
}

/// Get a translated string
pub fn t(key: &str) -> String {
    TRANSLATIONS.get()
        .map(|t| t.get(key))
        .unwrap_or_else(|| key.to_string())
}

/// Check if translations are initialized
pub fn is_initialized() -> bool {
    TRANSLATIONS.get().is_some()
}
