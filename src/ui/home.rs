//! Tela inicial: logo, título e o botão do tópico "Processos e Threads".
//!
//! Deliberadamente minimalista — sem subtítulo, sem rótulo de seção e sem
//! rodapé bibliográfico. As referências aparecem ao final de cada artigo.

use eframe::egui;

use crate::app::{OsPlaygroundApp, Screen};

pub fn show(app: &mut OsPlaygroundApp, ui: &mut egui::Ui) {
    let logo = app.logo().cloned();

    ui.vertical_centered(|ui| {
        ui.add_space(40.0);
        if let Some(tex) = &logo {
            let sized = egui::load::SizedTexture::from_handle(tex);
            ui.add(egui::Image::new(sized).max_height(220.0));
        }
        ui.add_space(14.0);
        ui.label(egui::RichText::new("OS Playground").size(34.0).strong());
        ui.add_space(36.0);

        let btn = egui::Button::new(egui::RichText::new("🧵  Processos e Threads").size(19.0))
            .min_size(egui::vec2(300.0, 56.0));
        if ui.add(btn).clicked() {
            app.navigate(Screen::ProcessesThreadsMenu);
        }

        ui.add_space(10.0);
        ui.label(
            egui::RichText::new("(mais tópicos de SO serão adicionados nas próximas fases)")
                .size(12.0)
                .color(egui::Color32::DARK_GRAY),
        );
    });
}
