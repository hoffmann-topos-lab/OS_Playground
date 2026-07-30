//! Tela "Race Conditions".
//!
//! O leitor de artigos é o de [`crate::ui::reader`]; o conteúdo vem de
//! [`crate::ui::content_race`]. Cada algoritmo traz uma demonstração passo a
//! passo, desenhada pelo próprio app.

use eframe::egui;

use crate::app::OsPlaygroundApp;
use crate::ui::{content_race, reader};

pub fn show(app: &mut OsPlaygroundApp, ui: &mut egui::Ui) {
    let goto = reader::show(
        ui,
        content_race::ARTICLES,
        "Capítulo 2 — seção 2.4",
        &mut app.race,
    );
    if let Some(screen) = goto {
        app.navigate(screen);
    }
}
