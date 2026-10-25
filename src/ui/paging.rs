//! Tela "Paginação" (seção 3.3).
//!
//! O leitor de artigos é o de [`crate::ui::reader`]; o conteúdo vem de
//! [`crate::ui::content_paging`]. Traz o tradutor de endereços virtuais, com
//! a tabela de páginas desenhada e a TLB acumulando acertos e faltas.

use eframe::egui;

use crate::app::OsPlaygroundApp;
use crate::ui::{content_paging, reader};

pub fn show(app: &mut OsPlaygroundApp, ui: &mut egui::Ui) {
    let goto = reader::show(
        ui,
        content_paging::ARTICLES,
        "Capítulo 3 — seção 3.3",
        &mut app.paging,
    );
    if let Some(screen) = goto {
        app.navigate(screen);
    }
}
