//! Tela "Projeto e Implementação" (seções 3.5 e 3.6).
//!
//! O leitor de artigos é o de [`crate::ui::reader`]; o conteúdo vem de
//! [`crate::ui::content_pagedesign`]. Traz a curva do desperdício em função
//! do tamanho da página, com o mínimo marcado.

use eframe::egui;

use crate::app::OsPlaygroundApp;
use crate::ui::{content_pagedesign, reader};

pub fn show(app: &mut OsPlaygroundApp, ui: &mut egui::Ui) {
    let goto = reader::show(
        ui,
        content_pagedesign::ARTICLES,
        "Capítulo 3 — seções 3.5 e 3.6",
        &mut app.page_design,
    );
    if let Some(screen) = goto {
        app.navigate(screen);
    }
}
