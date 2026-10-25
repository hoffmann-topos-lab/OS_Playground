//! Tela "Substituição de Páginas" (seção 3.4).
//!
//! O leitor de artigos é o de [`crate::ui::reader`]; o conteúdo vem de
//! [`crate::ui::content_replacement`]. Cada algoritmo traz o simulador de
//! substituição rodando a mesma cadeia de referências, passo a passo.

use eframe::egui;

use crate::app::OsPlaygroundApp;
use crate::ui::{content_replacement, reader};

pub fn show(app: &mut OsPlaygroundApp, ui: &mut egui::Ui) {
    let goto = reader::show(
        ui,
        content_replacement::ARTICLES,
        "Capítulo 3 — seção 3.4",
        &mut app.replacement,
    );
    if let Some(screen) = goto {
        app.navigate(screen);
    }
}
