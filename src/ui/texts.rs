//! Tela "Conceitos: Processos e Threads".
//!
//! O leitor de artigos é o de [`crate::ui::reader`]; o conteúdo vem de
//! [`crate::ui::content`].

use eframe::egui;

use crate::app::OsPlaygroundApp;
use crate::ui::{content, reader};

pub fn show(app: &mut OsPlaygroundApp, ui: &mut egui::Ui) {
    let goto = reader::show(
        ui,
        content::ARTICLES,
        "Capítulo 2 — seções 2.1 e 2.2",
        &mut app.texts,
    );
    if let Some(screen) = goto {
        app.navigate(screen);
    }
}
