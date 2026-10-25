//! Menu de um tópico: os cartões das sub-telas.
//!
//! Cada tópico tem a sua lista de opções ([`PROCESSES`], [`MEMORY`]); a tela é
//! a mesma para todos.

use eframe::egui;

use crate::app::{OsPlaygroundApp, Screen};

pub struct Opcao {
    icone: &'static str,
    titulo: &'static str,
    descricao: &'static str,
    tela: Screen,
}

/// As 4 opções de "Processos e Threads" (capítulo 2).
pub const PROCESSES: &[Opcao] = &[
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

/// As 4 opções de "Memória Virtual" (capítulo 3).
pub const MEMORY: &[Opcao] = &[
    Opcao {
        icone: "🗺",
        titulo: "Espaços de Endereçamento",
        descricao: "Da memória física exposta ao espaço de endereçamento: base e limite, troca de processos e o gerenciamento de memória livre (first, next, best e worst fit).",
        tela: Screen::Addressing,
    },
    Opcao {
        icone: "📄",
        titulo: "Paginação",
        descricao: "Páginas e quadros, a MMU, a decomposição do endereço virtual, a tabela de páginas e a TLB, com um tradutor de endereços interativo.",
        tela: Screen::Paging,
    },
    Opcao {
        icone: "♻",
        titulo: "Substituição de Páginas",
        descricao: "Ótimo, NRU, FIFO, segunda chance, relógio, LRU, envelhecimento, conjunto de trabalho e WSClock, com simulador passo a passo.",
        tela: Screen::Replacement,
    },
    Opcao {
        icone: "🛠",
        titulo: "Projeto e Implementação",
        descricao: "Alocação local × global, controle de carga, daemon de paginação, tamanho da página, páginas compartilhadas e o tratamento de uma falta de página.",
        tela: Screen::PageDesign,
    },
];

/// As 3 opções de "Sistema de Arquivos" (capítulo 4).
pub const FILES: &[Opcao] = &[
    Opcao {
        icone: "🔎",
        titulo: "Explorador de Arquivos",
        descricao: "O sistema de arquivos real, ao vivo: número de i-node, contagem de ligações, hard links e o stat de cada arquivo da sua máquina.",
        tela: Screen::FileExplorer,
    },
    Opcao {
        icone: "📄",
        titulo: "Arquivos e Diretórios",
        descricao: "A visão do usuário: nomes e i-nodes, tipos de arquivo, acesso e atributos, a árvore de diretórios, caminhos e ligações.",
        tela: Screen::FilesConcepts,
    },
    Opcao {
        icone: "🧩",
        titulo: "Implementação do Sistema de Arquivos",
        descricao: "Partições e inicialização, o layout no disco, alocação por i-node com índice de vários níveis, journaling e gerenciamento de espaço.",
        tela: Screen::FileSystemImpl,
    },
];

/// As 2 opções de "Entrada e Saída" (capítulo 5).
pub const IO: &[Opcao] = &[
    Opcao {
        icone: "🔌",
        titulo: "Dispositivos ao Vivo",
        descricao: "Os arquivos especiais de /dev da sua máquina: dispositivos de bloco e de caractere, com os números maior e menor de cada um.",
        tela: Screen::DeviceBrowser,
    },
    Opcao {
        icone: "⏱",
        titulo: "Conceitos de Entrada e Saída",
        descricao: "Hardware de E/S e controladores, DMA, e as três formas de fazer E/S (programa, interrupção e DMA) num simulador animado, mais as camadas de software.",
        tela: Screen::IoConcepts,
    },
];

pub fn show(app: &mut OsPlaygroundApp, ui: &mut egui::Ui, opcoes: &[Opcao]) {
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
        for opc in opcoes {
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
pub(crate) fn card(ui: &mut egui::Ui, opc: &Opcao) -> egui::Response {
    let width = ui.available_width().min(720.0);
    let frame = egui::Frame::group(ui.style())
        .inner_margin(egui::Margin::same(14))
        .corner_radius(10.0);

    let inner = frame
        .show(ui, |ui| {
            // Largura fixa e descrição com wrap: assim todos os cartões ficam do
            // mesmo tamanho. Sem o max_width e o wrap, o texto estica a caixa
            // (TextWrapMode::Extend) e cada cartão sai com uma largura diferente.
            ui.set_min_size(egui::vec2(width, 84.0));
            ui.set_max_width(width);
            ui.horizontal_top(|ui| {
                ui.label(egui::RichText::new(opc.icone).size(34.0));
                ui.add_space(10.0);
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(opc.titulo).size(18.0).strong());
                    ui.add_space(2.0);
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(opc.descricao)
                                .size(13.0)
                                .color(egui::Color32::GRAY),
                        )
                        .wrap(),
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
