//! OS Playground — ferramenta visual para a disciplina de Sistemas Operacionais.
//!
//! Fase 1: janela principal + tópico "Processos e Threads" com a árvore de
//! processos ao vivo funcional (as demais opções são telas iniciais).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod sys;
mod ui;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 720.0])
            .with_min_inner_size([820.0, 560.0])
            .with_title("OS Playground")
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "OS Playground",
        native_options,
        Box::new(|cc| Ok(Box::new(app::OsPlaygroundApp::new(cc)))),
    )
}

/// Ícone da janela, gerado por `cargo run --bin make_icons` (fundo removido).
fn load_icon() -> egui::IconData {
    let bytes = include_bytes!("../assets/window_icon.png");
    eframe::icon_data::from_png_bytes(bytes).expect("window_icon.png inválido")
}
