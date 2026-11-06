//! Automate › Connect to AI Apps («ربط مع الذكاء الاصطناعي»): connects جدول's MCP server
//! (`jadwal-cli mcp`, shipped beside the app) to the AI apps on this computer with one button
//! each, by writing their own settings (the `ai-connect` crate). Other MCP apps get the settings
//! to paste. Desktop only: the web build has no MCP server and no settings files.

use crate::SheetApp;
use crate::i18n::Language;

#[derive(Default)]
pub(crate) struct Runtime {
    pub(crate) open: bool,
    rows: Vec<(ai_connect::Client, ai_connect::State)>,
    /// The last action's result: text and whether it went wrong.
    message: Option<(String, bool)>,
}

/// The MCP server beside the running app.
fn server() -> Option<ai_connect::Server> {
    let exe = std::env::current_exe().ok()?;
    let command = ai_connect::find_server(&exe, &["jadwal-cli", "gridcraft-cli"])?;
    Some(ai_connect::Server { key: "jadwal".into(), command, args: vec!["mcp".into()] })
}

fn refresh(rt: &mut Runtime) {
    rt.rows.clear();
    if let (Some(dirs), Some(server)) = (ai_connect::Dirs::system(), server()) {
        rt.rows = ai_connect::Client::ALL.into_iter().map(|c| (c, ai_connect::state(c, &dirs, &server))).collect();
    }
}

/// Open the window (the `app.connectAi` command without `client`).
pub(crate) fn open(app: &mut SheetApp) {
    app.connect_ai.open = true;
    app.connect_ai.message = None;
    refresh(&mut app.connect_ai);
}

/// Connect or disconnect `client` without the window (`app.connectAi` with `client` and
/// `disconnect`): what the control channel, tests and agents use.
pub(crate) fn run(client: &str, disconnect: bool) -> Result<serde_json::Value, String> {
    let client = ai_connect::Client::from_id(client).ok_or_else(|| {
        let ids: Vec<&str> = ai_connect::Client::ALL.iter().map(|c| c.id()).collect();
        format!("unknown AI app {client:?}; one of: {}", ids.join(", "))
    })?;
    let dirs = ai_connect::Dirs::system().ok_or("the home folder is unknown")?;
    let server = server().ok_or("the MCP server (jadwal-cli) is not beside the app")?;
    if disconnect {
        ai_connect::disconnect(client, &dirs, &server).map_err(|e| e.to_string())?;
        return Ok(serde_json::json!({ "client": client.id(), "state": "notConnected" }));
    }
    match ai_connect::connect(client, &dirs, &server).map_err(|e| e.to_string())? {
        ai_connect::Done::Saved => Ok(serde_json::json!({ "client": client.id(), "state": "connected" })),
        ai_connect::Done::OpenLink(link) => Ok(serde_json::json!({ "client": client.id(), "link": link })),
    }
}

fn state_label(s: &ai_connect::State) -> &'static str {
    use ai_connect::State;
    match s {
        State::NotInstalled => "Not installed",
        State::NotConnected => "Not connected",
        State::Connected => "Connected ✓",
        State::Outdated => "Needs updating",
        State::Unreadable(_) => "Its settings can't be read",
    }
}

pub(crate) fn show(app: &mut SheetApp, ctx: &egui::Context) {
    if !app.connect_ai.open {
        return;
    }
    let lang = app.ui.language;
    let mut open = true;
    let mut close = false;
    let shown = egui::Window::new(lang.tr("Connect to AI Apps"))
        .id(egui::Id::unique("connect-ai-window"))
        .open(&mut open)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .default_width(440.0)
        .show(ctx, |ui| {
            ui.set_max_width(440.0);
            body(app, lang, ui);
            ui.add_space(8.0);
            if ui.button(lang.tr("Close")).clicked() {
                close = true;
            }
        });
    // Mirrored as a whole in a right-to-left interface, like every dialog.
    if let Some(shown) = shown {
        crate::mirror::overlay(ctx, shown.response.layer_id);
    }
    if !open || close {
        app.connect_ai.open = false;
    }
}

fn body(app: &mut SheetApp, lang: Language, ui: &mut egui::Ui) {
    use ai_connect::State;
    ui.label(lang.tr("Connect Jadwal to an AI app so it can build, fill and format your workbooks when you ask it to. Press Connect next to the app, then restart it."));
    ui.add_space(6.0);
    let Some(server) = server() else {
        ui.colored_label(ui.visuals().error_fg_color, lang.tr("The MCP server (jadwal-cli) isn't next to Jadwal. Download Jadwal again."));
        return;
    };
    if ai_connect::temporary_location(&server.command) {
        ui.colored_label(
            ui.visuals().warn_fg_color,
            lang.tr("Move Jadwal to the Applications folder and open it from there first, or the connection stops working once it closes."),
        );
        ui.add_space(4.0);
    }
    let Some(dirs) = ai_connect::Dirs::system() else { return };
    let mut action: Option<(ai_connect::Client, bool)> = None;
    let installed: Vec<_> = app.connect_ai.rows.iter().filter(|(_, s)| *s != State::NotInstalled).cloned().collect();
    if installed.is_empty() {
        ui.label(lang.tr("No AI app we know of is installed here. Install one, such as Claude, then open this window again."));
    } else {
        egui::Grid::new("connect_ai_grid").num_columns(3).spacing([16.0, 8.0]).show(ui, |ui| {
            for (client, state) in &installed {
                ui.label(egui::RichText::new(client.name()).strong());
                let label = ui.label(lang.tr(state_label(state)));
                if let State::Unreadable(why) = state {
                    label.on_hover_text(why);
                }
                let button = match state {
                    State::NotConnected => Some(("Connect", false)),
                    State::Outdated => Some(("Update", false)),
                    State::Connected => Some(("Disconnect", true)),
                    State::NotInstalled | State::Unreadable(_) => None,
                };
                match button {
                    Some((text, disconnect)) => {
                        if ui.button(lang.tr(text)).clicked() {
                            action = Some((*client, disconnect));
                        }
                    }
                    None => {
                        ui.label("");
                    }
                }
                ui.end_row();
            }
        });
    }
    let missing: Vec<&str> = app.connect_ai.rows.iter().filter(|(_, s)| *s == State::NotInstalled).map(|(c, _)| c.name()).collect();
    if !missing.is_empty() {
        ui.add_space(4.0);
        let list = missing.join(if lang.is_rtl() { "، " } else { ", " });
        ui.label(egui::RichText::new(format!("{} {list}", lang.tr("Not installed here:"))).small().weak());
    }
    if let Some((client, disconnect)) = action {
        let result = if disconnect {
            ai_connect::disconnect(client, &dirs, &server).map(|()| lang.tr("Disconnected.").to_string())
        } else {
            ai_connect::connect(client, &dirs, &server).map(|done| match done {
                ai_connect::Done::Saved if client.needs_restart() => lang.tr("Connected. Restart {app}.").replace("{app}", client.name()),
                ai_connect::Done::Saved => lang.tr("Connected.").to_string(),
                ai_connect::Done::OpenLink(link) => {
                    ui.ctx().open_url(egui::OpenUrl::new_tab(link));
                    lang.tr("Confirm the install in {app}.").replace("{app}", client.name())
                }
            })
        };
        app.connect_ai.message = Some(match result {
            Ok(text) => (text, false),
            Err(e) => {
                log::warn!("connecting {}: {e}", client.name());
                (format!("{} {e}", lang.tr("Couldn't connect:")), true)
            }
        });
        refresh(&mut app.connect_ai);
    }
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new(lang.tr("Tip: say jadwal in your request so the AI app uses this app, e.g. \"Use the jadwal tools to …\".")).small(),
    );
    if let Some((text, bad)) = &app.connect_ai.message {
        ui.add_space(6.0);
        if *bad {
            ui.colored_label(ui.visuals().error_fg_color, text);
        } else {
            ui.label(egui::RichText::new(text).strong());
        }
    }
    ui.add_space(8.0);
    ui.separator();
    ui.label(lang.tr("Another MCP app? Copy the settings and paste them into its MCP servers settings."));
    if ui.button(lang.tr("Copy Settings")).clicked() {
        ui.ctx().copy_text(server.snippet());
        app.connect_ai.message = Some((lang.tr("Settings copied.").to_string(), false));
    }
}

#[cfg(test)]
mod tests {
    use crate::{Services, SheetApp};
    use gridcraft_engine::Session;
    use serde_json::json;

    /// Automate › Connect to AI Apps opens its window, draws it, closes, and the command connects
    /// only the AI apps it knows.
    #[test]
    fn connect_ai_apps_window_opens_and_unknown_apps_are_refused() {
        let mut session = Session::new();
        session.new_workbook();
        let mut app = SheetApp::new(session, Services::default());
        assert_eq!(app.run("app.connectAi", json!({})).unwrap(), serde_json::Value::Null);
        assert!(app.connect_ai.open);
        let e = app.run("app.connectAi", json!({"client": "nope"})).unwrap_err();
        assert!(e.contains("unknown AI app") && e.contains("claudeCode"), "{e}");
        let ctx = egui::Context::default();
        for _ in 0..2 {
            let mut out = ctx.run_ui(egui::RawInput::default(), |ui| app.ui(ui));
            out.textures_delta.clear();
        }
        assert!(app.connect_ai.open, "drawing keeps it open");
        for lang in [crate::i18n::Language::Ar] {
            for text in ["Connect to AI Apps", "Connect", "Disconnect", "Copy Settings", "Connected ✓"] {
                assert_ne!(lang.tr(text), text, "{text} has an Arabic translation");
            }
        }
    }
}
