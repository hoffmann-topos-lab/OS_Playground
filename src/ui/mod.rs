//! Camada de interface (telas e componentes reutilizáveis).

// ---- Processos e Threads (capítulo 2) ----
pub mod content;
pub mod content_race;
pub mod content_sched;
pub mod demo;
pub mod gantt;
pub mod process_tree;
pub mod race;
pub mod scheduling;
pub mod texts;
pub mod tree_view;

// ---- Memória Virtual (capítulo 3) ----
pub mod addressing;
pub mod content_addressing;
pub mod content_pagedesign;
pub mod content_paging;
pub mod content_replacement;
pub mod memaddr;
pub mod memfit;
pub mod pagedesign;
pub mod pagerepl;
pub mod paging;
pub mod replacement;

// ---- Sistema de Arquivos (capítulo 4) ----
pub mod content_files;
pub mod content_fsimpl;
pub mod devices;
pub mod explorer;
pub mod files;
pub mod fsimpl;
pub mod fsview;

// ---- Entrada e Saída (capítulo 5) ----
pub mod content_io;
pub mod io;
pub mod ioview;

// ---- Impasses (capítulo 6) ----
pub mod avoidance;
pub mod content_avoidance;
pub mod content_deadlock;
pub mod content_detection;
pub mod content_philo;
pub mod deadlock;
pub mod deadlockview;
pub mod detection;
pub mod philo;
pub mod philosophers;

// ---- comuns ----
pub mod home;
pub mod reader;
pub mod theme;
pub mod topic_menu;

#[cfg(test)]
mod snapshots;

use eframe::egui;

/// Azul de destaque usado para a raiz da árvore (pid 0/1) e realces.
pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(0x6f, 0xb3, 0xff);

/// Moldura padrão dos widgets embutidos nos artigos (simuladores, diagramas
/// interativos): fundo discreto, borda fina, cantos arredondados.
pub fn widget_frame(ui: &egui::Ui) -> egui::Frame {
    egui::Frame::new()
        .inner_margin(egui::Margin::same(12))
        .corner_radius(9.0)
        .fill(ui.visuals().faint_bg_color)
        .stroke(egui::Stroke::new(
            1.0,
            ui.visuals().widgets.noninteractive.bg_stroke.color,
        ))
}

/// Parâmetros de um widget em duas colunas, rótulo | controle.
///
/// Sliders com `.text()` dentro de `horizontal_wrapped` quebram linha pela
/// largura do trilho e deixam o rótulo vazar; a grade não tem esse problema.
pub fn params_grid(ui: &mut egui::Ui, id_salt: impl egui::AsIdSalt, add: impl FnOnce(&mut egui::Ui)) {
    egui::Grid::new(id_salt)
        .num_columns(2)
        .spacing([10.0, 4.0])
        .show(ui, |ui| {
            ui.spacing_mut().slider_width = 110.0;
            add(ui);
        });
}

/// Uma linha de [`params_grid`]: rótulo à esquerda, controle à direita.
pub fn param_row(ui: &mut egui::Ui, label: &str, widget: impl egui::Widget) -> egui::Response {
    ui.label(egui::RichText::new(label).size(12.5));
    let resp = ui.add(widget);
    ui.end_row();
    resp
}

/// Cabeçalho dos widgets embutidos: rótulo em destaque + dica curta.
pub fn widget_header(ui: &mut egui::Ui, label: &str, hint: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new(label).size(14.0).strong().color(ACCENT));
        ui.label(
            egui::RichText::new(hint)
                .size(12.0)
                .color(egui::Color32::GRAY),
        );
    });
    ui.add_space(8.0);
}
