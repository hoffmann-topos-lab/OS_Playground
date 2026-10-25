//! Tela "Entrada e Saída" (seções 5.1, 5.2 e 5.3).
//!
//! Leitor de artigos de [`crate::ui::reader`], com o conteúdo de
//! [`crate::ui::content_io`] e os diagramas de [`crate::ui::ioview`].

use eframe::egui;

use crate::app::OsPlaygroundApp;
use crate::ui::{content_io, reader};

pub fn show(app: &mut OsPlaygroundApp, ui: &mut egui::Ui) {
    let goto = reader::show(
        ui,
        content_io::ARTICLES,
        "Capítulo 5 — seções 5.1 a 5.3",
        &mut app.io,
    );
    if let Some(screen) = goto {
        app.navigate(screen);
    }
}
