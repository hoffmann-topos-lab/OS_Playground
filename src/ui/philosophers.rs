//! Tela "O Jantar dos Filósofos" (seção 6.2).
//!
//! Leitor de artigos de [`crate::ui::reader`], com o conteúdo de
//! [`crate::ui::content_philo`] e o simulador de [`crate::ui::philo`].

use eframe::egui;

use crate::app::OsPlaygroundApp;
use crate::ui::{content_philo, reader};

pub fn show(app: &mut OsPlaygroundApp, ui: &mut egui::Ui) {
    let goto = reader::show(
        ui,
        content_philo::ARTICLES,
        "Capítulo 6 — seção 6.2",
        &mut app.philosophers,
    );
    if let Some(screen) = goto {
        app.navigate(screen);
    }
}
