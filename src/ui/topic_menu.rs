//! Menu do tópico "Processos e Threads" com as 4 opções (i–iv).

use eframe::egui;

use crate::app::{OsPlaygroundApp, Screen};

struct Opcao {
    icone: &'static str,
    titulo: &'static str,
    descricao: &'static str,
    tela: Screen,
}

const OPCOES: &[Opcao] = &[
    Opcao {
        icone: "🌳",
        titulo: "Árvore de Processos",
        descricao: "Hierarquia real do sistema ao vivo, a partir do pid 1 (init/launchd), com threads de cada processo.",
        tela: Screen::ProcessTree,
    },
    Opcao {
        icone: "📖",
        titulo: "Conceitos: Processos e Threads",
        descricao: "Textos explicativos: modelo de processo, criação/término, estados, e o modelo de threads.",
        tela: Screen::Texts,
    },
    Opcao {
        icone: "🔒",
        titulo: "Race Conditions",
        descricao: "Ilustrações dos algoritmos de exclusão mútua: Peterson, TSL, semáforos, mutexes.",
        tela: Screen::RaceConditions,
    },
    Opcao {
        icone: "⏱",
        titulo: "Escalonamento",
        descricao: "Visualização dos algoritmos: FCFS, SJF, Round-Robin, prioridade e loteria.",
        tela: Screen::Scheduling,
    },
];

pub fn show(app: &mut OsPlaygroundApp, ui: &mut egui::Ui) {
    ui.add_space(16.0);
    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new("Escolha o que deseja explorar")
                .size(18.0)
                .strong(),
        );
    });
    ui.add_space(16.0);

    let mut destino: Option<Screen> = None;
    egui::ScrollArea::vertical().show(ui, |ui| {
        for opc in OPCOES {
            if card(ui, opc).clicked() {
                destino = Some(opc.tela);
            }
            ui.add_space(12.0);
        }
    });

    if let Some(tela) = destino {
        app.navigate(tela);
    }
}

/// Um cartão clicável para uma opção do menu.
fn card(ui: &mut egui::Ui, opc: &Opcao) -> egui::Response {
    let desired = egui::vec2(ui.available_width().min(720.0), 84.0);
    let frame = egui::Frame::group(ui.style())
        .inner_margin(egui::Margin::same(14))
        .corner_radius(10.0);

    let inner = frame
        .show(ui, |ui| {
            ui.set_min_size(desired);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(opc.icone).size(34.0));
                ui.add_space(10.0);
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(opc.titulo).size(18.0).strong());
                    ui.add_space(2.0);
                    ui.label(
                        egui::RichText::new(opc.descricao)
                            .size(13.0)
                            .color(egui::Color32::GRAY),
                    );
                });
            });
        })
        .response;

    // Torna o cartão inteiro clicável e com cursor de "mãozinha".
    let resp = inner.interact(egui::Sense::click());
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}
