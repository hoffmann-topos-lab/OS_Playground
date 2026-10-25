//! Tela "Espaços de Endereçamento" (seções 3.1 e 3.2).
//!
//! O leitor de artigos é o de [`crate::ui::reader`]; o conteúdo vem de
//! [`crate::ui::content_addressing`]. Traz os registradores de base e limite
//! interativos e o simulador de gerenciamento de memória livre.

use eframe::egui;

use crate::app::OsPlaygroundApp;
use crate::ui::{content_addressing, reader};

pub fn show(app: &mut OsPlaygroundApp, ui: &mut egui::Ui) {
    let goto = reader::show(
        ui,
        content_addressing::ARTICLES,
        "Capítulo 3 — seções 3.1 e 3.2",
        &mut app.addressing,
    );
    if let Some(screen) = goto {
        app.navigate(screen);
    }
}
