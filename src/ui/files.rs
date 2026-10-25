//! Tela "Arquivos e Diretórios" (seções 4.1 e 4.2).
//!
//! Leitor de artigos de [`crate::ui::reader`], com o conteúdo de
//! [`crate::ui::content_files`].

use eframe::egui;

use crate::app::OsPlaygroundApp;
use crate::ui::{content_files, reader};

pub fn show(app: &mut OsPlaygroundApp, ui: &mut egui::Ui) {
    let goto = reader::show(
        ui,
        content_files::ARTICLES,
        "Capítulo 4 — seções 4.1 e 4.2",
        &mut app.files,
    );
    if let Some(screen) = goto {
        app.navigate(screen);
    }
}
