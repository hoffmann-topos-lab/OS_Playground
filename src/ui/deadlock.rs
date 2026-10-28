//! Tela "Impasses — Conceitos" (seções 6.0, 6.1, 6.3, 6.4 e 6.7).
//!
//! Leitor de artigos de [`crate::ui::reader`], com o conteúdo de
//! [`crate::ui::content_deadlock`] e os diagramas de [`crate::ui::deadlockview`].

use eframe::egui;

use crate::app::OsPlaygroundApp;
use crate::ui::{content_deadlock, reader};

pub fn show(app: &mut OsPlaygroundApp, ui: &mut egui::Ui) {
    let goto = reader::show(
        ui,
        content_deadlock::ARTICLES,
        "Capítulo 6 — conceitos e estratégias",
        &mut app.deadlock,
    );
    if let Some(screen) = goto {
        app.navigate(screen);
    }
}
