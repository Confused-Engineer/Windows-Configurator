//! Shared widgets, icons and theme.

use eframe::egui::{self, Color32, ImageSource, Response, RichText, Ui, Vec2, include_image};

pub const ROW_BUTTON: Vec2 = Vec2::splat(25.0);
pub const GREEN: Color32 = Color32::from_rgb(60, 200, 90);
pub const ORANGE: Color32 = Color32::from_rgb(240, 150, 40);
pub const RED: Color32 = Color32::from_rgb(230, 70, 70);
pub const NAV_TINT: Color32 = Color32::from_rgb(70, 110, 255);

/// Icons embedded in the binary.
pub mod icons {
    use super::*;

    macro_rules! icon {
        ($name:ident, $file:literal) => {
            pub fn $name() -> ImageSource<'static> {
                include_image!(concat!("../assets/resources/images/svg/", $file))
            }
        };
    }

    icon!(home, "home-outline.svg");
    icon!(download, "arrow-circle-down-outline.svg");
    icon!(key, "edit-2-outline.svg");
    icon!(launch, "external-link-outline.svg");
    icon!(tools, "printer-outline.svg");
    icon!(info, "info-outline.svg");
    icon!(settings, "settings-2-outline.svg");
    icon!(silent, "volume-off-outline.svg");
    icon!(upload, "upload-outline.svg");
    icon!(refresh, "refresh-outline.svg");
    icon!(copy, "copy-outline.svg");
}

pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.override_text_color = Some(Color32::from_rgb(253, 255, 230));
    visuals.faint_bg_color = Color32::from_rgb(20, 18, 40);
    visuals.extreme_bg_color = Color32::from_rgb(24, 45, 0); // text edits
    visuals.panel_fill = Color32::from_rgb(3, 0, 15);
    visuals.window_fill = Color32::from_rgb(10, 8, 30);
    ctx.set_visuals(visuals);

    ctx.global_style_mut(|style| {
        use egui::{FontFamily::Proportional, FontId, TextStyle};
        style
            .text_styles
            .insert(TextStyle::Heading, FontId::new(20.0, Proportional));
        style
            .text_styles
            .insert(TextStyle::Body, FontId::new(14.0, Proportional));
        style
            .text_styles
            .insert(TextStyle::Button, FontId::new(15.0, Proportional));
    });
}

/// Small tinted icon button used at the end of list rows.
pub fn icon_button(
    ui: &mut Ui,
    icon: ImageSource<'static>,
    tint: Color32,
    tooltip: &str,
) -> Response {
    let image = egui::Image::new(icon)
        .tint(tint)
        .fit_to_exact_size(ROW_BUTTON - Vec2::splat(6.0));
    ui.add_sized(ROW_BUTTON, egui::Button::image(image).corner_radius(5.0))
        .on_hover_text(tooltip)
}

/// A list row: label on the left, buttons (added right-to-left) on the
/// right, followed by a separator.
pub fn row(ui: &mut Ui, label: impl Into<egui::WidgetText>, buttons: impl FnOnce(&mut Ui)) {
    // Buttons are laid out first (from the right) so a long label is
    // truncated into the remaining space instead of pushing them off-screen.
    let size = egui::vec2(ui.available_width(), ROW_BUTTON.y);
    ui.allocate_ui_with_layout(
        size,
        egui::Layout::right_to_left(egui::Align::Center),
        |ui| {
            ui.add_space(8.0);
            buttons(ui);
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.add(egui::Label::new(label).truncate());
            });
        },
    );
    ui.separator();
}

/// Column heading followed by a separator.
pub fn heading(ui: &mut Ui, text: &str) {
    ui.heading(text);
    ui.separator();
}

/// Centered sub-heading for a group inside a column.
pub fn group_heading(ui: &mut Ui, text: &str) {
    ui.add_space(4.0);
    ui.vertical_centered(|ui| ui.label(RichText::new(text).strong().size(16.0)));
    ui.separator();
}

/// Block of tool output: monospace (tables line up) and selectable.
pub fn output_block(ui: &mut Ui, text: &str) {
    if text.is_empty() {
        ui.weak("…");
    } else {
        ui.add(
            egui::Label::new(RichText::new(text).monospace().size(12.0))
                .selectable(true)
                .extend(),
        );
    }
}

/// A coloured notice box.
pub fn notice(ui: &mut Ui, color: Color32, add_contents: impl FnOnce(&mut Ui)) {
    egui::Frame::group(ui.style())
        .stroke(egui::Stroke::new(1.0, color))
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add_contents(ui);
        });
    ui.add_space(4.0);
}
