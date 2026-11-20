use eframe::egui::{self, RichText, Ui};

use crate::app::Configurator;
use crate::ui::{self, icons};

impl Configurator {
    pub(crate) fn page_tokens(&mut self, ui: &mut Ui) {
        let tokens = self.config.tokens();
        if tokens.is_empty() {
            ui.weak("No tokens. Add a [Tokens] section to config.ini with `Name = value` lines.");
            return;
        }
        let mut copied = None;
        egui::ScrollArea::vertical()
            .id_salt("tokens")
            .show(ui, |ui| {
                egui::Grid::new("tokens_grid")
                    .num_columns(3)
                    .striped(true)
                    .spacing([16.0, 8.0])
                    .show(ui, |ui| {
                        for (name, token) in tokens {
                            ui.label(RichText::new(name).strong());
                            ui.add(
                                egui::Label::new(RichText::new(token).monospace()).selectable(true),
                            );
                            if ui::icon_button(ui, icons::copy(), ui::GREEN, "Copy to clipboard")
                                .clicked()
                            {
                                ui.ctx().copy_text(token.to_owned());
                                copied = Some(name.to_owned());
                            }
                            ui.end_row();
                        }
                    });
            });
        if let Some(name) = copied {
            self.status(format!("Copied token “{name}” to the clipboard"));
        }
    }
}
