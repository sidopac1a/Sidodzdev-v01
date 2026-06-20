//! UI Styles
//! 
//! Theme configuration for light and dark modes.

use egui::{Style, Visuals, Color32, FontFamily, FontId, TextStyle, Rounding, Stroke, Spacing};

/// Configure application style based on theme
pub fn configure_style(ctx: &egui::Context) -> Style {
    let mut style = Style::default();

    // Configure fonts
    let mut font_definitions = egui::FontDefinitions::default();

    // Add Arabic font support (using system fonts)
    font_definitions.font_data.insert(
        "arabic".to_owned(),
        egui::FontData::from_owned(include_bytes!("../../assets/fonts/NotoSansArabic-Regular.ttf").to_vec()),
    );

    font_definitions.families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "arabic".to_owned());

    ctx.set_fonts(font_definitions);

    // Configure text styles
    style.text_styles = [
        (TextStyle::Heading, FontId::new(24.0, FontFamily::Proportional)),
        (TextStyle::Body, FontId::new(16.0, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(14.0, FontFamily::Monospace)),
        (TextStyle::Button, FontId::new(16.0, FontFamily::Proportional)),
        (TextStyle::Small, FontId::new(12.0, FontFamily::Proportional)),
    ].into();

    // Configure spacing
    style.spacing = Spacing {
        item_spacing: egui::vec2(8.0, 8.0),
        window_margin: egui::Margin::same(10.0),
        button_padding: egui::vec2(12.0, 8.0),
        ..Default::default()
    };

    // Configure rounding
    style.visuals.widgets.noninteractive.rounding = Rounding::same(6.0);
    style.visuals.widgets.inactive.rounding = Rounding::same(6.0);
    style.visuals.widgets.hovered.rounding = Rounding::same(6.0);
    style.visuals.widgets.active.rounding = Rounding::same(6.0);

    style
}

/// Light theme colors
pub fn light_theme() -> Visuals {
    let mut visuals = Visuals::light();

    visuals.panel_fill = Color32::from_rgb(245, 245, 250);
    visuals.extreme_bg_color = Color32::from_rgb(255, 255, 255);
    visuals.faint_bg_color = Color32::from_rgb(235, 235, 240);
    visuals.window_fill = Color32::from_rgb(255, 255, 255);
    visuals.window_stroke = Stroke::new(1.0, Color32::from_rgb(200, 200, 210));

    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(250, 250, 255);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(66, 133, 244);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(51, 103, 214);
    visuals.widgets.active.bg_fill = Color32::from_rgb(25, 64, 142);

    visuals.selection.bg_fill = Color32::from_rgb(66, 133, 244);
    visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(255, 255, 255));

    visuals.hyperlink_color = Color32::from_rgb(26, 115, 232);
    visuals.faint_bg_color = Color32::from_rgb(232, 240, 254);

    visuals
}

/// Dark theme colors
pub fn dark_theme() -> Visuals {
    let mut visuals = Visuals::dark();

    visuals.panel_fill = Color32::from_rgb(30, 30, 35);
    visuals.extreme_bg_color = Color32::from_rgb(18, 18, 22);
    visuals.faint_bg_color = Color32::from_rgb(40, 40, 48);
    visuals.window_fill = Color32::from_rgb(35, 35, 42);
    visuals.window_stroke = Stroke::new(1.0, Color32::from_rgb(60, 60, 70));

    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(45, 45, 55);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(138, 180, 248);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(174, 203, 250);
    visuals.widgets.active.bg_fill = Color32::from_rgb(210, 227, 252);

    visuals.selection.bg_fill = Color32::from_rgb(138, 180, 248);
    visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(30, 30, 35));

    visuals.hyperlink_color = Color32::from_rgb(138, 180, 248);
    visuals.faint_bg_color = Color32::from_rgb(30, 42, 60);

    visuals
}

/// Apply theme to context
pub fn apply_theme(ctx: &egui::Context, is_dark: bool) {
    if is_dark {
        ctx.set_visuals(dark_theme());
    } else {
        ctx.set_visuals(light_theme());
    }
}

/// Success color
pub const SUCCESS_COLOR: Color32 = Color32::from_rgb(52, 168, 83);
/// Warning color
pub const WARNING_COLOR: Color32 = Color32::from_rgb(251, 188, 5);
/// Error color
pub const ERROR_COLOR: Color32 = Color32::from_rgb(234, 67, 53);
/// Info color
pub const INFO_COLOR: Color32 = Color32::from_rgb(66, 133, 244);
