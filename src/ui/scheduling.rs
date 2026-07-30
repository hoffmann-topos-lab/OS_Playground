//! Tela "Escalonamento".
//!
//! O leitor de artigos é o de [`crate::ui::reader`]; o conteúdo vem de
//! [`crate::ui::content_sched`]. Cada algoritmo traz um cenário vivo no
//! simulador de Gantt, com os tempos editáveis.

use eframe::egui;

use crate::app::OsPlaygroundApp;
use crate::ui::{content_sched, reader};

pub fn show(app: &mut OsPlaygroundApp, ui: &mut egui::Ui) {
    let goto = reader::show(
        ui,
        content_sched::ARTICLES,
        "Capítulo 2 — seção 2.5",
        &mut app.scheduling,
    );
    if let Some(screen) = goto {
        app.navigate(screen);
    }
}
