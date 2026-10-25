//! Tela "Implementação do Sistema de Arquivos" (seções 4.3 e 4.4).
//!
//! Leitor de artigos de [`crate::ui::reader`], com o conteúdo de
//! [`crate::ui::content_fsimpl`] e os diagramas de disco de
//! [`crate::ui::fsview`].

use eframe::egui;

use crate::app::OsPlaygroundApp;
use crate::ui::{content_fsimpl, reader};

pub fn show(app: &mut OsPlaygroundApp, ui: &mut egui::Ui) {
    let goto = reader::show(
        ui,
        content_fsimpl::ARTICLES,
        "Capítulo 4 — seções 4.3 e 4.4",
        &mut app.fsimpl,
    );
    if let Some(screen) = goto {
        app.navigate(screen);
    }
}
