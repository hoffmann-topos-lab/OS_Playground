//! Estado de topo do aplicativo: navegação entre telas e recursos globais.

use eframe::egui;

use crate::ui;

/// As telas navegáveis da fase 1.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Home,
    ProcessesThreadsMenu,
    ProcessTree,
    Texts,
    RaceConditions,
    Scheduling,
}

impl Screen {
    /// Título mostrado na barra superior.
    pub fn title(self) -> &'static str {
        match self {
            Screen::Home => "OS Playground",
            Screen::ProcessesThreadsMenu => "Processos e Threads",
            Screen::ProcessTree => "Árvore de Processos",
            Screen::Texts => "Processos e Threads — Conceitos",
            Screen::RaceConditions => "Race Conditions",
            Screen::Scheduling => "Escalonamento",
        }
    }
}

pub struct OsPlaygroundApp {
    screen: Screen,
    /// Pilha de navegação para o botão "voltar".
    history: Vec<Screen>,
    /// Textura do logo (fundo removido), carregada uma vez.
    logo: Option<egui::TextureHandle>,
    /// Estado da árvore de processos (snapshot, filtro, seleção).
    pub process_tree: ui::process_tree::ProcessTreeState,
    /// Estado das três telas-leitor (artigo aberto + widgets embutidos).
    pub texts: ui::reader::ReaderState,
    pub race: ui::reader::ReaderState,
    pub scheduling: ui::reader::ReaderState,
}

impl OsPlaygroundApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        ui::theme::apply(&cc.egui_ctx);
        let logo = load_logo_texture(&cc.egui_ctx);
        Self {
            screen: Screen::Home,
            history: Vec::new(),
            logo,
            process_tree: ui::process_tree::ProcessTreeState::default(),
            texts: ui::reader::ReaderState::default(),
            race: ui::reader::ReaderState::default(),
            scheduling: ui::reader::ReaderState::default(),
        }
    }

    /// Navega para `to`, empilhando a tela atual para permitir voltar.
    pub fn navigate(&mut self, to: Screen) {
        if to != self.screen {
            self.history.push(self.screen);
            self.screen = to;
        }
    }

    /// Volta para a tela anterior (se houver).
    pub fn back(&mut self) {
        if let Some(prev) = self.history.pop() {
            self.screen = prev;
        }
    }

    pub fn logo(&self) -> Option<&egui::TextureHandle> {
        self.logo.as_ref()
    }
}

impl eframe::App for OsPlaygroundApp {
    // No eframe 0.35 o método principal é `ui`, recebendo um `Ui` raiz.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Barra superior: botão voltar + título da tela atual.
        egui::Panel::top("appbar").exact_size(48.0).show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.add_space(8.0);
                let can_go_back = !self.history.is_empty();
                if ui
                    .add_enabled(can_go_back, egui::Button::new("  ⬅  Voltar  "))
                    .clicked()
                {
                    self.back();
                }
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);
                ui.heading(self.screen.title());
            });
        });

        egui::CentralPanel::default().show(ui, |ui| match self.screen {
            Screen::Home => ui::home::show(self, ui),
            Screen::ProcessesThreadsMenu => ui::topic_menu::show(self, ui),
            Screen::ProcessTree => ui::process_tree::show(self, ui),
            Screen::Texts => ui::texts::show(self, ui),
            Screen::RaceConditions => ui::race::show(self, ui),
            Screen::Scheduling => ui::scheduling::show(self, ui),
        });
    }
}

/// Carrega `assets/window_icon.png` como textura para exibir na tela inicial.
fn load_logo_texture(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let bytes = include_bytes!("../assets/window_icon.png");
    let img = image::load_from_memory(bytes).ok()?.to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    let color = egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw());
    Some(ctx.load_texture("logo", color, egui::TextureOptions::LINEAR))
}
