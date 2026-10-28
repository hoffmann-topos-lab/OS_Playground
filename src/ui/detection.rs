//! Tela "Detecção e Recuperação" (seção 6.5).
//!
//! Leitor de artigos de [`crate::ui::reader`], com o conteúdo de
//! [`crate::ui::content_detection`] e os simuladores de [`crate::ui::deadlockview`].

use eframe::egui;

use crate::app::OsPlaygroundApp;
use crate::ui::{content_detection, reader};

pub fn show(app: &mut OsPlaygroundApp, ui: &mut egui::Ui) {
    let goto = reader::show(
        ui,
        content_detection::ARTICLES,
        "Capítulo 6 — seção 6.5",
        &mut app.detection,
    );
    if let Some(screen) = goto {
        app.navigate(screen);
    }
}
