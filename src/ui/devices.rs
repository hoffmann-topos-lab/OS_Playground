//! Tela "Dispositivos ao Vivo": os arquivos especiais de `/dev`, ao vivo.
//!
//! Contraparte prática da seção 5.1.1. A divisão entre dispositivos de bloco e
//! de caractere, que o texto apresenta em abstrato, aparece aqui de forma
//! concreta: cada dispositivo do sistema é um arquivo especial em `/dev`,
//! marcado com `b` ou `c` e identificado por um número maior e um menor.

use eframe::egui;

use crate::app::OsPlaygroundApp;
use crate::sys::devices::{DevKind, DevList, Device};
use crate::ui::ACCENT;

#[derive(Default)]
pub struct DevicesState {
    list: Option<DevList>,
}

impl DevicesState {
    pub(crate) fn ui(&mut self, ui: &mut egui::Ui) {
        if self.list.is_none() {
            self.list = Some(DevList::read());
        }

        egui::Panel::top("dev_toolbar").exact_size(38.0).show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                if ui.button("🔄 Reler /dev").clicked() {
                    self.list = Some(DevList::read());
                }
                ui.separator();
                if let Some(list) = &self.list {
                    ui.label(
                        egui::RichText::new(format!(
                            "{} dispositivos  ·  {} de bloco  ·  {} de caractere",
                            list.devices.len(),
                            list.block_count,
                            list.char_count
                        ))
                        .color(egui::Color32::GRAY),
                    );
                }
            });
        });

        egui::CentralPanel::default().show(ui, |ui| {
            let list = self.list.as_ref().unwrap();
            if let Some(err) = &list.error {
                ui.add_space(16.0);
                ui.colored_label(egui::Color32::from_rgb(0xff, 0x8a, 0x5b), err);
                return;
            }

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(6.0);
                    ui.add(egui::Label::new(
                        egui::RichText::new(
                            "Cada linha abaixo é um arquivo especial de /dev. O número maior diz \
                             qual driver atende o dispositivo (seção 5.3.2); o número menor \
                             escolhe a unidade concreta. No terminal, ls -l /dev mostra os mesmos \
                             b e c na primeira coluna.",
                        )
                        .size(12.5),
                    ));
                    ui.add_space(10.0);

                    group(
                        ui,
                        DevKind::Block,
                        "Dispositivos de bloco",
                        "Leem e escrevem blocos endereçáveis e independentes. São os discos e as \
                         partições.",
                        &list.devices,
                    );
                    ui.add_space(14.0);
                    group(
                        ui,
                        DevKind::Char,
                        "Dispositivos de caractere",
                        "Entregam um fluxo de bytes, sem endereço nem busca. São terminais, \
                         pseudoterminais e fontes como null, zero e random.",
                        &list.devices,
                    );
                });
        });
    }
}

/// Ponto de entrada chamado pela navegação do app.
pub fn show(app: &mut OsPlaygroundApp, ui: &mut egui::Ui) {
    app.devices.ui(ui);
}

fn group(ui: &mut egui::Ui, kind: DevKind, title: &str, blurb: &str, all: &[Device]) {
    let color = match kind {
        DevKind::Block => ACCENT,
        DevKind::Char => egui::Color32::from_rgb(0xff, 0xb4, 0x5b),
    };
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("{}  {title}", kind.ls_char()))
                .size(15.0)
                .strong()
                .color(color),
        );
    });
    ui.add_space(2.0);
    ui.add(egui::Label::new(
        egui::RichText::new(blurb).size(12.0).color(egui::Color32::GRAY),
    ));
    ui.add_space(6.0);

    let devs: Vec<&Device> = all.iter().filter(|d| d.kind == kind).collect();
    if devs.is_empty() {
        ui.label(
            egui::RichText::new("nenhum encontrado neste sistema")
                .size(12.0)
                .italics()
                .color(egui::Color32::GRAY),
        );
        return;
    }

    egui::Grid::new(("dev_grid", title))
        .num_columns(4)
        .striped(true)
        .spacing([18.0, 4.0])
        .show(ui, |ui| {
            for h in ["tipo", "maior", "menor", "nome"] {
                ui.label(egui::RichText::new(h).size(11.0).strong().color(color));
            }
            ui.end_row();
            for d in devs {
                ui.label(
                    egui::RichText::new(d.kind.ls_char().to_string())
                        .monospace()
                        .size(12.5)
                        .color(color),
                );
                ui.label(egui::RichText::new(d.major.to_string()).monospace().size(12.0));
                ui.label(egui::RichText::new(d.minor.to_string()).monospace().size(12.0));
                ui.label(egui::RichText::new(&d.name).monospace().size(12.5));
                ui.end_row();
            }
        });
}
