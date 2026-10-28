//! Tela "Como Evitar Impasses" (seção 6.6).
//!
//! Leitor de artigos de [`crate::ui::reader`], com o conteúdo de
//! [`crate::ui::content_avoidance`] e o algoritmo do banqueiro de
//! [`crate::ui::deadlockview`].

use eframe::egui;

use crate::app::OsPlaygroundApp;
use crate::ui::{content_avoidance, reader};

pub fn show(app: &mut OsPlaygroundApp, ui: &mut egui::Ui) {
    let goto = reader::show(
        ui,
        content_avoidance::ARTICLES,
        "Capítulo 6 — seção 6.6",
        &mut app.avoidance,
    );
    if let Some(screen) = goto {
        app.navigate(screen);
    }
}
