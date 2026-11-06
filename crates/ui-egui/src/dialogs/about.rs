use super::Dialog;

/// Help ▸ About GridCraft: the app, its contributors and the AI models that helped. The credits
/// are compiled in (`crate::credits`, docs/contributors.md).
pub(super) fn about(ui: &mut egui::Ui, d: &mut Dialog) {
    ui.horizontal(|ui| {
        for tab in ["About", "Contributors", "Models"] {
            if ui.selectable_label(d.tab == tab, crate::i18n::current(ui.ctx()).tr(tab)).clicked() {
                d.tab = tab.to_string();
            }
        }
    });
    ui.separator();
    match d.tab.as_str() {
        "Contributors" => crate::credits::contributors_ui(ui),
        "Models" => crate::credits::models_ui(ui),
        _ => {
            let lang = crate::i18n::current(ui.ctx());
            ui.heading(crate::APP_NAME);
            ui.label(lang.tr("Version {version}").replace("{version}", crate::APP_VERSION));
            ui.add_space(4.0);
            ui.label(lang.tr(
                "An Arabic, right-to-left spreadsheet by Nasser Alqahtani, built on GridCraft, the open-source spreadsheet by the ArtCraft team.",
            ));
            ui.hyperlink_to("github.com/nasserfq/jadwal", crate::REPO_URL);
        }
    }
}
